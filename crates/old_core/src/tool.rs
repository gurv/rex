use crate::config::PluginType;
use crate::env::RexEnvironment;
use crate::helpers::get_rex_version;
use crate::id::Id;
use crate::layout::Inventory;
use crate::lockfile::LockRecord;
use crate::tool_context::ToolContext;
use crate::tool_error::RexToolError;
use crate::tool_spec::ToolSpec;
use crate::utils::{archive, git};
use rex_pdk_api::{
    PluginContext, PluginFunction, PluginUnresolvedContext, RegisterBackendInput,
    RegisterBackendOutput, RegisterToolInput, RegisterToolOutput, SourceLocation, VersionSpec,
};
use rustc_hash::FxHashMap;
use starbase_styles::color;
use starbase_utils::{fs, path};
use std::fmt::{self, Debug};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, instrument};
use rex_warpgate::{
    PluginContainer, PluginLocator, PluginManifest, VirtualPath, Wasm,
    host::{HostData, create_host_functions},
};

pub type ToolMetadata = RegisterToolOutput;

pub struct Tool {
    pub context: ToolContext,
    pub locator: Option<PluginLocator>,
    pub metadata: ToolMetadata,
    pub plugin: Arc<PluginContainer>,
    pub rex: Arc<RexEnvironment>,
    pub ty: PluginType,

    // Store
    pub inventory: Inventory,

    // Cache
    pub(crate) backend_registered: bool,
    pub(crate) cache: bool,
}

impl Tool {
    pub async fn new(
        context: ToolContext,
        rex: Arc<RexEnvironment>,
        plugin: Arc<PluginContainer>,
    ) -> Result<Self, RexToolError> {
        debug!(
            "Created tool {} and its WASM runtime",
            color::id(context.as_str())
        );

        let mut tool = Tool {
            backend_registered: false,
            cache: true,
            context,
            inventory: Inventory::default(),
            locator: None,
            metadata: ToolMetadata::default(),
            plugin,
            rex,
            ty: PluginType::Tool,
        };

        tool.register_tool().await?;

        if tool.context.backend.is_some() {
            tool.register_backend().await?;
        }

        Ok(tool)
    }

    #[instrument(name = "new_tool", skip(rex, wasm))]
    pub async fn load<I: AsRef<ToolContext> + fmt::Debug, P: AsRef<RexEnvironment>>(
        context: I,
        rex: P,
        wasm: Wasm,
    ) -> Result<Self, RexToolError> {
        let rex = rex.as_ref();

        Self::load_from_manifest(context, rex, Self::create_plugin_manifest(rex, wasm)?).await
    }

    pub async fn load_from_manifest<I: AsRef<ToolContext>, P: AsRef<RexEnvironment>>(
        context: I,
        rex: P,
        manifest: PluginManifest,
    ) -> Result<Self, RexToolError> {
        let context = context.as_ref();
        let rex = rex.as_ref();

        debug!(
            "Creating tool {} and instantiating plugin",
            color::id(context.as_str())
        );

        Self::new(
            context.to_owned(),
            Arc::new(rex.to_owned()),
            Arc::new(PluginContainer::new(
                context.id.clone(),
                manifest,
                create_host_functions(HostData {
                    cache_dir: rex.store.cache_dir.clone(),
                    http_client: Arc::clone(rex.get_plugin_loader()?.get_http_client()?),
                    virtual_paths: rex.get_virtual_paths(),
                    working_dir: rex.working_dir.clone(),
                }),
            )?),
        )
        .await
    }

    pub fn create_plugin_manifest<P: AsRef<RexEnvironment>>(
        rex: P,
        wasm: Wasm,
    ) -> Result<PluginManifest, RexToolError> {
        let rex = rex.as_ref();
        let mut virtual_paths = FxHashMap::default();

        for (host, guest) in rex.get_virtual_paths() {
            virtual_paths.insert(host.to_string_lossy().to_string(), guest);

            // The host path must exist or extism errors!
            fs::create_dir_all(host)?;
        }

        let mut manifest = PluginManifest::new([wasm]);
        manifest = manifest.with_allowed_host("*");
        manifest = manifest.with_allowed_paths(virtual_paths.into_iter());
        // manifest = manifest.with_timeout(Duration::from_secs(90));

        #[cfg(debug_assertions)]
        {
            manifest = manifest.with_timeout(std::time::Duration::from_secs(300));
        }

        Ok(manifest)
    }

