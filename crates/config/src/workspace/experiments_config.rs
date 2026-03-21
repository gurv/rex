use crate::config_struct;
use schematic::Config;

config_struct!(
    /// Configures experiments across the entire rex workspace.
    #[derive(Config)]
    pub struct ExperimentsConfig {}
);
