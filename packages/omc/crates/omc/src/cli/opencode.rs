use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::{OpencodeAction, OpencodeCommand};

const PLUGIN_NAME: &str = "oh-my-codes-opencode";

/// Oldest OpenCode 1.x release that supports object-shaped plugin entrypoints.
const MIN_OBJECT_ENTRY_VERSION: (u64, u64, u64) = (1, 18, 29);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenCodeVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl fmt::Display for OpenCodeVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl OpenCodeVersion {
    fn supports_dual_shape(self) -> bool {
        if self.major >= 2 {
            return true;
        }
        if self.major != 1 {
            return false;
        }
        (self.minor, self.patch) >= (MIN_OBJECT_ENTRY_VERSION.1, MIN_OBJECT_ENTRY_VERSION.2)
    }
}

fn parse_opencode_version(output: &str) -> Option<OpenCodeVersion> {
    for token in output.split_whitespace() {
        let trimmed = token.trim_start_matches(['v', 'V']);
        let parts: Vec<&str> = trimmed.split('.').collect();
        if parts.len() < 2 {
            continue;
        }
        let Ok(major) = parts[0].parse::<u64>() else {
            continue;
        };
        let Ok(minor) = parts[1].parse::<u64>() else {
            continue;
        };
        let patch = parts
            .get(2)
            .and_then(|p| p.split(['-', '+']).next())
            .and_then(|p| p.parse::<u64>().ok())
            .unwrap_or(0);
        return Some(OpenCodeVersion {
            major,
            minor,
            patch,
        });
    }
    None
}

fn detect_opencode_version() -> Option<OpenCodeVersion> {
    let output = Command::new("opencode").arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_opencode_version(&stdout)
        .or_else(|| parse_opencode_version(&String::from_utf8_lossy(&output.stderr)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigKey {
    /// OpenCode 1.x
    Plugin,
    /// OpenCode 2.x
    Plugins,
}

impl ConfigKey {
    fn as_str(self) -> &'static str {
        match self {
            ConfigKey::Plugin => "plugin",
            ConfigKey::Plugins => "plugins",
        }
    }
}

fn entry_matches(value: &serde_json::Value, name: &str) -> bool {
    if let Some(s) = value.as_str() {
        return s == name;
    }
    if let Some(arr) = value.as_array() {
        return arr.first().and_then(|f| f.as_str()) == Some(name);
    }
    if let Some(obj) = value.as_object() {
        return obj.get("package").and_then(|p| p.as_str()) == Some(name);
    }
    false
}

fn has_entry(value: &serde_json::Value, key: ConfigKey) -> bool {
    value
        .get(key.as_str())
        .and_then(|v| v.as_array())
        .is_some_and(|list| list.iter().any(|entry| entry_matches(entry, PLUGIN_NAME)))
}

fn remove_entry(value: &mut serde_json::Value, key: ConfigKey) -> usize {
    let Some(root) = value.as_object_mut() else {
        return 0;
    };
    let removed = match root.get_mut(key.as_str()) {
        Some(serde_json::Value::Array(list)) => {
            let before = list.len();
            list.retain(|entry| !entry_matches(entry, PLUGIN_NAME));
            before - list.len()
        }
        _ => 0,
    };
    if let Some(serde_json::Value::Array(list)) = root.get(key.as_str())
        && list.is_empty()
    {
        root.remove(key.as_str());
    }
    removed
}

fn add_entry(
    value: &mut serde_json::Value,
    key: ConfigKey,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = value
        .as_object_mut()
        .ok_or("config root is not an object")?;
    if !matches!(root.get(key.as_str()), Some(serde_json::Value::Array(_))) {
        root.insert(
            key.as_str().to_string(),
            serde_json::Value::Array(Vec::new()),
        );
    }
    let Some(serde_json::Value::Array(list)) = root.get_mut(key.as_str()) else {
        return Err(format!("`{}` must be an array", key.as_str()).into());
    };
    list.push(serde_json::Value::String(PLUGIN_NAME.to_string()));
    Ok(())
}

pub fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home_dir = dirs::home_dir().ok_or("could not determine home directory")?;
    let config_dir = home_dir.join(".config").join("opencode");
    let jsonc_path = config_dir.join("opencode.jsonc");
    let json_path = config_dir.join("opencode.json");

    if jsonc_path.exists() {
        Ok(jsonc_path)
    } else if json_path.exists() {
        Ok(json_path)
    } else {
        Ok(jsonc_path)
    }
}

fn read_config(path: &PathBuf) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)?;
    let value: serde_json::Value =
        jsonc_parser::parse_to_serde_value(&content, &Default::default())?;
    Ok(Some(value))
}

