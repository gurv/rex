pub use super::link_error::RexLinkError;
use crate::flow::locate::Locator;
use crate::tool::Tool;
use crate::tool_spec::ToolSpec;
use serde::Serialize;
use starbase_utils::{fs, path};
use std::path::PathBuf;
use tracing::{debug, instrument, warn};

#[derive(Debug, Default, Serialize)]
pub struct LinkerResponse {
    pub bins: Vec<PathBuf>,
}

/// Link binaries and shims for an installed tool.
pub struct Linker<'tool> {
    tool: &'tool Tool,
    spec: &'tool ToolSpec,
}

impl<'tool> Linker<'tool> {
    pub fn new(tool: &'tool Tool, spec: &'tool ToolSpec) -> Self {
        Self { tool, spec }
    }

    pub async fn link(
        tool: &'tool Tool,
        spec: &'tool ToolSpec,
        force: bool,
    ) -> Result<LinkerResponse, RexLinkError> {
        Self::new(tool, spec).link_all(force).await
    }

    /// Link both binaries and shims.
    pub async fn link_all(&self, force: bool) -> Result<LinkerResponse, RexLinkError> {
        Ok(LinkerResponse {
            bins: self.link_bins(force).await?,
        })
    }

    /// Symlink all primary and secondary binaries for the current tool.
    #[instrument(skip(self))]
    pub async fn link_bins(&self, force: bool) -> Result<Vec<PathBuf>, RexLinkError> {
        let bins = Locator::new(self.tool, self.spec)
            .locate_bins(if force {
                None
            } else {
                self.spec.version.as_ref()
            })
            .await?;

        if bins.is_empty() {
            return Ok(vec![]);
        }

        if force {
            debug!(
                tool = self.tool.context.as_str(),
                bins_dir = ?self.tool.rex.store.bin_dir,
                "Creating symlinks to the original tool executables"
            );
        }

        let mut to_create = vec![];

        for bin in bins {
            let Some(bin_version) = bin.version else {
                continue;
            };

            // Create a new product since we need to change the version for each bin
            let tool_dir = self.tool.inventory.get_product_dir(&bin_version);

            let input_path = tool_dir.join(path::normalize_separators(
                bin.config
                    .exe_link_path
                    .as_ref()
                    .or(bin.config.exe_path.as_ref())
                    .unwrap(),
            ));

            let output_path = bin.path;

            if !input_path.exists() {
                warn!(
                    tool = self.tool.context.as_str(),
                    source = ?input_path,
                    target = ?output_path,
                    "Unable to symlink binary, source file does not exist"
                );

                continue;
            }

            if !force && output_path.exists() {
                continue;
            }

            to_create.push((input_path, output_path));
        }

        // Only create bins if necessary
        let mut bins = vec![];

        if !to_create.is_empty() {
            let store = &self.tool.rex.store;

            fs::create_dir_all(&store.bin_dir)?;

            // Lock for our tests because of race conditions
            #[cfg(debug_assertions)]
            let _lock = fs::lock_directory(&store.bin_dir)?;

            for (input_path, output_path) in to_create {
                debug!(
                    tool = self.tool.context.as_str(),
                    source = ?input_path,
                    target = ?output_path,
                    "Creating binary symlink"
                );

                store.unlink_bin(&output_path)?;
                store.link_bin(&output_path, &input_path)?;

                bins.push(output_path);
            }
        }

        Ok(bins)
    }
}
