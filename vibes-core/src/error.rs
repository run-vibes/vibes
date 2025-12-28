//! Error types for vibes-core

use thiserror::Error;

/// Top-level error type for vibes-core
#[derive(Error, Debug)]
pub enum VibesError {
    #[error("Event bus error: {0}")]
    EventBus(#[from] EventBusError),

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

/// Errors from the event bus
#[derive(Error, Debug)]
pub enum EventBusError {
    #[error("Failed to publish event")]
    PublishFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_bus_error_publish_failed_displays_correctly() {
        let error = EventBusError::PublishFailed;
        assert!(error.to_string().contains("Failed to publish event"));
    }

    #[test]
    fn vibes_error_event_bus_displays_correctly() {
        let bus_error = EventBusError::PublishFailed;
        let error = VibesError::EventBus(bus_error);
        assert!(error.to_string().contains("Event bus error"));
    }

    #[test]
    fn vibes_error_converts_from_event_bus_error() {
        let bus_error = EventBusError::PublishFailed;
        let vibes_error: VibesError = bus_error.into();
        assert!(matches!(vibes_error, VibesError::EventBus(_)));
    }

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
