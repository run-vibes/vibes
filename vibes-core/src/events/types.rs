//! Event type definitions

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vibes_iggy::Partitionable;

use crate::hooks::HookEvent;

/// Source of user input for attribution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InputSource {
    /// Input from CLI client
    Cli,
    /// Input from Web UI client
    WebUi,
    /// Source unknown (e.g., historical data before migration)
    #[default]
    Unknown,
}

impl InputSource {
    /// Convert to database/JSON string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::WebUi => "web_ui",
            Self::Unknown => "unknown",
        }
    }

    /// Parse from database/JSON string
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "cli" => Some(Self::Cli),
            "web_ui" => Some(Self::WebUi),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VibesEvent {
    /// Event from Claude (wrapped)
    Claude {
        session_id: String,
        event: ClaudeEvent,
    },

    /// User input from any client
    UserInput {
        session_id: String,
        content: String,
        #[serde(default)]
        source: InputSource,
    },

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
    TunnelStateChanged { state: String, url: Option<String> },

    /// Session ownership was transferred
    OwnershipTransferred {
        session_id: String,
        new_owner_id: String,
    },

    /// Session was removed
    SessionRemoved { session_id: String, reason: String },

    /// Hook event from Claude Code
    Hook {
        session_id: Option<String>,
        event: HookEvent,
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
            VibesEvent::OwnershipTransferred { session_id, .. } => Some(session_id),
            VibesEvent::SessionRemoved { session_id, .. } => Some(session_id),
            VibesEvent::Hook { session_id, .. } => session_id.as_deref(),
            VibesEvent::ClientConnected { .. } => None,
            VibesEvent::ClientDisconnected { .. } => None,
            VibesEvent::TunnelStateChanged { .. } => None,
        }
    }
}

impl Partitionable for VibesEvent {
    fn partition_key(&self) -> Option<&str> {
        self.session_id()
    }
}

/// A VibesEvent with a globally unique UUIDv7 identifier.
///
/// This wrapper is used for storage in the EventLog. The event_id provides
/// a globally unique, time-ordered identifier that works across Iggy partitions
/// (unlike partition-scoped offsets).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredEvent {
    /// Globally unique, time-ordered event identifier (UUIDv7)
    pub event_id: Uuid,
    /// The event payload
    #[serde(flatten)]
    pub event: VibesEvent,
}

impl StoredEvent {
    /// Create a new StoredEvent with a fresh UUIDv7 identifier.
    #[must_use]
    pub fn new(event: VibesEvent) -> Self {
        Self {
            event_id: Uuid::now_v7(),
            event,
        }
    }

    /// Get the session ID from the inner event.
    #[must_use]
    pub fn session_id(&self) -> Option<&str> {
        self.event.session_id()
    }
}

impl Partitionable for StoredEvent {
    fn partition_key(&self) -> Option<&str> {
        self.event.partition_key()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== InputSource Tests ====================

    #[test]
    fn input_source_as_str_returns_correct_values() {
        assert_eq!(InputSource::Cli.as_str(), "cli");
        assert_eq!(InputSource::WebUi.as_str(), "web_ui");
        assert_eq!(InputSource::Unknown.as_str(), "unknown");
    }

    #[test]
    fn input_source_parse_returns_correct_variants() {
        assert_eq!(InputSource::parse("cli"), Some(InputSource::Cli));
        assert_eq!(InputSource::parse("web_ui"), Some(InputSource::WebUi));
        assert_eq!(InputSource::parse("unknown"), Some(InputSource::Unknown));
        assert_eq!(InputSource::parse("invalid"), None);
    }

    #[test]
    fn input_source_serialization_roundtrip() {
        for source in [InputSource::Cli, InputSource::WebUi, InputSource::Unknown] {
            let json = serde_json::to_string(&source).unwrap();
            let parsed: InputSource = serde_json::from_str(&json).unwrap();
            assert_eq!(source, parsed);
        }
    }

    #[test]
    fn input_source_serializes_to_snake_case() {
        let json = serde_json::to_string(&InputSource::WebUi).unwrap();
        assert_eq!(json, "\"web_ui\"");
    }

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
            source: InputSource::Unknown,
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: VibesEvent = serde_json::from_str(&json).unwrap();
        assert!(
            matches!(parsed, VibesEvent::UserInput { session_id, content, source }
            if session_id == "sess-456" && content == "Help me code" && source == InputSource::Unknown)
        );
    }

