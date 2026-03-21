use extism::{CurrentPlugin, Error, UserData, Val};
use serde_json::{Value, json};
use std::sync::Arc;

#[allow(clippy::enum_variant_names)]
#[derive(PartialEq)]
pub enum RexHostFunction {
    LoadActionConfig,
    LoadSubcommandConfig,
}

impl RexHostFunction {
    pub fn as_str(&self) -> &str {
        match self {
            Self::LoadActionConfig => "load_action_config_by_id",
            Self::LoadSubcommandConfig => "load_subcommand_config_by_id",
        }
    }
}

pub type LoadActionConfigHostFunc = Arc<dyn Fn(String) -> Value>;
pub type LoadSubcommandConfigHostFunc = Arc<dyn Fn(String) -> Value>;

#[derive(Clone, Default)]
pub struct MockedHostFuncs {
    load_action_config: Option<LoadActionConfigHostFunc>,
    load_subcommand_config: Option<LoadSubcommandConfigHostFunc>,
}

impl MockedHostFuncs {
    pub fn mock_load_action_config(&mut self, func: impl Fn(String) -> Value + 'static) {
        self.load_action_config = Some(Arc::new(func));
    }

    pub fn mock_load_subcommand_config(&mut self, func: impl Fn(String) -> Value + 'static) {
        self.load_subcommand_config = Some(Arc::new(func));
    }
}

pub fn mocked_host_func_impl(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    outputs: &mut [Val],
    user_data: UserData<(RexHostFunction, MockedHostFuncs)>,
) -> Result<(), Error> {
    let data = user_data.get()?;
    let data = data.lock().unwrap();
    let (func_type, mocked_funcs) = &*data;

    let value = match func_type {
        RexHostFunction::LoadActionConfig => {
            let action_id: String = plugin.memory_get_val(&inputs[0])?;

            mocked_funcs
                .load_action_config
                .as_ref()
                .map_or(json!({}), |func| func(action_id))
        }
        RexHostFunction::LoadSubcommandConfig => {
            let subcommand_id: String = plugin.memory_get_val(&inputs[0])?;

            mocked_funcs
                .load_subcommand_config
                .as_ref()
                .map_or(json!({}), |func| func(subcommand_id))
        }
    };

    plugin.memory_set_val(&mut outputs[0], serde_json::to_string(&value)?)?;

    Ok(())
}
