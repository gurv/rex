use crate::extension_plugin::ExtensionPlugin;
use miette::IntoDiagnostic;
use rex_common::Id;
use rex_config::ExtensionsConfig;
use rex_plugin::{
    PluginError, PluginRegistry, PluginType, RexHostData, serialize_config,
};
use starbase_utils::json::JsonValue;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::{debug, trace};

#[derive(Debug)]
pub struct ExtensionRegistry {
    pub config: Arc<ExtensionsConfig>,
    registry: Arc<PluginRegistry<ExtensionPlugin>>,
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self {
            config: Arc::new(ExtensionsConfig::default()),
            registry: Arc::new(PluginRegistry::new(
                PluginType::Extension,
                RexHostData::default(),
            )),
        }
    }
}

impl ExtensionRegistry {
    pub fn new(host_data: RexHostData, config: Arc<ExtensionsConfig>) -> Self {
        Self {
            config,
            registry: Arc::new(PluginRegistry::new(PluginType::Extension, host_data)),
        }
    }

    pub fn create_config(&self, id: &str) -> JsonValue {
        if let Some(config) = self.config.get_plugin_config(id) {
            return config.to_json();
        }

        JsonValue::default()
    }

    pub fn get_plugin_ids(&self) -> Vec<&Id> {
        self.config.plugins.keys().collect()
    }

    pub fn has_plugin_configs(&self) -> bool {
        !self.config.plugins.is_empty()
    }

    pub async fn load<T>(&self, id: T) -> miette::Result<Arc<ExtensionPlugin>>
    where
        T: AsRef<str>,
    {
        let id = Id::raw(id.as_ref());

        if !self.is_registered(&id).await {
            if !self.config.plugins.contains_key(&id) {
                return Err(PluginError::UnknownId {
                    id: id.to_string(),
                    ty: PluginType::Extension,
                }
                .into());
            }

            self.load_many([&id]).await?;
        }

        self.get_instance(&id).await
    }

    pub async fn load_all(&self) -> miette::Result<Vec<Arc<ExtensionPlugin>>> {
        if !self.has_plugin_configs() {
            return Ok(vec![]);
        }

        debug!("Loading all extension plugins");

        self.load_many(self.get_plugin_ids()).await
    }

    pub async fn load_many<I, T>(&self, ids: I) -> miette::Result<Vec<Arc<ExtensionPlugin>>>
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let mut set = JoinSet::<miette::Result<Arc<ExtensionPlugin>>>::new();
        let mut list = vec![];

        for id in ids {
            let id = Id::raw(id.as_ref());

            if self.registry.is_registered(&id).await {
                list.push(self.get_instance(&id).await?);
                continue;
            }

            let Some(config) = self.config.plugins.get(&id) else {
                continue;
            };

            let registry = Arc::clone(&self.registry);
            let config = config.to_owned();

            set.spawn(Box::pin(async move {
                let instance = registry
                    .load_with_config(&id, config.plugin.as_ref().unwrap(), |manifest| {
                        let value = serialize_config(config.config.iter())?;

                        trace!(
                            extension_id = id.as_str(),
                            config = %value,
                            "Storing rex extension configuration",
                        );

                        manifest
                            .config
                            .insert("rex_extension_config".to_owned(), value);

                        Ok(())
                    })
                    .await?;

                Ok(instance)
            }));
        }

        if !set.is_empty() {
            while let Some(result) = set.join_next().await {
                list.push(result.into_diagnostic()??);
            }
        }

        Ok(list)
    }
}

impl Deref for ExtensionRegistry {
    type Target = PluginRegistry<ExtensionPlugin>;

    fn deref(&self) -> &Self::Target {
        &self.registry
    }
}
