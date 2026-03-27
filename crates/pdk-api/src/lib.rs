mod common;
mod context;
mod extension;
mod host;
mod macros;
mod prompts;

pub use common::*;
pub use context::*;
pub use extension::*;
pub use host::*;
pub use rex_common::Id;
pub use prompts::*;
pub use rex_warpgate_api::*;

pub(crate) fn is_false(value: &bool) -> bool {
    !(*value)
}

pub(crate) fn is_zero(value: &u8) -> bool {
    *value == 0
}
