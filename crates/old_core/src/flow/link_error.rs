use super::locate_error::RexLocateError;
use crate::layout::RexLayoutError;
use starbase_styles::{Style, Stylize};
use starbase_utils::fs::FsError;
use starbase_utils::json::JsonError;
use thiserror::Error;

#[derive(Error, Debug, miette::Diagnostic)]
pub enum RexLinkError {
    #[diagnostic(transparent)]
    #[error(transparent)]
    Fs(#[from] Box<FsError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Json(#[from] Box<JsonError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Layout(#[from] Box<RexLayoutError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Locate(#[from] Box<RexLocateError>),

    #[diagnostic(code(rex::link::failed_args_parse))]
    #[error("Failed to parse shim arguments string {}.", .args.style(Style::Shell))]
    FailedArgsParse {
        args: String,
        #[source]
        error: Box<shell_words::ParseError>,
    },
}

impl From<FsError> for RexLinkError {
    fn from(e: FsError) -> RexLinkError {
        RexLinkError::Fs(Box::new(e))
    }
}

impl From<JsonError> for RexLinkError {
    fn from(e: JsonError) -> RexLinkError {
        RexLinkError::Json(Box::new(e))
    }
}

impl From<RexLayoutError> for RexLinkError {
    fn from(e: RexLayoutError) -> RexLinkError {
        RexLinkError::Layout(Box::new(e))
    }
}

impl From<RexLocateError> for RexLinkError {
    fn from(e: RexLocateError) -> RexLinkError {
        RexLinkError::Locate(Box::new(e))
    }
}
