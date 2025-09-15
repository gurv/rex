use std::io::BufWriter;

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use tracing::{Level, error, instrument, trace, warn};
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};

use rex::cli::{CliCommand, CliContext, CommandOutput, OutputKind};

#[derive(Debug, Clone, Parser)]
#[clap(
    name = "rex",
    about,
    version,
    arg_required_else_help = true,
    disable_help_subcommand = true,
    subcommand_required = true,
    subcommand_value_name = "COMMAND|PLUGIN",
    color = clap::ColorChoice::Auto
)]
struct Cli {
    #[clap(
        short = 'o',
        long = "output",
        default_value = "text",
        help = "Specify output format (text or json)",
        global = true
    )]
    pub(crate) output: OutputKind,

    #[clap(
        long = "help-markdown",
        help = "Print help in markdown format (conflicts with --help and --output json)",
        hide = true,
        global = true
    )]
    help_markdown: bool,

    #[clap(
        short = 'l',
        long = "log-level",
        default_value = "info",
        help = "Set the log level (trace, debug, info, warn, error)",
        global = true
    )]
    log_level: Level,

    #[clap(long = "verbose", help = "Enable verbose output", global = true)]
    verbose: bool,

    #[clap(subcommand)]
    command: Option<RexCliCommand>,
}

/// The main CLI commands for rex
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Subcommand)]
enum RexCliCommand {
    /// View configuration for rex
    #[clap(name = "config", subcommand)]
    Config(rex::cli::config::ConfigCommand),
    /// Push or pull Wasm components to/from an OCI registry
    #[clap(name = "oci", alias = "docker", subcommand)]
    Oci(rex::cli::oci::OciCommand),
    /// Manage rex plugins
    #[clap(name = "plugin", subcommand)]
    Plugin(rex::cli::plugin::PluginCommand),
}

impl CliCommand for RexCliCommand {
    /// Handle the rex command
    #[instrument(level = "debug", skip_all, name = "rex")]
    async fn handle(&self, ctx: &CliContext) -> anyhow::Result<CommandOutput> {
        match self {
            RexCliCommand::Config(cmd) => cmd.handle(ctx).await,
            RexCliCommand::Oci(cmd) => cmd.handle(ctx).await,
            RexCliCommand::Plugin(cmd) => cmd.handle(ctx).await,
        }
    }
    fn enable_pre_hook(
        &self,
    ) -> Option<rex::runtime::bindings::plugin::exports::vg::rex::plugin::HookType> {
        match self {
            RexCliCommand::Config(cmd) => cmd.enable_pre_hook(),
            RexCliCommand::Oci(cmd) => cmd.enable_pre_hook(),
            RexCliCommand::Plugin(cmd) => cmd.enable_pre_hook(),
        }
    }

    fn enable_post_hook(
        &self,
    ) -> Option<rex::runtime::bindings::plugin::exports::vg::rex::plugin::HookType> {
        match self {
            RexCliCommand::Config(cmd) => cmd.enable_post_hook(),
            RexCliCommand::Oci(cmd) => cmd.enable_post_hook(),
            RexCliCommand::Plugin(cmd) => cmd.enable_post_hook(),
        }
    }
}

#[tokio::main]
async fn main() {
    let rex_cmd = Cli::command();

    // Create global context with output kind and directory paths
    let ctx = match CliContext::new().await {
        Ok(ctx) => ctx,
        Err(e) => {
            error!(error = ?e, "failed to infer global context");
            // In the rare case that this fails, we'll parse and initialize the CLI here to output properly.
            let cli = Cli::parse();
            let (mut stdout, _stderr) = initialize_tracing(cli.log_level, cli.verbose);
            exit_with_output(
                &mut stdout,
                CommandOutput::error(e, None).with_output_kind(cli.output),
            );
        }
    };
    trace!(ctx = ?ctx, "inferred global context");

    let matches = rex_cmd.get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());

    trace!(cli = ?cli, "parsed CLI");

    // Implements clap_markdown for markdown generation of command line documentation. Most straightforward way to invoke is probably `rex app get --help-markdown > help.md`
    if cli.help_markdown {
        clap_markdown::print_help_markdown::<Cli>();
        std::process::exit(0);
    };

    // Initialize tracing with the specified log level
    let (stdout, _stderr) = initialize_tracing(cli.log_level, cli.verbose);
    // Use a buffered writer to prevent broken pipe errors
    let mut stdout_buf = BufWriter::new(stdout);

    // Since some interactive commands may hide the cursor, we need to ensure it is shown again on exit
    if let Err(e) = ctrlc::set_handler(move || {
        let term = dialoguer::console::Term::stdout();
        let _ = term.show_cursor();
        // TODO(IMPORTANT): If the runtime is executing a component here, we need to stop it.
    }) {
        warn!(err = ?e, "failed to set ctrl_c handler, interactive prompts may not restore cursor visibility");
    }

    let command_output = if let Some(command) = cli.command {
        run_command(ctx, command).await
    } else {
        Ok(CommandOutput::error(
            "No command provided. Use `rex --help` to see available commands.",
            None,
        ))
    };

    exit_with_output(
        &mut stdout_buf,
        command_output
            .unwrap_or_else(|e| CommandOutput::error(e, None).with_output_kind(cli.output))
            .with_output_kind(cli.output),
    )
}

/// Helper function to execute a command that impl's [`CliCommand`], returning the output
async fn run_command<C>(ctx: CliContext, command: C) -> anyhow::Result<CommandOutput>
where
    C: CliCommand + std::fmt::Debug,
{
    trace!(command = ?command, "handling command");
    let command_output = command.handle(&ctx).await;

    command_output
}

/// Initialize tracing with a custom format
///
/// Returns a tuple of stdout and stderr writers for consistency with the previous API.
fn initialize_tracing(
    log_level: Level,
    verbose: bool,
) -> (Box<dyn std::io::Write>, Box<dyn std::io::Write>) {
    // Display logs in a compact, CLI-friendly format
    if verbose {
        // Enable dynamic filtering from `RUST_LOG`, fallback to "info"
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(log_level.as_str()));

        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_level(true)
            .with_file(true)
            .with_line_number(true)
            .with_ansi(true); // Color output for TTY

        // Register all layers with the subscriber
        Registry::default().with(env_filter).with(fmt_layer).init();

        (Box::new(std::io::stdout()), Box::new(std::io::stderr()))
    } else {
        // Enable dynamic filtering from `RUST_LOG`, fallback to "info", but always set wasm_pkg_client=error
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(log_level.as_str()))
            // wasm_pkg_client/core are a little verbose so we set them to error level in non-verbose mode
            .add_directive(
                "wasm_pkg_client=error"
                    .parse()
                    .expect("failed to parse wasm_pkg_client directive"),
            )
            .add_directive(
                "wasm_pkg_core=error"
                    .parse()
                    .expect("failed to parse wasm_pkg_core directive"),
            );

        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr)
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_level(true)
            .with_file(false)
            .with_line_number(false)
            .with_ansi(true);

        // Register all layers with the subscriber
        Registry::default().with(env_filter).with(fmt_layer).init();

        (Box::new(std::io::stdout()), Box::new(std::io::stderr()))
    }
}

/// Helper function to ensure that we're exiting the program consistently and with the correct output format.
fn exit_with_output(stdout: &mut impl std::io::Write, output: CommandOutput) -> ! {
    let (message, success) = output.render();
    writeln!(stdout, "{message}").expect("failed to write output to stdout");
    stdout.flush().expect("failed to flush stdout");
    if success {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
