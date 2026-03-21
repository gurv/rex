use super::inventory::Inventory;
use super::layout_error::RexLayoutError;
use crate::id::Id;
use crate::tool_manifest::ToolManifest;
use rex_pdk_api::ToolInventoryOptions;
use serde::Serialize;
use starbase_styles::color;
use starbase_utils::{envx, fs, path};
use std::fmt;
use std::path::{Path, PathBuf};
use tracing::{debug, instrument};

#[derive(Clone, Default, Serialize)]
pub struct Store {
    pub dir: PathBuf,
    pub backends_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub builders_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub inventory_dir: PathBuf,
    pub plugins_dir: PathBuf,
    pub temp_dir: PathBuf,
}

impl Store {
    #[instrument(name = "create_store")]
    pub fn new(dir: &Path) -> Self {
        let temp_dir = match envx::path_var("REX_TEMP_DIR") {
            Some(custom) => {
                debug!(
                    temp_dir = ?custom,
                    "Using custom temp directory from {}",
                    color::symbol("REX_TEMP_DIR")
                );

                custom
            }
            None => dir.join("temp"),
        };

        Self {
            dir: dir.to_path_buf(),
            backends_dir: dir.join("backends"),
            bin_dir: dir.join("bin"),
            builders_dir: dir.join("builders"),
            cache_dir: dir.join("cache"),
            inventory_dir: dir.join("tools"),
            plugins_dir: dir.join("plugins"),
            temp_dir,
        }
    }

    pub fn create_inventory(
        &self,
        id: &Id,
        config: &ToolInventoryOptions,
    ) -> Result<Inventory, RexLayoutError> {
        let dir = self.inventory_dir.join(path::encode_component(id));

        Ok(Inventory {
            manifest: ToolManifest::load_from(&dir)?,
            dir,
            dir_original: None,
            temp_dir: self.temp_dir.join(path::encode_component(id)),
            config: config.to_owned(),
        })
    }

    pub fn load_uuid(&self) -> Result<String, RexLayoutError> {
        let id_path = self.dir.join("id");

        if id_path.exists() {
            return Ok(fs::read_file(id_path)?);
        }

        let id = uuid::Uuid::new_v4().to_string();

        fs::write_file(id_path, &id)?;

        Ok(id)
    }

    #[instrument(skip(self))]
    pub fn load_preferred_profile(&self) -> Result<Option<PathBuf>, RexLayoutError> {
        let profile_path = self.dir.join("profile");

        if profile_path.exists() {
            return Ok(Some(fs::read_file(profile_path)?.into()));
        }

        Ok(None)
    }

    #[instrument(skip(self))]
    pub fn save_preferred_profile(&self, path: &Path) -> Result<(), RexLayoutError> {
        fs::write_file(
            self.dir.join("profile"),
            path.as_os_str().as_encoded_bytes(),
        )?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub fn link_bin(&self, bin_path: &Path, src_path: &Path) -> Result<(), RexLayoutError> {
        // Windows requires admin privileges to create soft/hard links,
        // so just copy the binary... annoying...
        #[cfg(windows)]
        {
            fs::copy_file(src_path, bin_path)?;
        }

        #[cfg(unix)]
        {
            use starbase_utils::fs::FsError;

            std::os::unix::fs::symlink(src_path, bin_path).map_err(|error| {
                RexLayoutError::Fs(Box::new(FsError::Create {
                    path: src_path.to_path_buf(),
                    error: Box::new(error),
                }))
            })?;
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub fn unlink_bin(&self, bin_path: &Path) -> Result<(), RexLayoutError> {
        // Remove any file at this path, whether a symlink or not!
        if bin_path.is_symlink() {
            fs::remove_link(bin_path)?;
        } else if bin_path.is_file() {
            fs::remove_file(bin_path)?;
        }

        Ok(())
    }
}

impl fmt::Debug for Store {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Store")
            .field("dir", &self.dir)
            .field("bin_dir", &self.bin_dir)
            .field("cache_dir", &self.cache_dir)
            .field("inventory_dir", &self.inventory_dir)
            .field("plugins_dir", &self.plugins_dir)
            .field("temp_dir", &self.temp_dir)
            .finish()
    }
}
