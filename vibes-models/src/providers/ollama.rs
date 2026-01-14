//! Ollama local model provider.
//!
//! Connects to a local Ollama instance for running models like Llama, Mistral, etc.
//!
//! # Example
//!
//! ```ignore
//! use vibes_models::providers::OllamaProvider;
//!
//! let provider = OllamaProvider::new();  // Uses localhost:11434
//! let provider = OllamaProvider::with_base_url("http://192.168.1.100:11434");
//! ```

use std::sync::RwLock;

use serde::Deserialize;

use crate::{Capabilities, ModelInfo};

/// Default Ollama API base URL.
const DEFAULT_BASE_URL: &str = "http://localhost:11434";

// ────────────────────────────────────────────────────────────────────────────
// Ollama API Response Types
// ────────────────────────────────────────────────────────────────────────────

/// Response from Ollama's `/api/tags` endpoint.
#[derive(Debug, Deserialize)]
pub struct OllamaTagsResponse {
    pub models: Vec<OllamaModel>,
}

/// Model information from Ollama's API.
#[derive(Debug, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,
    /// Model digest hash (required for deserialization but not used in ModelInfo).
    #[allow(dead_code)]
    pub digest: String,
    pub modified_at: String,
}

impl OllamaModel {
    /// Convert to a `ModelInfo`.
    pub fn to_model_info(&self) -> ModelInfo {
        ModelInfo::builder("ollama", &self.name)
            .context_window(8192) // Default, varies by model
            .capabilities(Capabilities::chat())
            .local()
            .size_bytes(self.size)
            .modified_at(&self.modified_at)
            .build()
    }
}

/// Message in an Ollama chat request/response.
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct OllamaChatMessage {
    pub role: String,
    pub content: String,
}

/// Request body for Ollama's `/api/chat` endpoint.
#[derive(Debug, serde::Serialize)]
pub struct OllamaChatRequest {
    pub model: String,
    pub messages: Vec<OllamaChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OllamaChatOptions>,
}

/// Chat options for Ollama.
#[derive(Debug, serde::Serialize)]
pub struct OllamaChatOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

/// Response from Ollama's `/api/chat` endpoint.
#[derive(Debug, Deserialize)]
pub struct OllamaChatResponse {
    pub model: String,
    pub message: OllamaChatMessage,
    pub done: bool,
    #[serde(default)]
    pub prompt_eval_count: Option<u64>,
    #[serde(default)]
    pub eval_count: Option<u64>,
}

impl From<OllamaChatResponse> for super::ChatResponse {
    fn from(response: OllamaChatResponse) -> Self {
        Self {
            content: super::Content::text(response.message.content),
            stop_reason: super::StopReason::EndTurn,
            tool_calls: vec![],
            usage: super::Usage::new(
                response.prompt_eval_count.unwrap_or(0),
                response.eval_count.unwrap_or(0),
            ),
        }
    }
}

impl OllamaChatResponse {
    /// Convert to a streaming chunk.
    pub fn to_stream_chunk(&self) -> super::StreamChunk {
        super::StreamChunk {
            delta: if self.message.content.is_empty() {
                None
            } else {
                Some(self.message.content.clone())
            },
            stop_reason: if self.done {
                Some(super::StopReason::EndTurn)
            } else {
                None
            },
            tool_calls: vec![],
            usage: if self.done {
                Some(super::Usage::new(
                    self.prompt_eval_count.unwrap_or(0),
                    self.eval_count.unwrap_or(0),
                ))
            } else {
                None
            },
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// OllamaProvider
// ────────────────────────────────────────────────────────────────────────────

/// Ollama local model provider.
///
/// Connects to a local Ollama instance to run models like Llama, Mistral, etc.
pub struct OllamaProvider {
    base_url: String,
    client: reqwest::Client,
    cached_models: RwLock<Vec<ModelInfo>>,
}

impl OllamaProvider {
    /// Create a new Ollama provider with default URL (localhost:11434).
    pub fn new() -> Self {
        Self::with_base_url(DEFAULT_BASE_URL)
    }

    /// Create a new Ollama provider with a custom base URL.
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
            cached_models: RwLock::new(Vec::new()),
        }
    }

    /// Get the base URL for this provider.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the provider name.
    pub fn name(&self) -> &str {
        "ollama"
    }

    /// Get the list of cached models.
    ///
    /// Call [`refresh_models`](Self::refresh_models) to update from the Ollama API.
    pub fn models(&self) -> Vec<ModelInfo> {
        self.cached_models.read().unwrap().clone()
    }

    /// Refresh the model list from the Ollama API.
    ///
    /// Fetches models from `/api/tags` and updates the cache.
    pub async fn refresh_models(&self) -> crate::Result<()> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| crate::Error::Request(e.to_string()))?;

