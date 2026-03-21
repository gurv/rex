use starbase_styles::{Style, Stylize};
use starbase_utils::fs::FsError;
use starbase_utils::json::JsonError;
use thiserror::Error;

#[derive(Error, Debug, miette::Diagnostic)]
pub enum RexRegistryError {
    #[diagnostic(transparent)]
    #[error(transparent)]
    Fs(#[from] Box<FsError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Json(#[from] Box<JsonError>),

    #[diagnostic(code(rex::registry::parse_failed))]
    #[error("Failed to parse registry plugin data.")]
    FailedParse {
        #[source]
        error: Box<reqwest::Error>,
    },

    #[diagnostic(code(rex::registry::request_failed))]
    #[error("Failed to request plugins from registry {}.", .url.style(Style::Url))]
    FailedRequest {
        url: String,
        #[source]
        error: Box<reqwest::Error>,
    },
}

impl From<FsError> for RexRegistryError {
    fn from(e: FsError) -> RexRegistryError {
        RexRegistryError::Fs(Box::new(e))
    }
}

impl From<JsonError> for RexRegistryError {
    fn from(e: JsonError) -> RexRegistryError {
        RexRegistryError::Json(Box::new(e))
    }
}
