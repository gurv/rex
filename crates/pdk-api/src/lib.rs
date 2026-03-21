mod action;
mod api;
mod common;
mod context;
mod error;
mod extemsion;
mod hooks;
mod host;
mod prompts;
mod shapes;
mod subcommand;

pub use action::*;
pub use api::*;
pub use common::*;
pub use context::*;
pub use error::*;
pub use extemsion::*;
pub use hooks::*;
pub use host::*;
pub use shapes::*;
// pub use prompts::*;
pub use rex_system_env::{
    DependencyConfig, DependencyName, SystemDependency, SystemPackageManager as HostPackageManager,
};
pub use rex_version_spec::*;
pub use rex_warpgate_api::*;
pub use subcommand::*;

pub(crate) fn is_false(value: &bool) -> bool {
    !(*value)
}

pub(crate) fn is_zero(value: &u8) -> bool {
    *value == 0
}
