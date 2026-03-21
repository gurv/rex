use crate::utils::tool_record::ToolRecord;
use indexmap::{IndexMap, IndexSet};
use miette::IntoDiagnostic;
use rex_old_core::flow::locate::Locator;
use rex_old_core::flow::manage::RexManageError;
use rex_old_core::flow::resolve::Resolver;
use rex_old_core::{RexConfig, RexConfigEnvOptions, ToolContext, ToolSpec};
use rex_pdk_api::{
    ActivateEnvironmentInput, ActivateEnvironmentOutput, HookFunction, PluginFunction, RunHook,
    RunHookResult,
};
use rustc_hash::FxHashMap;
use starbase_shell::{BoxedShell, join_args};
use starbase_utils::envx;
use std::collections::VecDeque;
use std::env;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process::Command;
use tokio::task::JoinSet;
use tracing::trace;

#[derive(Default)]
pub struct ExecItem {
    context: ToolContext,
    active: bool,
    args: Vec<String>,
    env: IndexMap<String, Option<String>>,
    paths: IndexSet<PathBuf>,
}

impl ExecItem {
    pub fn add_args(&mut self, args: Vec<String>) {
        self.args.extend(args);
    }

    pub fn add_path(&mut self, path: PathBuf) {
        // Only add paths that exist
        if path.exists() {
            self.paths.insert(path);
        }
    }

    pub fn set_env(&mut self, key: String, value: String) {
        self.env.insert(key, Some(value));
    }

    // pub fn remove_env(&mut self, key: String) {
    //     self.env.insert(key, None);
    // }
}

#[derive(Clone, Default)]
pub struct ExecWorkflowParams {
    pub activate_environment: bool,
    pub check_process_env: bool,
    pub fallback_any_spec: bool,
    pub passthrough_args: Vec<String>,
    pub pre_run_hook: bool,
    pub version_env_vars: bool,
}

pub struct ExecWorkflow<'app> {
    pub args: Vec<String>,
    pub env: IndexMap<String, Option<String>>,
    pub paths: VecDeque<PathBuf>,

    config: &'app RexConfig,
    multiple: bool,
    tools: Vec<ToolRecord>,
}

impl<'app> ExecWorkflow<'app> {
    pub fn new(tools: Vec<ToolRecord>, config: &'app RexConfig) -> Self {
        Self {
            multiple: tools.len() > 1,
            tools,
            args: vec![],
            env: IndexMap::default(),
            paths: VecDeque::default(),
            config,
        }
    }

    pub fn collect_item(&mut self, item: ExecItem) {
        self.args.extend(item.args);

        for (key, value) in item.env {
            self.env.insert(key, value);
        }

        // Don't use a set as we need to persist the order!
        for path in item.paths {
            if !self.paths.contains(&path) {
                self.paths.push_back(path);
            }
        }
    }

    pub async fn prepare_environment(
        &mut self,
        mut specs: FxHashMap<ToolContext, ToolSpec>,
        params: ExecWorkflowParams,
    ) -> miette::Result<()> {
        let mut set = JoinSet::<Result<ExecItem, RexManageError>>::new();

        for tool in std::mem::take(&mut self.tools) {
            let provided_spec = specs.remove(&tool.context);
            let params = params.clone();

            // Extract in a background thread
            set.spawn(async move { prepare_tool(tool, provided_spec, params).await });
        }

        // Inherit shared environment variables
        self.env
            .extend(self.config.get_env_vars(RexConfigEnvOptions {
                include_shared: true,
                ..Default::default()
            })?);

        while let Some(item) = set.join_next().await {
            let item = item.into_diagnostic()??;

            if item.active {
                // Inherit tool environment variables
                self.env
                    .extend(self.config.get_env_vars(RexConfigEnvOptions {
                        context: Some(&item.context),
                        check_process: params.check_process_env,
                        ..Default::default()
                    })?);

                self.collect_item(item);
            }
        }

        Ok(())
    }

    #[cfg(unix)]
    pub fn create_wrapped_command<E, AI, A>(
        &self,
        shell: &BoxedShell,
        exe: E,
        args: AI,
        raw: bool,
    ) -> String
    where
        E: AsRef<OsStr>,
        AI: IntoIterator<Item = A>,
        A: AsRef<OsStr>,
    {
        let args = args.into_iter().collect::<Vec<_>>();
        let mut line = vec![exe.as_ref()];

        line.extend(args.iter().map(|arg| arg.as_ref()));

        if !self.multiple && !self.args.is_empty() {
            line.extend(self.args.iter().map(OsStr::new));
        }

        if raw {
            line.join(OsStr::new(" ")).into_string().unwrap()
        } else {
            join_args(shell, line)
        }
    }

    // `Quotable` doesn't support `OsStr` on Windows,
    // so we need to convert everything to strings...
    #[cfg(windows)]
    pub fn create_wrapped_command<E, AI, A>(
        &self,
        shell: &BoxedShell,
        exe: E,
        args: AI,
        raw: bool,
    ) -> String
    where
        E: AsRef<OsStr>,
        AI: IntoIterator<Item = A>,
        A: AsRef<OsStr>,
    {
        let args = args.into_iter().collect::<Vec<_>>();
        let mut line = vec![exe.as_ref().to_string_lossy().to_string()];

        line.extend(
            args.iter()
                .map(|arg| arg.as_ref().to_string_lossy().to_string()),
        );

        if !self.multiple && !self.args.is_empty() {
            line.extend(self.args.clone());
        }

        if raw {
            line.join(" ")
        } else {
            join_args(shell, line.iter().collect::<Vec<_>>())
        }
    }

