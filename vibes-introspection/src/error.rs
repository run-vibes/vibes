//! Error types for introspection

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum IntrospectionError {
    #[error("Could not determine home directory")]
    NoHomeDir,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Could not read config file at {path}: {source}")]
    ConfigRead {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Could not parse config file at {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("File watcher error: {0}")]
    Watcher(#[from] notify::Error),
}

pub type Result<T> = std::result::Result<T, IntrospectionError>;
