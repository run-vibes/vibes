//! Event type definitions

use serde::{Deserialize, Serialize};

/// Token usage statistics from Claude
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Events emitted by Claude backends (normalized across backends)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClaudeEvent {
    /// Text content being streamed
    TextDelta { text: String },

    /// Thinking/reasoning content being streamed
    ThinkingDelta { text: String },

    /// Tool use has started
    ToolUseStart { id: String, name: String },

    /// Tool input being streamed
    ToolInputDelta { id: String, delta: String },

    /// Tool execution result
    ToolResult {
        id: String,
        output: String,
        is_error: bool,
    },

    /// A new turn has started
    TurnStart,

    /// Turn has completed with usage stats
    TurnComplete { usage: Usage },

    /// An error occurred
    Error { message: String, recoverable: bool },

    /// Permission requested for tool use
    PermissionRequest {
        id: String,
        tool: String,
        description: String,
    },
}

/// Events on the VibesEventBus (includes client events)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VibesEvent {
    /// Event from Claude (wrapped)
    Claude {
        session_id: String,
        event: ClaudeEvent,
    },

    /// User input from any client
    UserInput { session_id: String, content: String },

    /// Permission response from client
    PermissionResponse {
        session_id: String,
        request_id: String,
        approved: bool,
    },

    /// New session was created
    SessionCreated {
        session_id: String,
        name: Option<String>,
    },

    /// Session state changed
    SessionStateChanged { session_id: String, state: String },

    /// Client connected to server
    ClientConnected { client_id: String },

    /// Client disconnected from server
    ClientDisconnected { client_id: String },

    /// Tunnel state changed
    TunnelStateChanged {
        state: String,
        url: Option<String>,
    },
}

impl VibesEvent {
    /// Extract session ID if this event is session-related
    pub fn session_id(&self) -> Option<&str> {
        match self {
            VibesEvent::Claude { session_id, .. } => Some(session_id),
            VibesEvent::UserInput { session_id, .. } => Some(session_id),
            VibesEvent::PermissionResponse { session_id, .. } => Some(session_id),
            VibesEvent::SessionCreated { session_id, .. } => Some(session_id),
            VibesEvent::SessionStateChanged { session_id, .. } => Some(session_id),
            VibesEvent::ClientConnected { .. } => None,
            VibesEvent::ClientDisconnected { .. } => None,
            VibesEvent::TunnelStateChanged { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Usage Tests ====================

    #[test]
    fn usage_default_is_zero() {
        let usage = Usage::default();
        assert_eq!(usage.input_tokens, 0);
        assert_eq!(usage.output_tokens, 0);
    }

    #[test]
    fn usage_serialization_roundtrip() {
        let usage = Usage {
            input_tokens: 100,
            output_tokens: 200,
        };
        let json = serde_json::to_string(&usage).unwrap();
        let parsed: Usage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.input_tokens, 100);
        assert_eq!(parsed.output_tokens, 200);
    }

    // ==================== ClaudeEvent Tests ====================

    #[test]
    fn claude_event_text_delta_clone_and_debug() {
        let event = ClaudeEvent::TextDelta {
            text: "Hello".to_string(),
        };
        let cloned = event.clone();
        assert!(format!("{:?}", cloned).contains("TextDelta"));
    }

    #[test]
    fn claude_event_text_delta_serialization_roundtrip() {
        let event = ClaudeEvent::TextDelta {
            text: "Hello world".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: ClaudeEvent = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, ClaudeEvent::TextDelta { text } if text == "Hello world"));
    }

    #[test]
    fn claude_event_thinking_delta_serialization_roundtrip() {
        let event = ClaudeEvent::ThinkingDelta {
            text: "Considering...".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: ClaudeEvent = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, ClaudeEvent::ThinkingDelta { text } if text == "Considering..."));
    }

    #[test]
    fn claude_event_tool_use_start_serialization_roundtrip() {
        let event = ClaudeEvent::ToolUseStart {
            id: "tool-123".to_string(),
            name: "Bash".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: ClaudeEvent = serde_json::from_str(&json).unwrap();
        assert!(
            matches!(parsed, ClaudeEvent::ToolUseStart { id, name } if id == "tool-123" && name == "Bash")
        );
    }

    #[test]
    fn claude_event_tool_input_delta_serialization_roundtrip() {
        let event = ClaudeEvent::ToolInputDelta {
            id: "tool-123".to_string(),
            delta: "ls -la".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: ClaudeEvent = serde_json::from_str(&json).unwrap();
        assert!(
            matches!(parsed, ClaudeEvent::ToolInputDelta { id, delta } if id == "tool-123" && delta == "ls -la")
        );
    }

    #[test]
    fn claude_event_tool_result_serialization_roundtrip() {
        let event = ClaudeEvent::ToolResult {
            id: "tool-123".to_string(),
            output: "file.txt".to_string(),
            is_error: false,
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: ClaudeEvent = serde_json::from_str(&json).unwrap();
        assert!(
            matches!(parsed, ClaudeEvent::ToolResult { id, output, is_error }
            if id == "tool-123" && output == "file.txt" && !is_error)
        );
    }

    #[test]
    fn claude_event_turn_start_serialization_roundtrip() {
        let event = ClaudeEvent::TurnStart;
        let json = serde_json::to_string(&event).unwrap();
        let parsed: ClaudeEvent = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, ClaudeEvent::TurnStart));
    }

    #[test]
    fn claude_event_turn_complete_serialization_roundtrip() {
        let event = ClaudeEvent::TurnComplete {
            usage: Usage {
                input_tokens: 50,
                output_tokens: 100,
            },
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: ClaudeEvent = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, ClaudeEvent::TurnComplete { usage }
            if usage.input_tokens == 50 && usage.output_tokens == 100));
    }

