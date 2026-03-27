use crate::app_options::*;
use crate::commands::run::RunArgs;
use crate::commands::plugin::PluginCommands;
use crate::systems::bootstrap;
use clap::builder::styling::{Color, Style, Styles};
use clap::{Parser, Subcommand};
use rex_env_var::GlobalEnvBag;
use starbase_styles::color::Color as ColorType;
use std::env;
use std::path::PathBuf;

#[cfg(windows)]
pub const EXE_NAME: &str = "rex.exe";

#[cfg(not(windows))]
pub const EXE_NAME: &str = "rex";

#[derive(Clone, Debug, Subcommand)]
pub enum Commands {
    #[command(name = "run", about = "Execute plugin.")]
    Run(RunArgs),

    #[command(name = "plugin", about = "Manage plugins.")]
    Plugin {
        #[command(subcommand)]
        command: PluginCommands,
    },

    // #[command(
    //     alias = "g",
    //     name = "generate",
    //     about = "Generate and scaffold files from a pre-defined template."
    // )]
    // Generate(GenerateArgs),

    // #[command(
    //     name = "mcp",
    //     about = "Start an MCP (model context protocol) server that can respond to AI agent requests."
    // )]
    // Mcp(McpArgs),

    // #[command(name = "template", about = "Manage templates.")]
    // Template {
    //     #[command(subcommand)]
    //     command: TemplateCommands,
    // },

    // #[command(
    //     alias = "up",
    //     name = "upgrade",
    //     about = "Upgrade to the latest version of rex."
    // )]
    // Upgrade,
}

fn fg(ty: ColorType) -> Style {
    Style::new().fg_color(Some(Color::from(ty as u8)))
}

fn create_styles() -> Styles {
    Styles::default()
        .error(fg(ColorType::Red))
        .header(Style::new().bold())
        .invalid(fg(ColorType::Yellow))
        .literal(fg(ColorType::Purple)) // args, options, etc
        .placeholder(fg(ColorType::GrayLight))
        .usage(fg(ColorType::Pink).bold())
        .valid(fg(ColorType::Green))
}

#[derive(Clone, Debug, Parser)]
#[command(
    bin_name = EXE_NAME,
    name = "rex",
    about = "This is the rex CLI!",
    version = env::var("REX_VERSION").unwrap_or_default(),
    disable_help_subcommand = true,
    next_line_help = false,
    propagate_version = true,
    styles = create_styles()
)]
pub struct Cli {
    #[arg(
        long,
        global = true,
        env = "REX_COLOR",
        help = "Force colored output",
        help_heading = "Global options"
    )]
    pub color: bool,

    #[arg(
        long,
        global = true,
        env = "REX_DUMP",
        help = "Dump a trace profile to the working directory",
        help_heading = "Global options"
    )]
    pub dump: bool,

    #[arg(
        value_enum,
        long,
        global = true,
        env = "REX_LOG",
        help = "Lowest log level to output",
        help_heading = "Global options",
        default_value_t
    )]
    pub log: LogLevel,

    #[arg(
        long,
        global = true,
        env = "REX_LOG_FILE",
        help = "Path to a file to write logs to",
        help_heading = "Global options"
    )]
    pub log_file: Option<PathBuf>,

    #[arg(
        long,
        short = 'q',
        global = true,
        env = "REX_QUIET",
        help = "Hide all rex console output",
        help_heading = "Global options"
    )]
    pub quiet: bool,

    #[arg(
        value_enum,
        long,
        global = true,
        env = "REX_THEME",
        help = "Terminal theme to print with",
        help_heading = "Global options",
        default_value_t
    )]
    pub theme: AppTheme,

    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn setup_env_vars(&self) {
        bootstrap::setup_colors(self.color);

        let bag = GlobalEnvBag::instance();
        bag.set("STARBASE_LOG", self.log.to_string());
        bag.set("STARBASE_THEME", self.theme.to_string());

        if !bag.has("REX_LOG") {
            bag.set("REX_LOG", self.log.to_string());
        }

        if !bag.has("REX_THEME") {
            bag.set("REX_THEME", self.theme.to_string());
        }

        if bag.should_debug_wasm() {
            bag.set("REX_WASM_LOG", "trace");
            bag.set("REX_DEBUG_WASM", "true");
            bag.set("EXTISM_DEBUG", "1");
            bag.set("EXTISM_ENABLE_WASI_OUTPUT", "1");
            bag.set("EXTISM_MEMDUMP", "wasm-plugin.mem");
            bag.set("EXTISM_COREDUMP", "wasm-plugin.core");
        }
    }
}
