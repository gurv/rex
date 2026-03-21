use crate::env::RexEnvironment;
use crate::registry::data::{PluginEntry, PluginRegistryDocument};
use crate::registry::registry_error::RexRegistryError;
use starbase_utils::{fs, json};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, instrument};

pub struct RexRegistry {
    env: Arc<RexEnvironment>,
    internal: Vec<PluginEntry>,
    external: Vec<PluginEntry>,
}

impl RexRegistry {
    pub fn new(env: Arc<RexEnvironment>) -> Self {
        debug!("Creating plugin registry");

        Self {
            env,
            internal: vec![],
            external: vec![],
        }
    }

    pub async fn load_plugins(&mut self) -> Result<Vec<&PluginEntry>, RexRegistryError> {
        self.load_internal_plugins().await?;
        self.load_external_plugins().await?;

        let mut plugins = vec![];
        plugins.extend(&self.internal);
        plugins.extend(&self.external);

        Ok(plugins)
    }

    #[instrument(skip(self))]
    pub async fn load_internal_plugins(&mut self) -> Result<Vec<&PluginEntry>, RexRegistryError> {
        if self.internal.is_empty() {
            debug!("Loading built-in plugins registry data");

            let plugins = self.load_plugins_from_registry(
                self.env
                    .store
                    .cache_dir
                    .join("registry/internal-plugins.json"),
                "https://raw.githubusercontent.com/gurv/rex/begin-proto/registry/data/built-in.json".into(),
            ).await?;

            self.internal.extend(plugins);
        }

        Ok(self.internal.iter().collect())
    }

    #[instrument(skip(self))]
    pub async fn load_external_plugins(&mut self) -> Result<Vec<&PluginEntry>, RexRegistryError> {
        if self.external.is_empty() {
            debug!("Loading third-party plugins registry data");

            let plugins = self.load_plugins_from_registry(
                self.env
                    .store
                    .cache_dir
                    .join("registry/external-plugins.json"),
                "https://raw.githubusercontent.com/gurv/rex/begin-proto/registry/data/third-party.json".into(),
            ).await?;

            self.external.extend(plugins);
        }

        Ok(self.external.iter().collect())
    }

    async fn load_plugins_from_registry(
        &self,
        temp_file: PathBuf,
        data_url: String,
    ) -> Result<Vec<PluginEntry>, RexRegistryError> {
        // Cache should refresh every 24 hours
        let duration = Duration::from_secs(86400);

        if temp_file.exists() && !fs::is_stale(&temp_file, false, duration)? {
            debug!(file = ?temp_file, "Reading plugins data from local cache");

            let plugins: Vec<PluginEntry> = json::read_file(&temp_file)?;

            return Ok(plugins);
        }

        // Otherwise fetch from the upstream URL
        debug!(url = &data_url, "Loading plugins data from remote URL");

        let data: PluginRegistryDocument = reqwest::get(&data_url)
            .await
            .map_err(|error| RexRegistryError::FailedRequest {
                url: data_url,
                error: Box::new(error),
            })?
            .json()
            .await
            .map_err(|error| RexRegistryError::FailedParse {
                error: Box::new(error),
            })?;

        // Cache the result for future requests
        json::write_file(temp_file, &data.plugins, false)?;

        Ok(data.plugins)
    }
}
