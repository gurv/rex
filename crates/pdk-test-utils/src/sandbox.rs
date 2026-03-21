use crate::action_wrapper::*;
use crate::host_func_mocker::*;
use crate::subcommand_wrapper::*;
use crate::wrapper::WasmTestWrapper;
use extism::{Function, UserData, ValType};
use rex_pdk_api::{RegisterActionInput, RegisterActionOutput, RegisterSubcommandInput, RegisterSubcommandOutput};
use rex_old_core::{RexEnvironment, Tool, ToolContext, inject_rex_manifest_config};
use starbase_id::Id;
use starbase_sandbox::{Sandbox, create_empty_sandbox, create_sandbox};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use rex_warpgate::test_utils::*;
use rex_warpgate::{PluginContainer, PluginLoader, PluginManifest, Wasm, host::*, inject_default_manifest_config};

pub struct RexWasmSandbox {
    pub sandbox: Sandbox,
    pub home_dir: PathBuf,
    pub host_funcs: MockedHostFuncs,
    pub rex_dir: PathBuf,
    pub root: PathBuf,
    pub wasm_file: PathBuf,
}

impl RexWasmSandbox {
    pub fn new(sandbox: Sandbox) -> Self {
        let root = sandbox.path().to_path_buf();
        let home_dir = root.join(".home");
        let rex_dir = root.join(".rex");
        let wasm_file = find_wasm_file();

        fs::create_dir_all(&home_dir).unwrap();
        fs::create_dir_all(&rex_dir).unwrap();

        Self {
            home_dir,
            rex_dir,
            root,
            sandbox,
            wasm_file,
            host_funcs: MockedHostFuncs::default(),
        }
    }

    pub fn create_config(&self) -> ConfigBuilder {
        ConfigBuilder::new(&self.root, &self.home_dir)
    }

    pub async fn create_plugin(&self, context: &str) -> WasmTestWrapper {
        self.create_plugin_with_config(context, |_| {}).await
    }

    pub async fn create_plugin_with_config(
        &self,
        context: &str,
        mut op: impl FnMut(&mut ConfigBuilder),
    ) -> WasmTestWrapper {
        let context = ToolContext::parse(context).unwrap();
        let mut rex = RexEnvironment::new_testing(&self.root).unwrap();
        rex.working_dir = self.root.clone();

        let mut manifest =
            Tool::create_plugin_manifest(&rex, Wasm::file(&self.wasm_file)).unwrap();

        inject_default_manifest_config(&context.id, &rex.home_dir, &mut manifest).unwrap();
        inject_rex_manifest_config(&context, &rex, &mut manifest).unwrap();

        let mut config = self.create_config();
        op(&mut config);

        manifest.config.extend(config.build());

        WasmTestWrapper {
            tool: Tool::load_from_manifest(context, rex, manifest)
                .await
                .unwrap(),
        }
    }

    pub async fn create_schema_plugin(
        &self,
        context: &str,
        schema_path: PathBuf,
    ) -> WasmTestWrapper {
        self.create_schema_plugin_with_config(context, schema_path, |_| {})
            .await
    }

    #[allow(unused_variables)]
    pub async fn create_schema_plugin_with_config(
        &self,
        context: &str,
        schema_path: PathBuf,
        mut op: impl FnMut(&mut ConfigBuilder),
    ) -> WasmTestWrapper {
        self.create_plugin_with_config(context, move |config| {
            op(config);

            #[cfg(feature = "schema")]
            {
                use crate::config_builder::RexConfigBuilder;

                config.schema_config(rex_old_core::load_schema_config(&schema_path).unwrap());
            }
        })
        .await
    }

    pub async fn create_action(&self, id: &str) -> ActionTestWrapper {
        self.create_action_with_config(id, |_| {}).await
    }

    pub async fn create_action_with_config(
        &self,
        id: &str,
        mut op: impl FnMut(&mut ConfigBuilder),
    ) -> ActionTestWrapper {
        let id = Id::raw(id);

        // Create manifest
        let mut manifest = PluginManifest::new([Wasm::file(self.wasm_file.clone())]);

        // Create config
        let mut config = self.create_config();
        config.plugin_id(&id);

        op(&mut config);

        manifest.config.extend(config.build());

        // Create plugin
        let plugin = self.create_plugin_container(id, manifest);
        let metadata: RegisterActionOutput = plugin
            .cache_func_with(
                "register_action",
                RegisterActionInput {
                    id: plugin.id.clone(),
                },
            )
            .await
            .unwrap();

        ActionTestWrapper {
            metadata,
            plugin,
            root: self.root.clone(),
        }
    }

