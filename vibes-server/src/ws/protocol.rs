//! WebSocket protocol message types
//!
//! Both CLI and Web UI use the same protocol for consistent behavior.

use serde::{Deserialize, Serialize};
use vibes_core::{AuthContext, ClaudeEvent};

/// Information about an active session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionInfo {
    pub id: String,
    pub name: Option<String>,
    pub state: String,
    pub owner_id: String,
    pub is_owner: bool,
    pub subscriber_count: u32,
    pub created_at: i64,
    pub last_activity_at: i64,
}

/// Reason a session was removed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RemovalReason {
    OwnerDisconnected,
    Killed,
    SessionFinished,
}

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
    /// Session created confirmation (response to CreateSession)
    SessionCreated {
        /// Original request ID
        request_id: String,
        /// New session ID
        session_id: String,
        /// Session name if provided
        name: Option<String>,
    },

    /// Session notification (broadcast when session is created by another client)
    SessionNotification {
        /// Session ID
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

    /// Tunnel state update
    TunnelState {
        /// Current state
        state: String,
        /// Public URL when connected
        url: Option<String>,
    },

    /// Auth context sent on connection
    AuthContext(AuthContext),
}

use vibes_core::VibesEvent;

/// Convert a VibesEvent to a ServerMessage for broadcasting
///
/// Returns None for events that shouldn't be broadcast to clients
/// (e.g., UserInput, PermissionResponse, ClientConnected/Disconnected)
pub fn vibes_event_to_server_message(event: &VibesEvent) -> Option<ServerMessage> {
    match event {
        VibesEvent::Claude { session_id, event } => Some(ServerMessage::Claude {
            session_id: session_id.clone(),
            event: event.clone(),
        }),
        VibesEvent::SessionStateChanged { session_id, state } => {
            Some(ServerMessage::SessionState {
                session_id: session_id.clone(),
                state: state.clone(),
            })
        }
        VibesEvent::SessionCreated { session_id, name } => {
            Some(ServerMessage::SessionNotification {
                session_id: session_id.clone(),
                name: name.clone(),
            })
        }
        // Tunnel state changes are broadcast to clients
        VibesEvent::TunnelStateChanged { state, url } => Some(ServerMessage::TunnelState {
            state: state.clone(),
            url: url.clone(),
        }),
        // These events are not broadcast to clients
        VibesEvent::UserInput { .. } => None,
        VibesEvent::PermissionResponse { .. } => None,
        VibesEvent::ClientConnected { .. } => None,
        VibesEvent::ClientDisconnected { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== SessionInfo Tests ====================

    #[test]
    fn test_session_info_serialization() {
        let info = SessionInfo {
            id: "sess-1".to_string(),
            name: Some("test".to_string()),
            state: "Idle".to_string(),
            owner_id: "client-1".to_string(),
            is_owner: true,
            subscriber_count: 2,
            created_at: 1234567890,
            last_activity_at: 1234567900,
        };

        let json = serde_json::to_string(&info).unwrap();
        let parsed: SessionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info, parsed);
    }

    #[test]
    fn test_removal_reason_serialization() {
        let reasons = vec![
            RemovalReason::OwnerDisconnected,
            RemovalReason::Killed,
            RemovalReason::SessionFinished,
        ];

        for reason in reasons {
            let json = serde_json::to_string(&reason).unwrap();
            let parsed: RemovalReason = serde_json::from_str(&json).unwrap();
            assert_eq!(reason, parsed);
        }
    }

    // ==================== ClientMessage Tests ====================

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

    #[test]
    fn test_server_message_session_notification_roundtrip() {
        let msg = ServerMessage::SessionNotification {
            session_id: "sess-abc".to_string(),
            name: Some("new-session".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);

        assert!(json.contains(r#""type":"session_notification""#));
    }

    // ==================== Event Conversion Tests ====================

    #[test]
    fn test_vibes_event_claude_converts_to_server_message() {
        let vibes_event = VibesEvent::Claude {
            session_id: "sess-1".to_string(),
            event: ClaudeEvent::TextDelta {
                text: "Hello".to_string(),
            },
        };

        let server_msg = vibes_event_to_server_message(&vibes_event);
        match server_msg {
            Some(ServerMessage::Claude { session_id, event }) => {
                assert_eq!(session_id, "sess-1");
                match event {
                    ClaudeEvent::TextDelta { text } => assert_eq!(text, "Hello"),
                    _ => panic!("Expected TextDelta event"),
                }
            }
            _ => panic!("Expected Some(Claude) message"),
        }
    }

    #[test]
    fn test_vibes_event_session_state_changed_converts() {
        let vibes_event = VibesEvent::SessionStateChanged {
            session_id: "sess-1".to_string(),
            state: "processing".to_string(),
        };

        let server_msg = vibes_event_to_server_message(&vibes_event);
        assert!(matches!(
            server_msg,
            Some(ServerMessage::SessionState { session_id, state })
            if session_id == "sess-1" && state == "processing"
        ));
    }

    #[test]
    fn test_vibes_event_session_created_converts_to_notification() {
        let vibes_event = VibesEvent::SessionCreated {
            session_id: "sess-new".to_string(),
            name: Some("my-session".to_string()),
        };

        let server_msg = vibes_event_to_server_message(&vibes_event);
        assert!(matches!(
            server_msg,
            Some(ServerMessage::SessionNotification { session_id, name })
            if session_id == "sess-new" && name == Some("my-session".to_string())
        ));
    }

    #[test]
    fn test_vibes_event_user_input_not_broadcast() {
        let vibes_event = VibesEvent::UserInput {
            session_id: "sess-1".to_string(),
            content: "test input".to_string(),
        };

        assert!(vibes_event_to_server_message(&vibes_event).is_none());
    }

    #[test]
    fn test_vibes_event_permission_response_not_broadcast() {
        let vibes_event = VibesEvent::PermissionResponse {
            session_id: "sess-1".to_string(),
            request_id: "req-1".to_string(),
            approved: true,
        };

        assert!(vibes_event_to_server_message(&vibes_event).is_none());
    }

    #[test]
    fn test_vibes_event_client_events_not_broadcast() {
        let connected = VibesEvent::ClientConnected {
            client_id: "client-1".to_string(),
        };
        let disconnected = VibesEvent::ClientDisconnected {
            client_id: "client-1".to_string(),
        };

        assert!(vibes_event_to_server_message(&connected).is_none());
        assert!(vibes_event_to_server_message(&disconnected).is_none());
    }

    // ==================== Tunnel State Tests ====================

    #[test]
    fn test_server_message_tunnel_state_serialization() {
        let msg = ServerMessage::TunnelState {
            state: "connected".to_string(),
            url: Some("https://test.trycloudflare.com".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("tunnel_state"));
        assert!(json.contains("connected"));
    }

    #[test]
    fn test_server_message_tunnel_state_roundtrip() {
        let msg = ServerMessage::TunnelState {
            state: "connected".to_string(),
            url: Some("https://test.trycloudflare.com".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
    }

    #[test]
    fn test_vibes_event_tunnel_state_changed_converts() {
        let vibes_event = VibesEvent::TunnelStateChanged {
            state: "connected".to_string(),
            url: Some("https://test.trycloudflare.com".to_string()),
        };

        let server_msg = vibes_event_to_server_message(&vibes_event);
        assert!(matches!(
            server_msg,
            Some(ServerMessage::TunnelState { state, url })
            if state == "connected" && url == Some("https://test.trycloudflare.com".to_string())
        ));
    }

    // ==================== Auth Context Tests ====================

    #[test]
    fn test_server_message_auth_context_local_roundtrip() {
        let msg = ServerMessage::AuthContext(AuthContext::Local);
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""type":"auth_context""#));
        assert!(json.contains(r#""source":"local""#));
    }

    #[test]
    fn test_server_message_auth_context_anonymous_roundtrip() {
        let msg = ServerMessage::AuthContext(AuthContext::Anonymous);
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""source":"anonymous""#));
    }

    #[test]
    fn test_server_message_auth_context_authenticated_roundtrip() {
        use chrono::Utc;
        use vibes_core::AccessIdentity;

        let identity = AccessIdentity::new("user@example.com".to_string(), Utc::now())
            .with_name("Test User".to_string());
        let msg = ServerMessage::AuthContext(AuthContext::Authenticated { identity });
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""source":"authenticated""#));
        assert!(json.contains(r#""email":"user@example.com""#));
    }
}
