//! Model provider trait and implementations.
//!
//! The [`ModelProvider`] trait defines the unified interface for all model providers,
//! whether cloud-based (Anthropic, OpenAI) or local (Ollama, llama.cpp).
//!
//! # Example
//!
//! ```ignore
//! use vibes_models::providers::{ModelProvider, ChatRequest, Message};
//!
//! async fn chat(provider: &dyn ModelProvider) {
//!     let request = ChatRequest::new(
//!         "claude-sonnet-4",
//!         vec![Message::user("Hello!")],
//!     );
//!
//!     let response = provider.chat(request).await?;
//!     println!("Response: {}", response.content.as_text());
//! }
//! ```

mod types;

use std::pin::Pin;

use async_trait::async_trait;
use tokio_stream::Stream;

pub use types::*;

use crate::{ModelInfo, Pricing, Result};

/// A stream of chat response chunks for streaming responses.
///
/// This is a pinned, boxed stream that yields [`StreamChunk`] items or errors.
pub type ChatStream = Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>;

/// Trait for model providers (cloud and local).
///
/// All model providers implement this trait to provide a unified interface
/// for chat completions, streaming, and embeddings.
///
/// # Required Methods
///
/// - [`name`](ModelProvider::name) - Provider identifier (e.g., "anthropic", "openai")
/// - [`models`](ModelProvider::models) - List of available models
/// - [`chat`](ModelProvider::chat) - Non-streaming chat completion
/// - [`chat_stream`](ModelProvider::chat_stream) - Streaming chat completion
///
/// # Optional Methods
///
/// - [`embed`](ModelProvider::embed) - Text embeddings (returns error by default)
/// - [`supports_tools`](ModelProvider::supports_tools) - Tool/function calling support
/// - [`supports_vision`](ModelProvider::supports_vision) - Image input support
/// - [`pricing`](ModelProvider::pricing) - Pricing information for a model
///
/// # Example Implementation
///
/// ```ignore
/// use async_trait::async_trait;
/// use vibes_models::providers::{ModelProvider, ChatRequest, ChatResponse, ChatStream};
/// use vibes_models::{ModelInfo, Pricing, Result};
///
/// struct MyProvider;
///
/// #[async_trait]
/// impl ModelProvider for MyProvider {
///     fn name(&self) -> &str {
///         "my-provider"
///     }
///
///     fn models(&self) -> Vec<ModelInfo> {
///         vec![/* ... */]
///     }
///
///     async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
///         // Implementation
///     }
///
///     async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream> {
///         // Implementation
///     }
/// }
/// ```
#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// Returns the provider name (e.g., "anthropic", "openai", "ollama").
    fn name(&self) -> &str;

    /// Returns the list of models available from this provider.
    fn models(&self) -> Vec<ModelInfo>;

    /// Perform a chat completion request.
    ///
    /// # Arguments
    ///
    /// * `request` - The chat request containing messages, model, and parameters
    ///
    /// # Returns
    ///
    /// The complete response with content, usage statistics, and any tool calls.
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;

    /// Perform a streaming chat completion request.
    ///
    /// # Arguments
    ///
    /// * `request` - The chat request (stream flag will be set automatically)
    ///
    /// # Returns
    ///
    /// A stream of response chunks. The final chunk will contain usage statistics.
    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream>;

    /// Generate text embeddings.
    ///
    /// # Arguments
    ///
    /// * `request` - The embedding request containing texts to embed
    ///
    /// # Returns
    ///
    /// Embeddings for each input text and usage statistics.
    ///
    /// # Default Implementation
    ///
    /// Returns an error indicating embeddings are not supported.
    async fn embed(&self, _request: EmbedRequest) -> Result<EmbedResponse> {
        Err(crate::Error::ProviderApi(format!(
            "embeddings not supported by provider '{}'",
            self.name()
        )))
    }

    /// Whether this provider supports tool/function calling.
    fn supports_tools(&self) -> bool {
        false
    }

    /// Whether this provider supports vision/image inputs.
    fn supports_vision(&self) -> bool {
        false
    }

    /// Get pricing information for a specific model.
    ///
    /// # Arguments
    ///
    /// * `model` - The model name to get pricing for
    ///
    /// # Returns
    ///
    /// Pricing information if available, None otherwise.
    fn pricing(&self, _model: &str) -> Option<Pricing> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Capabilities, ModelId};

    /// A mock provider for testing the trait.
    struct MockProvider {
        name: String,
        models: Vec<ModelInfo>,
    }

    impl MockProvider {
        fn new() -> Self {
            Self {
                name: "mock".to_string(),
                models: vec![ModelInfo {
                    id: ModelId::new("mock", "test-model"),
                    provider: "mock".to_string(),
                    name: "test-model".to_string(),
                    context_window: 4096,
                    max_output: Some(1024),
                    capabilities: Capabilities::chat(),
                    pricing: None,
                    local: false,
                }],
            }
        }
    }

    #[async_trait]
    impl ModelProvider for MockProvider {
        fn name(&self) -> &str {
            &self.name
        }

        fn models(&self) -> Vec<ModelInfo> {
            self.models.clone()
        }

        async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
            Ok(ChatResponse {
                content: Content::text(format!("Echo: {}", request.messages[0].content.as_text())),
                stop_reason: StopReason::EndTurn,
                tool_calls: vec![],
                usage: Usage::new(10, 5),
            })
        }

        async fn chat_stream(&self, _request: ChatRequest) -> Result<ChatStream> {
            use tokio_stream::iter;
            let chunks = vec![
                Ok(StreamChunk {
                    delta: Some("Hello".to_string()),
                    stop_reason: None,
                    tool_calls: vec![],
                    usage: None,
                }),
                Ok(StreamChunk {
                    delta: Some(" world".to_string()),
                    stop_reason: Some(StopReason::EndTurn),
                    tool_calls: vec![],
                    usage: Some(Usage::new(5, 10)),
                }),
            ];
            Ok(Box::pin(iter(chunks)))
        }
    }

    #[tokio::test]
    async fn mock_provider_chat_returns_response() {
        let provider = MockProvider::new();
        let request = ChatRequest::new("test-model", vec![Message::user("Hello")]);
        let response = provider.chat(request).await.unwrap();

        assert_eq!(response.content.as_text(), "Echo: Hello");
        assert_eq!(response.stop_reason, StopReason::EndTurn);
        assert_eq!(response.usage.total_tokens, 15);
    }

    #[tokio::test]
    async fn mock_provider_stream_yields_chunks() {
        use tokio_stream::StreamExt;

        let provider = MockProvider::new();
        let request = ChatRequest::new("test-model", vec![Message::user("Hello")]);
        let mut stream = provider.chat_stream(request).await.unwrap();

        let first = stream.next().await.unwrap().unwrap();
        assert_eq!(first.delta, Some("Hello".to_string()));
        assert!(first.stop_reason.is_none());

        let second = stream.next().await.unwrap().unwrap();
        assert_eq!(second.delta, Some(" world".to_string()));
        assert_eq!(second.stop_reason, Some(StopReason::EndTurn));
    }

    #[tokio::test]
    async fn default_embed_returns_error() {
        let provider = MockProvider::new();
        let request = EmbedRequest::new("test-model", vec!["hello".to_string()]);
        let result = provider.embed(request).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not supported"));
    }

    #[test]
    fn default_supports_methods_return_false() {
        let provider = MockProvider::new();
        assert!(!provider.supports_tools());
        assert!(!provider.supports_vision());
    }

    #[test]
    fn default_pricing_returns_none() {
        let provider = MockProvider::new();
        assert!(provider.pricing("test-model").is_none());
    }

    #[test]
    fn provider_returns_models() {
        let provider = MockProvider::new();
        let models = provider.models();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "test-model");
    }
}
