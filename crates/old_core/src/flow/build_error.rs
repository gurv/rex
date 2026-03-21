use crate::id::Id;
use crate::utils::archive::RexArchiveError;
use crate::utils::process::RexProcessError;
use starbase_console::ConsoleError;
use starbase_styles::{Style, Stylize};
use starbase_utils::fs::FsError;
use starbase_utils::net::NetError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, miette::Diagnostic)]
pub enum RexBuildError {
    #[diagnostic(transparent)]
    #[error(transparent)]
    Archive(#[from] Box<RexArchiveError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Console(#[from] Box<ConsoleError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Fs(#[from] Box<FsError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Net(#[from] Box<NetError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Process(#[from] Box<RexProcessError>),

    #[error(transparent)]
    System(#[from] Box<rex_system_env::Error>),

    #[diagnostic(code(rex::install::build::parse_version_failed))]
    #[error("Failed to parse version from {}.", .value.style(Style::Symbol))]
    FailedVersionParse {
        value: String,
        #[source]
        error: Box<semver::Error>,
    },

    #[diagnostic(code(rex::install::build::missing_builder))]
    #[error("Builder {} has not been installed.", .id.as_str().to_string().style(Style::Id))]
    MissingBuilder { id: Id },

    #[diagnostic(code(rex::install::build::missing_builder_exe))]
    #[error(
        "Executable {} from builder {} does not exist.",
        .exe.style(Style::Path),
        .id.to_string().style(Style::Id),
    )]
    MissingBuilderExe { exe: PathBuf, id: Id },

    #[diagnostic(code(rex::install::build::unmet_requirements))]
    #[error(
        "Build requirements have not been met, unable to proceed.\nPlease satisfy the requirements before attempting the build again."
    )]
    RequirementsNotMet,

    #[diagnostic(code(rex::install::build::cancelled))]
    #[error("Build has been cancelled.")]
    Cancelled,
}

impl From<RexArchiveError> for RexBuildError {
    fn from(e: RexArchiveError) -> RexBuildError {
        RexBuildError::Archive(Box::new(e))
    }
}

impl From<ConsoleError> for RexBuildError {
    fn from(e: ConsoleError) -> RexBuildError {
        RexBuildError::Console(Box::new(e))
    }
}

impl From<FsError> for RexBuildError {
    fn from(e: FsError) -> RexBuildError {
        RexBuildError::Fs(Box::new(e))
    }
}

impl From<NetError> for RexBuildError {
    fn from(e: NetError) -> RexBuildError {
        RexBuildError::Net(Box::new(e))
    }
}

impl From<RexProcessError> for RexBuildError {
    fn from(e: RexProcessError) -> RexBuildError {
        RexBuildError::Process(Box::new(e))
    }
}

impl From<rex_system_env::Error> for RexBuildError {
    fn from(e: rex_system_env::Error) -> RexBuildError {
        RexBuildError::System(Box::new(e))
    }
}
