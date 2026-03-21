#![allow(unused_assignments)]

use crate::config_error::RexConfigError;
use starbase_styles::{Style, Stylize};
use starbase_utils::fs::FsError;
use std::path::PathBuf;
use thiserror::Error;
use rex_warpgate::WarpgatePluginError;

#[derive(Error, Debug, miette::Diagnostic)]
pub enum RexDetectError {
    #[diagnostic(transparent)]
    #[error(transparent)]
    Config(#[from] Box<RexConfigError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Fs(#[from] Box<FsError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Plugin(#[from] Box<WarpgatePluginError>),

    #[diagnostic(code(rex::detect::invalid_version))]
    #[error(
      "Invalid version or requirement {} detected from {}.",
      .version.style(Style::Hash),
      .path.style(Style::Path),
    )]
    InvalidDetectedVersionSpec {
        #[source]
        error: Box<rex_version_spec::SpecError>,
        path: PathBuf,
        version: String,
    },

    #[diagnostic(code(rex::detect::failed), help = "Has the tool been installed?")]
    #[error(
        "Failed to detect an applicable version to run {tool} with. Try pinning a version with {} or explicitly passing the version as an argument or environment variable.",
        "rex pin".style(Style::Shell),
    )]
    FailedVersionDetect { tool: String },
}

impl From<RexConfigError> for RexDetectError {
    fn from(e: RexConfigError) -> RexDetectError {
        RexDetectError::Config(Box::new(e))
    }
}

impl From<FsError> for RexDetectError {
    fn from(e: FsError) -> RexDetectError {
        RexDetectError::Fs(Box::new(e))
    }
}

impl From<WarpgatePluginError> for RexDetectError {
    fn from(e: WarpgatePluginError) -> RexDetectError {
        RexDetectError::Plugin(Box::new(e))
    }
}
