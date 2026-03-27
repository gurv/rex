use crate::app_error::AppError;
use rex_env::RexEnvironment;
use rex_env_var::GlobalEnvBag;
use starbase_styles::color;
use starbase_utils::dirs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, instrument};

/// Recursively attempt to find the workspace root by locating the ".rex"
/// configuration folder, starting from the current working directory.
#[instrument]
pub fn find_workspace_root(working_dir: &Path) -> miette::Result<PathBuf> {
    debug!(
        working_dir = ?working_dir,
        "Attempting to find workspace root from current working directory",
    );

    let workspace_root = if let Some(root) = GlobalEnvBag::instance().get("REX_WORKSPACE_ROOT") {
        debug!(
            env_var = root,
            "Inheriting from {} environment variable",
            color::symbol("REX_WORKSPACE_ROOT")
        );

        let root: PathBuf = root
            .parse()
            .map_err(|_| AppError::InvalidWorkspaceRootEnvVar)?;

        if !root.join(".rex").exists() && !root.join(".config").join("rex").exists() {
            return Err(AppError::MissingConfigDir.into());
        }

        root
    } else {
        let mut current_dir = Some(working_dir);

        loop {
            if let Some(dir) = current_dir {
                if dir.join(".rex").exists() || dir.join(".config").join("rex").exists() {
                    break dir.to_path_buf();
                } else {
                    current_dir = dir.parent();
                }
            } else {
                return Err(AppError::MissingConfigDir.into());
            }
        }
    };

    // Avoid finding the ~/.rex directory
    let home_dir = dirs::home_dir().ok_or(AppError::MissingHomeDir)?;

    if home_dir == workspace_root {
        return Err(AppError::MissingConfigDir.into());
    }

    debug!(
        workspace_root = ?workspace_root,
        working_dir = ?working_dir,
        "Found workspace root",
    );

    Ok(workspace_root)
}

/// Detect information for rex from the environment.
#[instrument]
pub fn detect_rex_environment(
    working_dir: &Path,
    workspace_root: &Path,
) -> miette::Result<Arc<RexEnvironment>> {
    let mut env = RexEnvironment::new()?;
    env.working_dir = working_dir.to_path_buf();
    env.workspace_root = workspace_root.to_path_buf();

    Ok(Arc::new(env))
}
