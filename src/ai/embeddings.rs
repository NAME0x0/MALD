use anyhow::Result;

use crate::config::ConfigManager;

pub enum EmbeddingBackend {
    Ollama(super::ollama::OllamaClient),
}

impl EmbeddingBackend {
    pub fn from_config(config: &ConfigManager) -> Self {
        let backend = config
            .get_string("ai.backend")
            .unwrap_or_else(|| "ollama".into());

        let _ = backend; // reserved for future backends
        Self::Ollama(super::ollama::OllamaClient::from_config(config))
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        match self {
            Self::Ollama(client) => {
                let model = "nomic-embed-text";
                client.embeddings(model, text).await
            }
        }
    }
}
