use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::ConfigManager;

pub struct OllamaClient {
    base_url: String,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct StreamResponse {
    message: ChatMessage,
}

#[derive(Serialize)]
struct EmbeddingRequest {
    model: String,
    input: String,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    embeddings: Vec<Vec<f32>>,
}

#[derive(Deserialize)]
struct ListResponse {
    models: Vec<ModelInfo>,
}

#[derive(Deserialize)]
struct ModelInfo {
    name: String,
}

#[derive(Serialize)]
struct PullRequest {
    name: String,
    stream: bool,
}

impl OllamaClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn from_config(config: &ConfigManager) -> Self {
        let url = config
            .get_string("ai.ollama_url")
            .unwrap_or_else(|| "http://localhost:11434".into());
        Self::new(&url)
    }

    pub async fn is_running(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .is_ok()
    }

    pub async fn list_models(&self) -> Result<Vec<String>> {
        let resp: ListResponse = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await?
            .json()
            .await?;
        Ok(resp.models.into_iter().map(|m| m.name).collect())
    }

    pub async fn pull_model(&self, name: &str) -> Result<()> {
        let req = PullRequest {
            name: name.to_string(),
            stream: false,
        };
        self.client
            .post(format!("{}/api/pull", self.base_url))
            .json(&req)
            .send()
            .await?
            .error_for_status()
            .context("Failed to pull model")?;
        Ok(())
    }

    /// Pull a model with streaming progress output.
    pub async fn pull_model_stream(&self, name: &str) -> Result<()> {
        use futures_util::StreamExt;
        let req = PullRequest {
            name: name.to_string(),
            stream: true,
        };
        let resp = self
            .client
            .post(format!("{}/api/pull", self.base_url))
            .json(&req)
            .send()
            .await?
            .error_for_status()
            .context("Failed to pull model")?;

        let bar = indicatif::ProgressBar::new(0);
        bar.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("  {msg} [{bar:30}] {bytes}/{total_bytes}")
                .unwrap()
                .progress_chars("=> "),
        );
        bar.set_message(name.to_string());

        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let bytes = chunk?;
            let text = String::from_utf8_lossy(&bytes);
            for line in text.lines() {
                if line.is_empty() {
                    continue;
                }
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line) {
                    if let Some(total) = parsed.get("total").and_then(|v| v.as_u64()) {
                        bar.set_length(total);
                    }
                    if let Some(completed) = parsed.get("completed").and_then(|v| v.as_u64()) {
                        bar.set_position(completed);
                    }
                    if let Some(status) = parsed.get("status").and_then(|v| v.as_str()) {
                        if status == "success" {
                            bar.finish_with_message(format!("{} done", name));
                        }
                    }
                }
            }
        }
        bar.finish_and_clear();
        Ok(())
    }

    pub async fn chat(&self, model: &str, message: &str) -> Result<String> {
        self.chat_with_history(
            model,
            &[ChatMessage {
                role: "user".into(),
                content: message.into(),
            }],
        )
        .await
    }

    pub async fn chat_with_history(&self, model: &str, messages: &[ChatMessage]) -> Result<String> {
        let req = ChatRequest {
            model: model.to_string(),
            messages: messages.to_vec(),
            stream: false,
        };
        let resp: ChatResponse = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&req)
            .send()
            .await?
            .error_for_status()
            .context("Chat request failed")?
            .json()
            .await?;
        Ok(resp.message.content)
    }

    /// Stream a chat response, printing tokens as they arrive.
    pub async fn chat_streaming(&self, model: &str, messages: &[ChatMessage]) -> Result<String> {
        let req = ChatRequest {
            model: model.to_string(),
            messages: messages.to_vec(),
            stream: true,
        };
        let resp = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&req)
            .send()
            .await?
            .error_for_status()
            .context("Chat request failed")?;

        use std::io::Write;
        let mut full_response = String::new();
        let mut stream = resp.bytes_stream();
        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let bytes = chunk?;
            let text = String::from_utf8_lossy(&bytes);
            // Each line is a JSON object with a partial message
            for line in text.lines() {
                if line.is_empty() {
                    continue;
                }
                if let Ok(parsed) = serde_json::from_str::<StreamResponse>(line) {
                    let token = &parsed.message.content;
                    print!("{}", token);
                    std::io::stdout().flush().ok();
                    full_response.push_str(token);
                }
            }
        }
        println!(); // newline after streaming
        Ok(full_response)
    }

    pub async fn embeddings(&self, model: &str, text: &str) -> Result<Vec<f32>> {
        let req = EmbeddingRequest {
            model: model.to_string(),
            input: text.to_string(),
        };
        let resp: EmbeddingResponse = self
            .client
            .post(format!("{}/api/embed", self.base_url))
            .json(&req)
            .send()
            .await?
            .error_for_status()
            .context("Embedding request failed")?
            .json()
            .await?;
        resp.embeddings
            .into_iter()
            .next()
            .context("No embedding returned")
    }
}
