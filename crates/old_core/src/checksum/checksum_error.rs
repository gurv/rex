#![allow(unused_assignments)]

use starbase_styles::{Style, Stylize};
use starbase_utils::fs::FsError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, miette::Diagnostic)]
pub enum RexChecksumError {
    #[diagnostic(transparent)]
    #[error(transparent)]
    Fs(#[from] Box<FsError>),

    #[diagnostic(code(rex::checksum::minisign))]
    #[error("Failed to verify minisign checksum.")]
    Minisign {
        #[source]
        error: Box<minisign_verify::Error>,
    },

    #[diagnostic(code(rex::checksum::sha))]
    #[error("Failed to verify SHA checksum.")]
    Sha {
        #[source]
        error: Box<FsError>,
    },

    #[diagnostic(code(rex::checksum::missing_public_key))]
    #[error(
        "A {} is required to verify this tool. This setting must be implemented in the plugin.", "checksum_public_key".style(Style::Property)
    )]
    MissingPublicKey,

    #[diagnostic(
        code(rex::checksum::unknown_algorithm),
        help = "Try using a more explicit file extension."
    )]
    #[error(
        "Unknown checksum algorithm. Unable to derive from {}.",
        .path.style(Style::Path)
    )]
    UnknownAlgorithm { path: PathBuf },

    #[diagnostic(code(rex::checksum::unsupported_algorithm))]
    #[error(
        "Unsupported checksum algorithm {}.",
        .algo.style(Style::Symbol)
    )]
    UnsupportedAlgorithm { algo: String },
}

impl From<FsError> for RexChecksumError {
    fn from(e: FsError) -> RexChecksumError {
        RexChecksumError::Fs(Box::new(e))
    }
}
