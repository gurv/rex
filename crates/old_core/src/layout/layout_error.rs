use starbase_styles::{Style, Stylize};
use starbase_utils::fs::FsError;
use starbase_utils::json::JsonError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, miette::Diagnostic)]
pub enum RexLayoutError {
    #[diagnostic(transparent)]
    #[error(transparent)]
    Fs(#[from] Box<FsError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Json(#[from] Box<JsonError>),

    #[diagnostic(code(rex::store::shim::create_failed))]
    #[error("Failed to create shim {}.", .path.style(Style::Path))]
    FailedCreateShim {
        path: PathBuf,
        #[source]
        error: Box<std::io::Error>,
    },

    #[diagnostic(code(rex::store::shim::missing_binary))]
    #[error(
        "Unable to create shims as the {} binary cannot be found.\nLooked in the {} environment variable and {} directory.",
        "rex-shim".style(Style::Id),
        "REX_HOME".style(Style::Property),
        .bin_dir.style(Style::Path),
    )]
    MissingShimBinary { bin_dir: PathBuf },
}

impl From<FsError> for RexLayoutError {
    fn from(e: FsError) -> RexLayoutError {
        RexLayoutError::Fs(Box::new(e))
    }
}

impl From<JsonError> for RexLayoutError {
    fn from(e: JsonError) -> RexLayoutError {
        RexLayoutError::Json(Box::new(e))
    }
}
