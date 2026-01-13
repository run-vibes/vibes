//! Core types for model management.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a model in format `provider:model_name`.
///
/// # Examples
///
/// ```
/// use vibes_models::ModelId;
///
/// let id = ModelId::new("anthropic", "claude-sonnet-4");
/// assert_eq!(id.provider(), "anthropic");
/// assert_eq!(id.model(), "claude-sonnet-4");
/// assert_eq!(id.to_string(), "anthropic:claude-sonnet-4");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ModelId(String);

impl ModelId {
    /// Create a new model ID from provider and model name.
    pub fn new(provider: &str, model: &str) -> Self {
        Self(format!("{provider}:{model}"))
    }

    /// Parse a model ID from a string in `provider:model` format.
    pub fn parse(s: &str) -> Option<Self> {
        if s.contains(':') {
            Some(Self(s.to_string()))
        } else {
            None
        }
    }

    /// Get the provider portion of the ID.
    pub fn provider(&self) -> &str {
        self.0.split(':').next().unwrap_or("")
    }

    /// Get the model name portion of the ID.
    pub fn model(&self) -> &str {
        self.0.split(':').nth(1).unwrap_or("")
    }

    /// Get the full ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ModelId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Model capabilities indicating what features the model supports.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capabilities {
    /// Supports chat/conversation.
    pub chat: bool,
    /// Supports vision/image input.
    pub vision: bool,
    /// Supports tool/function calling.
    pub tools: bool,
    /// Supports text embeddings.
    pub embeddings: bool,
    /// Supports streaming responses.
    pub streaming: bool,
}

impl Capabilities {
    /// Create capabilities for a chat model.
    pub fn chat() -> Self {
        Self {
            chat: true,
            streaming: true,
            ..Default::default()
        }
    }

    /// Create capabilities for a full-featured model.
    pub fn full() -> Self {
        Self {
            chat: true,
            vision: true,
            tools: true,
            streaming: true,
            embeddings: false,
        }
    }

    /// Create capabilities for an embedding model.
    pub fn embeddings() -> Self {
        Self {
            embeddings: true,
            ..Default::default()
        }
    }

    /// Check if any capability matches the given filter.
    pub fn matches(&self, filter: &Capabilities) -> bool {
        (!filter.chat || self.chat)
            && (!filter.vision || self.vision)
            && (!filter.tools || self.tools)
            && (!filter.embeddings || self.embeddings)
            && (!filter.streaming || self.streaming)
    }
}

/// Pricing information for a model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pricing {
    /// Cost per million input tokens in USD.
    pub input_per_million: f64,
    /// Cost per million output tokens in USD.
    pub output_per_million: f64,
}

impl Pricing {
    /// Create new pricing information.
    pub fn new(input_per_million: f64, output_per_million: f64) -> Self {
        Self {
            input_per_million,
            output_per_million,
        }
    }

    /// Calculate cost for a given number of input and output tokens.
    pub fn calculate(&self, input_tokens: u64, output_tokens: u64) -> f64 {
        let input_cost = (input_tokens as f64 / 1_000_000.0) * self.input_per_million;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * self.output_per_million;
        input_cost + output_cost
    }
}

/// Information about a model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Unique model identifier.
    pub id: ModelId,
    /// Provider name (e.g., "anthropic", "openai").
    pub provider: String,
    /// Human-readable model name.
    pub name: String,
    /// Maximum context window size in tokens.
    pub context_window: u32,
    /// Maximum output tokens (if limited).
    pub max_output: Option<u32>,
    /// Model capabilities.
    pub capabilities: Capabilities,
    /// Pricing information (if available).
    pub pricing: Option<Pricing>,
    /// Whether this is a local model (not cloud-hosted).
    pub local: bool,
}

impl ModelInfo {
    /// Create a new model info builder.
    pub fn builder(provider: &str, name: &str) -> ModelInfoBuilder {
        ModelInfoBuilder::new(provider, name)
    }
}

/// Builder for constructing `ModelInfo`.
#[derive(Debug)]
pub struct ModelInfoBuilder {
    provider: String,
    name: String,
    context_window: u32,
    max_output: Option<u32>,
    capabilities: Capabilities,
    pricing: Option<Pricing>,
    local: bool,
}

