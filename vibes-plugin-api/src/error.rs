//! Error types for plugin authors

use thiserror::Error;

/// Errors that plugins can return
#[derive(Error, Debug)]
pub enum PluginError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Command execution failed
    #[error("Command failed: {0}")]
    Command(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Custom error with message
    #[error("{0}")]
    Custom(String),
}

impl PluginError {
    /// Create a custom error with a message
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }

    /// Create a configuration error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    /// Create a command error
    pub fn command(message: impl Into<String>) -> Self {
        Self::Command(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let config_err = PluginError::Config("missing key".to_string());
        assert_eq!(config_err.to_string(), "Configuration error: missing key");

        let cmd_err = PluginError::Command("exit code 1".to_string());
        assert_eq!(cmd_err.to_string(), "Command failed: exit code 1");

        let custom_err = PluginError::Custom("something happened".to_string());
        assert_eq!(custom_err.to_string(), "something happened");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let plugin_err: PluginError = io_err.into();

        assert!(matches!(plugin_err, PluginError::Io(_)));
        assert!(plugin_err.to_string().contains("file not found"));
    }

    #[test]
    fn test_helper_constructors() {
        let err = PluginError::custom("test");
        assert!(matches!(err, PluginError::Custom(_)));

        let err = PluginError::config("bad config");
        assert!(matches!(err, PluginError::Config(_)));

        let err = PluginError::command("failed");
        assert!(matches!(err, PluginError::Command(_)));
    }
}
