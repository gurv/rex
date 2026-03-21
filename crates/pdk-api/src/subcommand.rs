use crate::common::{InitializePluginInput, InitializePluginOutput};
use crate::context::RexContext;
// use rex_common::Id;
use rex_warpgate_api::Id;
use schematic::Schema;
use rex_warpgate_api::*;

pub type InitializeSubcommandInput = InitializePluginInput;
pub type InitializeSubcommandOutput = InitializePluginOutput;

// METADATA

api_struct!(
    /// Input passed to the `register_subcommand` function.
    pub struct RegisterSubcommandInput {
        /// ID of the subcommand, as it was configured.
        pub id: Id,
    }
);

api_struct!(
    /// Output returned from the `register_subcommand` function.
    pub struct RegisterSubcommandOutput {
        /// Name of the subcommand.
        pub name: String,

        /// Optional description about what the subcommand does.
        pub description: Option<String>,

        /// Version of the plugin.
        pub plugin_version: String,
    }
);

api_struct!(
    /// Output returned from the `define_subcommand_config` function.
    pub struct DefineSubcommandConfigOutput {
        /// Schema shape of the subcommand's configuration.
        pub schema: Schema,
    }
);

// EXECUTE

api_struct!(
    /// Input passed to the `execute_subcommand` function.
    pub struct ExecuteSubcommandInput {
        /// Custom arguments passed on the command line.
        pub args: Vec<String>,

        /// Current rex context.
        pub context: RexContext,
    }
);
