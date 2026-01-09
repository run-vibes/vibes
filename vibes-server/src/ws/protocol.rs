//! WebSocket protocol message types
//!
//! Both CLI and Web UI use the same protocol for consistent behavior.

use serde::{Deserialize, Serialize};
use vibes_core::{AuthContext, VibesEvent};

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

    /// Attach to a session (receive PTY output)
    /// Creates the session if it doesn't exist.
    Attach {
        /// Session ID to attach to
        session_id: String,
        /// Optional session name (used when creating new session)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        /// Optional working directory for the spawned process
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cwd: Option<String>,
        /// Initial terminal columns (used when creating new session)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cols: Option<u16>,
        /// Initial terminal rows (used when creating new session)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rows: Option<u16>,
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
    /// Session notification (broadcast when session is created by another client)
    SessionNotification {
        /// Session ID
        session_id: String,
        /// Session name if provided
        name: Option<String>,
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

    /// Replay scrollback buffer on attach
    PtyReplay {
        /// Session ID
        session_id: String,
        /// Scrollback data (base64 encoded)
        data: String,
    },
}

/// Convert a VibesEvent to a ServerMessage for broadcasting
///
/// Returns None for events that shouldn't be broadcast to WebSocket clients.
/// PTY output goes through the separate PTY broadcast channel, not this function.
pub fn vibes_event_to_server_message(event: &VibesEvent) -> Option<ServerMessage> {
    match event {
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
        VibesEvent::TunnelStateChanged { state, url } => Some(ServerMessage::TunnelState {
            state: state.clone(),
            url: url.clone(),
        }),
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
        // These events are not broadcast to WebSocket clients
        VibesEvent::Claude { .. } => None,
        VibesEvent::UserInput { .. } => None,
        VibesEvent::PermissionResponse { .. } => None,
        VibesEvent::OwnershipTransferred { .. } => None,
        VibesEvent::ClientConnected { .. } => None,
        VibesEvent::ClientDisconnected { .. } => None,
        VibesEvent::Hook { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vibes_core::ClaudeEvent;

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

    #[test]
    fn test_client_message_attach_roundtrip() {
        let msg = ClientMessage::Attach {
            session_id: "sess-1".to_string(),
            name: None,
            cwd: None,
            cols: None,
            rows: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""type":"attach""#));
    }

    #[test]
    fn test_client_message_attach_with_name_roundtrip() {
        let msg = ClientMessage::Attach {
            session_id: "sess-1".to_string(),
            name: Some("my-session".to_string()),
            cwd: None,
            cols: None,
            rows: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""name":"my-session""#));
    }

    #[test]
    fn test_client_message_attach_without_name_field_deserializes() {
        // Test backwards compatibility - old clients that don't send name field
        let json = r#"{"type":"attach","session_id":"sess-1"}"#;
        let parsed: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(
            parsed,
            ClientMessage::Attach { session_id, name, cwd, cols, rows }
            if session_id == "sess-1" && name.is_none() && cwd.is_none() && cols.is_none() && rows.is_none()
        ));
    }

    #[test]
    fn test_client_message_attach_with_cwd_roundtrip() {
        let msg = ClientMessage::Attach {
            session_id: "sess-1".to_string(),
            name: Some("my-session".to_string()),
            cwd: Some("/home/user/project".to_string()),
            cols: None,
            rows: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""cwd":"/home/user/project""#));
    }

    #[test]
    fn test_client_message_attach_without_cwd_field_deserializes() {
        // Backwards compatibility - old clients that don't send cwd field
        let json = r#"{"type":"attach","session_id":"sess-1"}"#;
        let parsed: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(
            parsed,
            ClientMessage::Attach { session_id, name, cwd, cols, rows }
            if session_id == "sess-1" && name.is_none() && cwd.is_none() && cols.is_none() && rows.is_none()
        ));
    }

    #[test]
    fn test_client_message_attach_with_dimensions_roundtrip() {
        let msg = ClientMessage::Attach {
            session_id: "sess-1".to_string(),
            name: None,
            cwd: None,
            cols: Some(80),
            rows: Some(24),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""cols":80"#));
        assert!(json.contains(r#""rows":24"#));
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

    // ==================== ServerMessage Tests ====================

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

    #[test]
    fn test_server_message_pty_replay_roundtrip() {
        let msg = ServerMessage::PtyReplay {
            session_id: "sess-1".to_string(),
            data: "aGVsbG8gd29ybGQ=".to_string(), // base64 for "hello world"
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ServerMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, parsed);
        assert!(json.contains(r#""type":"pty_replay""#));
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

    // ==================== Event Conversion Tests ====================

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

    #[test]
    fn test_vibes_event_claude_not_broadcast() {
        let vibes_event = VibesEvent::Claude {
            session_id: "sess-1".to_string(),
            event: ClaudeEvent::TextDelta {
                text: "Hello".to_string(),
            },
        };

        // Claude events are not broadcast to WebSocket clients (PTY handles output)
        assert!(vibes_event_to_server_message(&vibes_event).is_none());
    }

    #[test]
    fn test_vibes_event_user_input_not_broadcast() {
        use vibes_core::InputSource;

        let vibes_event = VibesEvent::UserInput {
            session_id: "sess-1".to_string(),
            content: "test input".to_string(),
            source: InputSource::Cli,
        };

        // UserInput events are not broadcast to WebSocket clients (PTY handles I/O)
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
}
