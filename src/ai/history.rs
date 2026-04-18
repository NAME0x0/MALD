use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::fs::mald_home;

use super::ollama::ChatMessage;

#[derive(Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub kb: String,
    pub created: String,
    pub messages: Vec<ChatMessage>,
}

impl ChatSession {
    pub fn new(kb: &str) -> Self {
        let id = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
        Self {
            id: id.clone(),
            kb: kb.to_string(),
            created: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
            messages: Vec::new(),
        }
    }

    pub fn add(&mut self, role: &str, content: &str) {
        self.messages.push(ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
        });
    }

    pub fn save(&self) -> Result<()> {
        let dir = chat_dir();
        crate::fs::ensure_directory(&dir)?;
        let path = dir.join(format!("{}.json", self.id));
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        let data = std::fs::read_to_string(path)?;
        serde_json::from_str(&data).context("Invalid chat session file")
    }
}

fn chat_dir() -> PathBuf {
    mald_home().join("sessions").join("chat")
}

/// Load the most recent chat session for a given KB, or None.
pub fn latest_session(kb: &str) -> Option<ChatSession> {
    let dir = chat_dir();
    if !dir.exists() {
        return None;
    }
    let mut sessions: Vec<PathBuf> = std::fs::read_dir(&dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .collect();
    sessions.sort();
    sessions.reverse();
    for path in sessions {
        if let Ok(session) = ChatSession::load(&path) {
            if session.kb == kb {
                return Some(session);
            }
        }
    }
    None
}

/// List all chat sessions.
pub fn list_sessions() -> Vec<ChatSession> {
    let dir = chat_dir();
    if !dir.exists() {
        return Vec::new();
    }
    let mut sessions: Vec<ChatSession> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .filter_map(|p| ChatSession::load(&p).ok())
        .collect();
    sessions.sort_by(|a, b| b.id.cmp(&a.id));
    sessions
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_session_roundtrip() {
        // Avoid std::env::set_var — it is unsound when tests run in parallel.
        // Instead, manually create the session dir and write the JSON directly.
        let dir = TempDir::new().unwrap();
        let chat_dir = dir.path().join("sessions").join("chat");
        std::fs::create_dir_all(&chat_dir).unwrap();

        let mut session = ChatSession::new("personal");
        session.add("user", "Hello");
        session.add("assistant", "Hi there");

        let path = chat_dir.join(format!("{}.json", session.id));
        let json = serde_json::to_string_pretty(&session).unwrap();
        std::fs::write(&path, json).unwrap();

        let loaded = ChatSession::load(&path).unwrap();
        assert_eq!(loaded.messages.len(), 2);
        assert_eq!(loaded.kb, "personal");
    }
}
