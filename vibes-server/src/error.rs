//! Server error types

use thiserror::Error;

/// Errors that can occur in the vibes server
#[derive(Debug, Error)]
pub enum ServerError {
    /// Failed to bind to the specified address
    #[error("failed to bind to {addr}: {source}")]
    Bind {
        addr: String,
        #[source]
        source: std::io::Error,
    },

    /// WebSocket error
    #[error("websocket error: {0}")]
    WebSocket(String),

    /// Session not found
    #[error("session not found: {0}")]
    SessionNotFound(String),

    /// Invalid message format
    #[error("invalid message: {0}")]
    InvalidMessage(String),

    /// Internal server error
    #[error("internal error: {0}")]
    Internal(String),
}