    pub async fn create_toolchain(&self, id: &str) -> SubcommandTestWrapper {
        self.create_subcommand_with_config(id, |_| {}).await
    }

    pub async fn create_subcommand_with_config(
        &self,
        id: &str,
        mut op: impl FnMut(&mut ConfigBuilder),
    ) -> SubcommandTestWrapper {
        let id = Id::raw(id);

        // Create manifest
        let mut manifest = PluginManifest::new([Wasm::file(self.wasm_file.clone())]);

        // Create config
        let mut config = self.create_config();
        config.plugin_id(&id);

        op(&mut config);

        manifest.config.extend(config.build());

        // Create plugin
        let plugin = Arc::new(self.create_plugin_container(id, manifest));
        let metadata: RegisterSubcommandOutput = plugin
            .cache_func_with(
                "register_subcommand",
                RegisterSubcommandInput {
                    id: plugin.id.clone(),
                },
            )
            .await
            .unwrap();

        SubcommandTestWrapper {
            metadata,
            plugin: plugin.clone(),
            root: self.root.clone(),
        }
    }

    pub fn enable_logging(&self) {
        enable_wasm_logging(&self.wasm_file);
    }

    fn create_plugin_container(
        &self,
        id: Id,
        mut manifest: PluginManifest,
    ) -> PluginContainer {
        let virtual_paths = BTreeMap::<PathBuf, PathBuf>::from_iter([
            (self.root.clone(), "/workspace".into()),
            (self.home_dir.clone(), "/userhome".into()),
            (self.rex_dir.clone(), "/rex".into()),
        ]);

        manifest.timeout_ms = None;
        manifest = manifest.with_allowed_host("*");
        manifest = manifest.with_allowed_paths(
            virtual_paths
                .iter()
                .map(|(key, value)| (key.to_string_lossy().to_string(), value.to_owned())),
        );

        inject_default_manifest_config(&id, &self.home_dir, &mut manifest).unwrap();

        PluginContainer::new(id, manifest, self.create_host_funcs(virtual_paths)).unwrap()
    }

    fn create_host_funcs(&self, virtual_paths: BTreeMap<PathBuf, PathBuf>) -> Vec<Function> {
        let loader = PluginLoader::new(self.rex_dir.join("plugins"), self.rex_dir.join("temp"));

        let host_data = HostData {
            cache_dir: self.rex_dir.join("cache"),
            http_client: loader.get_http_client().unwrap().clone(),
            virtual_paths,
            working_dir: self.root.clone(),
        };

        let mut funcs = create_host_functions(host_data.clone());

        for func_type in [
            // RexHostFunction::LoadExtensionConfig,
            RexHostFunction::LoadActionConfig,
            // RexHostFunction::LoadToolchainConfig,
            RexHostFunction::LoadSubcommandConfig,
        ] {
            funcs.push(Function::new(
                func_type.as_str().to_string(),
                vec![ValType::I64],
                [ValType::I64],
                UserData::new((func_type, self.host_funcs.clone())),
                mocked_host_func_impl,
            ));
        }

        funcs
    }
}

impl Deref for RexWasmSandbox {
    type Target = Sandbox;

    fn deref(&self) -> &Self::Target {
        &self.sandbox
    }
}

impl fmt::Debug for RexWasmSandbox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RexSandbox")
            .field("home_dir", &self.home_dir)
            .field("rex_dir", &self.rex_dir)
            .field("root", &self.root)
            .field("wasm_file", &self.wasm_file)
            .finish()
    }
}

pub fn create_rex_sandbox(fixture: &str) -> RexWasmSandbox {
    RexWasmSandbox::new(create_sandbox(fixture))
}

pub fn create_empty_rex_sandbox() -> RexWasmSandbox {
    RexWasmSandbox::new(create_empty_sandbox())
}