fn write_config(
    path: &PathBuf,
    value: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(value)?;
    fs::write(path, content + "\n")?;
    Ok(())
}

fn select_target(v1: bool, v2: bool, detected: Option<OpenCodeVersion>) -> ConfigKey {
    if v2 {
        return ConfigKey::Plugins;
    }
    if v1 {
        return ConfigKey::Plugin;
    }
    match detected {
        Some(version) if version.major >= 2 => ConfigKey::Plugins,
        Some(_) => ConfigKey::Plugin,
        None => {
            println!(
                "Could not detect opencode version, defaulting to OpenCode 2.x (`plugins` key). \
                 Use --v1 for OpenCode 1.x"
            );
            ConfigKey::Plugins
        }
    }
}

/// Installs the plugin entry under `target`. Returns whether the config changed.
fn install(
    value: &mut serde_json::Value,
    target: ConfigKey,
) -> Result<bool, Box<dyn std::error::Error>> {
    // For OpenCode 2.x, move the entry out of the legacy V1 `plugin` key.
    let moved = if target == ConfigKey::Plugins {
        remove_entry(value, ConfigKey::Plugin)
    } else {
        0
    };

    if has_entry(value, target) {
        if moved > 0 {
            println!("Removed stale entry from legacy `plugin` key");
        } else {
            println!("Plugin '{PLUGIN_NAME}' is already installed");
        }
        return Ok(moved > 0);
    }

    add_entry(value, target)?;
    if moved > 0 {
        println!("Moved entry from `plugin` to `plugins`");
    }
    println!("Plugin '{PLUGIN_NAME}' installed successfully");
    Ok(true)
}

