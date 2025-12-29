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

    /// Duplicate command registration
    #[error("Duplicate command: {0}")]
    DuplicateCommand(String),

    /// Duplicate route registration
    #[error("Duplicate route: {0}")]
    DuplicateRoute(String),

    /// Unknown command dispatch
    #[error("Unknown command: {0}")]
    UnknownCommand(String),

    /// Unknown route dispatch
    #[error("Unknown route: {0}")]
    UnknownRoute(String),

    /// Invalid input error
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(String),
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

    #[test]
    fn test_duplicate_command_error() {
        let err = PluginError::DuplicateCommand("trust levels".into());
        assert!(err.to_string().contains("trust levels"));
    }

    #[test]
    fn test_unknown_command_error() {
        let err = PluginError::UnknownCommand("foo bar".into());
        assert!(err.to_string().contains("foo bar"));
    }

    #[test]
    fn test_json_error() {
        let err = PluginError::Json("parse error".into());
        assert!(err.to_string().contains("parse error"));
    }

    #[test]
    fn test_duplicate_route_error() {
        let err = PluginError::DuplicateRoute("GET /foo".into());
        assert!(err.to_string().contains("GET /foo"));
    }

    #[test]
    fn test_unknown_route_error() {
        let err = PluginError::UnknownRoute("POST /bar".into());
        assert!(err.to_string().contains("POST /bar"));
    }

    #[test]
    fn test_invalid_input_error() {
        let err = PluginError::InvalidInput("missing param".into());
        assert!(err.to_string().contains("missing param"));
    }
}
