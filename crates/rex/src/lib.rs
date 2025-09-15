#![doc = include_str!("../../../README.md")]

/// Command line interface implementations for rex
pub mod cli;
/// Configuration management for rex
pub mod config;

/// The current version of the rex package, set at build time
pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
