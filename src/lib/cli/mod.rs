use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, Serialize, Deserialize, PartialEq)]
pub enum OutputKind {
    Text,
    Json,
}

impl FromStr for OutputKind {
    type Err = OutputParseErr;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "json" => Ok(Self::Json),
            "text" => Ok(Self::Text),
            _ => Err(OutputParseErr),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputParseErr;

impl Error for OutputParseErr {}

impl std::fmt::Display for OutputParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "error parsing output type, see help for the list of accepted outputs"
        )
    }
}

#[derive(Default)]
pub struct CommandOutput {
    pub map: HashMap<String, serde_json::Value>,
    pub text: String,
}

impl CommandOutput {
    pub fn new<S: Into<String>>(text: S, map: HashMap<String, serde_json::Value>) -> Self {
        Self {
            map,
            text: text.into(),
        }
    }

    pub fn from_key_and_text<K: Into<String>, S: Into<String>>(key: K, text: S) -> Self {
        let text_string: String = text.into();
        let mut map = HashMap::new();
        map.insert(key.into(), serde_json::Value::String(text_string.clone()));
        Self {
            map,
            text: text_string,
        }
    }
}

impl From<String> for CommandOutput {
    fn from(text: String) -> Self {
        let mut map = HashMap::new();
        map.insert(
            "result".to_string(),
            serde_json::Value::String(text.clone()),
        );
        Self { map, text }
    }
}

impl From<&str> for CommandOutput {
    fn from(text: &str) -> Self {
        Self::from(text.to_string())
    }
}
