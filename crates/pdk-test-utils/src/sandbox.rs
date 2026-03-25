use crate::extension_wrapper::*;
use crate::host_func_mocker::*;
use extism::{Function, UserData, ValType};

// };
use rex_pdk_api::{
    Id, RegisterExtensionInput, RegisterExtensionOutput
};
use proto_core::{ProtoEnvironment, ToolContext, inject_proto_manifest_config};
use starbase_sandbox::{Sandbox, create_empty_sandbox, create_sandbox};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use rex_warpgate::{
    PluginContainer, PluginLoader, PluginManifest, Wasm, host::*, inject_default_manifest_config,
    test_utils::*,
};

pub struct RexWasmSandbox {
    pub sandbox: Sandbox,
    pub home_dir: PathBuf,
    pub host_funcs: MockedHostFuncs,
    pub rex_dir: PathBuf,
    pub proto: Arc<ProtoEnvironment>,
    pub proto_dir: PathBuf,
    pub root: PathBuf,
    pub wasm_file: PathBuf,
}

impl RexWasmSandbox {
    pub fn new(sandbox: Sandbox) -> Self {
        let root = sandbox.path().to_path_buf();
        let home_dir = root.join(".home");
        let rex_dir = root.join(".rex");
        let proto_dir = root.join(".proto");
        let wasm_file = find_wasm_file();

        // Folders must exist for WASM to compile correctly!
        fs::create_dir_all(&home_dir).unwrap();
        fs::create_dir_all(&rex_dir).unwrap();
        fs::create_dir_all(&proto_dir).unwrap();

        // Required for toolchains
        let mut proto = ProtoEnvironment::new_testing(&root).unwrap();
        proto.working_dir = root.clone();

        Self {
            home_dir,
            rex_dir,
            proto: Arc::new(proto),
            proto_dir,
            root,
            sandbox,
            wasm_file,
            host_funcs: MockedHostFuncs::default(),
        }
    }

    pub fn create_config(&self) -> ConfigBuilder {
        ConfigBuilder::new(&self.root, &self.home_dir)
    }

    pub async fn create_extension(&self, id: &str) -> ExtensionTestWrapper {
        self.create_extension_with_config(id, |_| {}).await
    }

    pub async fn create_extension_with_config(
        &self,
        id: &str,
        mut op: impl FnMut(&mut ConfigBuilder),
    ) -> ExtensionTestWrapper {
        let id = Id::raw(id);

        // Create manifest
        let mut manifest = PluginManifest::new([Wasm::file(self.wasm_file.clone())]);

        // Create config
        let mut config = self.create_config();
        config.plugin_id(&id);

        op(&mut config);

        manifest.config.extend(config.build());

        // Create plugin
        let plugin = self.create_plugin_container(id, manifest, false);
        let metadata: RegisterExtensionOutput = plugin
            .cache_func_with(
                "register_extension",
                RegisterExtensionInput {
                    id: plugin.id.clone(),
                },
            )
            .await
            .unwrap();

        ExtensionTestWrapper {
            metadata,
            plugin,
            root: self.root.clone(),
        }
    }

    // pub async fn create_toolchain(&self, id: &str) -> ToolchainTestWrapper {
    //     self.create_toolchain_with_config(id, |_| {}).await
    // }

    // pub async fn create_toolchain_with_config(
    //     &self,
    //     id: &str,
    //     mut op: impl FnMut(&mut ConfigBuilder),
    // ) -> ToolchainTestWrapper {
    //     let id = Id::raw(id);

    //     // Create manifest
    //     let mut manifest = PluginManifest::new([Wasm::file(self.wasm_file.clone())]);

    //     // Create config
    //     let mut config = self.create_config();
    //     config.plugin_id(&id);

    //     op(&mut config);

    //     manifest.config.extend(config.build());

    //     // Create plugin
    //     let plugin = Arc::new(self.create_plugin_container(id, manifest, true));
    //     let metadata: RegisterToolchainOutput = plugin
    //         .cache_func_with(
    //             "register_toolchain",
    //             RegisterToolchainInput {
    //                 id: plugin.id.clone(),
    //             },
    //         )
    //         .await
    //         .unwrap();

    //     ToolchainTestWrapper {
    //         metadata,
    //         plugin: plugin.clone(),
    //         root: self.root.clone(),
    //         tool: if plugin.has_func("register_tool").await {
    //             Some(ToolTestWrapper {
    //                 tool: Tool::new(
    //                     ToolContext::new(plugin.id.clone()),
    //                     self.proto.clone(),
    //                     plugin,
    //                 )
    //                 .await
    //                 .unwrap(),
    //             })
    //         } else {
    //             None
    //         },
    //     }
    // }

    pub fn enable_logging(&self) {
        enable_wasm_logging(&self.wasm_file);
    }

    fn create_plugin_container(
        &self,
        id: Id,
        mut manifest: PluginManifest,
        with_proto: bool,
    ) -> PluginContainer {
        let virtual_paths = BTreeMap::<PathBuf, PathBuf>::from_iter([
            (self.root.clone(), "/workspace".into()),
            (self.home_dir.clone(), "/userhome".into()),
            (self.rex_dir.clone(), "/rex".into()),
            (self.proto_dir.clone(), "/proto".into()),
        ]);

        manifest.timeout_ms = None;
        manifest = manifest.with_allowed_host("*");
        manifest = manifest.with_allowed_paths(
            virtual_paths
                .iter()
                .map(|(key, value)| (key.to_string_lossy().to_string(), value.to_owned())),
        );

        inject_default_manifest_config(&id, &self.home_dir, &mut manifest).unwrap();

        if with_proto {
            let context = ToolContext::new(id.clone());

            inject_proto_manifest_config(&context, &self.proto, &mut manifest).unwrap();
        }

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
            RexHostFunction::LoadExtensionConfig,
            // RexHostFunction::LoadProject,
            // RexHostFunction::LoadProjects,
            // RexHostFunction::LoadTask,
            // RexHostFunction::LoadTasks,
            // RexHostFunction::LoadToolchainConfig,
        ] {
            funcs.push(Function::new(
                func_type.as_str().to_string(),
                // if func_type == RexHostFunction::LoadToolchainConfig {
                //     vec![ValType::I64, ValType::I64]
                // } else {
                //     vec![ValType::I64]
                // },
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
        f.debug_struct("RexWasmSandbox")
            .field("home_dir", &self.home_dir)
            .field("rex_dir", &self.rex_dir)
            .field("proto_dir", &self.proto_dir)
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