    pub fn create_command<E, AI, A>(self, exe: E, args: AI) -> miette::Result<Command>
    where
        E: AsRef<OsStr>,
        AI: IntoIterator<Item = A>,
        A: AsRef<OsStr>,
    {
        let mut command = Command::new(exe);
        command.args(args);

        self.apply_to_command(&mut command, false)?;

        Ok(command)
    }

    pub fn create_command_with_shell<E, AI, A>(
        self,
        shell: BoxedShell,
        exe: E,
        args: AI,
        raw: bool,
    ) -> miette::Result<Command>
    where
        E: AsRef<OsStr>,
        AI: IntoIterator<Item = A>,
        A: AsRef<OsStr>,
    {
        let mut command = Command::new(shell.to_string());
        command.args(shell.get_exec_command().shell_args);
        command.arg(self.create_wrapped_command(&shell, exe, args, raw));

        self.apply_to_command(&mut command, true)?;

        Ok(command)
    }

    pub fn apply_to_command(self, command: &mut Command, with_shell: bool) -> miette::Result<()> {
        if let Some(path) = self.join_paths()? {
            command.env("PATH", path);
        }

        for (key, value) in self.env {
            match value {
                Some(value) => command.env(key, value),
                None => command.env_remove(key),
            };
        }

        if !with_shell && !self.multiple && !self.args.is_empty() {
            command.args(self.args);
        }

        trace!(
            exe = ?command.get_program().to_string_lossy(),
            args = ?command.get_args().map(|arg| arg.to_string_lossy()).collect::<Vec<_>>(),
            "Created command to execute",
        );

        Ok(())
    }

    pub fn join_paths(&self) -> miette::Result<Option<OsString>> {
        if !self.paths.is_empty() {
            let mut list = self.paths.clone().into_iter().collect::<Vec<_>>();
            list.extend(envx::paths());

            return Ok(Some(env::join_paths(list).into_diagnostic()?));
        }

        Ok(None)
    }
}

async fn prepare_tool(
    tool: ToolRecord,
    provided_spec: Option<ToolSpec>,
    params: ExecWorkflowParams,
) -> Result<ExecItem, RexManageError> {
    let mut item = ExecItem {
        context: tool.context.clone(),
        ..Default::default()
    };

    // Extract the spec, otherwise return early
    let mut spec = match provided_spec {
        Some(inner) => inner,
        None => {
            if params.fallback_any_spec {
                ToolSpec::parse("*")?
            } else {
                return Ok(item);
            }
        }
    };

    item.active = true;

    // Resolve the version and locate executables
    Resolver::resolve(&tool, &mut spec, true).await?;

    if !tool.is_installed(&spec) {
        return Ok(item);
    }

    if params.version_env_vars {
        item.set_env(
            format!("{}_VERSION", tool.get_env_var_prefix()),
            spec.get_resolved_version().to_string(),
        );
    }

    // Extract vars/paths for environment
    let locations = Locator::locate(&tool, &spec).await?;

    if params.activate_environment
        && tool
            .plugin
            .has_func(PluginFunction::ActivateEnvironment)
            .await
    {
        let output: ActivateEnvironmentOutput = tool
            .plugin
            .call_func_with(
                PluginFunction::ActivateEnvironment,
                ActivateEnvironmentInput {
                    context: tool.create_plugin_context(&spec),
                    globals_dir: locations
                        .globals_dir
                        .as_ref()
                        .map(|dir| tool.to_virtual_path(dir)),
                },
            )
            .await?;

        for (key, value) in output.env {
            item.set_env(key, value);
        }

        for path in output.paths {
            item.add_path(path);
        }
    }

    if params.pre_run_hook && tool.plugin.has_func(HookFunction::PreRun).await {
        let output: RunHookResult = tool
            .plugin
            .call_func_with(
                HookFunction::PreRun,
                RunHook {
                    context: tool.create_plugin_context(&spec),
                    globals_dir: locations
                        .globals_dir
                        .as_ref()
                        .map(|dir| tool.to_virtual_path(dir)),
                    globals_prefix: locations.globals_prefix,
                    passthrough_args: params.passthrough_args,
                },
            )
            .await?;

        if let Some(value) = output.args {
            item.add_args(value);
        }

        if let Some(env) = output.env {
            for (key, value) in env {
                item.set_env(key, value);
            }
        }

        if let Some(paths) = output.paths {
            for path in paths {
                item.add_path(path);
            }
        }
    }

    // Extract executable directories
    if let Some(dir) = locations.exe_file.parent() {
        item.add_path(dir.to_path_buf());
    }

    for exes_dir in locations.exes_dirs {
        item.add_path(exes_dir);
    }

    for globals_dir in locations.globals_dirs {
        item.add_path(globals_dir);
    }

    // Mark it as used so that auto-clean doesn't remove it!
    if std::env::var("REX_SKIP_USED_AT").is_err()
        && let Some(version) = &spec.version
    {
        let _ = tool.inventory.create_product(version).track_used_at();
    }

    Ok(item)
}
