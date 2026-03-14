use std::collections::HashMap;
use std::io::{BufWriter, Write, stdout};

use clap::{self, FromArgMatches, Parser, Subcommand};
use rex::cli::plugin::{self, PluginCommand};
use rex::lib::cli::{CommandOutput, OutputKind};
use serde_json::json;

fn version(output: OutputKind) -> String {
    match output {
        OutputKind::Text => format!("rex v{}", clap::crate_version!(),),
        OutputKind::Json => {
            let versions = serde_json::json!({
                "rex": format!("v{}", clap::crate_version!()),
            });
            serde_json::to_string_pretty(&versions).unwrap()
        }
    }
}

#[derive(Debug, Clone, Parser)]
#[clap(name = "rex", disable_version_flag = true)]
#[command(version, about, long_about = None)]
struct Cli {
    #[clap(
        short = 'o',
        long = "output",
        default_value = "text",
        help = "Specify output format (text or json)",
        global = true
    )]
    pub(crate) output: OutputKind,

    #[clap(short = 'V', long = "version", help = "Print version")]
    version: bool,

    #[clap(subcommand)]
    command: Option<CliCommand>,
}

#[derive(Debug, Clone, Subcommand)]
enum CliCommand {
    /// Manage rex plugins
    #[clap(name = "plugin", subcommand)]
    Plugin(PluginCommand),
}

#[tokio::main]
async fn main() {
    use clap::CommandFactory;
    let mut command = Cli::command();
    command.build();
    let matches = command.get_matches_mut();
    let cli = Cli::from_arg_matches(&matches).unwrap();
    let output_kind = cli.output;
    if cli.version {
        println!("{}", version(cli.output));
        std::process::exit(0);
    }
    let cli_command = cli.command.unwrap_or_else(|| {
        eprintln!("{}", command.render_help());
        std::process::exit(2);
    });
    let res: anyhow::Result<CommandOutput> = match cli_command {
        CliCommand::Plugin(plugin_cli) => plugin::handle_command(plugin_cli, output_kind).await,
    };
    let mut stdout_buf = BufWriter::new(stdout().lock());
    let exit_code: i32 = match res {
        Ok(out) => match output_kind {
            OutputKind::Json => {
                let map = out.map;
                let _ = writeln!(
                    stdout_buf,
                    "\n{}",
                    serde_json::to_string_pretty(&map).unwrap()
                );
                0
            }
            OutputKind::Text => {
                let _ = writeln!(stdout_buf, "\n{}", out.text);
                0
            }
        },
        Err(e) => {
            match output_kind {
                OutputKind::Json => {
                    let mut map = HashMap::new();
                    map.insert("success".to_string(), json!(false));
                    map.insert("error".to_string(), json!(e.to_string()));

                    let error_chain = e
                        .chain()
                        .skip(1)
                        .map(|e| format!("{e}"))
                        .collect::<Vec<String>>();

                    if !error_chain.is_empty() {
                        map.insert("error_chain".to_string(), json!(error_chain));
                    }

                    let backtrace = e.backtrace().to_string();

                    if !backtrace.is_empty() && backtrace != "disabled backtrace" {
                        map.insert("backtrace".to_string(), json!(backtrace));
                    }

                    eprintln!("\n{}", serde_json::to_string_pretty(&map).unwrap());
                }
                OutputKind::Text => {
                    eprintln!("\n{e:?}");
                }
            }
            1
        }
    };
    let _ = stdout_buf.flush();
    std::process::exit(exit_code);
}
