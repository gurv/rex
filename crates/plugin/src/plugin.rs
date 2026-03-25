use async_trait::async_trait;
use proto_core::ProtoEnvironment;
use rex_env::RexEnvironment;
use rex_warpgate::{Id, PluginContainer, PluginLocator};
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;

pub struct PluginRegistration {
    pub container: PluginContainer,
    pub id: Id,        // unstable_foo
    pub id_stable: Id, // foo
    pub locator: PluginLocator,
    pub rex_env: Arc<RexEnvironment>,
    pub proto_env: Arc<ProtoEnvironment>,
    pub wasm_file: PathBuf,
}

#[derive(Clone, Copy, Debug)]
pub enum PluginType {
    Extension,
    Toolchain,
}

impl PluginType {
    pub fn get_dir_name(&self) -> &str {
        match self {
            PluginType::Extension => "extensions",
            PluginType::Toolchain => "toolchains",
        }
    }

    pub fn get_label(&self) -> &str {
        match self {
            PluginType::Extension => "extension",
            PluginType::Toolchain => "toolchain",
        }
    }
}

#[async_trait]
pub trait Plugin: Debug + Sized {
    async fn new(registration: PluginRegistration) -> miette::Result<Self>;
    fn get_id(&self) -> &Id;
    fn get_type(&self) -> PluginType;
}
