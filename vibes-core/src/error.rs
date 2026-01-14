//! Error types for vibes-core

use thiserror::Error;

/// Top-level error type for vibes-core
#[derive(Error, Debug)]
pub enum VibesError {
    #[error("Notification error: {0}")]
    Notification(#[from] NotificationError),

    #[error("PTY error: {0}")]
    Pty(#[from] crate::pty::PtyError),

    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),
}

/// Result type alias for vibes-core operations
pub type VibesResult<T> = std::result::Result<T, VibesError>;

/// Errors related to agent operations
#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Operation not supported: {0}")]
    NotSupported(String),

    #[error("Agent is not in expected state: expected {expected}, got {actual}")]
    InvalidState { expected: String, actual: String },

    #[error("Task execution failed: {0}")]
    TaskFailed(String),

    #[error("Agent was cancelled")]
    Cancelled,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_error_config_displays_correctly() {
        let error = NotificationError::Config("invalid key".to_string());
        assert!(error.to_string().contains("Configuration error"));
        assert!(error.to_string().contains("invalid key"));
    }

    #[test]
    fn vibes_error_notification_displays_correctly() {
        let notif_error = NotificationError::SendFailed("timeout".to_string());
        let error = VibesError::Notification(notif_error);
        assert!(error.to_string().contains("Notification error"));
    }
}
