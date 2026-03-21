#![allow(unused_assignments)]

use miette::Diagnostic;
use rex_old_core::flow::link::RexLinkError;
use rex_old_core::flow::manage::RexManageError;
use rex_old_core::flow::resolve::RexResolveError;
use rex_old_core::layout::RexLayoutError;
use rex_old_core::rex_warpgate::WarpgatePluginError;
use rex_old_core::{IdError, REX_CONFIG_NAME, RexConfigError};
use starbase_console::ConsoleError;
use starbase_shell::ShellError;
use starbase_styles::{Style, Stylize};
use starbase_utils::fs::FsError;
use std::path::PathBuf;
use thiserror::Error;

// Convention: <command><action><component>

#[derive(Error, Debug, Diagnostic)]
pub enum RexCliError {
    #[diagnostic(transparent)]
    #[error(transparent)]
    Config(#[from] Box<RexConfigError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Console(#[from] Box<ConsoleError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Fs(#[from] Box<FsError>),

    #[error(transparent)]
    Http(#[from] Box<reqwest::Error>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Id(#[from] Box<IdError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Layout(#[from] Box<RexLayoutError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Link(#[from] Box<RexLinkError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Manage(#[from] Box<RexManageError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Plugin(#[from] Box<WarpgatePluginError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Resolve(#[from] Box<RexResolveError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Shell(#[from] Box<ShellError>),

    #[diagnostic(code(rex::missing_tools_config))]
    #[error(
        "No {} has been found in current directory. Attempted to find at {}.",
        REX_CONFIG_NAME.style(Style::File),
        .path.style(Style::Path),
    )]
    MissingToolsConfigInCwd { path: PathBuf },
}

impl From<RexConfigError> for RexCliError {
    fn from(e: RexConfigError) -> RexCliError {
        RexCliError::Config(Box::new(e))
    }
}

impl From<ConsoleError> for RexCliError {
    fn from(e: ConsoleError) -> RexCliError {
        RexCliError::Console(Box::new(e))
    }
}

impl From<FsError> for RexCliError {
    fn from(e: FsError) -> RexCliError {
        RexCliError::Fs(Box::new(e))
    }
}

impl From<IdError> for RexCliError {
    fn from(e: IdError) -> RexCliError {
        RexCliError::Id(Box::new(e))
    }
}

impl From<reqwest::Error> for RexCliError {
    fn from(e: reqwest::Error) -> RexCliError {
        RexCliError::Http(Box::new(e))
    }
}

impl From<RexLayoutError> for RexCliError {
    fn from(e: RexLayoutError) -> RexCliError {
        RexCliError::Layout(Box::new(e))
    }
}

impl From<RexLinkError> for RexCliError {
    fn from(e: RexLinkError) -> RexCliError {
        RexCliError::Link(Box::new(e))
    }
}

impl From<WarpgatePluginError> for RexCliError {
    fn from(e: WarpgatePluginError) -> RexCliError {
        RexCliError::Plugin(Box::new(e))
    }
}

impl From<RexResolveError> for RexCliError {
    fn from(e: RexResolveError) -> RexCliError {
        RexCliError::Resolve(Box::new(e))
    }
}

impl From<RexManageError> for RexCliError {
    fn from(e: RexManageError) -> RexCliError {
        RexCliError::Manage(Box::new(e))
    }
}

impl From<ShellError> for RexCliError {
    fn from(e: ShellError) -> RexCliError {
        RexCliError::Shell(Box::new(e))
    }
}
