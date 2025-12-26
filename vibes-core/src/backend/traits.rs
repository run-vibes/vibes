//! ClaudeBackend trait and related types
//!
//! The backend abstraction (ADR-008) enables swapping interaction models:
//! PrintMode, PTY, StreamJson, or custom implementations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::error::BackendError;
use crate::events::ClaudeEvent;

/// State of a Claude backend
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BackendState {
    /// Ready for input
    Idle,
    /// Processing a request
    Processing,
    /// Waiting for permission approval
    WaitingPermission { request_id: String, tool: String },
    /// Completed successfully
    Finished,
    /// Failed with error
    Failed { message: String, recoverable: bool },
}

/// Trait for Claude Code backends
///
/// Implementations handle the actual communication with Claude,
/// whether via subprocess, PTY, or direct API.
#[async_trait]
pub trait ClaudeBackend: Send + Sync {
    /// Send user input to Claude
    async fn send(&mut self, input: &str) -> Result<(), BackendError>;

    /// Subscribe to events from this backend
    fn subscribe(&self) -> broadcast::Receiver<ClaudeEvent>;

    /// Respond to a permission request (for interactive backends)
    async fn respond_permission(
        &mut self,
        request_id: &str,
        approved: bool,
    ) -> Result<(), BackendError>;

    /// Claude's session ID for continuity across turns
    fn claude_session_id(&self) -> &str;

    /// Current state of the backend
    fn state(&self) -> BackendState;

    /// Graceful shutdown
    async fn shutdown(&mut self) -> Result<(), BackendError>;
}

/// Factory for creating Claude backends
///
/// Enables dependency injection of backend implementations.
pub trait BackendFactory: Send + Sync {
    /// Create a new backend instance
    ///
    /// If `claude_session_id` is None, a new session ID will be generated.
    fn create(&self, claude_session_id: Option<String>) -> Box<dyn ClaudeBackend>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== BackendState Tests ====================

    #[test]
    fn backend_state_idle_debug() {
        let state = BackendState::Idle;
        assert!(format!("{:?}", state).contains("Idle"));
    }

    #[test]
    fn backend_state_processing_debug() {
        let state = BackendState::Processing;
        assert!(format!("{:?}", state).contains("Processing"));
    }

    #[test]
    fn backend_state_waiting_permission_debug() {
        let state = BackendState::WaitingPermission {
            request_id: "req-1".to_string(),
            tool: "Bash".to_string(),
        };
        let debug = format!("{:?}", state);
        assert!(debug.contains("WaitingPermission"));
        assert!(debug.contains("req-1"));
        assert!(debug.contains("Bash"));
    }

    #[test]
    fn backend_state_finished_debug() {
        let state = BackendState::Finished;
        assert!(format!("{:?}", state).contains("Finished"));
    }

    #[test]
    fn backend_state_failed_debug() {
        let state = BackendState::Failed {
            message: "Connection lost".to_string(),
            recoverable: true,
        };
        let debug = format!("{:?}", state);
        assert!(debug.contains("Failed"));
        assert!(debug.contains("Connection lost"));
    }

    #[test]
    fn backend_state_clone_works() {
        let state = BackendState::WaitingPermission {
            request_id: "req-1".to_string(),
            tool: "Bash".to_string(),
        };
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn backend_state_serialization_roundtrip() {
        let states = vec![
            BackendState::Idle,
            BackendState::Processing,
            BackendState::WaitingPermission {
                request_id: "req-1".to_string(),
                tool: "Bash".to_string(),
            },
            BackendState::Finished,
            BackendState::Failed {
                message: "Error".to_string(),
                recoverable: false,
            },
        ];

        for state in states {
            let json = serde_json::to_string(&state).unwrap();
            let parsed: BackendState = serde_json::from_str(&json).unwrap();
            assert_eq!(state, parsed);
        }
    }
}
