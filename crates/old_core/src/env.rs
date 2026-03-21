use crate::config::{ConfigMode, REX_CONFIG_NAME, PinLocation, RexConfig};
use crate::config_error::RexConfigError;
use crate::env_error::RexEnvError;
use crate::file_manager::{RexConfigFile, RexDirEntry, RexFileManager};
use crate::helpers::is_offline;
use crate::layout::Store;
use crate::lockfile::RexLock;
use once_cell::sync::OnceCell;
use starbase_console::{Console, EmptyReporter};
use starbase_utils::dirs::home_dir;
use starbase_utils::envx;
use std::collections::BTreeMap;
use std::env;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLockReadGuard, RwLockWriteGuard};
use std::time::Duration;
use rex_system_env::{SystemArch, SystemOS};
use tracing::debug;
use rex_warpgate::PluginLoader;

pub type RexConsole = Console<EmptyReporter>;

#[derive(Clone, Default)]
pub struct RexEnvironment {
    pub config_mode: ConfigMode,
    pub env_mode: Option<String>,
    pub home_dir: PathBuf, // ~
    pub store: Store,
    pub test_only: bool,
    pub working_dir: PathBuf,

    pub os: SystemOS,
    pub arch: SystemArch,

    file_manager: Arc<OnceCell<RexFileManager>>,
    plugin_loader: Arc<OnceCell<PluginLoader>>,
}

impl RexEnvironment {
    pub fn new() -> Result<Self, RexEnvError> {
        let home = home_dir().ok_or(RexEnvError::MissingHomeDir)?;
        let mut root = envx::path_var("REX_HOME")
            .or_else(|| envx::path_var("XDG_DATA_HOME").map(|xdg| xdg.join("rex")))
            .unwrap_or_else(|| home.join(".rex"));

        if let Ok(rel_root) = root.strip_prefix("~") {
            root = home.join(rel_root);
        }

        Self::from(root, home)
    }

    pub fn new_testing(sandbox: &Path) -> Result<Self, RexEnvError> {
        let mut env = Self::from(sandbox.join(".rex"), sandbox.join(".home"))?;
        env.test_only = true;

        Ok(env)
    }

    pub fn from<R: AsRef<Path>, H: AsRef<Path>>(root: R, home: H) -> Result<Self, RexEnvError> {
        let root = root.as_ref();
        let home = home.as_ref();

        debug!(
            store = ?root,
            home = ?home,
            "Creating rex environment, detecting store",
        );

        Ok(RexEnvironment {
            config_mode: ConfigMode::Upwards,
            working_dir: env::current_dir().map_err(|_| RexEnvError::MissingWorkingDir)?,
            env_mode: env::var("REX_ENV").ok(),
            home_dir: home.to_owned(),
            file_manager: Arc::new(OnceCell::new()),
            plugin_loader: Arc::new(OnceCell::new()),
            test_only: env::var("REX_TEST").is_ok(),
            store: Store::new(root),
            os: SystemOS::default(),
            arch: SystemArch::default(),
        })
    }

    pub fn get_config_dir(&self, pin: PinLocation) -> &Path {
        match pin {
            PinLocation::Global => &self.store.dir,
            PinLocation::Local => &self.working_dir,
            PinLocation::User => &self.home_dir,
        }
    }

    pub fn get_plugin_loader(&self) -> Result<&PluginLoader, RexConfigError> {
        let config = self.load_config()?;

        self.plugin_loader.get_or_try_init(|| {
            let mut options = config.settings.http.clone();
            options.cache_dir = Some(self.store.cache_dir.join("requests-v2"));

            let mut loader =
                PluginLoader::new(&self.store.plugins_dir, self.store.temp_dir.join("plugins"));

            if let Some(secs) = config.settings.cache_duration {
                loader.set_cache_duration(Duration::from_secs(secs));
            }

            loader.set_http_client_options(&options);
            loader.set_offline_checker(is_offline);
            loader.add_registries(config.settings.registries.clone());

            Ok(loader)
        })
    }

