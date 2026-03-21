use crate::app::{App as CLI};
use crate::helpers::create_console_theme;
use crate::systems::*;
use crate::utils::tool_record::ToolRecord;
use async_trait::async_trait;
use rex_old_core::flow::resolve::Resolver;
use rex_old_core::{
    ConfigMode, RexConfig, RexEnvironment, DEBUG_PLUGIN_KEY, ToolContext, ToolSpec,
    // load_debug_plugin_with_rex, load_tool, registry::RexRegistry,
    load_tool, registry::RexRegistry,
};
use rex_old_core::{RexConfigError, RexLoaderError};
use rustc_hash::FxHashSet;
use semver::Version;
use starbase::{AppResult, AppSession};
use starbase_console::{Console, EmptyReporter};
use std::sync::Arc;
use tokio::task::JoinSet;

#[derive(Debug, Default)]
pub struct LoadToolOptions {
    pub all: bool,
    pub contexts: FxHashSet<ToolContext>,
    pub detect_version: bool,
    pub inherit_local: bool,
    pub inherit_remote: bool,
}

pub type RexConsole = Console<EmptyReporter>;

#[derive(Clone)]
pub struct RexSession {
    pub cli: CLI,
    pub cli_version: Version,
    pub console: RexConsole,
    pub env: Arc<RexEnvironment>,
}

impl RexSession {
    pub fn new(cli: CLI) -> Self {
        let env = RexEnvironment::default();

        let mut console = Console::<EmptyReporter>::new(false);
        console.set_theme(create_console_theme());
        console.set_reporter(EmptyReporter);

        Self {
            cli,
            cli_version: Version::parse(env!("CARGO_PKG_VERSION")).unwrap(),
            console,
            env: Arc::new(env),
        }
    }

    pub fn create_registry(&self) -> RexRegistry {
        RexRegistry::new(Arc::clone(&self.env))
    }

    pub fn load_config(&self) -> Result<&RexConfig, RexConfigError> {
        self.env.load_config()
    }

    pub fn load_config_with_mode(
        &self,
        mode: ConfigMode,
    ) -> Result<&RexConfig, RexConfigError> {
        self.env.load_config_with_mode(mode)
    }

    #[tracing::instrument(name = "load_tool", skip(self))]
    pub async fn load_tool_with_options(
        &self,
        context: &ToolContext,
        options: LoadToolOptions,
    ) -> Result<ToolRecord, RexLoaderError> {
        let mut record = ToolRecord::new(load_tool(context, &self.env).await?);

        if options.inherit_remote {
            record.inherit_from_remote().await?;
        }

        if options.inherit_local {
            record.inherit_from_local(self.load_config()?);
        }

        if options.detect_version {
            record.detect_version_and_source().await;

            let mut spec = record
                .detected_version
                .clone()
                .unwrap_or_else(|| ToolSpec::parse("*").unwrap());

            Resolver::resolve(&record.tool, &mut spec, false).await?;

            record.spec = spec;
        }

        Ok(record)
    }

    /// Load tools that have a configured version.
    pub async fn load_tools(&self) -> Result<Vec<ToolRecord>, RexLoaderError> {
        self.load_tools_with_options(LoadToolOptions::default())
            .await
    }

    #[tracing::instrument(name = "load_tools", skip(self))]
    pub async fn load_tools_with_options(
        &self,
        mut options: LoadToolOptions,
    ) -> Result<Vec<ToolRecord>, RexLoaderError> {
        let config = self.env.load_config()?;

        // Gather the IDs of all possible tools. We can't just use the
        // `plugins` map, because some tools may not have a plugin entry,
        // for example, those using backends.
        let mut contexts = FxHashSet::default();
        contexts.extend(
            config
                .plugins
                .tools
                .keys()
                .map(|id| ToolContext::new(id.to_owned())),
        );
        contexts.extend(config.versions.keys().cloned());

        // If no filter IDs provided, inherit the IDs from the current
        // config for every tool that has a version. Otherwise, we'll
        // load all tools, even built-ins, when the user isn't using them.
        // This causes quite a performance hit.
        if options.contexts.is_empty() {
            if options.all {
                options.contexts.extend(contexts.clone());
            } else {
                options.contexts.extend(config.versions.keys().cloned());
            }
        }

        // Download the schema plugin before loading plugins.
        // We must do this here, otherwise when multiple schema
        // based tools are installed in parallel, they will
        // collide when attempting to download the schema plugin!
        // if !contexts.is_empty() {
        //     load_debug_plugin_with_rex(&self.env).await?;
        // }

        let mut set = JoinSet::<Result<ToolRecord, RexLoaderError>>::new();
        let mut records = vec![];
        let opt_inherit_remote = options.inherit_remote;
        let opt_detect_version = options.detect_version;

        for context in contexts {
            if !options.contexts.contains(&context) {
                continue;
            }

            // These shouldn't be treated as a "normal plugin"
            if context.id == DEBUG_PLUGIN_KEY {
                continue;
            }

            let rex = Arc::clone(&self.env);

            set.spawn(async move {
                let mut record = ToolRecord::new(load_tool(&context, &rex).await?);

                if opt_inherit_remote {
                    record.inherit_from_remote().await?;
                }

                if opt_detect_version {
                    record.detect_version_and_source().await;
                }

                Ok(record)
            });
        }

        while let Some(result) = set.join_next().await {
            let mut record: ToolRecord = result.unwrap()?;

            if options.inherit_local {
                record.inherit_from_local(config);
            }

            records.push(record);
        }

        Ok(records)
    }

    pub fn should_print_json(&self) -> bool {
        self.cli.json
    }
}

#[async_trait]
impl AppSession for RexSession {
    async fn startup(&mut self) -> AppResult {
        self.env = Arc::new(detect_rex_env(&self.cli)?);

        Ok(None)
    }

    async fn analyze(&mut self) -> AppResult {
        load_rex_configs(&self.env)?;

        Ok(None)
    }

    async fn execute(&mut self) -> AppResult {
        clean_rex_backups(&self.env)?;

        Ok(None)
    }

    async fn shutdown(&mut self) -> AppResult {
        self.console.out.flush()?;
        self.console.err.flush()?;

        Ok(None)
    }
}
