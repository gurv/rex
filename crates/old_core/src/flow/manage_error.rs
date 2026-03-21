use crate::config_error::RexConfigError;
use crate::flow::install::RexInstallError;
use crate::flow::link::RexLinkError;
use crate::flow::locate::RexLocateError;
use crate::flow::lock::RexLockError;
use crate::flow::resolve::RexResolveError;
use crate::layout::RexLayoutError;
use starbase_utils::json::JsonError;
use thiserror::Error;
use rex_warpgate::WarpgatePluginError;

#[derive(Error, Debug, miette::Diagnostic)]
pub enum RexManageError {
    #[diagnostic(transparent)]
    #[error(transparent)]
    Config(#[from] Box<RexConfigError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Install(#[from] Box<RexInstallError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Json(#[from] Box<JsonError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Layout(#[from] Box<RexLayoutError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Link(#[from] Box<RexLinkError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Locate(#[from] Box<RexLocateError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Lock(#[from] Box<RexLockError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Plugin(#[from] Box<WarpgatePluginError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Resolve(#[from] Box<RexResolveError>),
}

impl From<RexConfigError> for RexManageError {
    fn from(e: RexConfigError) -> RexManageError {
        RexManageError::Config(Box::new(e))
    }
}

impl From<RexInstallError> for RexManageError {
    fn from(e: RexInstallError) -> RexManageError {
        RexManageError::Install(Box::new(e))
    }
}

impl From<JsonError> for RexManageError {
    fn from(e: JsonError) -> RexManageError {
        RexManageError::Json(Box::new(e))
    }
}

impl From<RexLayoutError> for RexManageError {
    fn from(e: RexLayoutError) -> RexManageError {
        RexManageError::Layout(Box::new(e))
    }
}

impl From<RexLinkError> for RexManageError {
    fn from(e: RexLinkError) -> RexManageError {
        RexManageError::Link(Box::new(e))
    }
}

impl From<RexLocateError> for RexManageError {
    fn from(e: RexLocateError) -> RexManageError {
        RexManageError::Locate(Box::new(e))
    }
}

impl From<RexLockError> for RexManageError {
    fn from(e: RexLockError) -> RexManageError {
        RexManageError::Lock(Box::new(e))
    }
}

impl From<WarpgatePluginError> for RexManageError {
    fn from(e: WarpgatePluginError) -> RexManageError {
        RexManageError::Plugin(Box::new(e))
    }
}

impl From<RexResolveError> for RexManageError {
    fn from(e: RexResolveError) -> RexManageError {
        RexManageError::Resolve(Box::new(e))
    }
}
