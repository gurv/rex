use crate::context::RexContext;
use crate::prompts::SettingPrompt;
use rex_warpgate_api::*;
use rustc_hash::FxHashMap;
use schematic::Schema;

pub type ConfigSchema = Schema;

api_struct!(
    /// Input passed to the initialize functions.
    pub struct InitializePluginInput {
        /// Current rex context.
        pub context: RexContext,
    }
);

api_struct!(
    /// Output returned from the initialize functions.
    #[serde(default)]
    pub struct InitializePluginOutput {
        /// A URL to documentation about available configuration settings.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub config_url: Option<String>,

        /// Settings to include in the injected toolchain config file.
        /// Supports dot notation for the keys.
        #[serde(skip_serializing_if = "FxHashMap::is_empty")]
        pub default_settings: FxHashMap<String, serde_json::Value>,

        /// A URL to documentation about the toolchain.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub docs_url: Option<String>,

        /// A list of questions to prompt the user about configuration
        /// settings and the values to inject.
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub prompts: Vec<SettingPrompt>,
    }
);

api_enum!(
    /// Type of extend/merge strategy.
    #[serde(tag = "strategy", content = "value")]
    pub enum Extend<T> {
        /// Empty the data.
        Empty,

        /// Append to the data.
        Append(T),

        /// Prepend to the data.
        Prepend(T),

        /// Replace the data.
        Replace(T),
    }
);
