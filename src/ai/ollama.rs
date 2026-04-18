use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::time::Duration;

use crate::config::ConfigManager;

/// Default connect timeout (fast fail if Ollama not reachable).
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
/// Default request timeout for non-streaming requests.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);
/// Short timeout for health checks.
const HEALTH_TIMEOUT: Duration = Duration::from_secs(2);

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
        let client = reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        }
    }

    pub fn from_config(config: &ConfigManager) -> Self {
        let url = config.typed().ai.ollama_url.clone();
        Self::new(&url)
    }

    /// Check if Ollama is running. Uses short timeout for fast failure.
    pub async fn is_running(&self) -> bool {
        // Use a dedicated client with short timeout for health checks
        let health_client = reqwest::Client::builder()
            .connect_timeout(HEALTH_TIMEOUT)
            .timeout(HEALTH_TIMEOUT)
            .build()
            .unwrap_or_else(|_| self.client.clone());

        health_client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .is_ok()
    }

    pub async fn list_models(&self) -> Result<Vec<String>> {
        let resp: ListResponse = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .timeout(Duration::from_secs(10)) // Short timeout for listing
            .send()
            .await
            .context("Failed to connect to Ollama (is it running?)")?
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
            // Model pulls can take a long time (downloading GBs)
            .timeout(Duration::from_secs(3600))
            .send()
            .await
            .context("Failed to connect to Ollama for model pull")?
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
        // Create client without total timeout for streaming (only connect timeout)
        let stream_client = reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .build()
            .unwrap_or_else(|_| self.client.clone());

        let resp = stream_client
            .post(format!("{}/api/pull", self.base_url))
            .json(&req)
            .send()
            .await
            .context("Failed to connect to Ollama for model pull")?
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
                            bar.finish_with_message(format!("{name} done"));
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
            .await
            .context("Failed to connect to Ollama (is it running?)")?
            .error_for_status()
            .context("Chat request failed")?
            .json()
            .await
            .context("Failed to parse chat response")?;
        Ok(resp.message.content)
    }

    /// Stream a chat response, printing tokens as they arrive.
    pub async fn chat_streaming(&self, model: &str, messages: &[ChatMessage]) -> Result<String> {
        let response = self
            .chat_streaming_with_callback(model, messages, |token| {
                print!("{token}");
                std::io::stdout().flush().ok();
            })
            .await?;
        println!();
        Ok(response)
    }

    pub async fn chat_streaming_with_callback<F>(
        &self,
        model: &str,
        messages: &[ChatMessage],
        mut on_chunk: F,
    ) -> Result<String>
    where
        F: FnMut(&str),
    {
        let req = ChatRequest {
            model: model.to_string(),
            messages: messages.to_vec(),
            stream: true,
        };
        // Streaming client: only connect timeout, no total timeout
        let stream_client = reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .build()
            .unwrap_or_else(|_| self.client.clone());

        let resp = stream_client
            .post(format!("{}/api/chat", self.base_url))
            .json(&req)
            .send()
            .await
            .context("Failed to connect to Ollama (is it running?)")?
            .error_for_status()
            .context("Chat request failed")?;

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
                    on_chunk(token);
                    full_response.push_str(token);
                }
            }
        }
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
            // Embeddings are fast, use shorter timeout
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .context("Failed to connect to Ollama for embeddings")?
            .error_for_status()
            .context("Embedding request failed")?
            .json()
            .await
            .context("Failed to parse embedding response")?;
        resp.embeddings
            .into_iter()
            .next()
            .context("No embedding returned")
    }
}
