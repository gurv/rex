use starbase_utils::{dirs, envx};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tracing::debug;

#[derive(Debug, Default, Clone)]
pub struct RexEnvironment {
    pub id_file: PathBuf,
    pub cache_dir: PathBuf,
    pub plugins_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub templates_dir: PathBuf,
    pub home_dir: PathBuf,   // ~
    pub store_root: PathBuf, // ~/.rex
    pub test_only: bool,
    pub working_dir: PathBuf,
    pub workspace_root: PathBuf,
}

impl RexEnvironment {
    pub fn new() -> miette::Result<Self> {
        Self::from(envx::vendor_home_var("REX_HOME", |user_dir| {
            envx::path_var("XDG_DATA_HOME")
                .map(|xdg| xdg.join("rex"))
                .unwrap_or_else(|| user_dir.join(".rex"))
        }))
    }

    pub fn from<P: AsRef<Path>>(root: P) -> miette::Result<Self> {
        let store_root = root.as_ref();

        debug!(store = ?store_root, "Creating rex environment, detecting store");

        Ok(RexEnvironment {
            id_file: store_root.join("id"),
            cache_dir: store_root.join("cache"),
            plugins_dir: store_root.join("plugins"),
            temp_dir: store_root.join("temp"),
            templates_dir: store_root.join("templates"),
            home_dir: dirs::home_dir().unwrap(),
            store_root: store_root.to_owned(),
            test_only: false,
            working_dir: PathBuf::new(),
            workspace_root: PathBuf::new(),
        })
    }

    pub fn new_testing(sandbox: &Path) -> Self {
        let mut env = Self::from(sandbox.join(".rex")).unwrap();
        env.working_dir = sandbox.to_path_buf();
        env.workspace_root = sandbox.to_path_buf();
        env.home_dir = sandbox.join(".home");
        env.test_only = true;
        env
    }

    pub fn get_virtual_paths(&self) -> BTreeMap<PathBuf, PathBuf> {
        BTreeMap::from_iter([
            (self.store_root.clone(), "/rex".into()),
            (self.home_dir.clone(), "/userhome".into()),
            (self.workspace_root.clone(), "/workspace".into()),
        ])
    }
}

impl AsRef<RexEnvironment> for RexEnvironment {
    fn as_ref(&self) -> &RexEnvironment {
        self
    }
}