        if !response.status().is_success() {
            return Err(crate::Error::ProviderApi(format!(
                "Ollama API returned status {}",
                response.status()
            )));
        }

        let tags: OllamaTagsResponse = response
            .json()
            .await
            .map_err(|e| crate::Error::Request(e.to_string()))?;

        let models: Vec<ModelInfo> = tags.models.iter().map(|m| m.to_model_info()).collect();

        *self.cached_models.write().unwrap() = models;
        Ok(())
    }

    /// Set the cached models directly (for testing).
    #[cfg(test)]
    fn set_models(&self, models: Vec<ModelInfo>) {
        *self.cached_models.write().unwrap() = models;
    }

    /// Perform a chat completion request.
    pub async fn chat(&self, request: super::ChatRequest) -> crate::Result<super::ChatResponse> {
        let url = format!("{}/api/chat", self.base_url);

        // Convert messages to Ollama format
        let messages: Vec<OllamaChatMessage> = request
            .messages
            .iter()
            .map(|m| OllamaChatMessage {
                role: match m.role {
                    super::Role::System => "system".to_string(),
                    super::Role::User => "user".to_string(),
                    super::Role::Assistant => "assistant".to_string(),
                    super::Role::Tool => "tool".to_string(),
                },
                content: m.content.as_text(),
            })
            .collect();

        let options = if request.temperature.is_some()
            || request.max_tokens.is_some()
            || request.stop.is_some()
        {
            Some(OllamaChatOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens,
                stop: request.stop,
            })
        } else {
            None
        };

        let ollama_request = OllamaChatRequest {
            model: request.model,
            messages,
            stream: Some(false),
            options,
        };

        let response = self
            .client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| crate::Error::Request(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(crate::Error::ProviderApi(format!(
                "Ollama API returned {}: {}",
                status, body
            )));
        }

        let ollama_response: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| crate::Error::Request(e.to_string()))?;

        Ok(ollama_response.into())
    }

    /// Perform a streaming chat completion request.
    pub async fn chat_stream(
        &self,
        request: super::ChatRequest,
    ) -> crate::Result<super::ChatStream> {
        use futures_util::StreamExt;

        let url = format!("{}/api/chat", self.base_url);

        // Convert messages to Ollama format
        let messages: Vec<OllamaChatMessage> = request
            .messages
            .iter()
            .map(|m| OllamaChatMessage {
                role: match m.role {
                    super::Role::System => "system".to_string(),
                    super::Role::User => "user".to_string(),
                    super::Role::Assistant => "assistant".to_string(),
                    super::Role::Tool => "tool".to_string(),
                },
                content: m.content.as_text(),
            })
            .collect();

        let options = if request.temperature.is_some()
            || request.max_tokens.is_some()
            || request.stop.is_some()
        {
            Some(OllamaChatOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens,
                stop: request.stop,
            })
        } else {
            None
        };

        let ollama_request = OllamaChatRequest {
            model: request.model,
            messages,
            stream: Some(true),
            options,
        };

        let response = self
            .client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| crate::Error::Request(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(crate::Error::ProviderApi(format!(
                "Ollama API returned {}: {}",
                status, body
            )));
        }

        // Stream NDJSON responses
        let byte_stream = response.bytes_stream();
        let stream = byte_stream
            .map(|result| {
                result
                    .map_err(|e| crate::Error::Request(e.to_string()))
                    .and_then(|bytes| {
                        // Parse NDJSON line
                        let text = String::from_utf8_lossy(&bytes);
                        // Skip empty lines
                        let trimmed = text.trim();
                        if trimmed.is_empty() {
                            return Ok(super::StreamChunk {
                                delta: None,
                                stop_reason: None,
                                tool_calls: vec![],
                                usage: None,
                            });
                        }
                        serde_json::from_str::<OllamaChatResponse>(trimmed)
                            .map(|r| r.to_stream_chunk())
                            .map_err(crate::Error::Serialization)
                    })
            })
            // Filter out empty chunks
            .filter(|result| {
                std::future::ready(match result {
                    Ok(chunk) => chunk.delta.is_some() || chunk.stop_reason.is_some(),
                    Err(_) => true, // Keep errors to propagate them
                })
            });

        Ok(Box::pin(stream))
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl super::ModelProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    fn models(&self) -> Vec<ModelInfo> {
        self.cached_models.read().unwrap().clone()
    }

    async fn chat(&self, request: super::ChatRequest) -> crate::Result<super::ChatResponse> {
        self.chat(request).await
    }

    async fn chat_stream(&self, request: super::ChatRequest) -> crate::Result<super::ChatStream> {
        self.chat_stream(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_provider_with_default_url() {
        let provider = OllamaProvider::new();
        assert_eq!(provider.base_url(), "http://localhost:11434");
    }

    #[test]
    fn with_base_url_creates_provider_with_custom_url() {
        let provider = OllamaProvider::with_base_url("http://192.168.1.100:11434");
        assert_eq!(provider.base_url(), "http://192.168.1.100:11434");
    }

    #[test]
    fn name_returns_ollama() {
        let provider = OllamaProvider::new();
        assert_eq!(provider.name(), "ollama");
    }

    // ────────────────────────────────────────────────────────────────────────────
    // Model Discovery Tests
    // ────────────────────────────────────────────────────────────────────────────

    #[test]
    fn parse_tags_response_extracts_models() {
        let json = r#"{
            "models": [
                {
                    "name": "llama3:latest",
                    "model": "llama3:latest",
                    "modified_at": "2024-01-15T10:00:00Z",
                    "size": 4661224676,
                    "digest": "abc123"
                },
                {
                    "name": "mistral:7b",
                    "model": "mistral:7b",
                    "modified_at": "2024-01-14T10:00:00Z",
                    "size": 4109865159,
                    "digest": "def456"
                }
            ]
        }"#;

        let response: OllamaTagsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.models.len(), 2);
        assert_eq!(response.models[0].name, "llama3:latest");
        assert_eq!(response.models[1].name, "mistral:7b");
    }

    #[test]
    fn ollama_model_converts_to_model_info() {
        let ollama_model = OllamaModel {
            name: "llama3:latest".to_string(),
            size: 4661224676,
            digest: "abc123".to_string(),
            modified_at: "2024-01-15T10:00:00Z".to_string(),
        };

        let info = ollama_model.to_model_info();

        assert_eq!(info.provider, "ollama");
        assert_eq!(info.name, "llama3:latest");
        assert_eq!(info.id.to_string(), "ollama:llama3:latest");
        assert!(info.local, "Ollama models should be marked as local");
        assert!(info.capabilities.chat);
        assert!(info.capabilities.streaming);
        assert!(info.pricing.is_none(), "Local models have no pricing");
    }

    #[test]
    fn models_returns_empty_when_not_refreshed() {
        let provider = OllamaProvider::new();
        assert!(provider.models().is_empty());
    }

    #[tokio::test]
    async fn refresh_models_updates_cached_models() {
        // This test uses a mock response - full integration test is separate
        let provider = OllamaProvider::new();

        // Simulate what refresh would do by setting cached models directly
        let models = vec![
            OllamaModel {
                name: "llama3:latest".to_string(),
                size: 4661224676,
                digest: "abc123".to_string(),
                modified_at: "2024-01-15T10:00:00Z".to_string(),
            }
            .to_model_info(),
        ];
        provider.set_models(models);

        let cached = provider.models();
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].name, "llama3:latest");
        assert!(cached[0].local);
    }

    // ────────────────────────────────────────────────────────────────────────────
    // Chat API Tests
    // ────────────────────────────────────────────────────────────────────────────

    #[test]
    fn parse_chat_response_extracts_content() {
        let json = r#"{
            "model": "llama3",
            "created_at": "2024-01-15T10:00:00Z",
            "message": {
                "role": "assistant",
                "content": "Hello! How can I help you today?"
            },
            "done": true,
            "total_duration": 1234567890,
            "load_duration": 123456789,
            "prompt_eval_count": 10,
            "prompt_eval_duration": 12345678,
            "eval_count": 15,
            "eval_duration": 23456789
        }"#;

        let response: OllamaChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.message.content, "Hello! How can I help you today?");
        assert_eq!(response.message.role, "assistant");
        assert!(response.done);
        assert_eq!(response.prompt_eval_count, Some(10));
        assert_eq!(response.eval_count, Some(15));
    }

    #[test]
    fn ollama_chat_response_converts_to_chat_response() {
        use crate::providers::{ChatResponse, StopReason};

        let ollama_response = OllamaChatResponse {
            model: "llama3".to_string(),
            message: OllamaChatMessage {
                role: "assistant".to_string(),
                content: "Hello!".to_string(),
            },
            done: true,
            prompt_eval_count: Some(10),
            eval_count: Some(15),
        };

        let response: ChatResponse = ollama_response.into();

        assert_eq!(response.content.as_text(), "Hello!");
        assert_eq!(response.stop_reason, StopReason::EndTurn);
        assert_eq!(response.usage.input_tokens, 10);
        assert_eq!(response.usage.output_tokens, 15);
        assert!(response.tool_calls.is_empty());
    }

    // ────────────────────────────────────────────────────────────────────────────
    // Streaming Tests
    // ────────────────────────────────────────────────────────────────────────────

    #[test]
    fn parse_streaming_chunk_extracts_delta() {
        // Streaming response has same structure but done=false for intermediate chunks
        let json = r#"{
            "model": "llama3",
            "created_at": "2024-01-15T10:00:00Z",
            "message": {
                "role": "assistant",
                "content": "Hello"
            },
            "done": false
        }"#;

        let chunk: OllamaChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(chunk.message.content, "Hello");
        assert!(!chunk.done);
    }

    #[test]
    fn streaming_chunk_converts_to_stream_chunk() {
        use crate::providers::StreamChunk;

        let ollama_chunk = OllamaChatResponse {
            model: "llama3".to_string(),
            message: OllamaChatMessage {
                role: "assistant".to_string(),
                content: "Hello".to_string(),
            },
            done: false,
            prompt_eval_count: None,
            eval_count: None,
        };

        let chunk: StreamChunk = ollama_chunk.to_stream_chunk();

        assert_eq!(chunk.delta, Some("Hello".to_string()));
        assert!(chunk.stop_reason.is_none());
        assert!(chunk.usage.is_none());
    }

    #[test]
    fn final_streaming_chunk_has_stop_reason_and_usage() {
        use crate::providers::{StopReason, StreamChunk};

        let ollama_final = OllamaChatResponse {
            model: "llama3".to_string(),
            message: OllamaChatMessage {
                role: "assistant".to_string(),
                content: "!".to_string(),
            },
            done: true,
            prompt_eval_count: Some(10),
            eval_count: Some(15),
        };

        let chunk: StreamChunk = ollama_final.to_stream_chunk();

        assert_eq!(chunk.delta, Some("!".to_string()));
        assert_eq!(chunk.stop_reason, Some(StopReason::EndTurn));
        assert!(chunk.usage.is_some());
        let usage = chunk.usage.unwrap();
        assert_eq!(usage.input_tokens, 10);
        assert_eq!(usage.output_tokens, 15);
    }

    // ────────────────────────────────────────────────────────────────────────────
    // Integration Tests (require Ollama running)
    // ────────────────────────────────────────────────────────────────────────────

    /// Check if Ollama is available at the given URL.
    async fn ollama_available(base_url: &str) -> bool {
        let client = reqwest::Client::new();
        client
            .get(format!("{}/api/tags", base_url))
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
            .is_ok()
    }

    #[tokio::test]
    #[ignore = "requires Ollama running locally"]
    async fn integration_refresh_models_fetches_from_ollama() {
        let base_url =
            std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string());

        if !ollama_available(&base_url).await {
            eprintln!("Skipping: Ollama not available at {}", base_url);
            return;
        }

        let provider = OllamaProvider::with_base_url(&base_url);
        provider
            .refresh_models()
            .await
            .expect("refresh should succeed");

        let models = provider.models();
        // We don't assert specific models since that depends on what's installed
        println!("Found {} models from Ollama", models.len());
        for model in &models {
            println!("  - {} (local: {})", model.name, model.local);
            assert!(model.local, "Ollama models should be marked as local");
        }
    }

    #[tokio::test]
    #[ignore = "requires Ollama running locally with a model installed"]
    async fn integration_chat_sends_request_to_ollama() {
        use crate::providers::{ChatRequest, Message};

        let base_url =
            std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string());

        if !ollama_available(&base_url).await {
            eprintln!("Skipping: Ollama not available at {}", base_url);
            return;
        }

        let provider = OllamaProvider::with_base_url(&base_url);
        provider
            .refresh_models()
            .await
            .expect("refresh should succeed");

        let models = provider.models();
        if models.is_empty() {
            eprintln!("Skipping: No models installed in Ollama");
            return;
        }

        let model = &models[0].name;
        let request = ChatRequest::new(model, vec![Message::user("Say 'hello' and nothing else.")]);

        let response = provider.chat(request).await.expect("chat should succeed");
        println!("Response: {}", response.content.as_text());
        assert!(!response.content.as_text().is_empty());
    }
}