pub fn run(cmd: OpencodeCommand) -> Result<(), Box<dyn std::error::Error>> {
    let path = config_path()?;

    match cmd.action {
        OpencodeAction::Install { v1, v2 } => {
            let detected = detect_opencode_version();
            let target = select_target(v1, v2, detected);

            let mut value = read_config(&path)?.unwrap_or_else(|| serde_json::json!({}));
            if install(&mut value, target)? {
                write_config(&path, &value)?;
                println!(
                    "Config updated: {} (`{}` key)",
                    path.display(),
                    target.as_str()
                );
            }

            if target == ConfigKey::Plugin
                && let Some(version) = detected
                && !version.supports_dual_shape()
            {
                println!(
                    "warning: opencode {version} is older than 1.18.29; \
                     oh-my-codes-opencode >= 0.3 requires opencode >= 1.18.29 \
                     (or pin version 0.2.x on older releases)"
                );
            }
        }
        OpencodeAction::Uninstall => {
            let Some(mut value) = read_config(&path)? else {
                println!("Config file not found: {}", path.display());
                return Ok(());
            };

            let removed = remove_entry(&mut value, ConfigKey::Plugin)
                + remove_entry(&mut value, ConfigKey::Plugins);

            if removed > 0 {
                write_config(&path, &value)?;
                println!("Plugin '{PLUGIN_NAME}' uninstalled successfully");
                println!("Config updated: {}", path.display());
            } else {
                println!("Plugin '{PLUGIN_NAME}' is not installed");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn v(major: u64, minor: u64, patch: u64) -> OpenCodeVersion {
        OpenCodeVersion {
            major,
            minor,
            patch,
        }
    }

    #[test]
    fn parses_v2_version_output() {
        assert_eq!(parse_opencode_version("opencode v2.2.0"), Some(v(2, 2, 0)));
        assert_eq!(parse_opencode_version("v2.0.1"), Some(v(2, 0, 1)));
        assert_eq!(parse_opencode_version("2.10.3"), Some(v(2, 10, 3)));
    }

    #[test]
    fn parses_v1_version_output() {
        assert_eq!(parse_opencode_version("1.18.5"), Some(v(1, 18, 5)));
        assert_eq!(
            parse_opencode_version("opencode 1.14.0 something"),
            Some(v(1, 14, 0))
        );
        assert_eq!(parse_opencode_version("1.18.29-beta.1"), Some(v(1, 18, 29)));
    }

    #[test]
    fn rejects_unparsable_output() {
        assert_eq!(parse_opencode_version(""), None);
        assert_eq!(parse_opencode_version("opencode dev build"), None);
        assert_eq!(parse_opencode_version("not-a-version"), None);
    }

    #[test]
    fn dual_shape_floor() {
        assert!(v(2, 0, 0).supports_dual_shape());
        assert!(v(1, 18, 29).supports_dual_shape());
        assert!(v(1, 19, 0).supports_dual_shape());
        assert!(!v(1, 18, 28).supports_dual_shape());
        assert!(!v(1, 17, 99).supports_dual_shape());
        assert!(!v(0, 9, 9).supports_dual_shape());
    }

    #[test]
    fn install_v2_moves_entry_from_legacy_key() {
        let mut value = json!({ "plugin": ["oh-my-codes-opencode", "other-plugin"] });
        install(&mut value, ConfigKey::Plugins).unwrap();
        let plugins = value["plugins"].as_array().unwrap();
        assert!(
            plugins
                .iter()
                .any(|e| e.as_str() == Some("oh-my-codes-opencode"))
        );
        let plugin = value["plugin"].as_array().unwrap();
        assert_eq!(plugin.len(), 1);
        assert_eq!(plugin[0].as_str(), Some("other-plugin"));
    }

    #[test]
    fn install_v2_removes_empty_legacy_key() {
        let mut value = json!({ "plugin": ["oh-my-codes-opencode"] });
        install(&mut value, ConfigKey::Plugins).unwrap();
        assert!(value.get("plugin").is_none());
        assert!(has_entry(&value, ConfigKey::Plugins));
    }

    #[test]
    fn install_v2_leaves_tuple_entries_alone() {
        let mut value = json!({ "plugin": [["other-plugin", { "enabled": true }]] });
        install(&mut value, ConfigKey::Plugins).unwrap();
        let plugin = value["plugin"].as_array().unwrap();
        assert_eq!(plugin.len(), 1);
        assert!(has_entry(&value, ConfigKey::Plugins));
    }

    #[test]
    fn select_target_prefers_flags_then_detection() {
        assert_eq!(
            select_target(false, true, Some(v(1, 18, 29))),
            ConfigKey::Plugins
        );
        assert_eq!(
            select_target(true, false, Some(v(2, 2, 0))),
            ConfigKey::Plugin
        );
        assert_eq!(
            select_target(false, false, Some(v(2, 0, 0))),
            ConfigKey::Plugins
        );
        assert_eq!(
            select_target(false, false, Some(v(1, 14, 0))),
            ConfigKey::Plugin
        );
        assert_eq!(select_target(false, false, None), ConfigKey::Plugins);
    }

    #[test]
    fn install_is_idempotent_per_key() {
        let mut value = json!({});
        install(&mut value, ConfigKey::Plugins).unwrap();
        install(&mut value, ConfigKey::Plugins).unwrap();
        assert_eq!(value["plugins"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn install_detects_object_form_entry() {
        let mut value = json!({ "plugins": [{ "package": "oh-my-codes-opencode" }] });
        install(&mut value, ConfigKey::Plugins).unwrap();
        assert_eq!(value["plugins"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn install_v1_keeps_legacy_key() {
        let mut value = json!({});
        install(&mut value, ConfigKey::Plugin).unwrap();
        assert!(has_entry(&value, ConfigKey::Plugin));
        assert!(value.get("plugins").is_none());
    }

    #[test]
    fn uninstall_removes_from_both_keys() {
        let mut value = json!({
            "plugin": ["oh-my-codes-opencode", "other"],
            "plugins": ["oh-my-codes-opencode"],
        });
        let removed = remove_entry(&mut value, ConfigKey::Plugin)
            + remove_entry(&mut value, ConfigKey::Plugins);
        assert_eq!(removed, 2);
        assert!(value.get("plugins").is_none());
        let plugin = value["plugin"].as_array().unwrap();
        assert_eq!(plugin.len(), 1);
        assert_eq!(plugin[0].as_str(), Some("other"));
    }

    #[test]
    fn uninstall_removes_tuple_and_object_entries() {
        let mut value = json!({
            "plugin": [["oh-my-codes-opencode", { "x": 1 }]],
            "plugins": [{ "package": "oh-my-codes-opencode", "options": {} }],
        });
        let removed = remove_entry(&mut value, ConfigKey::Plugin)
            + remove_entry(&mut value, ConfigKey::Plugins);
        assert_eq!(removed, 2);
        assert!(value.get("plugin").is_none());
        assert!(value.get("plugins").is_none());
    }
}