    /// Disable caching when applicable.
    pub fn disable_caching(&mut self) {
        self.cache = false;
    }

    /// Return the backend identifier.
    pub fn get_backend(&self) -> Option<&Id> {
        self.context.backend.as_ref()
    }

    pub fn get_env_var_prefix(&self) -> String {
        format!("REX_{}", self.get_id().to_env_var())
    }

    /// Return the tool identifier for use within file names.
    pub fn get_file_name(&self) -> &str {
        let id = self.get_id().as_str();

        // May be an npm package with a scope,
        // so remove the scope and return the name
        match id.rfind('/') {
            Some(index) => &id[index + 1..],
            None => id,
        }
    }

    /// Return the tool identifier.
    pub fn get_id(&self) -> &Id {
        &self.context.id
    }

    /// Return an absolute path to the tool's inventory directory.
    /// The inventory houses installed versions, the manifest, and more.
    pub fn get_inventory_dir(&self) -> &Path {
        &self.inventory.dir
    }

    /// Return a human readable name for the tool.
    pub fn get_name(&self) -> &str {
        &self.metadata.name
    }

    /// Return an absolute path to a temp directory solely for this tool.
    pub fn get_temp_dir(&self) -> &Path {
        &self.inventory.temp_dir
    }

    /// Return an absolute path to the tool's install directory for the currently resolved version.
    pub fn get_product_dir(&self, spec: &ToolSpec) -> PathBuf {
        match &spec.version {
            Some(version) => self.inventory.get_product_dir(version),
            None => self.inventory.get_product_dir(&VersionSpec::default()),
        }
    }

    /// Return true if this tool instance is a backend plugin.
    pub fn is_backend_plugin(&self) -> bool {
        self.ty == PluginType::Backend
    }

    /// Return true if the tool has been installed. This *requires* the spec to
    /// have been resolved before hand.
    pub fn is_installed(&self, spec: &ToolSpec) -> bool {
        let dir = self.get_product_dir(spec);

        debug!(
            tool = self.context.as_str(),
            install_dir = ?dir,
            "Checking if tool is installed",
        );

        let installed = spec.version.as_ref().is_some_and(|v| {
            !v.is_latest() && self.inventory.manifest.installed_versions.contains(v)
        }) && dir.exists()
            && !fs::is_dir_locked(&dir);

        if installed {
            debug!(
                tool = self.context.as_str(),
                install_dir = ?dir,
                "Tool has already been installed",
            );
        } else {
            debug!(tool = self.context.as_str(), "Tool has not been installed");
        }

        installed
    }

    /// Convert a virtual path to a real path.
    pub fn from_virtual_path(&self, path: impl AsRef<Path> + Debug) -> PathBuf {
        self.plugin.from_virtual_path(path)
    }

    /// Convert a real path to a virtual path.
    pub fn to_virtual_path(&self, path: impl AsRef<Path> + Debug) -> VirtualPath {
        self.plugin.to_virtual_path(path)
    }
}

// APIs

impl Tool {
    /// Return contextual information to pass to WASM plugin functions.
    pub fn create_plugin_context(&self, spec: &ToolSpec) -> PluginContext {
        PluginContext {
            rex_version: Some(get_rex_version().to_owned()),
            temp_dir: self.to_virtual_path(self.get_temp_dir()),
            tool_dir: self.to_virtual_path(self.get_product_dir(spec)),
            version: spec.get_resolved_version(),
        }
    }

    /// Return contextual information to pass to WASM plugin functions,
    /// representing an unresolved state, which has no version or tool
    /// data.
    #[allow(deprecated)]
    pub fn create_plugin_unresolved_context(&self) -> PluginUnresolvedContext {
        PluginUnresolvedContext {
            rex_version: Some(get_rex_version().to_owned()),
            temp_dir: self.to_virtual_path(&self.inventory.temp_dir),
            // TODO: temporary until 3rd-party plugins update their PDKs
            tool_dir: self.to_virtual_path(&self.rex.store.inventory_dir),
            version: Some(VersionSpec::default()),
        }
    }

    /// Create an initial lock record.
    pub fn create_locked_record(&self) -> LockRecord {
        let mut record = LockRecord {
            backend: self.context.backend.clone(),
            ..Default::default()
        };

        if !self.metadata.lock_options.ignore_os_arch {
            record.os = Some(self.rex.os);
            record.arch = Some(self.rex.arch);
        }

        record
    }

