#![allow(unused_assignments)]

use crate::config_error::RexConfigError;
use crate::flow::lock::RexLockError;
use crate::id::IdError;
use crate::layout::RexLayoutError;
use starbase_styles::{Style, Stylize};
use starbase_utils::fs::FsError;
use std::path::PathBuf;
use thiserror::Error;
use rex_warpgate::{WarpgateHttpClientError, WarpgatePluginError};

#[derive(Error, Debug, miette::Diagnostic)]
pub enum RexResolveError {
    #[diagnostic(transparent)]
    #[error(transparent)]
    Config(#[from] Box<RexConfigError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Fs(#[from] Box<FsError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    HttpClient(#[from] Box<WarpgateHttpClientError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Id(#[from] Box<IdError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Layout(#[from] Box<RexLayoutError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Lock(#[from] Box<RexLockError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Plugin(#[from] Box<WarpgatePluginError>),

    #[diagnostic(code(rex::resolve::offline::version_required))]
    #[error(
        "Internet connection required to load and resolve a valid version. To work around this:\n - Pass a fully-qualified version explicitly: {}\n - Execute the non-shim binaries instead: {}",
        .command.style(Style::Shell),
        .bin_dir.style(Style::Path)
    )]
    RequiredInternetConnectionForVersion { command: String, bin_dir: PathBuf },

    #[diagnostic(code(rex::resolve::invalid_version))]
    #[error("Invalid version or requirement in tool specification {}.", .version.style(Style::Hash))]
    InvalidVersionSpec {
        version: String,
        #[source]
        error: Box<rex_version_spec::SpecError>,
    },

    #[diagnostic(
        code(rex::resolve::undetected_version),
        help = "Has the tool been installed?"
    )]
    #[error(
        "Failed to detect an applicable version to run {tool} with. Try pinning a version with {} or explicitly passing the version as an argument or environment variable.",
        "rex pin".style(Style::Shell),
    )]
    FailedVersionDetect { tool: String },

    #[diagnostic(
        code(rex::resolve::unresolved_version),
        help = "Does this version exist and has it been released?"
    )]
    #[error(
        "Failed to resolve {} to a valid supported version for {tool}.",
        .version.style(Style::Hash),
    )]
    FailedVersionResolve { tool: String, version: String },
}

impl From<WarpgateHttpClientError> for RexResolveError {
    fn from(e: WarpgateHttpClientError) -> RexResolveError {
        RexResolveError::HttpClient(Box::new(e))
    }
}

impl From<RexConfigError> for RexResolveError {
    fn from(e: RexConfigError) -> RexResolveError {
        RexResolveError::Config(Box::new(e))
    }
}

impl From<FsError> for RexResolveError {
    fn from(e: FsError) -> RexResolveError {
        RexResolveError::Fs(Box::new(e))
    }
}

impl From<RexLayoutError> for RexResolveError {
    fn from(e: RexLayoutError) -> RexResolveError {
        RexResolveError::Layout(Box::new(e))
    }
}

impl From<RexLockError> for RexResolveError {
    fn from(e: RexLockError) -> RexResolveError {
        RexResolveError::Lock(Box::new(e))
    }
}

impl From<WarpgatePluginError> for RexResolveError {
    fn from(e: WarpgatePluginError) -> RexResolveError {
        RexResolveError::Plugin(Box::new(e))
    }
}

impl From<IdError> for RexResolveError {
    fn from(e: IdError) -> RexResolveError {
        RexResolveError::Id(Box::new(e))
    }
}
