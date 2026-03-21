use super::build_error::RexBuildError;
use super::lock_error::RexLockError;
use crate::checksum::RexChecksumError;
use crate::config_error::RexConfigError;
use crate::utils::archive::RexArchiveError;
use crate::utils::process::RexProcessError;
use starbase_styles::{Style, Stylize, apply_style_tags};
use starbase_utils::fs::FsError;
use starbase_utils::net::NetError;
use std::path::PathBuf;
use thiserror::Error;
use rex_warpgate::{WarpgateHttpClientError, WarpgatePluginError};

#[derive(Error, Debug, miette::Diagnostic)]
pub enum RexInstallError {
    #[diagnostic(transparent)]
    #[error(transparent)]
    Archive(#[from] Box<RexArchiveError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Build(#[from] Box<RexBuildError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Checksum(#[from] Box<RexChecksumError>),

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
    Net(#[from] Box<NetError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Lock(#[from] Box<RexLockError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Plugin(#[from] Box<WarpgatePluginError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Process(#[from] Box<RexProcessError>),

    #[diagnostic(code(rex::install::failed))]
    #[error("Failed to install {tool}. {}", apply_style_tags(.error))]
    FailedInstall { tool: String, error: String },

    #[diagnostic(code(rex::uninstall::failed))]
    #[error("Failed to uninstall {tool}. {}", apply_style_tags(.error))]
    FailedUninstall { tool: String, error: String },

    #[diagnostic(code(rex::install::invalid_checksum))]
    #[error(
        "Checksum has failed for {}, which was verified using {}.",
        .download.style(Style::Path),
        .checksum.style(Style::Path),
    )]
    InvalidChecksum {
        checksum: PathBuf,
        download: PathBuf,
    },

    #[diagnostic(code(rex::install::prebuilt_unsupported))]
    #[error("Downloading a pre-built is not supported for {tool}. Try building from source by passing {}.", "--build".style(Style::Shell))]
    UnsupportedDownloadPrebuilt { tool: String },

    #[diagnostic(code(rex::install::build_unsupported))]
    #[error("Building from source is not supported for {tool}. Try downloading a pre-built by passing {}.", "--no-build".style(Style::Shell))]
    UnsupportedBuildFromSource { tool: String },

    #[diagnostic(code(rex::offline))]
    #[error("Internet connection required, unable to download, install, or run tools.")]
    RequiredInternetConnection,
}

impl From<RexArchiveError> for RexInstallError {
    fn from(e: RexArchiveError) -> RexInstallError {
        RexInstallError::Archive(Box::new(e))
    }
}

impl From<RexBuildError> for RexInstallError {
    fn from(e: RexBuildError) -> RexInstallError {
        RexInstallError::Build(Box::new(e))
    }
}

impl From<RexChecksumError> for RexInstallError {
    fn from(e: RexChecksumError) -> RexInstallError {
        RexInstallError::Checksum(Box::new(e))
    }
}

impl From<WarpgateHttpClientError> for RexInstallError {
    fn from(e: WarpgateHttpClientError) -> RexInstallError {
        RexInstallError::HttpClient(Box::new(e))
    }
}

impl From<RexConfigError> for RexInstallError {
    fn from(e: RexConfigError) -> RexInstallError {
        RexInstallError::Config(Box::new(e))
    }
}

impl From<FsError> for RexInstallError {
    fn from(e: FsError) -> RexInstallError {
        RexInstallError::Fs(Box::new(e))
    }
}

impl From<RexLockError> for RexInstallError {
    fn from(e: RexLockError) -> RexInstallError {
        RexInstallError::Lock(Box::new(e))
    }
}

impl From<NetError> for RexInstallError {
    fn from(e: NetError) -> RexInstallError {
        RexInstallError::Net(Box::new(e))
    }
}

impl From<WarpgatePluginError> for RexInstallError {
    fn from(e: WarpgatePluginError) -> RexInstallError {
        RexInstallError::Plugin(Box::new(e))
    }
}

impl From<RexProcessError> for RexInstallError {
    fn from(e: RexProcessError) -> RexInstallError {
        RexInstallError::Process(Box::new(e))
    }
}
