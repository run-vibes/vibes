//! WebSocket protocol message types
//!
//! Both CLI and Web UI use the same protocol for consistent behavior.

use serde::{Deserialize, Serialize};
use vibes_core::{AuthContext, ClaudeEvent, InputSource, VibesEvent};

/// A historical event with sequence number for catch-up
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryEvent {
    /// Sequence number for ordering
    pub seq: u64,
    /// The actual event
    pub event: VibesEvent,
    /// Unix timestamp in milliseconds
    pub timestamp: i64,
}

fn default_history_limit() -> u32 {
    50
}

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
        /// Request catch-up with historical events
        #[serde(default)]
        catch_up: bool,
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

    /// Request list of all active sessions
    ListSessions {
        /// Request ID for correlation
        request_id: String,
    },

    /// Terminate a session
    KillSession {
        /// Session ID to terminate
        session_id: String,
    },

    /// Request additional history page
    RequestHistory {
        /// Session ID
        session_id: String,
        /// Return events with seq < before_seq
        before_seq: u64,
        /// Max events to return
        #[serde(default = "default_history_limit")]
        limit: u32,
    },

    // ==================== PTY Messages ====================

    /// Attach to a session (receive PTY output)
    Attach {
        /// Session ID to attach to
        session_id: String,
    },

    /// Detach from a session
    Detach {
        /// Session ID to detach from
        session_id: String,
    },

    /// Send input to PTY
    PtyInput {
        /// Target session ID
        session_id: String,
        /// Input data (base64 encoded)
        data: String,
    },

    /// Resize PTY
    PtyResize {
        /// Target session ID
        session_id: String,
        /// New column count
        cols: u16,
        /// New row count
        rows: u16,
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

    /// Full session list response
    SessionList {
        /// Original request ID
        request_id: String,
        /// List of session info
        sessions: Vec<SessionInfo>,
    },

    /// Session was removed
    SessionRemoved {
        /// Session ID that was removed
        session_id: String,
        /// Reason for removal
        reason: RemovalReason,
    },

    /// Ownership transferred to a new client
    OwnershipTransferred {
        /// Session ID
        session_id: String,
        /// New owner client ID
        new_owner_id: String,
        /// Whether the recipient is the new owner
        you_are_owner: bool,
    },

    /// Subscribe acknowledgment with history catch-up
    SubscribeAck {
        /// Session ID
        session_id: String,
        /// Current sequence number (live events continue from current_seq + 1)
        current_seq: u64,
        /// Historical events (most recent page)
        history: Vec<HistoryEvent>,
        /// Whether more history pages are available
        has_more: bool,
    },

    /// Additional history page response
    HistoryPage {
        /// Session ID
        session_id: String,
        /// Historical events for this page
        events: Vec<HistoryEvent>,
        /// Whether more pages exist before oldest_seq
        has_more: bool,
        /// Oldest sequence number in this page
        oldest_seq: u64,
    },

    /// User input broadcast to other subscribers
    UserInput {
        /// Session ID
        session_id: String,
        /// Input content
        content: String,
        /// Source of the input
        source: InputSource,
    },

    // ==================== PTY Messages ====================

    /// PTY output data
    PtyOutput {
        /// Source session ID
        session_id: String,
        /// Output data (base64 encoded)
        data: String,
    },

    /// PTY process exited
    PtyExit {
        /// Session ID
        session_id: String,
        /// Exit code (None if killed by signal)
        exit_code: Option<i32>,
    },

    /// Attach acknowledged
    AttachAck {
        /// Session ID
        session_id: String,
        /// Current terminal columns
        cols: u16,
        /// Current terminal rows
        rows: u16,
    },
}

