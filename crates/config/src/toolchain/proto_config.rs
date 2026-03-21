use crate::config_struct;
use rex_version_spec::VersionSpec;
use schematic::{Config, DefaultValueResult};

fn default_version(_: &()) -> DefaultValueResult<VersionSpec> {
    Ok(VersionSpec::parse("0.55.4").ok())
}

config_struct!(
    /// Configures how rex integrates with proto.
    #[derive(Config)]
    pub struct ProtoConfig {
        /// The version of proto to download and install,
        /// and to use for installing and running other toolchains.
        #[setting(default = default_version)]
        pub version: VersionSpec,
    }
);
