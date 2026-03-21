use crate::config::*;
use crate::config_error::RexConfigError;
use crate::id::Id;
use crate::lockfile::*;
use crate::tool_context::ToolContext;
use once_cell::sync::OnceCell;
use schematic::{Config, PartialConfig};
use serde::Serialize;
use starbase_utils::fs;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tracing::debug;

#[derive(Debug, Serialize)]
pub struct RexConfigFile {
    pub exists: bool,
    pub path: PathBuf,
    pub config: PartialRexConfig,
}

#[derive(Debug, Serialize)]
pub struct RexDirEntry {
    pub path: PathBuf,
    pub location: PinLocation,
    pub configs: Vec<RexConfigFile>,
    pub locked: bool,
}

#[derive(Debug)]
pub struct RexFileManager {
    // Paths are sorted from current working directory,
    // up until the root or user directory, whichever is first.
    // The special `~/.rex/.rextools` config is always
    // loaded last, and is the last entry in the list.
    // For directories without a config, we still insert
    // an empty entry. This helps with traversal logic.
    pub entries: Vec<RexDirEntry>,

    all_config: Arc<OnceCell<RexConfig>>,
    all_config_no_global: Arc<OnceCell<RexConfig>>,
    global_config: Arc<OnceCell<RexConfig>>,
    local_config: Arc<OnceCell<RexConfig>>,

    lock: Arc<RwLock<RexLock>>,
    locked: bool,
}

impl RexFileManager {
    pub fn load(
        start_dir: impl AsRef<Path>,
        end_dir: Option<&Path>,
        env_mode: Option<&String>,
    ) -> Result<Self, RexConfigError> {
        let mut current_dir = Some(start_dir.as_ref());
        let mut entries = vec![];
        let mut locked_dirs: Vec<PathBuf> = vec![];
        let mut locked = false;
        let mut lock = RexLock::default();

        while let Some(dir) = current_dir {
            let mut configs = vec![];
            let is_end = end_dir.is_some_and(|end| end == dir);
            let location = if is_end {
                PinLocation::User
            } else {
                PinLocation::Local
            };

            if let Some(env) = env_mode {
                let env_path = dir.join(format!("{REX_CONFIG_NAME}.{env}"));

                configs.push(RexConfigFile {
                    config: RexConfig::load(&env_path, false)?,
                    exists: env_path.exists(),
                    path: env_path,
                });
            }

            let lock_path = dir.join(REX_LOCK_NAME);
            let path = dir.join(REX_CONFIG_NAME);

            configs.push(RexConfigFile {
                config: RexConfig::load(&path, false)?,
                exists: path.exists(),
                path,
            });

            // Only load the lockfile if any of the configs
            // in the current directory are enabled
            let load_lockfile = location == PinLocation::Local
                && configs.iter().any(|file| {
                    file.config
                        .settings
                        .as_ref()
                        .and_then(|settings| settings.lockfile)
                        .unwrap_or(false)
                });

            if load_lockfile {
                if let Some(locked_dir) = locked_dirs
                    .iter()
                    .find(|child_dir| child_dir.starts_with(dir))
                {
                    return Err(RexConfigError::AlreadyLocked {
                        child_dir: locked_dir.into(),
                        parent_dir: dir.into(),
                    });
                } else {
                    locked_dirs.push(dir.to_path_buf());
                }

                lock = RexLock::load_from(dir)?;
                locked = true;
            } else if lock_path.exists() {
                fs::remove_file(lock_path)?;
            }

            entries.push(RexDirEntry {
                path: dir.to_path_buf(),
                location,
                configs,
                locked: load_lockfile,
            });

            if is_end {
                break;
            }

            current_dir = dir.parent();
        }

        Ok(Self {
            entries,
            all_config: Arc::new(OnceCell::new()),
            all_config_no_global: Arc::new(OnceCell::new()),
            global_config: Arc::new(OnceCell::new()),
            local_config: Arc::new(OnceCell::new()),
            locked,
            lock: Arc::new(RwLock::new(lock)),
        })
    }

    pub fn get_lock(&self) -> Option<RwLockReadGuard<'_, RexLock>> {
        self.locked.then(|| self.lock.read().unwrap())
    }

    pub fn get_lock_mut(&self) -> Option<RwLockWriteGuard<'_, RexLock>> {
        self.locked.then(|| self.lock.write().unwrap())
    }

    pub fn get_config_files(&self) -> Vec<&RexConfigFile> {
        self.entries.iter().flat_map(|dir| &dir.configs).collect()
    }

    pub fn get_global_config(&self) -> Result<&RexConfig, RexConfigError> {
        self.global_config.get_or_try_init(|| {
            debug!("Loading global config only");

            self.merge_configs(
                self.entries
                    .iter()
                    .filter_map(|dir| {
                        if dir.location == PinLocation::Global {
                            Some(dir.configs.iter())
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .collect(),
            )
        })
    }

    pub fn get_local_config(&self, cwd: &Path) -> Result<&RexConfig, RexConfigError> {
        self.local_config.get_or_try_init(|| {
            debug!("Loading local config only");

            self.merge_configs(
                self.entries
                    .iter()
                    .filter_map(|dir| {
                        if dir.path == cwd {
                            Some(dir.configs.iter())
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .collect(),
            )
        })
    }

    pub fn get_merged_config(&self) -> Result<&RexConfig, RexConfigError> {
        self.all_config.get_or_try_init(|| {
            debug!("Merging loaded configs with global");

            self.merge_configs(
                self.entries
                    .iter()
                    .flat_map(|dir| dir.configs.iter())
                    .collect(),
            )
        })
    }

    pub fn get_merged_config_without_global(&self) -> Result<&RexConfig, RexConfigError> {
        self.all_config_no_global.get_or_try_init(|| {
            debug!("Merging loaded configs without global");

            self.merge_configs(
                self.entries
                    .iter()
                    .filter_map(|dir| {
                        if dir.location != PinLocation::Global {
                            Some(dir.configs.iter())
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .collect(),
            )
        })
    }

    pub(crate) fn remove_rex_pins(&mut self) {
        let context = ToolContext::new(Id::raw(REX_PLUGIN_KEY));

        self.entries.iter_mut().for_each(|dir| {
            if dir.location != PinLocation::Local {
                dir.configs.iter_mut().for_each(|file| {
                    if let Some(versions) = &mut file.config.versions {
                        versions.remove(&context);
                    }

                    if let Some(unknown) = &mut file.config.unknown {
                        unknown.remove(REX_PLUGIN_KEY);
                    }
                });
            }
        });
    }

    fn merge_configs(&self, files: Vec<&RexConfigFile>) -> Result<RexConfig, RexConfigError> {
        let mut partial = PartialRexConfig::default();
        let mut count = 0;
        let context = &();

        for file in files.iter().rev() {
            if file.exists {
                partial.merge(context, file.config.to_owned())?;
                count += 1;
            }
        }

        let mut config = RexConfig::from_partial(partial.finalize(context)?);
        config.inherit_builtin_plugins();
        config.setup_env_vars();

        debug!("Merged {} configs", count);

        Ok(config)
    }
}
