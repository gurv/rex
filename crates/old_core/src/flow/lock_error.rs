#![allow(unused_assignments)]

use crate::config_error::RexConfigError;
use starbase_styles::{Style, Stylize};
use starbase_utils::toml::TomlError;
use thiserror::Error;

#[derive(Error, Debug, miette::Diagnostic)]
pub enum RexLockError {
    #[diagnostic(transparent)]
    #[error(transparent)]
    Config(#[from] Box<RexConfigError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Toml(#[from] Box<TomlError>),

    #[diagnostic(
        code(rex::install::mismatched_checksum),
        help = "Is this install legitimate?"
    )]
    #[error(
        "Checksum mismatch! Received {} but expected {}.",
        .checksum.style(Style::Hash),
        .lockfile_checksum.style(Style::Hash),
    )]
    MismatchedChecksum {
        checksum: String,
        lockfile_checksum: String,
    },

    #[diagnostic(
        code(rex::install::mismatched_checksum),
        help = "Is this install legitimate?"
    )]
    #[error(
        "Checksum mismatch for {}! Received {} but expected {}.",
        .source_url.style(Style::Url),
        .checksum.style(Style::Hash),
        .lockfile_checksum.style(Style::Hash),
    )]
    MismatchedChecksumWithSource {
        checksum: String,
        lockfile_checksum: String,
        source_url: String,
    },

    #[diagnostic(code(rex::install::mismatched_arch))]
    #[error(
        "System architecture mismatch! Received {} but expected {}.",
        .arch.style(Style::Hash),
        .lockfile_arch.style(Style::Hash),
    )]
    MismatchedArch { arch: String, lockfile_arch: String },

    #[diagnostic(code(rex::install::mismatched_os))]
    #[error(
        "Operating system mismatch! Received {} but expected {}.",
        .os.style(Style::Hash),
        .lockfile_os.style(Style::Hash),
    )]
    MismatchedOs { os: String, lockfile_os: String },
}

impl From<RexConfigError> for RexLockError {
    fn from(e: RexConfigError) -> RexLockError {
        RexLockError::Config(Box::new(e))
    }
}

impl From<TomlError> for RexLockError {
    fn from(e: TomlError) -> RexLockError {
        RexLockError::Toml(Box::new(e))
    }
}
