use anyhow::{Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

const CURRENT_CONFIG_VERSION: u64 = 2;

pub struct ConfigManager {
    path: PathBuf,
    data: Value,
}

/// Recursively merge `defaults` into `target`, only adding keys that don't exist.
fn merge_missing(target: &mut Value, defaults: &Value) {
    if let (Some(t), Some(d)) = (target.as_object_mut(), defaults.as_object()) {
        for (key, val) in d {
            if !t.contains_key(key) {
                t.insert(key.clone(), val.clone());
            } else if val.is_object() {
                merge_missing(&mut t[key], val);
            }
        }
    }
}

impl ConfigManager {
    pub fn load(path: &Path) -> Result<Self> {
        let data = if path.exists() {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read config: {}", path.display()))?;
            match serde_json::from_str(&content) {
                Ok(parsed) => parsed,
                Err(e) => {
                    eprintln!(
                        "Warning: config at {} is corrupted ({}), using defaults.",
                        path.display(),
                        e
                    );
                    Self::default_config()
                }
            }
        } else {
            Self::default_config()
        };
        let mut mgr = Self {
            path: path.to_path_buf(),
            data,
        };
        Self::migrate(&mut mgr.data);
        Ok(mgr)
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = &self.data;
        for part in parts {
            current = current.get(part)?;
        }
        Some(current)
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|v| v.as_str().map(String::from))
    }

    pub fn set(&mut self, key: &str, value: Value) -> Result<()> {
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = &mut self.data;
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                current[part] = value;
                return self.save();
            }
            if !current.get(part).is_some_and(|v| v.is_object()) {
                current[part] = Value::Object(serde_json::Map::new());
            }
            current = &mut current[part];
        }
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.data)?;
        std::fs::write(&self.path, content)?;
        Ok(())
    }

    /// Migrate config from older versions. Adds missing keys without overwriting.
    fn migrate(data: &mut Value) {
        let version = data
            .get("config_version")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        if version < CURRENT_CONFIG_VERSION {
            // Merge defaults for any missing keys
            let defaults = Self::default_config();
            merge_missing(data, &defaults);
            data["config_version"] = serde_json::json!(CURRENT_CONFIG_VERSION);
        }
    }

    pub fn default_config() -> Value {
        serde_json::json!({
            "config_version": CURRENT_CONFIG_VERSION,
            "editor": "nvim",
            "default_kb": "personal",
            "ai": {
                "backend": "ollama",
                "default_model": "llama3.2",
                "ollama_url": "http://localhost:11434",
                "gguf_model_path": "",
                "embedding_model": "nomic-embed-text"
            },
            "daemon": {
                "port": 7433,
                "auto_start": true
            },
            "session": {
                "shell": "powershell",
                "tmux_enabled": false
            },
            "hooks": {}
        })
    }

    pub fn data(&self) -> &Value {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        let config = ConfigManager::load(&path).unwrap();
        assert_eq!(config.get_string("editor").unwrap(), "nvim");
        assert_eq!(config.get_string("ai.backend").unwrap(), "ollama");
    }

    #[test]
    fn test_set_and_get() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        let mut config = ConfigManager::load(&path).unwrap();
        config
            .set("ai.backend", Value::String("gguf".into()))
            .unwrap();
        assert_eq!(config.get_string("ai.backend").unwrap(), "gguf");

        // Reload and verify persistence
        let config2 = ConfigManager::load(&path).unwrap();
        assert_eq!(config2.get_string("ai.backend").unwrap(), "gguf");
    }

    #[test]
    fn test_nested_set_creates_path() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        let mut config = ConfigManager::load(&path).unwrap();
        config
            .set("new.nested.key", Value::String("value".into()))
            .unwrap();
        assert_eq!(config.get_string("new.nested.key").unwrap(), "value");
    }

    #[test]
    fn test_migration_adds_missing_keys() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        // Write an old config without hooks or config_version
        std::fs::write(
            &path,
            r#"{"editor": "vim", "default_kb": "work"}"#,
        )
        .unwrap();

        let config = ConfigManager::load(&path).unwrap();
        // Should have migrated: added config_version and ai defaults
        assert!(config.get("config_version").is_some());
        assert!(config.get("ai.backend").is_some());
        // Should NOT overwrite existing values
        assert_eq!(config.get_string("editor").unwrap(), "vim");
        assert_eq!(config.get_string("default_kb").unwrap(), "work");
    }

    #[test]
    fn test_get_nonexistent_key() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        let config = ConfigManager::load(&path).unwrap();
        assert!(config.get("nonexistent.key").is_none());
    }
}
