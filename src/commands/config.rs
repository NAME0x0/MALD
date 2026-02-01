use anyhow::Result;
use serde_json::Value;

use crate::config::ConfigManager;
use crate::fs::mald_home;

pub async fn get(key: &str) -> Result<()> {
    let config_path = mald_home().join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;

    match config.get(key) {
        Some(value) => {
            match value {
                Value::String(s) => println!("{s}"),
                other => println!("{}", serde_json::to_string_pretty(other)?),
            }
            Ok(())
        }
        None => {
            println!("Key '{key}' not found");
            Ok(())
        }
    }
}

pub async fn set(key: &str, value: &str) -> Result<()> {
    let config_path = mald_home().join("config").join("config.json");
    let mut config = ConfigManager::load(&config_path)?;

    // Try to parse as JSON value, fall back to string
    let json_value = serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.into()));

    config.set(key, json_value)?;
    println!("Set {key} = {value}");
    Ok(())
}
