//! Error types for introspection

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum IntrospectionError {
    #[error("Could not determine home directory")]
    NoHomeDir,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, IntrospectionError>;
