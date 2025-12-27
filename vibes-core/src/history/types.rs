//! Core history types

use crate::session::SessionState;
use serde::{Deserialize, Serialize};

/// Role of a message in the conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    ToolUse,
    ToolResult,
}

impl MessageRole {
    /// Convert to database string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::ToolUse => "tool_use",
            Self::ToolResult => "tool_result",
        }
    }

    /// Parse from database string
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "user" => Some(Self::User),
            "assistant" => Some(Self::Assistant),
            "tool_use" => Some(Self::ToolUse),
            "tool_result" => Some(Self::ToolResult),
            _ => None,
        }
    }
}

/// Aggregated message stored in history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalMessage {
    /// Auto-incremented database ID
    pub id: i64,
    /// Parent session ID
    pub session_id: String,
    /// Message role
    pub role: MessageRole,
    /// Message content (aggregated from deltas)
    pub content: String,
    /// Tool name for tool_use/tool_result messages
    pub tool_name: Option<String>,
    /// Tool invocation ID linking tool_use to tool_result
    pub tool_id: Option<String>,
    /// Unix timestamp (seconds)
    pub created_at: i64,
    /// Input tokens for this message
    pub input_tokens: Option<u32>,
    /// Output tokens for this message
    pub output_tokens: Option<u32>,
}

impl HistoricalMessage {
    /// Create a new user message
    pub fn user(session_id: String, content: String, created_at: i64) -> Self {
        Self {
            id: 0, // Set by database
            session_id,
            role: MessageRole::User,
            content,
            tool_name: None,
            tool_id: None,
            created_at,
            input_tokens: None,
            output_tokens: None,
        }
    }

    /// Create a new assistant message
    pub fn assistant(session_id: String, content: String, created_at: i64) -> Self {
        Self {
            id: 0,
            session_id,
            role: MessageRole::Assistant,
            content,
            tool_name: None,
            tool_id: None,
            created_at,
            input_tokens: None,
            output_tokens: None,
        }
    }

    /// Create a tool use message
    pub fn tool_use(
        session_id: String,
        tool_id: String,
        tool_name: String,
        content: String,
        created_at: i64,
    ) -> Self {
        Self {
            id: 0,
            session_id,
            role: MessageRole::ToolUse,
            content,
            tool_name: Some(tool_name),
            tool_id: Some(tool_id),
            created_at,
            input_tokens: None,
            output_tokens: None,
        }
    }

    /// Create a tool result message
    pub fn tool_result(
        session_id: String,
        tool_id: String,
        tool_name: String,
        content: String,
        created_at: i64,
    ) -> Self {
        Self {
            id: 0,
            session_id,
            role: MessageRole::ToolResult,
            content,
            tool_name: Some(tool_name),
            tool_id: Some(tool_id),
            created_at,
            input_tokens: None,
            output_tokens: None,
        }
    }
}

/// Persisted session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalSession {
    /// Session UUID
    pub id: String,
    /// Human-readable name
    pub name: Option<String>,
    /// Claude's session ID for resume
    pub claude_session_id: Option<String>,
    /// Final session state (stored as simplified enum)
    pub state: SessionState,
    /// Unix timestamp (seconds)
    pub created_at: i64,
    /// Last activity timestamp
    pub last_accessed_at: i64,
    /// Total input tokens used
    pub total_input_tokens: u32,
    /// Total output tokens used
    pub total_output_tokens: u32,
    /// Number of messages
    pub message_count: u32,
    /// Error message if state is Failed
    pub error_message: Option<String>,
}

impl HistoricalSession {
    /// Create a new session with timestamps set to now
    pub fn new(id: String, name: Option<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Self {
            id,
            name,
            claude_session_id: None,
            state: SessionState::Idle,
            created_at: now,
            last_accessed_at: now,
            total_input_tokens: 0,
            total_output_tokens: 0,
            message_count: 0,
            error_message: None,
        }
    }
}

/// Session summary for list views (lighter than full HistoricalSession)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub name: Option<String>,
    pub state: SessionState,
    pub created_at: i64,
    pub last_accessed_at: i64,
    pub message_count: u32,
    pub total_tokens: u32,
    /// First ~100 chars of first user message
    pub preview: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_role_roundtrip() {
        for role in [
            MessageRole::User,
            MessageRole::Assistant,
            MessageRole::ToolUse,
            MessageRole::ToolResult,
        ] {
            let s = role.as_str();
            let parsed = MessageRole::parse(s);
            assert_eq!(parsed, Some(role));
        }
    }

    #[test]
    fn test_message_role_serde() {
        let role = MessageRole::ToolUse;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"tool_use\"");

        let parsed: MessageRole = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, role);
    }

    #[test]
    fn test_historical_message_user() {
        let msg = HistoricalMessage::user("sess-1".into(), "Hello".into(), 1234567890);
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.session_id, "sess-1");
        assert_eq!(msg.content, "Hello");
        assert!(msg.tool_name.is_none());
    }

    #[test]
    fn test_historical_message_tool_use() {
        let msg = HistoricalMessage::tool_use(
            "sess-1".into(),
            "tool-123".into(),
            "Read".into(),
            "{\"path\": \"/tmp\"}".into(),
            1234567890,
        );
        assert_eq!(msg.role, MessageRole::ToolUse);
        assert_eq!(msg.tool_name, Some("Read".into()));
        assert_eq!(msg.tool_id, Some("tool-123".into()));
    }

    #[test]
    fn test_historical_session_new() {
        let session = HistoricalSession::new("sess-123".into(), Some("Test".into()));
        assert_eq!(session.id, "sess-123");
        assert_eq!(session.name, Some("Test".into()));
        assert!(matches!(session.state, SessionState::Idle));
        assert!(session.created_at > 0);
        assert_eq!(session.total_input_tokens, 0);
    }
}
