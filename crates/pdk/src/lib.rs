mod args;
mod action;
mod helpers;
mod funcs;
mod macros;
mod subcommand;

pub use args::*;
pub use action::*;
pub use helpers::*;
pub use funcs::*;
pub use rex_pdk_api::*;
pub use rex_warpgate_pdk::*;
pub use subcommand::*;

/// Map a `miette` (or similar error) to an `extism` Error.
pub fn map_miette_error(error: impl std::fmt::Display) -> extism_pdk::Error {
    anyhow!("{error}")
}
