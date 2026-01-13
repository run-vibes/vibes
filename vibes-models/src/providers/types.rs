//! Request and response types for model providers.

use serde::{Deserialize, Serialize};

/// Role of a message in a conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System message setting context/behavior.
    System,
    /// User message.
    User,
    /// Assistant response.
    Assistant,
    /// Tool/function result.
    Tool,
}

/// Content of a message, either text or structured.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    /// Simple text content.
    Text(String),
    /// Structured content parts (text, images, etc.).
    Parts(Vec<ContentPart>),
}

impl Content {
    /// Create text content.
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text(s.into())
    }

    /// Get content as text, joining parts if necessary.
    pub fn as_text(&self) -> String {
        match self {
            Content::Text(s) => s.clone(),
            Content::Parts(parts) => parts
                .iter()
                .filter_map(|p| match p {
                    ContentPart::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(""),
        }
    }
}

impl From<String> for Content {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for Content {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string())
    }
}

/// A part of structured content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    /// Text content.
    Text { text: String },
    /// Image content with base64 data or URL.
    Image {
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        base64: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        media_type: Option<String>,
    },
}

/// A message in a conversation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message sender.
    pub role: Role,
    /// Content of the message.
    pub content: Content,
    /// Tool call ID if this is a tool result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    /// Create a system message.
    pub fn system(content: impl Into<Content>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
            tool_call_id: None,
        }
    }

    /// Create a user message.
    pub fn user(content: impl Into<Content>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            tool_call_id: None,
        }
    }

    /// Create an assistant message.
    pub fn assistant(content: impl Into<Content>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            tool_call_id: None,
        }
    }

    /// Create a tool result message.
    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<Content>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

/// Tool definition for function calling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name.
    pub name: String,
    /// Tool description.
    pub description: String,
    /// JSON schema for parameters.
    pub parameters: serde_json::Value,
}

/// A tool call made by the model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique ID for this tool call.
    pub id: String,
    /// Name of the tool to call.
    pub name: String,
    /// Arguments as JSON string.
    pub arguments: String,
}

/// Token usage statistics.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    /// Number of input/prompt tokens.
    pub input_tokens: u64,
    /// Number of output/completion tokens.
    pub output_tokens: u64,
    /// Total tokens (input + output).
    pub total_tokens: u64,
}

impl Usage {
    /// Create new usage statistics.
    pub fn new(input_tokens: u64, output_tokens: u64) -> Self {
        Self {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
        }
    }
}

/// Request for a chat completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// Model ID to use.
    pub model: String,
    /// Conversation messages.
    pub messages: Vec<Message>,
    /// Sampling temperature (0.0 to 2.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Maximum tokens to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Stop sequences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    /// Available tools for function calling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    /// Whether to stream the response.
    #[serde(default)]
    pub stream: bool,
}

impl ChatRequest {
    /// Create a new chat request.
    pub fn new(model: impl Into<String>, messages: Vec<Message>) -> Self {
        Self {
            model: model.into(),
            messages,
            temperature: None,
            max_tokens: None,
            stop: None,
            tools: None,
            stream: false,
        }
    }

    /// Set the temperature.
    pub fn temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    /// Set the maximum tokens.
    pub fn max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = Some(tokens);
        self
    }

    /// Set stop sequences.
    pub fn stop(mut self, sequences: Vec<String>) -> Self {
        self.stop = Some(sequences);
        self
    }

    /// Set available tools.
    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Enable streaming.
    pub fn stream(mut self) -> Self {
        self.stream = true;
        self
    }
}

/// Reason why generation stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Reached end of response naturally.
    EndTurn,
    /// Hit a stop sequence.
    StopSequence,
    /// Reached max tokens limit.
    MaxTokens,
    /// Model wants to call a tool.
    ToolUse,
}

/// Response from a chat completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Response content.
    pub content: Content,
    /// Why generation stopped.
    pub stop_reason: StopReason,
    /// Tool calls requested by the model.
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
    /// Token usage statistics.
    pub usage: Usage,
}

/// A chunk from a streaming response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// Delta content (incremental text).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<String>,
    /// Stop reason if this is the final chunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<StopReason>,
    /// Tool calls if present.
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
    /// Usage statistics (typically only in final chunk).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Request for text embeddings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedRequest {
    /// Model ID to use.
    pub model: String,
    /// Texts to embed.
    pub texts: Vec<String>,
}

impl EmbedRequest {
    /// Create a new embedding request.
    pub fn new(model: impl Into<String>, texts: Vec<String>) -> Self {
        Self {
            model: model.into(),
            texts,
        }
    }
}

/// Response from an embedding request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedResponse {
    /// Embeddings for each input text.
    pub embeddings: Vec<Vec<f32>>,
    /// Token usage statistics.
    pub usage: Usage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_constructors_work() {
        let sys = Message::system("You are helpful");
        assert_eq!(sys.role, Role::System);
        assert_eq!(sys.content.as_text(), "You are helpful");

        let user = Message::user("Hello");
        assert_eq!(user.role, Role::User);

        let asst = Message::assistant("Hi there!");
        assert_eq!(asst.role, Role::Assistant);

        let tool = Message::tool_result("call_123", "result");
        assert_eq!(tool.role, Role::Tool);
        assert_eq!(tool.tool_call_id, Some("call_123".to_string()));
    }

    #[test]
    fn chat_request_builder_works() {
        let req = ChatRequest::new("gpt-4o", vec![Message::user("Hello")])
            .temperature(0.7)
            .max_tokens(1000)
            .stream();

        assert_eq!(req.model, "gpt-4o");
        assert_eq!(req.temperature, Some(0.7));
        assert_eq!(req.max_tokens, Some(1000));
        assert!(req.stream);
    }

    #[test]
    fn content_from_string() {
        let content: Content = "hello".into();
        assert_eq!(content.as_text(), "hello");

        let content: Content = String::from("world").into();
        assert_eq!(content.as_text(), "world");
    }

    #[test]
    fn usage_calculates_total() {
        let usage = Usage::new(100, 50);
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn message_serializes_correctly() {
        let msg = Message::user("Hello");
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"Hello\""));
    }

    #[test]
    fn role_serializes_lowercase() {
        let json = serde_json::to_string(&Role::Assistant).unwrap();
        assert_eq!(json, "\"assistant\"");
    }

    #[test]
    fn stop_reason_serializes_snake_case() {
        let json = serde_json::to_string(&StopReason::EndTurn).unwrap();
        assert_eq!(json, "\"end_turn\"");

        let json = serde_json::to_string(&StopReason::ToolUse).unwrap();
        assert_eq!(json, "\"tool_use\"");
    }

    #[test]
    fn content_parts_serialize_with_type_tag() {
        let parts = Content::Parts(vec![
            ContentPart::Text {
                text: "Hello".to_string(),
            },
            ContentPart::Image {
                url: Some("https://example.com/img.png".to_string()),
                base64: None,
                media_type: None,
            },
        ]);
        let json = serde_json::to_string(&parts).unwrap();
        assert!(json.contains("\"type\":\"text\""));
        assert!(json.contains("\"type\":\"image\""));
    }

    #[test]
    fn embed_request_creates_correctly() {
        let req = EmbedRequest::new("text-embedding-3-large", vec!["hello".to_string()]);
        assert_eq!(req.model, "text-embedding-3-large");
        assert_eq!(req.texts.len(), 1);
    }
}
