mod common;
mod context;
mod extension;
mod host;
mod macros;
mod prompts;
// mod toolchain;

pub use common::*;
pub use context::*;
pub use extension::*;
pub use host::*;
pub use rex_common::Id;
// pub use rex_project::ProjectFragment;
// pub use rex_task::TaskFragment;
pub use prompts::*;
pub use proto_pdk_api::{
    CalVer, ExecCommandInput, SemVer, UnresolvedVersionSpec, Version, VersionReq, VersionSpec,
};
// pub use toolchain::*;
pub use rex_warpgate_api::*;

pub(crate) fn is_false(value: &bool) -> bool {
    !(*value)
}

pub(crate) fn is_zero(value: &u8) -> bool {
    *value == 0
}
