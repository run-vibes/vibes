//! Error types for vibes-core

use thiserror::Error;

/// Top-level error type for vibes-core
#[derive(Error, Debug)]
pub enum VibesError {
    #[error("Notification error: {0}")]
    Notification(#[from] NotificationError),

    #[error("PTY error: {0}")]
    Pty(#[from] crate::pty::PtyError),
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
