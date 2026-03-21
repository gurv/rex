use crate::session::RexSession;
use clap::Args;
// use moon_common::Id;
use rex_old_core::Id;
use starbase::AppResult;
use tracing::instrument;

#[derive(Args, Clone, Debug)]
pub struct XxxArgs {
    #[arg(required = true, help = "Action ID to execute")]
    id: Id,

    // Passthrough args (after --)
    #[arg(last = true, help = "Arguments to pass through to the action")]
    passthrough: Vec<String>,
}

#[instrument(skip(_session))]
pub async fn xxx(_session: RexSession, args: XxxArgs) -> AppResult {
    // let extension_registry = session.get_extension_registry().await?;

    // extension_registry
    //     .load(&args.id)
    //     .await?
    //     .execute(args.passthrough, extension_registry.create_context())
    //     .await?;

    Ok(None)
}
