use schematic::ConfigError;
use starbase_styles::{Style, Stylize};
use starbase_utils::fs::FsError;
use starbase_utils::toml::TomlError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, miette::Diagnostic)]
pub enum RexConfigError {
    #[diagnostic(transparent)]
    #[error(transparent)]
    Fs(#[from] Box<FsError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Schematic(#[from] Box<ConfigError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Toml(#[from] Box<TomlError>),

    #[diagnostic(code(rex::config::lockfile_already_exists))]
    #[error(
        "Unable to lock the directory {} as a lock file already exists in the child directory {}. Nested lock files are not supported. Instead, lock the parent directory.",
        .parent_dir.style(Style::Path),
        .child_dir.style(Style::Path),
    )]
    AlreadyLocked {
        child_dir: PathBuf,
        parent_dir: PathBuf,
    },

    #[diagnostic(code(rex::config::env_parse_failed))]
    #[error(
        "Failed to parse .env file {}.",
        .path.style(Style::Path),
    )]
    FailedParseEnvFile {
        path: PathBuf,
        #[source]
        error: Box<dotenvy::Error>,
    },

    #[diagnostic(code(rex::config::failed_update))]
    #[error(
        "Failed to update config {}.",
        .path.style(Style::Path),
    )]
    FailedUpdate {
        path: PathBuf,
        #[source]
        error: Box<toml_edit::TomlError>,
    },

    #[diagnostic(code(rex::config::missing_env_file))]
    #[error(
        "The .env file {} does not exist. This was configured as {} in the config {}.",
        .path.style(Style::Path),
        .config.style(Style::File),
        .config_path.style(Style::Path),
    )]
    MissingEnvFile {
        path: PathBuf,
        config: String,
        config_path: PathBuf,
    },
}

impl From<FsError> for RexConfigError {
    fn from(e: FsError) -> RexConfigError {
        RexConfigError::Fs(Box::new(e))
    }
}

impl From<ConfigError> for RexConfigError {
    fn from(e: ConfigError) -> RexConfigError {
        RexConfigError::Schematic(Box::new(e))
    }
}

impl From<TomlError> for RexConfigError {
    fn from(e: TomlError) -> RexConfigError {
        RexConfigError::Toml(Box::new(e))
    }
}
