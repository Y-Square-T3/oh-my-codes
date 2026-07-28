use std::fs;
use std::path::PathBuf;

use super::{OpencodeAction, OpencodeCommand};

const PLUGIN_NAME: &str = "oh-my-codes-opencode";

fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_dir = dirs::config_dir().ok_or("could not determine config directory")?;
    Ok(config_dir.join("opencode").join("opencode.json"))
}

fn read_config(path: &PathBuf) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)?;
    let value: serde_json::Value = serde_json::from_str(&content)?;
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

pub fn run(cmd: OpencodeCommand) -> Result<(), Box<dyn std::error::Error>> {
    let path = config_path()?;

    match cmd.action {
        OpencodeAction::Install => {
            let mut value = read_config(&path)?.unwrap_or_else(|| serde_json::json!({}));

            let plugins = value.get_mut("plugin").and_then(|v| v.as_array_mut());

            if let Some(plugins) = plugins {
                if plugins.iter().any(|p| p.as_str() == Some(PLUGIN_NAME)) {
                    println!("Plugin '{PLUGIN_NAME}' is already installed");
                    return Ok(());
                }
                plugins.push(serde_json::Value::String(PLUGIN_NAME.to_string()));
            } else {
                value["plugin"] = serde_json::json!([PLUGIN_NAME]);
            }

            write_config(&path, &value)?;
            println!("Plugin '{PLUGIN_NAME}' installed successfully");
            println!("Config updated: {}", path.display());
        }
        OpencodeAction::Uninstall => {
            let Some(mut value) = read_config(&path)? else {
                println!("Config file not found: {}", path.display());
                return Ok(());
            };

            let removed =
                if let Some(plugins) = value.get_mut("plugin").and_then(|v| v.as_array_mut()) {
                    let before = plugins.len();
                    plugins.retain(|p| p.as_str() != Some(PLUGIN_NAME));
                    before != plugins.len()
                } else {
                    false
                };

            if removed {
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