impl ModelInfoBuilder {
    fn new(provider: &str, name: &str) -> Self {
        Self {
            provider: provider.to_string(),
            name: name.to_string(),
            context_window: 4096,
            max_output: None,
            capabilities: Capabilities::default(),
            pricing: None,
            local: false,
        }
    }

    /// Set the context window size.
    pub fn context_window(mut self, tokens: u32) -> Self {
        self.context_window = tokens;
        self
    }

    /// Set the maximum output tokens.
    pub fn max_output(mut self, tokens: u32) -> Self {
        self.max_output = Some(tokens);
        self
    }

    /// Set the model capabilities.
    pub fn capabilities(mut self, caps: Capabilities) -> Self {
        self.capabilities = caps;
        self
    }

    /// Set the pricing information.
    pub fn pricing(mut self, pricing: Pricing) -> Self {
        self.pricing = Some(pricing);
        self
    }

    /// Mark as a local model.
    pub fn local(mut self) -> Self {
        self.local = true;
        self
    }

    /// Build the `ModelInfo`.
    pub fn build(self) -> ModelInfo {
        ModelInfo {
            id: ModelId::new(&self.provider, &self.name),
            provider: self.provider,
            name: self.name,
            context_window: self.context_window,
            max_output: self.max_output,
            capabilities: self.capabilities,
            pricing: self.pricing,
            local: self.local,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_id_new_creates_correct_format() {
        let id = ModelId::new("anthropic", "claude-sonnet-4");
        assert_eq!(id.to_string(), "anthropic:claude-sonnet-4");
    }

    #[test]
    fn model_id_parse_extracts_parts() {
        let id = ModelId::parse("openai:gpt-4o").unwrap();
        assert_eq!(id.provider(), "openai");
        assert_eq!(id.model(), "gpt-4o");
    }

    #[test]
    fn model_id_parse_returns_none_for_invalid() {
        assert!(ModelId::parse("invalid").is_none());
    }

    #[test]
    fn model_id_serializes_as_string() {
        let id = ModelId::new("anthropic", "claude-sonnet-4");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"anthropic:claude-sonnet-4\"");
    }

    #[test]
    fn model_id_deserializes_from_string() {
        let id: ModelId = serde_json::from_str("\"openai:gpt-4o\"").unwrap();
        assert_eq!(id.provider(), "openai");
        assert_eq!(id.model(), "gpt-4o");
    }

    #[test]
    fn capabilities_matches_filters_correctly() {
        let full = Capabilities::full();
        let chat_only = Capabilities::chat();

        // Full caps should match chat filter
        let chat_filter = Capabilities {
            chat: true,
            ..Default::default()
        };
        assert!(full.matches(&chat_filter));
        assert!(chat_only.matches(&chat_filter));

        // Chat-only should not match vision filter
        let vision_filter = Capabilities {
            vision: true,
            ..Default::default()
        };
        assert!(full.matches(&vision_filter));
        assert!(!chat_only.matches(&vision_filter));
    }

    #[test]
    fn pricing_calculates_correctly() {
        let pricing = Pricing::new(3.0, 15.0); // $3/M input, $15/M output
        let cost = pricing.calculate(1_000_000, 100_000);
        assert!((cost - 4.5).abs() < 0.001); // $3 + $1.50 = $4.50
    }

    #[test]
    fn model_info_builder_creates_model() {
        let info = ModelInfo::builder("anthropic", "claude-sonnet-4")
            .context_window(200_000)
            .max_output(8192)
            .capabilities(Capabilities::full())
            .pricing(Pricing::new(3.0, 15.0))
            .build();

        assert_eq!(info.id.to_string(), "anthropic:claude-sonnet-4");
        assert_eq!(info.provider, "anthropic");
        assert_eq!(info.context_window, 200_000);
        assert!(info.capabilities.vision);
        assert!(!info.local);
    }

    #[test]
    fn model_info_serializes_to_json() {
        let info = ModelInfo::builder("ollama", "llama3")
            .context_window(8192)
            .capabilities(Capabilities::chat())
            .local()
            .build();

        let json = serde_json::to_string_pretty(&info).unwrap();
        assert!(json.contains("\"local\": true"));
        assert!(json.contains("\"ollama:llama3\""));
    }
}
