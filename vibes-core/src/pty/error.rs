//! PTY error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PtyError {
    #[error("Failed to create PTY: {0}")]
    CreateFailed(String),

    #[error("Failed to spawn process: {0}")]
    SpawnFailed(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("PTY I/O error: {0}")]
    IoError(#[from] std::io::Error),
}
