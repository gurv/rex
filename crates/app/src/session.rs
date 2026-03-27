use crate::app::Cli;
use crate::app_error::AppError;
use crate::systems::*;
use async_trait::async_trait;
use rex_app_context::AppContext;
use rex_common::is_formatted_output;
use rex_config::{ConfigLoader, ExtensionsConfig};
use rex_console::{Console, RexReporter, create_console_theme};
use rex_env::RexEnvironment;
use rex_extension_plugin::*;
use rex_plugin::RexHostData;
use rex_process::ProcessRegistry;
use semver::Version;
use starbase::{AppResult, AppSession};
use std::env;
use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tracing::debug;

#[derive(Clone)]
pub struct RexSession {
    pub cli: Cli,
    pub cli_version: Version,
    pub config_loader: ConfigLoader,
    pub console: Console,
    pub rex_env: Arc<RexEnvironment>,
    extension_registry: OnceLock<Arc<ExtensionRegistry>>,
    pub extensions_config: Arc<ExtensionsConfig>,
    pub config_dir: PathBuf,
    pub working_dir: PathBuf,
    pub workspace_root: PathBuf,
}

impl RexSession {
    pub fn new(cli: Cli, cli_version: String) -> Self {
        debug!("Creating new application session");

        Self {
            cli_version: Version::parse(&cli_version).unwrap(),
            config_dir: PathBuf::new(),
            config_loader: ConfigLoader::default(),
            console: Console::new(cli.quiet || is_formatted_output()),
            extensions_config: Arc::new(ExtensionsConfig::default()),
            extension_registry: OnceLock::new(),
            rex_env: Arc::new(RexEnvironment::default()),
            working_dir: PathBuf::new(),
            workspace_root: PathBuf::new(),
            cli,
        }
    }

    pub async fn get_app_context(&self) -> miette::Result<Arc<AppContext>> {
        Ok(Arc::new(AppContext {
            cli_version: self.cli_version.clone(),
            config_dir: self.config_dir.clone(),
            console: self.get_console()?,
            rex_env: Arc::clone(&self.rex_env),
            extensions_config: Arc::clone(&self.extensions_config),
            extension_registry: self.get_extension_registry().await?,
            working_dir: self.working_dir.clone(),
            workspace_root: self.workspace_root.clone(),
        }))
    }

    pub fn get_console(&self) -> miette::Result<Arc<Console>> {
        Ok(Arc::new(self.console.clone()))
    }

    pub async fn get_extension_registry(&self) -> miette::Result<Arc<ExtensionRegistry>> {
        let item = self.extension_registry.get_or_init(|| {
            Arc::new(ExtensionRegistry::new(
                RexHostData {
                    rex_env: Arc::clone(&self.rex_env),
                    extensions_config: Arc::clone(&self.extensions_config),
                },
                Arc::clone(&self.extensions_config),
            ))
        });

        Ok(Arc::clone(item))
    }
}

#[async_trait]
impl AppSession for RexSession {
    /// Setup initial state for the session. Order is very important!!!
    async fn startup(&mut self) -> AppResult {
        self.console.set_reporter(RexReporter::default());
        self.console.set_theme(create_console_theme());

        self.working_dir = env::current_dir().map_err(|_| AppError::MissingWorkingDir)?;
        self.workspace_root = self.working_dir.clone();
        self.config_dir = self.config_loader.locate_dir(&self.workspace_root);

        self.rex_env = startup::detect_rex_environment(&self.working_dir, &self.workspace_root)?;

        Ok(None)
    }

    /// Analyze the current state and install/registery necessary functionality.
    async fn analyze(&mut self) -> AppResult {
        Ok(None)
    }

    async fn execute(&mut self) -> AppResult {
        Ok(None)
    }

    async fn shutdown(&mut self) -> AppResult {
        // Ensure all child processes have finished running
        ProcessRegistry::instance()
            .wait_for_running_to_shutdown()
            .await;

        self.console.close()?;

        Ok(None)
    }
}

impl fmt::Debug for RexSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CliSession")
            .field("cli", &self.cli)
            .field("cli_version", &self.cli_version)
            .field("rex_env", &self.rex_env)
            .field("extensions_config", &self.extensions_config)
            .field("working_dir", &self.working_dir)
            .field("workspace_root", &self.workspace_root)
            .finish()
    }
}
