use extism_pdk::*;
use rex_pdk_api::AnyResult;
use serde::de::DeserializeOwned;

#[host_fn]
extern "ExtismHost" {
    fn load_action_config_by_id<T: DeserializeOwned>(action_id: String) -> Json<T>;
    fn load_subcommand_config_by_id<T: DeserializeOwned>(subcommand_id: String) -> Json<T>;
}

/// Load configuration for a action by ID.
pub fn load_action_config<T: DeserializeOwned>(action_id: impl AsRef<str>) -> AnyResult<T> {
    let config = unsafe { load_action_config_by_id(action_id.as_ref().into())? };

    Ok(config.0)
}

/// Load configuration for a subcommand by ID.
pub fn load_subcommand_config<T: DeserializeOwned>(subcommand_id: impl AsRef<str>) -> AnyResult<T> {
    let config = unsafe { load_subcommand_config_by_id(subcommand_id.as_ref().into())? };

    Ok(config.0)
}
