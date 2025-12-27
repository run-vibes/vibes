//! Error types for vibes-core

use thiserror::Error;

use crate::history::HistoryError;

/// Top-level error type for vibes-core
#[derive(Error, Debug)]
pub enum VibesError {
    #[error("Session error: {0}")]
    Session(#[from] SessionError),

    #[error("Backend error: {0}")]
    Backend(#[from] BackendError),

    #[error("Event bus error: {0}")]
    EventBus(#[from] EventBusError),

    #[error("Notification error: {0}")]
    Notification(#[from] NotificationError),

    #[error("History error: {0}")]
    History(#[from] HistoryError),
}

/// Errors related to push notifications
#[derive(Error, Debug)]
pub enum NotificationError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Failed to send notification: {0}")]
    SendFailed(String),

    #[error("Storage error: {0}")]
    Storage(String),
}

/// Errors related to session management
#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Session not found: {0}")]
    NotFound(String),

    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Invalid state: expected {expected}, got {actual}")]
    InvalidState { expected: String, actual: String },

    #[error("Failure is not recoverable")]
    NotRecoverable,

    #[error("Backend error: {0}")]
    Backend(#[from] BackendError),
}

/// Errors from Claude Code backends
#[derive(Error, Debug)]
pub enum BackendError {
    #[error("Failed to spawn Claude process: {0}")]
    SpawnFailed(#[source] std::io::Error),

    #[error("Claude process exited unexpectedly: code {code:?}")]
    ProcessCrashed { code: Option<i32> },

    #[error("Claude binary not found. Is Claude Code installed?")]
    ClaudeNotFound,

    #[error("Parse error: {message}")]
    ParseError { message: String, recoverable: bool },

    #[error("Claude error: {0}")]
    ClaudeError(String),
}

/// Errors from the event bus
#[derive(Error, Debug)]
pub enum EventBusError {
    #[error("Failed to publish event")]
    PublishFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test BackendError Display implementations
    #[test]
    fn backend_error_spawn_failed_displays_correctly() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = BackendError::SpawnFailed(io_error);
        assert!(error.to_string().contains("Failed to spawn Claude process"));
    }

    #[test]
    fn backend_error_process_crashed_displays_correctly() {
        let error = BackendError::ProcessCrashed { code: Some(1) };
        assert!(error.to_string().contains("exited unexpectedly"));
        assert!(error.to_string().contains("1"));
    }

    #[test]
    fn backend_error_claude_not_found_displays_correctly() {
        let error = BackendError::ClaudeNotFound;
        assert!(error.to_string().contains("Claude binary not found"));
    }

    #[test]
    fn backend_error_parse_error_displays_correctly() {
        let error = BackendError::ParseError {
            message: "invalid JSON".to_string(),
            recoverable: true,
        };
        assert!(error.to_string().contains("Parse error"));
        assert!(error.to_string().contains("invalid JSON"));
    }

    #[test]
    fn backend_error_claude_error_displays_correctly() {
        let error = BackendError::ClaudeError("rate limited".to_string());
        assert!(error.to_string().contains("rate limited"));
    }

    // Test SessionError Display implementations
    #[test]
    fn session_error_not_found_displays_correctly() {
        let error = SessionError::NotFound("abc123".to_string());
        assert!(error.to_string().contains("Session not found"));
        assert!(error.to_string().contains("abc123"));
    }

    #[test]
    fn session_error_invalid_state_displays_correctly() {
        let error = SessionError::InvalidStateTransition {
            from: "Idle".to_string(),
            to: "Completed".to_string(),
        };
        assert!(error.to_string().contains("Invalid state transition"));
    }

    // Test EventBusError Display implementations
    #[test]
    fn event_bus_error_publish_failed_displays_correctly() {
        let error = EventBusError::PublishFailed;
        assert!(error.to_string().contains("Failed to publish event"));
    }

    // Test VibesError Display implementations
    #[test]
    fn vibes_error_session_displays_correctly() {
        let session_error = SessionError::NotFound("xyz".to_string());
        let error = VibesError::Session(session_error);
        assert!(error.to_string().contains("Session error"));
    }

    #[test]
    fn vibes_error_backend_displays_correctly() {
        let backend_error = BackendError::ClaudeNotFound;
        let error = VibesError::Backend(backend_error);
        assert!(error.to_string().contains("Backend error"));
    }

    #[test]
    fn vibes_error_event_bus_displays_correctly() {
        let bus_error = EventBusError::PublishFailed;
        let error = VibesError::EventBus(bus_error);
        assert!(error.to_string().contains("Event bus error"));
    }

    // Test From conversions
    #[test]
    fn session_error_converts_from_backend_error() {
        let backend_error = BackendError::ClaudeNotFound;
        let session_error: SessionError = backend_error.into();
        assert!(matches!(session_error, SessionError::Backend(_)));
    }

    #[test]
    fn vibes_error_converts_from_session_error() {
        let session_error = SessionError::NotFound("test".to_string());
        let vibes_error: VibesError = session_error.into();
        assert!(matches!(vibes_error, VibesError::Session(_)));
    }

    #[test]
    fn vibes_error_converts_from_backend_error() {
        let backend_error = BackendError::ClaudeNotFound;
        let vibes_error: VibesError = backend_error.into();
        assert!(matches!(vibes_error, VibesError::Backend(_)));
    }

    #[test]
    fn vibes_error_converts_from_event_bus_error() {
        let bus_error = EventBusError::PublishFailed;
        let vibes_error: VibesError = bus_error.into();
        assert!(matches!(vibes_error, VibesError::EventBus(_)));
    }
}
