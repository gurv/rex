use std::path::PathBuf;

use crate::lib::cli::{CommandOutput, OutputKind};
use clap::{Parser, Subcommand};

#[derive(Debug, Clone, Subcommand)]
pub enum PluginCommand {
    /// Install a rex plugin
    #[clap(name = "install")]
    Install(PluginInstallCommand),
    /// Uninstall a plugin
    #[clap(name = "uninstall", alias = "delete", alias = "rm")]
    Uninstall(PluginUninstallCommand),
    /// List installed plugins
    #[clap(name = "list", alias = "ls")]
    List(PluginListCommand),
}

#[derive(Debug, Clone, Parser)]
pub struct PluginCommonOpts {
    /// Path to plugin directory. Defaults to $HOME/.rex/plugins.
    #[clap(long = "plugin-dir", env = "REX_PLUGIN_DIR")]
    pub plugin_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Parser)]
pub struct PluginInstallCommand {
    /// URL of the plugin to install. Can be a file:// URL.
    #[clap(name = "url")]
    pub url: String,

    /// Whether or not to update the plugin if it is already installed. Defaults to false
    #[clap(long = "update")]
    pub update: bool,

    #[clap(flatten)]
    pub opts: PluginCommonOpts,
}

#[derive(Debug, Clone, Parser)]
pub struct PluginUninstallCommand {
    /// ID of the plugin to uninstall
    #[clap(name = "id")]
    pub plugin: String,

    #[clap(flatten)]
    pub opts: PluginCommonOpts,
}

#[derive(Debug, Clone, Parser)]
pub struct PluginListCommand {
    #[clap(flatten)]
    pub opts: PluginCommonOpts,
}

pub async fn handle_command(
    cmd: PluginCommand,
    output_kind: OutputKind,
) -> anyhow::Result<CommandOutput> {
    match cmd {
        PluginCommand::Install(cmd) => handle_install(cmd, output_kind).await,
        PluginCommand::Uninstall(cmd) => handle_uninstall(cmd, output_kind).await,
        PluginCommand::List(cmd) => handle_list(cmd, output_kind).await,
    }
}

#[allow(unused_variables)]
pub async fn handle_install(
    cmd: PluginInstallCommand,
    output_kind: OutputKind,
) -> anyhow::Result<CommandOutput> {
    Ok(CommandOutput {
        text: "TODO".to_string(),
        map: [].into(),
    })
}

#[allow(unused_variables)]
pub async fn handle_uninstall(
    cmd: PluginUninstallCommand,
    output_kind: OutputKind,
) -> anyhow::Result<CommandOutput> {
    Ok(CommandOutput {
        text: "TODO".to_string(),
        map: [].into(),
    })
}

#[allow(unused_variables)]
pub async fn handle_list(
    cmd: PluginListCommand,
    output_kind: OutputKind,
) -> anyhow::Result<CommandOutput> {
    Ok(CommandOutput {
        text: "TODO".to_string(),
        map: [].into(),
    })
}
