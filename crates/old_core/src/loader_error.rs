use crate::config::REX_CONFIG_NAME;
use crate::config_error::RexConfigError;
use crate::flow::resolve::RexResolveError;
use crate::id::Id;
use crate::tool_error::RexToolError;
use starbase_styles::{Style, Stylize};
use starbase_utils::json::JsonError;
use starbase_utils::toml::TomlError;
use starbase_utils::yaml::YamlError;
use thiserror::Error;
use rex_warpgate::{WarpgateLoaderError, WarpgatePluginError};

#[derive(Error, Debug, miette::Diagnostic)]
pub enum RexLoaderError {
    #[diagnostic(transparent)]
    #[error(transparent)]
    Config(#[from] Box<RexConfigError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Json(#[from] Box<JsonError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Loader(#[from] Box<WarpgateLoaderError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Plugin(#[from] Box<WarpgatePluginError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Resolve(#[from] Box<RexResolveError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Toml(#[from] Box<TomlError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Tool(#[from] Box<RexToolError>),

    #[diagnostic(transparent)]
    #[error(transparent)]
    Yaml(#[from] Box<YamlError>),

    #[diagnostic(code(rex::tool::unknown_id))]
    #[error(
        "Unable to proceed, {} is not a built-in plugin and has not been configured with {} in a {} file.\n\nLearn more about plugins: {}\nSearch community plugins: {}",
        .id.to_string().style(Style::Id),
        "[plugins]".style(Style::Property),
        REX_CONFIG_NAME.style(Style::File),
        "https://moonrepo.dev/docs/rex/plugins".style(Style::Url),
        format!("rex plugin search {}", .id).style(Style::Shell),
    )]
    UnknownTool { id: Id },
}

impl From<RexConfigError> for RexLoaderError {
    fn from(e: RexConfigError) -> RexLoaderError {
        RexLoaderError::Config(Box::new(e))
    }
}

impl From<JsonError> for RexLoaderError {
    fn from(e: JsonError) -> RexLoaderError {
        RexLoaderError::Json(Box::new(e))
    }
}

impl From<WarpgateLoaderError> for RexLoaderError {
    fn from(e: WarpgateLoaderError) -> RexLoaderError {
        RexLoaderError::Loader(Box::new(e))
    }
}

impl From<WarpgatePluginError> for RexLoaderError {
    fn from(e: WarpgatePluginError) -> RexLoaderError {
        RexLoaderError::Plugin(Box::new(e))
    }
}

impl From<RexResolveError> for RexLoaderError {
    fn from(e: RexResolveError) -> RexLoaderError {
        RexLoaderError::Resolve(Box::new(e))
    }
}

impl From<TomlError> for RexLoaderError {
    fn from(e: TomlError) -> RexLoaderError {
        RexLoaderError::Toml(Box::new(e))
    }
}

impl From<RexToolError> for RexLoaderError {
    fn from(e: RexToolError) -> RexLoaderError {
        RexLoaderError::Tool(Box::new(e))
    }
}

impl From<YamlError> for RexLoaderError {
    fn from(e: YamlError) -> RexLoaderError {
        RexLoaderError::Yaml(Box::new(e))
    }
}
