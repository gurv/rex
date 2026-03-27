#![allow(clippy::disallowed_types)] // schematic

mod config_finder;
#[cfg(feature = "loader")]
mod config_loader;
mod extensions_config;
#[cfg(feature = "loader")]
mod formats;
mod macros;
pub mod patterns;
mod shapes;

pub use config_finder::*;
#[cfg(feature = "loader")]
pub use config_loader::*;
pub use extensions_config::*;
pub use schematic;
pub use semver::{Version, VersionReq};
pub use shapes::*;
pub use rex_version_spec::{CalVer, SemVer, UnresolvedVersionSpec, VersionSpec};

use schematic::{Config, PartialConfig};

pub fn finalize_config<T: Config>(config: T::Partial) -> miette::Result<T> {
    Ok(T::from_partial(config.finalize(&Default::default())?))
}
