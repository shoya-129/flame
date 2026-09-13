use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Finds the root directory of a Flame project by locating `flame.toml` or `Flame.toml`.
pub fn find_manifest_root(start: &Path) -> Option<PathBuf> {
    let mut current = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        if current.join("flame.toml").exists() || current.join("Flame.toml").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Parses key-value pairs inside a specified TOML section (e.g. `[plugins]`).
pub fn parse_manifest_section(content: &str, section_name: &str) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    let mut in_section = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == section_name {
            in_section = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_section = false;
            continue;
        }
        if !in_section || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((name, value)) = trimmed.split_once('=') {
            entries.push((
                name.trim().to_string(),
                value.trim().trim_matches('"').to_string(),
            ));
        }
    }

    entries
}

/// Alias for `parse_manifest_section` for backward compatibility.
pub fn parse_section_entries(content: &str, section_name: &str) -> Vec<(String, String)> {
    parse_manifest_section(content, section_name)
}

/// Parses declared permissions from `[permissions]` section of a manifest.
pub fn parse_manifest_permissions(content: &str) -> HashSet<String> {
    let mut perms = HashSet::new();
    let mut in_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[permissions]" {
            in_section = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_section = false;
            continue;
        }
        if !in_section || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(eq_idx) = trimmed.find('=') {
            let key = trimmed[..eq_idx].trim().to_string();
            let val = trimmed[eq_idx + 1..].trim();
            if val == "true" {
                perms.insert(key);
            }
        } else {
            perms.insert(trimmed.to_string());
        }
    }
    perms
}
