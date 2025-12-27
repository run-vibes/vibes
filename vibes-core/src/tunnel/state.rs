//! Tunnel state and event types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Current state of the tunnel connection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TunnelState {
    /// Tunnel is disabled in configuration
    Disabled,
    /// Tunnel is starting up
    Starting,
    /// Tunnel is connected and ready
    Connected {
        url: String,
        connected_at: DateTime<Utc>,
    },
    /// Tunnel lost connection, attempting to reconnect
    Reconnecting {
        attempt: u32,
        last_error: String,
    },
    /// Tunnel failed to connect
    Failed {
        error: String,
        can_retry: bool,
    },
    /// Tunnel was explicitly stopped
    Stopped,
}

impl Default for TunnelState {
    fn default() -> Self {
        Self::Disabled
    }
}

/// Events emitted by the tunnel manager
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TunnelEvent {
    /// Tunnel is starting
    Starting,
    /// Tunnel connected successfully
    Connected { url: String },
    /// Tunnel disconnected
    Disconnected { reason: String },
    /// Tunnel is reconnecting
    Reconnecting { attempt: u32 },
    /// Tunnel failed
    Failed { error: String },
    /// Tunnel stopped
    Stopped,
    /// Log message from cloudflared
    Log { level: LogLevel, message: String },
}

/// Log levels for tunnel events
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tunnel_state_default_is_disabled() {
        let state = TunnelState::default();
        assert!(matches!(state, TunnelState::Disabled));
    }

    #[test]
    fn tunnel_state_serialization_roundtrip() {
        let state = TunnelState::Connected {
            url: "https://example.trycloudflare.com".to_string(),
            connected_at: Utc::now(),
        };
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("connected"));
        let parsed: TunnelState = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, TunnelState::Connected { .. }));
    }

    #[test]
    fn tunnel_event_serialization_roundtrip() {
        let event = TunnelEvent::Connected {
            url: "https://test.trycloudflare.com".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: TunnelEvent = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, TunnelEvent::Connected { .. }));
    }
}