    /// Register the tool by loading initial metadata and persisting it.
    #[instrument(skip_all)]
    pub async fn register_tool(&mut self) -> Result<(), RexToolError> {
        let metadata: RegisterToolOutput = self
            .plugin
            .cache_func_with(
                PluginFunction::RegisterTool,
                RegisterToolInput {
                    id: self.get_id().to_owned(),
                },
            )
            .await?;

        #[cfg(not(debug_assertions))]
        if let Some(expected_version) = &metadata.minimum_rex_version {
            let actual_version = get_rex_version();

            if actual_version < expected_version {
                return Err(RexToolError::InvalidMinimumVersion {
                    tool: metadata.name,
                    id: self.get_id().clone(),
                    expected: expected_version.to_string(),
                    actual: actual_version.to_string(),
                }
                .into());
            }
        }

        let inventory_id =
            if metadata.inventory_options.scoped_backend_dir && self.context.backend.is_some() {
                Id::raw(self.context.as_str().replace(':', "-"))
            } else {
                self.context.id.clone()
            };

        let mut inventory = self
            .rex
            .store
            .create_inventory(&inventory_id, &metadata.inventory_options)?;

        if let Some(override_dir) = &metadata.inventory_options.override_dir {
            let override_dir_path = override_dir.real_path();

            debug!(
                tool = self.context.as_str(),
                override_virtual = ?override_dir.virtual_path(),
                override_real = ?override_dir_path,
                "Attempting to override inventory directory"
            );

            if override_dir_path.as_ref().is_none_or(|p| p.is_relative()) {
                return Err(RexToolError::RequiredAbsoluteInventoryDir {
                    tool: metadata.name.clone(),
                    dir: override_dir_path.unwrap_or_else(|| PathBuf::from("<unknown>")),
                });
            }

            inventory.dir_original = Some(inventory.dir);
            inventory.dir = self.from_virtual_path(override_dir);
        }

        self.inventory = inventory;
        self.metadata = metadata;

        Ok(())
    }

    /// Register the backend by acquiring necessary source files.
    #[instrument(skip_all)]
    pub async fn register_backend(&mut self) -> Result<(), RexToolError> {
        if !self.plugin.has_func(PluginFunction::RegisterBackend).await || self.backend_registered {
            return Ok(());
        }

        let Some(backend) = &self.context.backend else {
            return Ok(());
        };

        let metadata: RegisterBackendOutput = self
            .plugin
            .cache_func_with(
                PluginFunction::RegisterBackend,
                RegisterBackendInput {
                    context: self.create_plugin_unresolved_context(),
                    id: self.get_id().to_owned(),
                },
            )
            .await?;

        let Some(source) = metadata.source else {
            self.backend_registered = true;

            return Ok(());
        };

        let backend_id = metadata.backend_id;
        let backend_dir = self
            .rex
            .store
            .backends_dir
            .join(path::encode_component(backend)) // asdf
            .join(path::encode_component(&backend_id)); // node
        let update_perms = !backend_dir.exists();
        let config = self.rex.load_config()?;

        debug!(
            tool = self.context.as_str(),
            backend_id = ?backend_id,
            backend_dir = ?backend_dir,
            "Acquiring backend sources",
        );

        match source {
            SourceLocation::Archive(mut src) => {
                if !backend_dir.exists() {
                    src.url = config.rewrite_url(src.url);

                    debug!(
                        tool = self.context.as_str(),
                        url = &src.url,
                        "Downloading backend archive",
                    );

                    archive::download_and_unpack(
                        &src,
                        &backend_dir,
                        &self.rex.store.temp_dir,
                        self.rex
                            .get_plugin_loader()?
                            .get_http_client()?
                            .to_inner(),
                    )
                    .await?;
                }
            }
            SourceLocation::Git(src) => {
                debug!(
                    tool = self.context.as_str(),
                    url = &src.url,
                    "Cloning backend repository",
                );

                git::clone_or_pull_repo(&src, &backend_dir).await?;
            }
        };

        if update_perms {
            for exe in metadata.exes {
                let exe_path = backend_dir.join(path::normalize_separators(exe));

                if exe_path.exists() {
                    fs::update_perms(exe_path, None)?;
                }
            }
        }

        self.ty = PluginType::Backend;
        self.backend_registered = true;

        Ok(())
    }
}

impl fmt::Debug for Tool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tool")
            .field("id", self.get_id())
            .field("metadata", &self.metadata)
            .field("locator", &self.locator)
            .field("rex", &self.rex)
            .field("inventory", &self.inventory)
            .finish()
    }
}
