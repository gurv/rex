use miette::IntoDiagnostic;
use rex_common::Id;
use serde::Serialize;
use starbase_utils::{fs, json, toml, yaml};
use std::path::{Path, PathBuf};

pub fn serialize_config_based_on_extension(
    plugin_id: &Id,
    path: &Path,
    config: impl Serialize,
) -> miette::Result<String> {
    let template = match path.extension().and_then(|ext| ext.to_str()) {
        Some("json" | "jsonc") => json::format(
            &json::JsonMap::from_iter([(
                plugin_id.to_string(),
                json::serde_json::to_value(config).into_diagnostic()?,
            )]),
            true,
        )
        .into_diagnostic()?,
        Some("toml") => toml::format(
            &toml::TomlTable::from_iter([(
                plugin_id.to_string(),
                toml::TomlValue::try_from(config).into_diagnostic()?,
            )]),
            true,
        )
        .into_diagnostic()?,
        Some("yml" | "yaml") => yaml::format(&yaml::YamlMapping::from_iter([(
            yaml::YamlValue::String(plugin_id.to_string()),
            yaml::serde_yaml::to_value(config).into_diagnostic()?,
        )]))
        .into_diagnostic()?,
        _ => unimplemented!(),
    };

    Ok(template)
}

pub fn append_plugin_to_config_file(
    plugin_id: &Id,
    config_paths: Vec<PathBuf>,
    config: impl Serialize,
) -> miette::Result<PathBuf> {
    let path = config_paths
        .iter()
        .find(|path| path.exists())
        .unwrap_or(&config_paths[0]);

    fs::append_file(
        path,
        format!(
            "\n\n{}",
            serialize_config_based_on_extension(plugin_id, path, config)?
        ),
    )?;

    Ok(path.to_path_buf())
}
