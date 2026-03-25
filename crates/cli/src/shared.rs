use crate::lookup::*;
use clap::Parser;
use mimalloc::MiMalloc;
use rex_app::commands::plugin::PluginCommands;
use rex_app::{Cli, Commands, RexSession, commands};
use rex_env_var::GlobalEnvBag;
use starbase::diagnostics::IntoDiagnostic;
use starbase::tracing::TracingOptions;
use starbase::{App, MainResult};
use starbase_styles::color;
use starbase_utils::{dirs, string_vec};
use std::env;
use std::ffi::OsString;
use std::process::{Command, ExitCode};
use tracing::debug;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn get_version() -> String {
    let version = env!("CARGO_PKG_VERSION");

    GlobalEnvBag::instance().set("REX_VERSION", version);

    version.to_owned()
}

fn get_tracing_modules() -> Vec<String> {
    let bag = GlobalEnvBag::instance();
    let mut modules = string_vec![
        "rex",
        "proto",
        "starbase",
        "rex_warpgate",
    ];

    if bag.should_debug_mcp() {
        modules.push("rust_mcp_sdk".into());
        modules.push("rust_mcp_transport".into());
    }

    if bag.should_debug_wasm() {
        modules.push("extism".into());
    } else {
        modules.push("extism::pdk".into());
    }

    if bag.should_debug_remote() {
        modules.push("tonic".into());
    }

    modules
}

#[cfg(unix)]
fn exec_local_bin(mut command: Command) -> std::io::Result<u8> {
    use std::os::unix::process::CommandExt;

    Err(command.exec())
}

#[cfg(windows)]
fn exec_local_bin(mut command: Command) -> std::io::Result<u8> {
    let result = command.spawn()?.wait()?;

    if !result.success() {
        return Ok(result.code().unwrap_or(1) as u8);
    }

    Ok(0)
}

pub async fn run_cli(args: Vec<OsString>) -> MainResult {
    sigpipe::reset();

    let version = get_version();

    let cli = Cli::parse_from(&args);
    cli.setup_env_vars();

    let app = App::default();
    app.setup_diagnostics();

    let _guard = app.setup_tracing(TracingOptions {
        dump_trace: cli.dump,
        filter_modules: get_tracing_modules(),
        log_env: "STARBASE_LOG".into(),
        log_file: cli.log_file.clone(),
        show_spans: cli.log.is_verbose(),
        ..TracingOptions::default()
    });

    if let Ok(exe) = env::current_exe() {
        debug!(
            args = ?args,
            "Running rex v{} (with {})",
            version,
            color::path(exe),
        );
    } else {
        debug!(args = ?args, "Running rex v{}", version);
    }

    if let (Some(home_dir), Ok(current_dir)) = (dirs::home_dir(), env::current_dir())
        && is_globally_installed(&home_dir)
        && let Some(local_bin) = has_locally_installed(&home_dir, &current_dir)
    {
        debug!(local = ?local_bin, "Binary is running from a global path, but we found a local binary to use instead");
        debug!("Will now execute the local binary and replace this running process");

        let mut command = Command::new(local_bin);
        command.args(&args);
        command.current_dir(current_dir);

        let exit_code = exec_local_bin(command).into_diagnostic()?;

        return Ok(ExitCode::from(exit_code));
    }

    let exit_code = app
        .run(RexSession::new(cli, version), |session| async {
            match session.cli.command.clone() {
                Commands::Run(args) => commands::run::run(session, args).await,
                Commands::Plugin { command } => match command {
                    PluginCommands::Add(args) => {
                        commands::plugin::add::add(session, args).await
                    }
                    PluginCommands::Info(args) => {
                        commands::plugin::info::info(session, args).await
                    }
                },
            }
        })
        .await?;

    Ok(ExitCode::from(exit_code))
}
