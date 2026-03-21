use crate::common::{InitializePluginInput, InitializePluginOutput};
use crate::context::RexContext;
// use rex_common::Id;
use rex_warpgate_api::Id;
use schematic::Schema;
use rex_warpgate_api::*;

pub type InitializeActionInput = InitializePluginInput;
pub type InitializeActionOutput = InitializePluginOutput;

api_struct!(
    /// Input passed to the `register_action` function.
    pub struct RegisterActionInput {
        /// ID of the action, as it was configured.
        pub id: Id,
    }
);

api_struct!(
    /// Output returned from the `register_action` function.
    pub struct RegisterActionOutput {
        /// Name of the action.
        pub name: String,

        /// Optional description about what the action does.
        pub description: Option<String>,

        /// Version of the plugin.
        pub plugin_version: String,
    }
);

api_struct!(
    /// Output returned from the `define_action_config` function.
    pub struct DefineActionConfigOutput {
        /// Schema shape of the action's configuration.
        pub schema: Schema,
    }
);

api_struct!(
    /// Input passed to the `execute_action` function.
    pub struct ExecuteActionInput {
        /// Custom arguments passed on the command line.
        pub args: Vec<String>,

        /// Current rex context.
        pub context: RexContext,
    }
);
