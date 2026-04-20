use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

const CURRENT_CONFIG_VERSION: u64 = 2;

fn default_editor() -> &'static str {
    if cfg!(windows) {
        "code"
    } else {
        "nvim"
    }
}

// ---------------------------------------------------------------------------
// Typed configuration structs (read-only, deserialized from the same JSON)
// ---------------------------------------------------------------------------

/// Typed, compile-time validated configuration.
///
/// Deserializes from the same JSON file that `ConfigManager` uses. All fields
/// have sensible defaults that match `ConfigManager::default_config()`. This
/// struct is **read-only** -- writing still goes through `ConfigManager::set()`.
#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct TypedConfig {
    pub config_version: u64,
    pub editor: String,
    pub default_kb: String,
    pub ai: AiConfig,
    pub daemon: DaemonConfig,
    pub session: SessionConfig,
    pub hooks: HooksConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct AiConfig {
    pub backend: String,
    pub default_model: String,
    pub ollama_url: String,
    pub gguf_model_path: String,
    pub embedding_model: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct DaemonConfig {
    pub port: u16,
    pub auto_start: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct SessionConfig {
    pub shell: String,
    pub tmux_enabled: bool,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
pub struct HooksConfig {
    pub on_create: Option<String>,
    pub on_save: Option<String>,
    pub on_daily: Option<String>,
}

// --- Default impls (match ConfigManager::default_config()) ---

impl Default for TypedConfig {
    fn default() -> Self {
        Self {
            config_version: CURRENT_CONFIG_VERSION,
            editor: default_editor().into(),
            default_kb: "personal".into(),
            ai: AiConfig::default(),
            daemon: DaemonConfig::default(),
            session: SessionConfig::default(),
            hooks: HooksConfig::default(),
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            backend: "ollama".into(),
            default_model: "llama3.2".into(),
            ollama_url: "http://localhost:11434".into(),
            gguf_model_path: String::new(),
            embedding_model: "nomic-embed-text".into(),
        }
    }
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            port: 7433,
            auto_start: true,
        }
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            shell: "powershell".into(),
            tmux_enabled: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: resolve a KB name from an optional CLI arg + config default
// ---------------------------------------------------------------------------

/// Resolve the knowledge base name and path from an optional CLI argument.
///
/// Returns `(ConfigManager, TypedConfig, kb_name, kb_path)`.
/// The KB path is **not** validated for existence -- callers should check if needed.
pub fn resolve_kb(kb: Option<&str>) -> Result<(ConfigManager, TypedConfig, String, PathBuf)> {
    let home = crate::fs::mald_home();
    let config_path = home.join("config").join("config.json");
    let config = ConfigManager::load(&config_path)?;
    let typed = config.typed();
    let kb_root = home.join("kb");
    let available = list_kb_names(&kb_root);
    let requested = kb
        .map(String::from)
        .unwrap_or_else(|| typed.default_kb.clone());
    let requested_path = kb_root.join(&requested);

    let kb_name = if requested_path.exists() {
        requested
    } else {
        let exact = available
            .iter()
            .find(|name| name.eq_ignore_ascii_case(&requested))
            .cloned();

        if let Some(exact) = exact {
            exact
        } else if let Some(best) = fuzzy_resolve_kb_name(&available, &requested) {
            best
        } else if kb.is_none() {
            available.into_iter().next().unwrap_or(requested)
        } else {
            requested
        }
    };
    let kb_path = kb_root.join(&kb_name);
    Ok((config, typed, kb_name, kb_path))
}

fn fuzzy_resolve_kb_name(available: &[String], requested: &str) -> Option<String> {
    let requested = requested.trim();
    if requested.is_empty() {
        return None;
    }

    let requested_lower = requested.to_ascii_lowercase();
    let requested_norm = normalize_kb_name(requested);
    let tokens: Vec<&str> = requested_norm
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .collect();

    let mut matches: Vec<(i32, &String)> = available
        .iter()
        .filter_map(|name| {
            let lower = name.to_ascii_lowercase();
            let normalized = normalize_kb_name(name);

            let score = if lower == requested_lower || normalized == requested_norm {
                500
            } else if lower.starts_with(&requested_lower) || normalized.starts_with(&requested_norm)
            {
                350
            } else if !tokens.is_empty() && tokens.iter().all(|token| normalized.contains(token)) {
                250
            } else if lower.contains(&requested_lower) || normalized.contains(&requested_norm) {
                200
            } else {
                return None;
            };

            Some((score, name))
        })
        .collect();

    matches.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| a.1.len().cmp(&b.1.len()))
            .then_with(|| a.1.cmp(b.1))
    });

    matches.into_iter().next().map(|(_, name)| name.clone())
}

