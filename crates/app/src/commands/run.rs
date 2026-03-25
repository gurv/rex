use crate::session::RexSession;
use clap::Args;
use rex_common::Id;
use starbase::AppResult;
use tracing::instrument;

#[derive(Args, Clone, Debug)]
pub struct RunArgs {
    #[arg(required = true, help = "Plugin ID to execute")]
    id: Id,

    #[arg(last = true, help = "Arguments to pass through to the plugin")]
    passthrough: Vec<String>,
}

#[instrument(skip(session))]
pub async fn run(session: RexSession, args: RunArgs) -> AppResult {
    let extension_registry = session.get_extension_registry().await?;

    extension_registry
        .load(&args.id)
        .await?
        .execute(args.passthrough, extension_registry.create_context())
        .await?;

    Ok(None)
}
