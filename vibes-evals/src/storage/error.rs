//! Error types for evaluation storage.

use thiserror::Error;

/// Result type for storage operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during storage operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Database error from libSQL.
    #[error("database error: {0}")]
    Database(#[from] libsql::Error),

    /// JSON serialization/deserialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid data in the database.
    #[error("invalid data: {0}")]
    InvalidData(String),

    /// Study not found.
    #[error("study not found: {0}")]
    StudyNotFound(String),
}
