//! Plugin host error types

use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// Errors that can occur in the plugin host
#[derive(Error, Debug)]
pub enum PluginHostError {
    /// Plugin directory not found
    #[error("Plugin directory not found: {path}")]
    PluginDirNotFound { path: PathBuf },

    /// Plugin library not found in directory
    #[error("Plugin library not found in {dir}")]
    LibraryNotFound { dir: PathBuf },

    /// API version mismatch between vibes and plugin
    #[error("API version mismatch: vibes expects {expected}, plugin has {found}")]
    ApiVersionMismatch { expected: u32, found: u32 },

    /// Failed to load dynamic library
    #[error("Failed to load plugin library: {0}")]
    LibraryLoad(#[from] libloading::Error),

    /// Plugin initialization failed
    #[error("Plugin initialization failed: {0}")]
    InitFailed(#[from] vibes_plugin_api::PluginError),

    /// Registry error (parsing, saving, etc.)
    #[error("Registry error: {0}")]
    Registry(String),

    /// Plugin not found
    #[error("Plugin '{name}' not found")]
    NotFound { name: String },

    /// Plugin timed out
    #[error("Plugin '{name}' timed out after {timeout:?}")]
    Timeout { name: String, timeout: Duration },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Command conflict: another plugin already registered this command
    #[error(
        "Command conflict: '{command}' already registered by plugin '{existing_plugin}', cannot register for '{new_plugin}'"
    )]
    CommandConflict {
        command: String,
        existing_plugin: String,
        new_plugin: String,
    },

    /// Route conflict: another plugin already registered this route
    #[error(
        "Route conflict: '{route}' already registered by plugin '{existing_plugin}', cannot register for '{new_plugin}'"
    )]
    RouteConflict {
        route: String,
        existing_plugin: String,
        new_plugin: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_dir_not_found_display() {
        let err = PluginHostError::PluginDirNotFound {
            path: PathBuf::from("/some/path"),
        };
        assert!(err.to_string().contains("/some/path"));
    }

    #[test]
    fn test_api_version_mismatch_display() {
        let err = PluginHostError::ApiVersionMismatch {
            expected: 1,
            found: 2,
        };
        let msg = err.to_string();
        assert!(msg.contains("1"));
        assert!(msg.contains("2"));
    }

    #[test]
    fn test_not_found_display() {
        let err = PluginHostError::NotFound {
            name: "test-plugin".to_string(),
        };
        assert!(err.to_string().contains("test-plugin"));
    }

    #[test]
    fn test_timeout_display() {
        let err = PluginHostError::Timeout {
            name: "slow-plugin".to_string(),
            timeout: Duration::from_secs(5),
        };
        let msg = err.to_string();
        assert!(msg.contains("slow-plugin"));
        assert!(msg.contains("5"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: PluginHostError = io_err.into();
        assert!(matches!(err, PluginHostError::Io(_)));
    }

    #[test]
    fn test_command_conflict_error() {
        let err = PluginHostError::CommandConflict {
            command: "trust levels".into(),
            existing_plugin: "groove".into(),
            new_plugin: "other".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("trust levels"));
        assert!(msg.contains("groove"));
    }

    #[test]
    fn test_route_conflict_error() {
        let err = PluginHostError::RouteConflict {
            route: "GET /api/groove/policy".into(),
            existing_plugin: "groove".into(),
            new_plugin: "other".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("/api/groove/policy"));
    }
}
