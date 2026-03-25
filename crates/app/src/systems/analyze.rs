use crate::app_error::AppError;
use semver::{Version, VersionReq};
use starbase::AppResult;
use tracing::instrument;

// #[instrument(skip_all)]
// pub async fn extract_repo_info(vcs: &BoxedVcs) -> miette::Result<()> {
//     let bag = GlobalEnvBag::instance();

//     if vcs.is_enabled()
//         && !bag.has("REX_VCS_REPO_SLUG")
//         && let Ok(slug) = vcs.get_repository_slug().await
//     {
//         bag.set("REX_VCS_REPO_SLUG", slug.as_str());
//     }

//     Ok(())
// }

#[instrument]
pub fn validate_version_constraint(constraint: &VersionReq, version: &Version) -> AppResult {
    if !constraint.matches(version) {
        return Err(AppError::InvalidRexVersion {
            actual: version.to_string(),
            expected: constraint.to_string(),
        }
        .into());
    }

    Ok(None)
}
