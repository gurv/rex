use crate::commands::{
    McpArgs,
    debug::{DebugConfigArgs, DebugEnvArgs},
    plugin::{AddPluginArgs, InfoPluginArgs, ListPluginsArgs, RemovePluginArgs, SearchPluginArgs},
    xxx::{XxxArgs},
};
use clap::builder::styling::{Color, Style, Styles};
use clap::{Parser, Subcommand, ValueEnum};
use rex_old_core::ConfigMode;
use starbase_styles::color::Color as ColorType;
use std::{
    env,
    fmt::{Display, Error, Formatter},
    path::PathBuf,
};

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum AppTheme {
    #[default]
    Dark,
    Light,
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
    Verbose,
}

impl LogLevel {
    pub fn is_verbose(&self) -> bool {
        matches!(self, Self::Verbose)
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{}",
            match self {
                LogLevel::Off => "off",
                LogLevel::Error => "error",
                LogLevel::Warn => "warn",
                LogLevel::Info => "info",
                LogLevel::Debug => "debug",
                // Must map to tracing levels
                LogLevel::Trace | LogLevel::Verbose => "trace",
            }
        )?;

        Ok(())
    }
}

fn fg(ty: ColorType) -> Style {
    Style::new().fg_color(Some(Color::from(ty as u8)))
}

fn create_styles() -> Styles {
    Styles::default()
        .error(fg(ColorType::Red))
        .header(Style::new().bold())
        .invalid(fg(ColorType::Yellow))
        .literal(fg(ColorType::Pink)) // args, options, etc
        .placeholder(fg(ColorType::GrayLight))
        .usage(fg(ColorType::Purple).bold())
        .valid(fg(ColorType::Green))
}

#[derive(Clone, Debug, Parser)]
#[command(
    name = "rex",
    version,
    about,
    long_about = None,
    disable_help_subcommand = true,
    propagate_version = true,
    next_line_help = false,
    styles = create_styles()
)]
pub struct App {
    #[arg(
        value_enum,
        long,
        short = 'c',
        global = true,
        env = "REX_CONFIG_MODE",
        help = "Mode in which to load configuration"
    )]
    pub config_mode: Option<ConfigMode>,

    #[arg(
        long,
        global = true,
        env = "REX_DUMP",
        help = "Dump a trace profile to the working directory"
    )]
    pub dump: bool,

    #[arg(
        value_enum,
        default_value_t,
        long,
        global = true,
        env = "REX_LOG",
        help = "Lowest log level to output"
    )]
    pub log: LogLevel,

    #[arg(
        long,
        global = true,
        env = "REX_LOG_FILE",
        help = "Path to a file to write logs to"
    )]
    pub log_file: Option<PathBuf>,

    #[arg(
        long,
        global = true,
        env = "REX_JSON",
        help = "Print as JSON (when applicable)"
    )]
    pub json: bool,

    #[arg(
        value_enum,
        default_value_t,
        long,
        global = true,
        env = "REX_THEME",
        help = "Terminal theme to print with"
    )]
    pub theme: AppTheme,

    #[arg(
        long,
        short = 'y',
        global = true,
        env = "REX_YES",
        help = "Avoid all interactive prompts and use defaults"
    )]
    pub yes: bool,

    #[command(subcommand)]
    pub command: Commands,
}

impl App {
    pub fn setup_env_vars(&self) {
        unsafe {
            env::set_var("REX_APP_LOG", self.log.to_string());
            env::set_var("REX_VERSION", env!("CARGO_PKG_VERSION"));

            if let Ok(value) = env::var("REX_DEBUG_COMMAND") {
                env::set_var("WARPGATE_DEBUG_COMMAND", value);
            }

            env::set_var(
                "STARBASE_THEME",
                match self.theme {
                    AppTheme::Dark => "dark",
                    AppTheme::Light => "light",
                },
            );

            // Disable ANSI colors in JSON output
            if self.json {
                env::set_var("NO_COLOR", "1");
                env::remove_var("FORCE_COLOR");
            }
        }
    }
}

#[derive(Clone, Debug, Subcommand)]
pub enum Commands {
    #[command(name = "debug", about = "Debug the current rex environment.")]
    Debug {
        #[command(subcommand)]
        command: DebugCommands,
    },

    #[command(
        name = "mcp",
        about = "Start an MCP server to handle tool, resource, and prompt requests for AI agents."
    )]
    Mcp(McpArgs),

    #[command(
        alias = "tool", // Deprecated
        name = "plugin",
        about = "Operations for managing tool plugins."
    )]
    Plugin {
        #[command(subcommand)]
        command: PluginCommands,
    },

    #[command(name = "xxx", about = "xxx.")]
    Xxx(XxxArgs),
}

#[derive(Clone, Debug, Subcommand)]
pub enum DebugCommands {
    #[command(
        name = "config",
        about = "Debug all loaded .rextools config's for the current directory."
    )]
    Config(DebugConfigArgs),

    #[command(name = "env", about = "Debug the current rex environment and store.")]
    Env(DebugEnvArgs),
}

#[derive(Clone, Debug, Subcommand)]
pub enum PluginCommands {
    #[command(
        name = "add",
        about = "Add a plugin to manage a tool.",
        long_about = "Add a plugin to a .rextools config file to enable and manage that tool."
    )]
    Add(AddPluginArgs),

    #[command(
        name = "info",
        about = "Display information about an installed plugin and its inventory."
    )]
    Info(InfoPluginArgs),

    #[command(
        name = "list",
        about = "List all configured and built-in plugins, and optionally include inventory."
    )]
    List(ListPluginsArgs),

    #[command(
        name = "remove",
        about = "Remove a plugin and unmanage a tool.",
        long_about = "Remove a plugin from a .rextools config file and unmanage that tool."
    )]
    Remove(RemovePluginArgs),

    #[command(
        name = "search",
        about = "Search for available plugins provided by the community."
    )]
    Search(SearchPluginArgs),
}
