//! WebSocket protocol message types
//!
//! Both CLI and Web UI use the same protocol for consistent behavior.

use serde::{Deserialize, Serialize};
use vibes_core::ClaudeEvent;

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Subscribe to session events
    Subscribe {
        /// Session IDs to subscribe to
        session_ids: Vec<String>,
    },

    /// Unsubscribe from session events
    Unsubscribe {
        /// Session IDs to unsubscribe from
        session_ids: Vec<String>,
    },

    /// Create a new session
    CreateSession {
        /// Optional session name
        name: Option<String>,
        /// Request ID for correlation
        request_id: String,
    },

    /// Send user input to a session
    Input {
        /// Target session ID
        session_id: String,
        /// Input content
        content: String,
    },

    /// Respond to a permission request
    PermissionResponse {
        /// Target session ID
        session_id: String,
        /// Permission request ID
        request_id: String,
        /// Whether to approve
        approved: bool,
    },
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Session created confirmation
    SessionCreated {
        /// Original request ID
        request_id: String,
        /// New session ID
        session_id: String,
        /// Session name if provided
        name: Option<String>,
    },

    /// Claude event from a session
    Claude {
        /// Source session ID
        session_id: String,
        /// The Claude event
        event: ClaudeEvent,
    },

    /// Session state change
    SessionState {
        /// Session ID
        session_id: String,
        /// New state
        state: String,
    },

    /// Error message
    Error {
        /// Session ID if applicable
        session_id: Option<String>,
        /// Error message
        message: String,
        /// Error code
        code: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_subscribe_roundtrip() {
        let msg = ClientMessage::Subscribe {
            session_ids: vec!["sess-1".to_string(), "sess-2".to_string()],
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);

        // Verify JSON structure
        assert!(json.contains(r#""type":"subscribe""#));
        assert!(json.contains(r#""session_ids""#));
    }

    #[test]
    fn test_client_message_unsubscribe_roundtrip() {
        let msg = ClientMessage::Unsubscribe {
            session_ids: vec!["sess-abc".to_string()],
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
    }

    #[test]
    fn test_client_message_create_session_roundtrip() {
        let msg = ClientMessage::CreateSession {
            name: Some("my-feature".to_string()),
            request_id: "req-1".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);

        assert!(json.contains(r#""type":"create_session""#));
    }

    #[test]
    fn test_client_message_input_roundtrip() {
        let msg = ClientMessage::Input {
            session_id: "sess-abc".to_string(),
            content: "Help me refactor this".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
    }

    #[test]
    fn test_client_message_permission_response_roundtrip() {
        let msg = ClientMessage::PermissionResponse {
            session_id: "sess-abc".to_string(),
            request_id: "perm-1".to_string(),
            approved: true,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);

        assert!(json.contains(r#""type":"permission_response""#));
    }

    #[test]
    fn test_server_message_session_created_roundtrip() {
        let msg = ServerMessage::SessionCreated {
            request_id: "req-1".to_string(),
            session_id: "sess-abc".to_string(),
            name: Some("my-feature".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);

        assert!(json.contains(r#""type":"session_created""#));
    }

    #[test]
    fn test_server_message_claude_roundtrip() {
        let msg = ServerMessage::Claude {
            session_id: "sess-abc".to_string(),
            event: ClaudeEvent::TextDelta {
                text: "Here's how...".to_string(),
            },
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);

        assert!(json.contains(r#""type":"claude""#));
    }

    #[test]
    fn test_server_message_session_state_roundtrip() {
        let msg = ServerMessage::SessionState {
            session_id: "sess-abc".to_string(),
            state: "processing".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
    }

    #[test]
    fn test_server_message_error_roundtrip() {
        let msg = ServerMessage::Error {
            session_id: Some("sess-abc".to_string()),
            message: "Session not found".to_string(),
            code: "NOT_FOUND".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
    }
}