fn normalize_kb_name(value: &str) -> String {
    value
        .chars()
        .map(|ch| match ch {
            '-' | '_' | '/' | '\\' => ' ',
            _ => ch.to_ascii_lowercase(),
        })
        .collect()
}

pub fn list_kb_names(kb_root: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(kb_root)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            entry
                .file_type()
                .ok()
                .filter(|ft| ft.is_dir())
                .map(|_| entry.file_name().to_string_lossy().to_string())
        })
        .collect();
    names.sort();
    names
}

/// Load config and return both the mutable manager and the typed snapshot.
pub fn load_typed(path: &Path) -> Result<(ConfigManager, TypedConfig)> {
    let mgr = ConfigManager::load(path)?;
    let typed = mgr.typed();
    Ok((mgr, typed))
}

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

/// Validate a dot-notation config key.
/// Rejects empty keys, empty segments, leading/trailing dots, and control characters.
fn validate_key(key: &str) -> Result<()> {
    if key.is_empty() {
        anyhow::bail!("Config key cannot be empty");
    }
    if key.starts_with('.') || key.ends_with('.') {
        anyhow::bail!("Config key cannot start or end with a dot: {key:?}");
    }
    if key.contains("..") {
        anyhow::bail!("Config key cannot contain empty segments (double dot): {key:?}");
    }
    if key.chars().any(|c| c.is_control()) {
        anyhow::bail!("Config key cannot contain control characters: {key:?}");
    }
    // Reject excessively long keys (defense against abuse)
    if key.len() > 128 {
        anyhow::bail!("Config key too long (max 128 chars): {key:?}");
    }
    Ok(())
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
        // Silently return None for invalid keys (get is infallible)
        if validate_key(key).is_err() {
            return None;
        }
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
        validate_key(key)?;
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

    /// Atomic save: write to temp file, then rename.
    /// Prevents corruption if the process crashes mid-write or two instances race.
    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.data)?;

        // Write to a sibling temp file first
        let tmp_path = self.path.with_extension("json.tmp");
        std::fs::write(&tmp_path, &content)
            .with_context(|| format!("Failed to write temp config: {}", tmp_path.display()))?;

        // Atomic rename (on the same filesystem, this is atomic on most OSes)
        std::fs::rename(&tmp_path, &self.path).or_else(|_| {
            // Fallback for cross-device moves (shouldn't happen for same-dir rename)
            std::fs::write(&self.path, &content)?;
            let _ = std::fs::remove_file(&tmp_path);
            Ok(())
        })
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
            "editor": default_editor(),
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

    /// Deserialize the current config JSON into a typed, validated struct.
    ///
    /// Unknown keys are silently ignored (forward-compatible). Missing keys
    /// receive their `Default` values.
    pub fn typed(&self) -> TypedConfig {
        serde_json::from_value(self.data.clone()).unwrap_or_default()
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
        assert_eq!(config.get_string("editor").unwrap(), default_editor());
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
        std::fs::write(&path, r#"{"editor": "vim", "default_kb": "work"}"#).unwrap();

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

    #[test]
    fn test_invalid_key_empty() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        let mut config = ConfigManager::load(&path).unwrap();
        assert!(config.set("", Value::String("x".into())).is_err());
        assert!(config.get("").is_none());
    }

    #[test]
    fn test_invalid_key_leading_dot() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        let mut config = ConfigManager::load(&path).unwrap();
        assert!(config.set(".leading", Value::String("x".into())).is_err());
    }

    #[test]
    fn test_invalid_key_trailing_dot() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        let mut config = ConfigManager::load(&path).unwrap();
        assert!(config.set("trailing.", Value::String("x".into())).is_err());
    }

    #[test]
    fn test_invalid_key_double_dot() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        let mut config = ConfigManager::load(&path).unwrap();
        assert!(config.set("a..b", Value::String("x".into())).is_err());
    }

    #[test]
    fn test_invalid_key_control_chars() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        let mut config = ConfigManager::load(&path).unwrap();
        assert!(config.set("a\0b", Value::String("x".into())).is_err());
        assert!(config.set("a\nb", Value::String("x".into())).is_err());
    }

    // -----------------------------------------------------------------------
    // TypedConfig tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_typed_defaults_match_json_defaults() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        let config = ConfigManager::load(&path).unwrap();
        let typed = config.typed();

        assert_eq!(typed.config_version, 2);
        assert_eq!(typed.editor, default_editor());
        assert_eq!(typed.default_kb, "personal");
        assert_eq!(typed.ai.backend, "ollama");
        assert_eq!(typed.ai.default_model, "llama3.2");
        assert_eq!(typed.ai.ollama_url, "http://localhost:11434");
        assert_eq!(typed.ai.gguf_model_path, "");
        assert_eq!(typed.ai.embedding_model, "nomic-embed-text");
        assert_eq!(typed.daemon.port, 7433);
        assert!(typed.daemon.auto_start);
        assert_eq!(typed.session.shell, "powershell");
        assert!(!typed.session.tmux_enabled);
        assert!(typed.hooks.on_create.is_none());
        assert!(typed.hooks.on_save.is_none());
        assert!(typed.hooks.on_daily.is_none());
    }

    #[test]
    fn test_typed_reads_custom_values() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        let mut config = ConfigManager::load(&path).unwrap();
        config.set("editor", Value::String("code".into())).unwrap();
        config
            .set("ai.default_model", Value::String("mistral".into()))
            .unwrap();
        config.set("daemon.port", serde_json::json!(9999)).unwrap();

        let typed = config.typed();
        assert_eq!(typed.editor, "code");
        assert_eq!(typed.ai.default_model, "mistral");
        assert_eq!(typed.daemon.port, 9999);
        // Unchanged fields keep defaults
        assert_eq!(typed.default_kb, "personal");
    }

    #[test]
    fn test_typed_survives_unknown_keys() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(
            &path,
            r#"{
                "editor": "emacs",
                "future_feature": true,
                "ai": { "default_model": "phi3", "new_ai_key": 42 }
            }"#,
        )
        .unwrap();

        let config = ConfigManager::load(&path).unwrap();
        let typed = config.typed();
        assert_eq!(typed.editor, "emacs");
        assert_eq!(typed.ai.default_model, "phi3");
        // Unknown keys silently ignored, missing keys get defaults
        assert_eq!(typed.default_kb, "personal");
        assert_eq!(typed.daemon.port, 7433);
    }

    #[test]
    fn test_typed_from_empty_json_uses_defaults() {
        let typed: TypedConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(typed.editor, default_editor());
        assert_eq!(typed.ai.default_model, "llama3.2");
        assert_eq!(typed.daemon.port, 7433);
    }

    #[test]
    fn test_load_typed_helper() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("config.json");
        let (mgr, typed) = load_typed(&path).unwrap();
        assert_eq!(mgr.get_string("editor").unwrap(), default_editor());
        assert_eq!(typed.editor, default_editor());
        assert_eq!(typed.ai.backend, "ollama");
    }
}
