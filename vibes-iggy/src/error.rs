//! Error types for vibes-iggy.

use std::io;
use thiserror::Error;

/// Result type for vibes-iggy operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in vibes-iggy.
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error (file system, process, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Iggy client error
    #[error("Iggy error: {0}")]
    Iggy(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Consumer group not found
    #[error("Consumer group not found: {0}")]
    ConsumerNotFound(String),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Server not running
    #[error("Iggy server not running")]
    ServerNotRunning,

    /// Binary not found
    #[error("iggy-server binary not found")]
    BinaryNotFound,

    /// Invalid offset
    #[error("Invalid offset: {0}")]
    InvalidOffset(u64),
}

impl From<iggy::error::IggyError> for Error {
    fn from(err: iggy::error::IggyError) -> Self {
        Error::Iggy(err.to_string())
    }
}
