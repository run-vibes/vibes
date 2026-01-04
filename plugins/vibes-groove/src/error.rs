//! Error types for vibes-groove

use thiserror::Error;
use uuid::Uuid;

/// Error type for groove storage operations
#[derive(Debug, Error)]
pub enum GrooveError {
    /// Database operation failed
    #[error("Database error: {0}")]
    Database(String),

    /// No project context available for project-scoped operations
    #[error("No project context available")]
    NoProjectContext,

    /// No enterprise context available for enterprise-scoped operations
    #[error("No enterprise context for org: {0}")]
    NoEnterpriseContext(String),

    /// Invalid scope specification
    #[error("Invalid scope: {0}")]
    InvalidScope(String),

    /// Learning not found
    #[error("Learning not found: {0}")]
    NotFound(Uuid),

    /// Schema migration failed
    #[error("Schema migration failed: {0}")]
    Migration(String),

    /// Export or import operation failed
    #[error("Export/import error: {0}")]
    Export(String),

    /// Embedding generation or search failed
    #[error("Embedding error: {0}")]
    Embedding(String),

    /// Assessment system error (Iggy communication, etc.)
    #[error("Assessment error: {0}")]
    Assessment(String),

    /// IO operation failed
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization or deserialization failed
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Result type alias for groove operations
pub type Result<T> = std::result::Result<T, GrooveError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = GrooveError::Database("connection failed".into());
        assert!(err.to_string().contains("connection failed"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let groove_err: GrooveError = io_err.into();
        assert!(matches!(groove_err, GrooveError::Io(_)));
    }

    #[test]
    fn test_no_project_context() {
        let err = GrooveError::NoProjectContext;
        assert_eq!(err.to_string(), "No project context available");
    }
}
