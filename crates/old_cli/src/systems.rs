#![allow(unused_imports)]

use crate::app::{App as CLI};
use rex_old_core::{ConfigMode, RexEnvironment, get_exe_file_name};
use starbase_utils::fs;
use std::env;
use tracing::{debug, instrument};

// STARTUP

#[instrument(skip_all)]
pub fn detect_rex_env(_cli: &CLI) -> miette::Result<RexEnvironment> {
    #[cfg(debug_assertions)]
    let mut env = if let Ok(sandbox) = env::var("REX_SANDBOX") {
        RexEnvironment::new_testing(&std::path::PathBuf::from(&sandbox))?
    } else {
        RexEnvironment::new()?
    };

    #[cfg(not(debug_assertions))]
    let mut env = RexEnvironment::new()?;

    env.config_mode = ConfigMode::UpwardsGlobal;

    Ok(env)
}

// ANALYZE

#[instrument(skip_all)]
pub fn load_rex_configs(env: &RexEnvironment) -> miette::Result<()> {
    debug!(
        working_dir = ?env.working_dir,
        "Loading configuration in {} mode",
        env.config_mode.to_string()
    );

    env.load_config()?;

    Ok(())
}

// EXECUTE

#[instrument(skip_all)]
pub fn clean_rex_backups(env: &RexEnvironment) -> miette::Result<()> {
    for bin_name in [get_exe_file_name("rex"), get_exe_file_name("rex-shim")] {
        let backup_path = env.store.bin_dir.join(format!("{bin_name}.backup"));

        if backup_path.exists() {
            let _ = fs::remove_file(backup_path);
        }
    }

    Ok(())
}
