use extism::{CurrentPlugin, Error, Function, UserData, Val, ValType};
use proto_core::ProtoEnvironment;
use rex_common::{Id, color};
use rex_config::ExtensionsConfig;
use rex_env::RexEnvironment;
use rex_warpgate::host::{HostData, create_host_functions as create_shared_host_functions};
use std::fmt;
use std::sync::Arc;
use tracing::{instrument, trace};

#[derive(Clone, Default)]
pub struct RexHostData {
    pub rex_env: Arc<RexEnvironment>,
    pub proto_env: Arc<ProtoEnvironment>,
    pub extensions_config: Arc<ExtensionsConfig>,
}

impl fmt::Debug for RexHostData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RexHostData")
            .field("rex_env", &self.rex_env)
            .field("proto_env", &self.proto_env)
            .field("extensions_config", &self.extensions_config)
            .finish()
    }
}

pub fn create_host_functions(data: RexHostData, shared_data: HostData) -> Vec<Function> {
    let mut functions = vec![];
    functions.extend(create_shared_host_functions(shared_data));
    functions.extend(vec![Function::new(
        "load_extension_config_by_id",
        [ValType::I64],
        [ValType::I64],
        UserData::new(data.clone()),
        load_extension_config_by_id,
    )]);
    functions
}

#[instrument(name = "host_load_extension_config_by_id", skip_all)]
fn load_extension_config_by_id(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    outputs: &mut [Val],
    user_data: UserData<RexHostData>,
) -> Result<(), Error> {
    let uuid = plugin.id().to_string();
    let extension_id = Id::new(plugin.memory_get_val::<String>(&inputs[0])?)?;

    trace!(
        plugin = &uuid,
        extension_id = extension_id.as_str(),
        "Calling host function {}",
        color::label("load_extension_config_by_id"),
    );

    let data = user_data.get()?;
    let data = data.lock().unwrap();

    let config = data
        .extensions_config
        .get_plugin_config(&extension_id)
        .ok_or_else(|| {
            Error::msg(format!(
                "Unable to load extension configuration. Extension {extension_id} does not exist."
            ))
        })?;

    plugin.memory_set_val(&mut outputs[0], serde_json::to_string(&config.to_json())?)?;

    trace!(
        plugin = &uuid,
        extension_id = extension_id.as_str(),
        "Called host function {}",
        color::label("load_extension_config_by_id"),
    );

    Ok(())
}
