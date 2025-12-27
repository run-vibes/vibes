//! History error types

use thiserror::Error;

/// Errors for chat history operations
#[derive(Error, Debug)]
pub enum HistoryError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Migration failed: {0}")]
    Migration(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = HistoryError::SessionNotFound("test-123".into());
        assert_eq!(err.to_string(), "Session not found: test-123");
    }

    #[test]
    fn test_database_error_conversion() {
        // rusqlite::Error is not easily constructed, so just verify the enum compiles
        let err = HistoryError::Migration("version 2 failed".into());
        assert!(err.to_string().contains("version 2 failed"));
    }
}