    pub fn get_virtual_paths(&self) -> BTreeMap<PathBuf, PathBuf> {
        let mut paths = BTreeMap::from_iter([
            (self.store.temp_dir.clone(), "/temp".into()),
            (self.store.dir.clone(), "/rex".into()),
            (self.home_dir.clone(), "/userhome".into()),
        ]);

        if !paths.contains_key(&self.working_dir) {
            // This is required for situtations where users are using rex
            // outside of the home directory, and the WASM plugin will need
            // access to it!
            paths.insert(
                self.working_dir.clone(),
                if self.test_only {
                    "/sandbox".into()
                } else {
                    "/cwd".into()
                },
            );
        }

        paths
    }

    pub fn load_config(&self) -> Result<&RexConfig, RexConfigError> {
        self.load_config_with_mode(self.config_mode)
    }

    pub fn load_config_with_mode(
        &self,
        mode: ConfigMode,
    ) -> Result<&RexConfig, RexConfigError> {
        let manager = self.load_file_manager()?;

        match mode {
            ConfigMode::Global => manager.get_global_config(),
            ConfigMode::Local => manager.get_local_config(&self.working_dir),
            ConfigMode::Upwards => manager.get_merged_config_without_global(),
            ConfigMode::UpwardsGlobal => manager.get_merged_config(),
        }
    }

    pub fn load_config_files(&self) -> Result<Vec<&RexConfigFile>, RexConfigError> {
        Ok(self
            .load_file_manager()?
            .entries
            .iter()
            .filter_map(|dir| {
                if !self.config_mode.includes_global() && dir.location == PinLocation::Global
                    || self.config_mode.only_local() && dir.path != self.working_dir
                    || self.config_mode.only_global() && dir.path != self.store.dir
                {
                    None
                } else {
                    Some(&dir.configs)
                }
            })
            .flatten()
            .collect())
    }

    pub fn load_lock(&self) -> Result<Option<RwLockReadGuard<'_, RexLock>>, RexConfigError> {
        Ok(self.load_file_manager()?.get_lock())
    }

    pub fn load_lock_mut(
        &self,
    ) -> Result<Option<RwLockWriteGuard<'_, RexLock>>, RexConfigError> {
        Ok(self.load_file_manager()?.get_lock_mut())
    }

    #[tracing::instrument(name = "load_all", skip_all)]
    pub fn load_file_manager(&self) -> Result<&RexFileManager, RexConfigError> {
        self.file_manager.get_or_try_init(|| {
            // Don't traverse passed the home directory,
            // but only if working directory is within it!
            let end_dir = if self.working_dir.starts_with(&self.home_dir) {
                Some(self.home_dir.as_path())
            } else {
                None
            };

            let mut manager =
                RexFileManager::load(&self.working_dir, end_dir, self.env_mode.as_ref())?;

            // Always load the rex home/root config last
            let path = self.store.dir.join(REX_CONFIG_NAME);

            manager.entries.push(RexDirEntry {
                path: self.store.dir.clone(),
                location: PinLocation::Global,
                configs: vec![RexConfigFile {
                    exists: path.exists(),
                    path,
                    config: RexConfig::load_from(&self.store.dir, true)?,
                }],
                locked: false,
            });

            // Remove the pinned `rex` version from global/user configs,
            // as it causes massive recursion and `rex` process chains
            manager.remove_rex_pins();

            Ok(manager)
        })
    }
}

impl AsRef<RexEnvironment> for RexEnvironment {
    fn as_ref(&self) -> &RexEnvironment {
        self
    }
}

impl fmt::Debug for RexEnvironment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RexEnvironment")
            .field("config_mode", &self.config_mode)
            .field("env_mode", &self.env_mode)
            .field("home_dir", &self.home_dir)
            .field("store", &self.store)
            .field("test_only", &self.test_only)
            .field("working_dir", &self.working_dir)
            .finish()
    }
}
