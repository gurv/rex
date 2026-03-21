mod app;
mod commands;
mod components;
mod error;
mod helpers;
mod mcp;
mod session;
mod systems;
mod utils;

use app::{App as CLI, Commands, DebugCommands, PluginCommands};
use clap::Parser;
pub use session::RexSession;
use starbase::{
    App, MainResult,
    tracing::{LogLevel, TracingOptions},
};
use starbase_utils::{envx, string_vec};
use std::env;
use std::process::ExitCode;
use tracing::debug;

fn get_tracing_modules() -> Vec<String> {
    let mut modules = string_vec!["rex", "schematic", "starbase", "warpgate"];

    if envx::bool_var("REX_DEBUG_WASM") || envx::bool_var("REX_WASM_LOG") {
        modules.push("extism".into());
    } else {
        modules.push("extism::pdk".into());
    }

    modules
}

#[tokio::main]
async fn main() -> MainResult {
    sigpipe::reset();

    let cli = CLI::parse();
    cli.setup_env_vars();

    let app = App::default();
    app.setup_diagnostics();

    let _guard = app.setup_tracing(TracingOptions {
        default_level: LogLevel::Info,
        dump_trace: cli.dump,
        filter_modules: get_tracing_modules(),
        log_env: "REX_APP_LOG".into(),
        log_file: cli.log_file.clone(),
        show_spans: cli.log.is_verbose(),
        // test_env: "REX_TEST".into(),
        ..TracingOptions::default()
    });

    let session = RexSession::new(cli);
    let mut args = env::args_os().collect::<Vec<_>>();

    debug!(
        exe = ?args.remove(0),
        args = ?args,
        shim = env::var("REX_SHIM_NAME").ok(),
        shim_exe = env::var("REX_SHIM_PATH").ok(),
        pid = std::process::id(),
        "Running rex v{}",
        session.cli_version
    );

    let exit_code = app
        .run(session, |session| async {
            match session.cli.command.clone() {
                Commands::Debug { command } => match command {
                    DebugCommands::Config(args) => commands::debug::config(session, args).await,
                    DebugCommands::Env(args) => commands::debug::env(session, args).await,
                },
                Commands::Mcp(args) => commands::mcp(session, args).await,
                Commands::Plugin { command } => match command {
                    PluginCommands::Add(args) => commands::plugin::add(session, args).await,
                    PluginCommands::Info(args) => commands::plugin::info(session, args).await,
                    PluginCommands::List(args) => commands::plugin::list(session, args).await,
                    PluginCommands::Remove(args) => commands::plugin::remove(session, args).await,
                    PluginCommands::Search(args) => commands::plugin::search(session, args).await,
                },
                Commands::Xxx(args) => commands::xxx(session, args).await,
            }
        })
        .await?;

    Ok(ExitCode::from(exit_code))
}
