use crate::config_error::RexConfigError;
use crate::id::Id;
use crate::layout::RexLayoutError;
use crate::utils::archive::RexArchiveError;
use crate::utils::process::RexProcessError;
use starbase_styles::{Style, Stylize};
use starbase_utils::fs::FsError;
use std::path::PathBuf;
use thiserror::Error;
use rex_warpgate::{WarpgateHttpClientError, WarpgatePluginError};

#[derive(Error, Debug, miette::Diagnostic)]
pub enum RexToolError {
    #[diagnostic(transparent)]
    #[error(transparent)]
    Archive(#[from] Box<RexArchiveError>),

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
    Layout(#[from] Box<RexLayoutError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Plugin(#[from] Box<WarpgatePluginError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Process(#[from] Box<RexProcessError>),

    #[diagnostic(code(rex::tool::minimum_version_requirement))]
    #[error(
        "Unable to use the {tool} plugin with identifier {}, as it requires a minimum rex version of {}, but found {} instead.",
        .id.to_string().style(Style::Id),
        .expected.style(Style::Hash),
        .actual.style(Style::Hash)
    )]
    InvalidMinimumVersion {
        tool: String,
        id: Id,
        expected: String,
        actual: String,
    },

    #[diagnostic(code(rex::tool::invalid_inventory_dir))]
    #[error(
        "{tool} inventory directory has been overridden with {} but it's not an absolute path. Only absolute paths are supported.",
        .dir.style(Style::Path),
    )]
    RequiredAbsoluteInventoryDir { tool: String, dir: PathBuf },
}

impl From<RexArchiveError> for RexToolError {
    fn from(e: RexArchiveError) -> RexToolError {
        RexToolError::Archive(Box::new(e))
    }
}

impl From<WarpgateHttpClientError> for RexToolError {
    fn from(e: WarpgateHttpClientError) -> RexToolError {
        RexToolError::HttpClient(Box::new(e))
    }
}

impl From<RexConfigError> for RexToolError {
    fn from(e: RexConfigError) -> RexToolError {
        RexToolError::Config(Box::new(e))
    }
}

impl From<FsError> for RexToolError {
    fn from(e: FsError) -> RexToolError {
        RexToolError::Fs(Box::new(e))
    }
}

impl From<RexLayoutError> for RexToolError {
    fn from(e: RexLayoutError) -> RexToolError {
        RexToolError::Layout(Box::new(e))
    }
}

impl From<WarpgatePluginError> for RexToolError {
    fn from(e: WarpgatePluginError) -> RexToolError {
        RexToolError::Plugin(Box::new(e))
    }
}

impl From<RexProcessError> for RexToolError {
    fn from(e: RexProcessError) -> RexToolError {
        RexToolError::Process(Box::new(e))
    }
}