/// Convert a VibesEvent to a ServerMessage for broadcasting
///
/// Returns None for events that shouldn't be broadcast to clients
/// (e.g., PermissionResponse, ClientConnected/Disconnected)
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
        // SessionRemoved is converted directly
        VibesEvent::SessionRemoved { session_id, reason } => {
            let removal_reason = match reason.as_str() {
                "killed" => RemovalReason::Killed,
                "session_finished" => RemovalReason::SessionFinished,
                _ => RemovalReason::OwnerDisconnected,
            };
            Some(ServerMessage::SessionRemoved {
                session_id: session_id.clone(),
                reason: removal_reason,
            })
        }
        // OwnershipTransferred needs special handling (client-specific you_are_owner)
        // It will be handled in handle_broadcast_event
        VibesEvent::OwnershipTransferred { .. } => None,
        // UserInput is broadcast to all subscribers (clients filter by source)
        VibesEvent::UserInput {
            session_id,
            content,
            source,
        } => Some(ServerMessage::UserInput {
            session_id: session_id.clone(),
            content: content.clone(),
            source: *source,
        }),
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
            catch_up: false,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);

        // Verify JSON structure
        assert!(json.contains(r#""type":"subscribe""#));
        assert!(json.contains(r#""session_ids""#));
    }

    #[test]
    fn test_client_message_subscribe_with_catch_up() {
        let msg = ClientMessage::Subscribe {
            session_ids: vec!["sess-1".to_string()],
            catch_up: true,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""catch_up":true"#));
    }

    #[test]
    fn test_client_message_subscribe_catch_up_defaults_false() {
        // When catch_up is not provided in JSON, it should default to false
        let json = r#"{"type":"subscribe","session_ids":["sess-1"]}"#;
        let parsed: ClientMessage = serde_json::from_str(json).unwrap();
        match parsed {
            ClientMessage::Subscribe { catch_up, .. } => assert!(!catch_up),
            _ => panic!("Expected Subscribe message"),
        }
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
    fn test_client_message_list_sessions_roundtrip() {
        let msg = ClientMessage::ListSessions {
            request_id: "req-1".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""type":"list_sessions""#));
    }

    #[test]
    fn test_client_message_kill_session_roundtrip() {
        let msg = ClientMessage::KillSession {
            session_id: "sess-1".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""type":"kill_session""#));
    }

    // ==================== ServerMessage Tests ====================

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
    fn test_vibes_event_user_input_converts() {
        let vibes_event = VibesEvent::UserInput {
            session_id: "sess-1".to_string(),
            content: "test input".to_string(),
            source: InputSource::Cli,
        };

        let server_msg = vibes_event_to_server_message(&vibes_event);
        assert!(matches!(
            server_msg,
            Some(ServerMessage::UserInput {
                session_id,
                content,
                source: InputSource::Cli,
            }) if session_id == "sess-1" && content == "test input"
        ));
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
    fn test_server_message_session_list_roundtrip() {
        let msg = ServerMessage::SessionList {
            request_id: "req-1".to_string(),
            sessions: vec![SessionInfo {
                id: "sess-1".to_string(),
                name: None,
                state: "Idle".to_string(),
                owner_id: "client-1".to_string(),
                is_owner: true,
                subscriber_count: 1,
                created_at: 0,
                last_activity_at: 0,
            }],
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""type":"session_list""#));
    }

    #[test]
    fn test_server_message_session_removed_roundtrip() {
        let msg = ServerMessage::SessionRemoved {
            session_id: "sess-1".to_string(),
            reason: RemovalReason::OwnerDisconnected,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""type":"session_removed""#));
    }

    #[test]
    fn test_server_message_ownership_transferred_roundtrip() {
        let msg = ServerMessage::OwnershipTransferred {
            session_id: "sess-1".to_string(),
            new_owner_id: "client-2".to_string(),
            you_are_owner: false,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""type":"ownership_transferred""#));
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

    // ==================== History Event Tests ====================

    #[test]
    fn test_history_event_serialization() {
        let event = HistoryEvent {
            seq: 42,
            event: VibesEvent::SessionStateChanged {
                session_id: "sess-1".to_string(),
                state: "processing".to_string(),
            },
            timestamp: 1234567890000,
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: HistoryEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed);
        assert!(json.contains(r#""seq":42"#));
        assert!(json.contains(r#""timestamp":1234567890000"#));
    }

    #[test]
    fn test_client_message_request_history_roundtrip() {
        let msg = ClientMessage::RequestHistory {
            session_id: "sess-1".to_string(),
            before_seq: 100,
            limit: 25,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""type":"request_history""#));
        assert!(json.contains(r#""before_seq":100"#));
    }

    #[test]
    fn test_client_message_request_history_default_limit() {
        let json = r#"{"type":"request_history","session_id":"sess-1","before_seq":50}"#;
        let parsed: ClientMessage = serde_json::from_str(json).unwrap();
        match parsed {
            ClientMessage::RequestHistory { limit, .. } => assert_eq!(limit, 50),
            _ => panic!("Expected RequestHistory message"),
        }
    }

    #[test]
    fn test_server_message_subscribe_ack_roundtrip() {
        let msg = ServerMessage::SubscribeAck {
            session_id: "sess-1".to_string(),
            current_seq: 42,
            history: vec![HistoryEvent {
                seq: 40,
                event: VibesEvent::SessionStateChanged {
                    session_id: "sess-1".to_string(),
                    state: "idle".to_string(),
                },
                timestamp: 1234567890000,
            }],
            has_more: true,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""type":"subscribe_ack""#));
        assert!(json.contains(r#""current_seq":42"#));
        assert!(json.contains(r#""has_more":true"#));
    }

    #[test]
    fn test_server_message_history_page_roundtrip() {
        let msg = ServerMessage::HistoryPage {
            session_id: "sess-1".to_string(),
            events: vec![],
            has_more: false,
            oldest_seq: 0,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""type":"history_page""#));
        assert!(json.contains(r#""oldest_seq":0"#));
    }

    #[test]
    fn test_server_message_user_input_roundtrip() {
        let msg = ServerMessage::UserInput {
            session_id: "sess-1".to_string(),
            content: "Hello from CLI".to_string(),
            source: InputSource::Cli,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""type":"user_input""#));
        assert!(json.contains(r#""source":"cli""#));
    }

    #[test]
    fn test_server_message_user_input_web_ui_source() {
        let msg = ServerMessage::UserInput {
            session_id: "sess-1".to_string(),
            content: "Hello from Web".to_string(),
            source: InputSource::WebUi,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""source":"web_ui""#));
    }

    // ==================== PTY Message Tests ====================

    #[test]
    fn test_client_message_attach_roundtrip() {
        let msg = ClientMessage::Attach {
            session_id: "sess-1".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""type":"attach""#));
    }

    #[test]
    fn test_client_message_detach_roundtrip() {
        let msg = ClientMessage::Detach {
            session_id: "sess-1".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""type":"detach""#));
    }

    #[test]
    fn test_client_message_pty_input_roundtrip() {
        let msg = ClientMessage::PtyInput {
            session_id: "sess-1".to_string(),
            data: "aGVsbG8=".to_string(), // base64 for "hello"
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""type":"pty_input""#));
    }

    #[test]
    fn test_client_message_pty_resize_roundtrip() {
        let msg = ClientMessage::PtyResize {
            session_id: "sess-1".to_string(),
            cols: 120,
            rows: 40,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""type":"pty_resize""#));
        assert!(json.contains(r#""cols":120"#));
        assert!(json.contains(r#""rows":40"#));
    }

    #[test]
    fn test_server_message_pty_output_roundtrip() {
        let msg = ServerMessage::PtyOutput {
            session_id: "sess-1".to_string(),
            data: "aGVsbG8gd29ybGQ=".to_string(), // base64 for "hello world"
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""type":"pty_output""#));
    }

    #[test]
    fn test_server_message_pty_exit_roundtrip() {
        let msg = ServerMessage::PtyExit {
            session_id: "sess-1".to_string(),
            exit_code: Some(0),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""type":"pty_exit""#));
        assert!(json.contains(r#""exit_code":0"#));
    }

    #[test]
    fn test_server_message_pty_exit_no_exit_code() {
        let msg = ServerMessage::PtyExit {
            session_id: "sess-1".to_string(),
            exit_code: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""exit_code":null"#));
    }

    #[test]
    fn test_server_message_attach_ack_roundtrip() {
        let msg = ServerMessage::AttachAck {
            session_id: "sess-1".to_string(),
            cols: 80,
            rows: 24,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""type":"attach_ack""#));
        assert!(json.contains(r#""cols":80"#));
        assert!(json.contains(r#""rows":24"#));
    }
}