    #[test]
    fn claude_event_error_serialization_roundtrip() {
        let event = ClaudeEvent::Error {
            message: "Something went wrong".to_string(),
            recoverable: true,
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: ClaudeEvent = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, ClaudeEvent::Error { message, recoverable }
            if message == "Something went wrong" && recoverable));
    }

    // ==================== VibesEvent Tests ====================

    #[test]
    fn vibes_event_claude_serialization_roundtrip() {
        let event = VibesEvent::Claude {
            session_id: "sess-123".to_string(),
            event: ClaudeEvent::TextDelta {
                text: "Hi".to_string(),
            },
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: VibesEvent = serde_json::from_str(&json).unwrap();
        assert!(
            matches!(parsed, VibesEvent::Claude { session_id, .. } if session_id == "sess-123")
        );
    }

    #[test]
    fn vibes_event_user_input_serialization_roundtrip() {
        let event = VibesEvent::UserInput {
            session_id: "sess-456".to_string(),
            content: "Help me code".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: VibesEvent = serde_json::from_str(&json).unwrap();
        assert!(
            matches!(parsed, VibesEvent::UserInput { session_id, content }
            if session_id == "sess-456" && content == "Help me code")
        );
    }

    #[test]
    fn vibes_event_permission_response_serialization_roundtrip() {
        let event = VibesEvent::PermissionResponse {
            session_id: "sess-789".to_string(),
            request_id: "req-1".to_string(),
            approved: true,
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: VibesEvent = serde_json::from_str(&json).unwrap();
        assert!(
            matches!(parsed, VibesEvent::PermissionResponse { session_id, request_id, approved }
            if session_id == "sess-789" && request_id == "req-1" && approved)
        );
    }

    #[test]
    fn vibes_event_session_created_serialization_roundtrip() {
        let event = VibesEvent::SessionCreated {
            session_id: "sess-new".to_string(),
            name: Some("My Session".to_string()),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: VibesEvent = serde_json::from_str(&json).unwrap();
        assert!(
            matches!(parsed, VibesEvent::SessionCreated { session_id, name }
            if session_id == "sess-new" && name == Some("My Session".to_string()))
        );
    }

    #[test]
    fn vibes_event_session_state_changed_serialization_roundtrip() {
        let event = VibesEvent::SessionStateChanged {
            session_id: "sess-1".to_string(),
            state: "Processing".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: VibesEvent = serde_json::from_str(&json).unwrap();
        assert!(
            matches!(parsed, VibesEvent::SessionStateChanged { session_id, state }
            if session_id == "sess-1" && state == "Processing")
        );
    }

    #[test]
    fn vibes_event_client_connected_serialization_roundtrip() {
        let event = VibesEvent::ClientConnected {
            client_id: "client-abc".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: VibesEvent = serde_json::from_str(&json).unwrap();
        assert!(
            matches!(parsed, VibesEvent::ClientConnected { client_id } if client_id == "client-abc")
        );
    }

    #[test]
    fn vibes_event_client_disconnected_serialization_roundtrip() {
        let event = VibesEvent::ClientDisconnected {
            client_id: "client-xyz".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: VibesEvent = serde_json::from_str(&json).unwrap();
        assert!(
            matches!(parsed, VibesEvent::ClientDisconnected { client_id } if client_id == "client-xyz")
        );
    }

    // ==================== Session ID Extraction Tests ====================

    #[test]
    fn vibes_event_session_id_returns_some_for_session_events() {
        let event = VibesEvent::Claude {
            session_id: "sess-123".to_string(),
            event: ClaudeEvent::TurnStart,
        };
        assert_eq!(event.session_id(), Some("sess-123"));
    }

    #[test]
    fn vibes_event_session_id_returns_none_for_client_events() {
        let event = VibesEvent::ClientConnected {
            client_id: "client-1".to_string(),
        };
        assert_eq!(event.session_id(), None);
    }

    #[test]
    fn vibes_event_session_id_works_for_all_session_event_types() {
        let events = [
            VibesEvent::Claude {
                session_id: "s1".to_string(),
                event: ClaudeEvent::TurnStart,
            },
            VibesEvent::UserInput {
                session_id: "s2".to_string(),
                content: "test".to_string(),
            },
            VibesEvent::PermissionResponse {
                session_id: "s3".to_string(),
                request_id: "r1".to_string(),
                approved: true,
            },
            VibesEvent::SessionCreated {
                session_id: "s4".to_string(),
                name: None,
            },
            VibesEvent::SessionStateChanged {
                session_id: "s5".to_string(),
                state: "Idle".to_string(),
            },
        ];

        for (i, event) in events.iter().enumerate() {
            assert_eq!(event.session_id(), Some(format!("s{}", i + 1).as_str()));
        }
    }

    // ==================== TunnelStateChanged Tests ====================

    #[test]
    fn vibes_event_tunnel_state_changed_serialization_roundtrip() {
        let event = VibesEvent::TunnelStateChanged {
            state: "connected".to_string(),
            url: Some("https://test.trycloudflare.com".to_string()),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: VibesEvent = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, VibesEvent::TunnelStateChanged { state, url }
            if state == "connected" && url == Some("https://test.trycloudflare.com".to_string())));
    }

    #[test]
    fn vibes_event_tunnel_state_changed_session_id_is_none() {
        let event = VibesEvent::TunnelStateChanged {
            state: "starting".to_string(),
            url: None,
        };
        assert_eq!(event.session_id(), None);
    }
}
