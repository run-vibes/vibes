//! Error types for model management.

use thiserror::Error;

/// Result type alias using the crate's error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during model operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Model not found in registry.
    #[error("model not found: {0}")]
    ModelNotFound(String),

    /// Provider not found in registry.
    #[error("provider not found: {0}")]
    ProviderNotFound(String),

    /// Credentials not found for provider.
    #[error("credentials not found for provider: {0}")]
    CredentialsNotFound(String),

    /// Failed to access system keyring.
    #[error("keyring error: {0}")]
    Keyring(String),

    /// Invalid API key format.
    #[error("invalid API key format")]
    InvalidApiKey,

    /// Provider API error.
    #[error("provider API error: {0}")]
    ProviderApi(String),

    /// Request failed.
    #[error("request failed: {0}")]
    Request(String),

    /// Serialization/deserialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_formats_correctly() {
        let err = Error::ModelNotFound("gpt-5".to_string());
        assert_eq!(err.to_string(), "model not found: gpt-5");
    }

    #[test]
    fn error_from_serde_json() {
        let json_err: serde_json::Error = serde_json::from_str::<String>("invalid").unwrap_err();
        let err: Error = json_err.into();
        assert!(matches!(err, Error::Serialization(_)));
    }
}
