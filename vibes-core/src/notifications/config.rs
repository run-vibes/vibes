//! Configuration for push notifications

use serde::{Deserialize, Serialize};

/// Configuration for push notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Whether notifications are enabled globally
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Notify when Claude needs permission approval
    #[serde(default = "default_true")]
    pub notify_permission: bool,

    /// Notify when session completes successfully
    #[serde(default = "default_true")]
    pub notify_completed: bool,

    /// Notify when session fails with an error
    #[serde(default = "default_true")]
    pub notify_error: bool,
}

fn default_true() -> bool {
    true
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            notify_permission: true,
            notify_completed: true,
            notify_error: true,
        }
    }
}

impl NotificationConfig {
    /// Create a config with all notifications enabled
    pub fn all_enabled() -> Self {
        Self::default()
    }

    /// Create a config with all notifications disabled
    pub fn all_disabled() -> Self {
        Self {
            enabled: false,
            notify_permission: false,
            notify_completed: false,
            notify_error: false,
        }
    }

    /// Check if any notification type is enabled
    pub fn any_enabled(&self) -> bool {
        self.enabled && (self.notify_permission || self.notify_completed || self.notify_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NotificationConfig::default();
        assert!(config.enabled);
        assert!(config.notify_permission);
        assert!(config.notify_completed);
        assert!(config.notify_error);
    }

    #[test]
    fn test_all_disabled() {
        let config = NotificationConfig::all_disabled();
        assert!(!config.enabled);
        assert!(!config.any_enabled());
    }

    #[test]
    fn test_any_enabled() {
        let mut config = NotificationConfig::default();
        assert!(config.any_enabled());

        config.enabled = false;
        assert!(!config.any_enabled());
    }

    #[test]
    fn test_deserialize_toml() {
        let toml = r#"
            enabled = true
            notify_permission = true
            notify_completed = false
            notify_error = true
        "#;
        let config: NotificationConfig = toml::from_str(toml).unwrap();
        assert!(config.enabled);
        assert!(config.notify_permission);
        assert!(!config.notify_completed);
        assert!(config.notify_error);
    }

    #[test]
    fn test_deserialize_toml_defaults() {
        let toml = r#""#;
        let config: NotificationConfig = toml::from_str(toml).unwrap();
        assert!(config.enabled); // defaults to true
    }
}
