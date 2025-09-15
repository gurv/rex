#![doc = include_str!("../../../README.md")]

/// Command line interface implementations for rex
pub mod cli;
/// Build Wasm components
pub mod component_build;
/// Configuration management for rex
pub mod config;
/// Implementations for the developer loop, including component plugin management
pub mod dev;
/// Component inspection and analysis
pub mod inspect;
/// OCI registry operations for WebAssembly components
pub mod oci;
/// Plugin management for rex
pub mod plugin;
/// [`wasmcloud_runtime::Runtime`] management for rex
pub mod runtime;

/// Manage WebAssembly Interface Types (WIT) for wash components
pub(crate) mod wit;

/// The current version of the rex package, set at build time
pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
