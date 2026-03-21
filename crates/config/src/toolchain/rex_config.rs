use crate::config_struct;
use schematic::{Config, validate};

config_struct!(
    /// Configures how and where rex updates will be received.
    #[derive(Config)]
    pub struct MoonConfig {
        /// A secure URL to lookup the latest available version.
        #[setting(validate = validate::url_secure, default = "https://launch.rexrepo.app/rex/check_version")]
        pub manifest_url: String,

        /// A secure URL for downloading the rex binary itself.
        #[setting(validate = validate::url_secure, default = "https://github.com/rexrepo/rex/releases/latest/download/{file}")]
        pub download_url: String,
    }
);
