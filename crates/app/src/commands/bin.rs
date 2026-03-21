use crate::session::RexSession;
use clap::Args;
use rex_common::Id;
use rex_env_var::GlobalEnvBag;
use rex_process_augment::AugmentedCommand;
use starbase::AppResult;
use tracing::instrument;

#[derive(Args, Clone, Debug)]
pub struct BinArgs {
    #[arg(required = true, help = "The toolchain to query")]
    toolchain: Id,
}

#[instrument(skip(session))]
pub async fn bin(session: RexSession, args: BinArgs) -> AppResult {
    session.console.quiet();

    // let app_context = session.get_app_context().await?;

    // let mut command = AugmentedCommand::new(&app_context, GlobalEnvBag::instance(), "proto");
    // command.arg("bin").arg(args.toolchain.as_str());
    // command.inherit_from_plugins(None, None).await?;
    // command.inherit_proto();

    // let result = command.exec_stream_output().await?;

    // if !result.success() {
    //     return Ok(Some(result.code().unwrap_or(1) as u8));
    // }

    Ok(None)
}