    #[test]
    fn vibes_event_user_input_with_source_serialization_roundtrip() {
        let event = VibesEvent::UserInput {
            session_id: "sess-456".to_string(),
            content: "Help me code".to_string(),
            source: InputSource::Cli,
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: VibesEvent = serde_json::from_str(&json).unwrap();
        assert!(
            matches!(parsed, VibesEvent::UserInput { session_id, content, source }
            if session_id == "sess-456" && content == "Help me code" && source == InputSource::Cli)
        );
    }

    #[test]
    fn vibes_event_user_input_deserializes_without_source() {
        // Backward compatibility: old messages without source field
        let json = r#"{"type":"user_input","session_id":"sess-1","content":"hello"}"#;
        let parsed: VibesEvent = serde_json::from_str(json).unwrap();
        assert!(
            matches!(parsed, VibesEvent::UserInput { session_id, content, source }
            if session_id == "sess-1" && content == "hello" && source == InputSource::Unknown)
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
                source: InputSource::Unknown,
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
        assert!(
            matches!(parsed, VibesEvent::TunnelStateChanged { state, url }
            if state == "connected" && url == Some("https://test.trycloudflare.com".to_string()))
        );
    }

    #[test]
    fn vibes_event_tunnel_state_changed_session_id_is_none() {
        let event = VibesEvent::TunnelStateChanged {
            state: "starting".to_string(),
            url: None,
        };
        assert_eq!(event.session_id(), None);
    }

    // ==================== Hook Event Tests ====================

    #[test]
    fn vibes_event_hook_serialization_roundtrip() {
        use crate::hooks::HookEvent;
        use crate::hooks::PreToolUseData;

        let hook_event = HookEvent::PreToolUse(PreToolUseData {
            tool_name: "Bash".to_string(),
            input: r#"{"command": "ls"}"#.to_string(),
            session_id: Some("sess-123".to_string()),
        });

        let event = VibesEvent::Hook {
            session_id: Some("sess-123".to_string()),
            event: hook_event,
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: VibesEvent = serde_json::from_str(&json).unwrap();

        assert!(
            matches!(parsed, VibesEvent::Hook { session_id: Some(id), .. } if id == "sess-123")
        );
    }

    #[test]
    fn vibes_event_hook_session_id_extraction() {
        use crate::hooks::HookEvent;
        use crate::hooks::StopData;

        let hook_event = HookEvent::Stop(StopData {
            transcript_path: None,
            reason: Some("user".to_string()),
            session_id: Some("sess-456".to_string()),
        });

        let event = VibesEvent::Hook {
            session_id: Some("sess-456".to_string()),
            event: hook_event,
        };

        assert_eq!(event.session_id(), Some("sess-456"));
    }

    // ==================== StoredEvent Tests ====================

    #[test]
    fn stored_event_new_generates_uuidv7() {
        let event = VibesEvent::ClientConnected {
            client_id: "c1".to_string(),
        };
        let stored = StoredEvent::new(event);

        // UUIDv7 has version 7 in bits 48-51
        assert_eq!(stored.event_id.get_version_num(), 7);
    }

    #[test]
    fn stored_event_ids_are_unique() {
        let event1 = StoredEvent::new(VibesEvent::ClientConnected {
            client_id: "c1".to_string(),
        });
        let event2 = StoredEvent::new(VibesEvent::ClientConnected {
            client_id: "c2".to_string(),
        });

        assert_ne!(event1.event_id, event2.event_id);
    }

    #[test]
    fn stored_event_ids_are_time_ordered() {
        let event1 = StoredEvent::new(VibesEvent::ClientConnected {
            client_id: "c1".to_string(),
        });
        // Small delay to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_millis(1));
        let event2 = StoredEvent::new(VibesEvent::ClientConnected {
            client_id: "c2".to_string(),
        });

        // UUIDv7 is lexicographically sortable by time
        assert!(event1.event_id < event2.event_id);
    }

    #[test]
    fn stored_event_session_id_delegates_to_inner() {
        let event = StoredEvent::new(VibesEvent::SessionCreated {
            session_id: "sess-123".to_string(),
            name: None,
        });

        assert_eq!(event.session_id(), Some("sess-123"));
    }

    #[test]
    fn stored_event_partition_key_delegates_to_inner() {
        let event = StoredEvent::new(VibesEvent::SessionCreated {
            session_id: "sess-123".to_string(),
            name: None,
        });

        assert_eq!(event.partition_key(), Some("sess-123"));
    }

    #[test]
    fn stored_event_serialization_flattens_event() {
        let event = StoredEvent::new(VibesEvent::ClientConnected {
            client_id: "c1".to_string(),
        });
        let json = serde_json::to_string(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // event_id should be at top level
        assert!(parsed["event_id"].is_string());

        // Inner event fields should be flattened (not nested under "event")
        assert_eq!(parsed["type"], "client_connected");
        assert_eq!(parsed["client_id"], "c1");

        // Should NOT have a nested "event" object
        assert!(parsed.get("event").is_none());
    }

    #[test]
    fn stored_event_deserialization_roundtrip() {
        let original = StoredEvent::new(VibesEvent::SessionCreated {
            session_id: "sess-456".to_string(),
            name: Some("Test Session".to_string()),
        });

        let json = serde_json::to_string(&original).unwrap();
        let parsed: StoredEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.event_id, original.event_id);
        assert_eq!(parsed.event, original.event);
    }

    #[test]
    fn stored_event_clone_preserves_event_id() {
        let original = StoredEvent::new(VibesEvent::ClientConnected {
            client_id: "c1".to_string(),
        });
        let cloned = original.clone();

        assert_eq!(original.event_id, cloned.event_id);
        assert_eq!(original.event, cloned.event);
    }
}
