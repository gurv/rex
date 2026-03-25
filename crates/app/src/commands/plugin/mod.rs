pub mod add;
pub mod info;

use clap::Subcommand;

#[derive(Clone, Debug, Subcommand)]
pub enum PluginCommands {
    #[command(name = "add", about = "Add and configure plugin.")]
    Add(add::PluginAddArgs),

    #[command(
        name = "info",
        about = "Show detailed information about plugin."
    )]
    Info(info::PluginInfoArgs),
}
