use crate::common::{InitializePluginInput, InitializePluginOutput};
use crate::context::RexContext;
use rex_common::Id;
use rex_warpgate_api::*;
use schematic::Schema;

pub type InitializeExtensionInput = InitializePluginInput;
pub type InitializeExtensionOutput = InitializePluginOutput;

// METADATA

api_struct!(
    /// Input passed to the `register_extension` function.
    pub struct RegisterExtensionInput {
        /// ID of the extension, as it was configured.
        pub id: Id,
    }
);

api_struct!(
    /// Output returned from the `register_extension` function.
    pub struct RegisterExtensionOutput {
        /// Name of the extension.
        pub name: String,

        /// Optional description about what the extension does.
        pub description: Option<String>,

        /// Version of the plugin.
        pub plugin_version: String,
    }
);

api_struct!(
    /// Output returned from the `define_extension_config` function.
    pub struct DefineExtensionConfigOutput {
        /// Schema shape of the extension's configuration.
        pub schema: Schema,
    }
);

// EXECUTE

api_struct!(
    /// Input passed to the `execute_extension` function.
    pub struct ExecuteExtensionInput {
        /// Custom arguments passed on the command line.
        pub args: Vec<String>,

        /// Current rex context.
        pub context: RexContext,
    }
);
