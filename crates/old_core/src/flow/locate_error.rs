use starbase_styles::{Style, Stylize};
use starbase_utils::fs::FsError;
use std::path::PathBuf;
use thiserror::Error;
use rex_warpgate::WarpgatePluginError;

#[derive(Error, Debug, miette::Diagnostic)]
pub enum RexLocateError {
    #[diagnostic(transparent)]
    #[error(transparent)]
    Fs(#[from] Box<FsError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Plugin(#[from] Box<WarpgatePluginError>),

    #[diagnostic(code(rex::locate::missing_executable))]
    #[error(
      "Unable to find an executable for {tool}, expected file {} does not exist.",
      .path.style(Style::Path),
    )]
    MissingToolExecutable { tool: String, path: PathBuf },

    #[diagnostic(code(rex::locate::no_primary_executable))]
    #[error(
      "{tool} does not support a primary (default) executable. You can run a secondary executable by passing {} with the executable name.",
      "--exe".style(Style::Shell),
    )]
    NoPrimaryExecutable { tool: String },
}

impl From<FsError> for RexLocateError {
    fn from(e: FsError) -> RexLocateError {
        RexLocateError::Fs(Box::new(e))
    }
}

impl From<WarpgatePluginError> for RexLocateError {
    fn from(e: WarpgatePluginError) -> RexLocateError {
        RexLocateError::Plugin(Box::new(e))
    }
}
