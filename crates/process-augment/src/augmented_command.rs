use rex_app_context::AppContext;
use rex_pdk_api::{
    ExecCommandInput, Extend, ExtendCommandInput, ExtendCommandOutput,
};
use rex_process::{Command, CommandArg};
use std::ffi::{OsStr, OsString};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

// Order of operations:
// - For task based commands:
//   - Inherit exe, args, and env from task
//   - Extend with `extend_command` plugin calls
//   - Extend with `extend_task_command` or `extend_task_script` plugin calls
// - For general commands:
//   - Inherit exe, args, and env from caller
//   - Extend with `extend_command` plugin calls

// Path ordering:
// - Plugin `extend_*` injected paths
// - Toolchain executable paths
// - proto store/shims/bin paths
// - rex store paths

pub struct AugmentedCommand<'app> {
    command: Command,
    context: &'app AppContext,
}

impl<'app> AugmentedCommand<'app> {
    pub fn new(context: &'app AppContext, bin: impl AsRef<OsStr>) -> Self {
        AugmentedCommand {
            command: Command::new(bin),
            context,
        }
    }

    pub fn from_input(
        context: &'app AppContext,
        input: &ExecCommandInput,
    ) -> Self {
        let mut builder = Self::new(context, &input.command);
        builder.args(&input.args);
        builder.envs(&input.env);
        builder
    }

    // pub fn from_task(context: &'app AppContext, bag: &'app GlobalEnvBag, task: &Task) -> Self {
    //     let mut builder = Self::new(context, bag, &task.command.value);

    //     if let Some(script) = &task.script {
    //         builder.set_script(script);
    //     } else {
    //         if let Some(quoted_command) = &task.command.quoted_value {
    //             builder.set_bin(CommandArg {
    //                 quoted_value: Some(OsString::from(quoted_command)),
    //                 value: OsString::from(&task.command.value),
    //             });
    //         }

    //         for arg in &task.args {
    //             builder.args.push_back(CommandArg {
    //                 quoted_value: arg.quoted_value.as_ref().map(OsString::from),
    //                 value: OsString::from(&arg.value),
    //             });
    //         }
    //     }

    //     for (key, value) in &task.env {
    //         builder.env_with_behavior(
    //             key,
    //             match value {
    //                 // Only set if system var not set
    //                 Some(val) => EnvBehavior::SetIfMissing(OsString::from(val)),
    //                 // Don't inherit system var
    //                 None => EnvBehavior::Unset,
    //             },
    //         );
    //     }

    pub fn augment(self) -> Command {
        self.command
    }

    pub fn apply_command_outputs(&mut self, outputs: Vec<ExtendCommandOutput>) {
        for output in outputs {
            if let Some(new_bin) = output.command {
                self.set_bin(new_bin);
            }

            if let Some(new_args) = output.args {
                self.apply_args(new_args);
            }

            self.envs(output.env);
            self.envs_remove(output.env_remove);
            self.apply_paths(output.paths);
        }
    }

    pub fn apply_args(&mut self, args: Extend<Vec<String>>) {
        match args {
            Extend::Empty => {
                self.args.clear();
            }
            Extend::Append(next) => {
                self.args(next);
            }
            Extend::Prepend(next) => {
                for arg in next.into_iter().rev() {
                    self.args.push_front(CommandArg {
                        quoted_value: None,
                        value: OsString::from(arg),
                    });
                }
            }
            Extend::Replace(next) => {
                self.args.clear();
                self.args(next);
            }
        }
    }

    pub fn apply_paths(&mut self, paths: Vec<PathBuf>) {
        if paths.is_empty() {
            return;
        }

        // Normalize separators since WASM always uses forward slashes
        #[cfg(windows)]
        let paths = paths.into_iter().map(|path| {
            PathBuf::from(rex_common::path::normalize_separators(
                path.to_string_lossy(),
            ))
        });

        self.append_paths(paths);
    }

    pub async fn inherit_from_plugins(
        &mut self,
    ) -> miette::Result<()> {
        let current_dir = &self.context.working_dir;

        self.apply_command_outputs(
            self.context
                .extension_registry
                .extend_command_all(|registry, extension| ExtendCommandInput {
                    context: registry.create_context(),
                    command: self.get_bin_name(),
                    args: self.get_args_list(),
                    current_dir: registry.to_virtual_path(current_dir),
                    extension_config: registry.create_config(&extension.id),
                    ..Default::default()
                })
                .await?,
        );

        Ok(())
    }

    pub fn inherit_proto(&mut self) {
        self.env("PROTO_AUTO_INSTALL", "false");
        self.env("PROTO_IGNORE_MIGRATE_WARNING", "true");
        self.env("PROTO_NO_PROGRESS", "true");
        self.env("PROTO_VERSION", "v0.55.4");
        self.env("STARBASE_FORCE_TTY", "true");
    }
}

impl Deref for AugmentedCommand<'_> {
    type Target = Command;

    fn deref(&self) -> &Self::Target {
        &self.command
    }
}

impl DerefMut for AugmentedCommand<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.command
    }
}
