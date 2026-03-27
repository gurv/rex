use rex_config::{ExtensionsConfig, Version};
use rex_console::Console;
use rex_env::RexEnvironment;
use rex_extension_plugin::ExtensionRegistry;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AppContext {
    pub cli_version: Version,
    pub rex_env: Arc<RexEnvironment>,
    pub console: Arc<Console>,
    pub extensions_config: Arc<ExtensionsConfig>,
    pub extension_registry: Arc<ExtensionRegistry>,
    pub config_dir: PathBuf,
    pub working_dir: PathBuf,
    pub workspace_root: PathBuf,
}
