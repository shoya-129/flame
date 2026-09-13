use std::fs;
use std::path::Path;
use super::meta::PluginSpec;
pub use crate::utils::manifest::{parse_manifest_permissions, parse_section_entries};

fn parse_plugin_entries(content: &str) -> Vec<(String, String)> {
    parse_section_entries(content, "[plugins]")
}

pub fn list_plugins() -> Vec<PluginSpec> {
    let toml_path = Path::new("flame.toml");
    if !toml_path.exists() {
        return Vec::new();
    }

    let content = match fs::read_to_string(toml_path) {
        Ok(content) => content,
        Err(_) => return Vec::new(),
    };

    parse_plugin_entries(&content)
        .into_iter()
        .map(|(name, source)| {
            let version = if source == "*" || source.is_empty() {
                None
            } else if let Some((_, version)) = source.rsplit_once('@') {
                Some(version.to_string())
            } else {
                None
            };
            let is_local = source.starts_with('.') || source.starts_with('/') || source == "*";
            PluginSpec {
                name,
                source,
                version,
                is_local,
            }
        })
        .collect()
}

