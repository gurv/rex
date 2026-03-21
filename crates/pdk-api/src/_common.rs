// use crate::context::RexContext;
// // use crate::prompts::SettingPrompt;
// use rustc_hash::FxHashMap;
// // use schematic::Schema;
// use rex_warpgate_api::*;
// use std::path::PathBuf;

// // pub type ConfigSchema = Schema;

// // INIT

// api_struct!(
//     /// Input passed to the initialize functions.
//     pub struct InitializePluginInput {
//         /// Current rex context.
//         pub context: RexContext,
//     }
// );

// api_struct!(
//     /// Output returned from the initialize functions.
//     #[serde(default)]
//     pub struct InitializePluginOutput {
//         /// A URL to documentation about available configuration settings.
//         #[serde(skip_serializing_if = "Option::is_none")]
//         pub config_url: Option<String>,

//         /// Settings to include in the injected config file.
//         /// Supports dot notation for the keys.
//         #[serde(skip_serializing_if = "FxHashMap::is_empty")]
//         pub default_settings: FxHashMap<String, serde_json::Value>,

//         /// A URL to documentation about the plugin.
//         #[serde(skip_serializing_if = "Option::is_none")]
//         pub docs_url: Option<String>,

//         // /// A list of questions to prompt the user about configuration
//         // /// settings and the values to inject.
//         // #[serde(skip_serializing_if = "Vec::is_empty")]
//         // pub prompts: Vec<SettingPrompt>,
//     }
// );

// api_enum!(
//     /// Type of extend/merge strategy.
//     #[serde(tag = "strategy", content = "value")]
//     pub enum Extend<T> {
//         /// Empty the data.
//         Empty,

//         /// Append to the data.
//         Append(T),

//         /// Prepend to the data.
//         Prepend(T),

//         /// Replace the data.
//         Replace(T),
//     }
// );

// api_struct!(
//     /// Input passed to the `extend_command` function.
//     pub struct ExtendCommandInput {
//         /// The current arguments, after the command.
//         pub args: Vec<String>,

//         /// Current rex context.
//         pub context: RexContext,

//         /// The current command (binary/program).
//         pub command: String,

//         /// The current working directory in which the command will run.
//         pub current_dir: VirtualPath,

//         /// Workspace extension configuration.
//         /// Is null when within toolchains.
//         pub extension_config: serde_json::Value,

//         /// Workspace and project merged toolchain configuration,
//         /// with the latter taking precedence. Is null when
//         /// within extensions.
//         pub toolchain_config: serde_json::Value,
//     }
// );

// api_struct!(
//     /// Output returned from the `extend_command` and `extend_task_command` functions.
//     #[serde(default)]
//     pub struct ExtendCommandOutput {
//         /// The command (binary/program) to use. Will replace the existing
//         /// command. Can be overwritten by subsequent extend calls.
//         #[serde(skip_serializing_if = "Option::is_none")]
//         pub command: Option<String>,

//         /// List of arguments to merge with.
//         /// Can be modified by subsequent extend calls.
//         #[serde(skip_serializing_if = "Option::is_none")]
//         pub args: Option<Extend<Vec<String>>>,

//         /// Map of environment variables to add.
//         /// Can be overwritten by subsequent extend calls.
//         #[serde(skip_serializing_if = "FxHashMap::is_empty")]
//         pub env: FxHashMap<String, String>,

//         /// List of environment variables to remove.
//         /// Can be overwritten by subsequent extend calls.
//         #[serde(skip_serializing_if = "Vec::is_empty")]
//         pub env_remove: Vec<String>,

//         /// List of absolute paths to prepend into the `PATH` environment
//         /// variable, but after the proto prepended paths. These *must*
//         /// be real paths, not virtual!
//         #[serde(skip_serializing_if = "Vec::is_empty")]
//         pub paths: Vec<PathBuf>,
//     }
// );
