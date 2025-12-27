//! Authentication error types

use thiserror::Error;

/// Errors that can occur during authentication
#[derive(Debug, Error)]
pub enum AuthError {
    /// No authentication token was provided in the request
    #[error("no authentication token provided")]
    MissingToken,

    /// The token format is invalid (not a valid JWT)
    #[error("invalid token format: {0}")]
    InvalidFormat(String),

    /// The token signature verification failed
    #[error("token signature verification failed")]
    InvalidSignature,

    /// The token has expired
    #[error("token has expired")]
    Expired,

    /// The token's audience claim doesn't match the expected value
    #[error("invalid audience claim")]
    InvalidAudience,

    /// The key ID in the token doesn't match any known keys
    #[error("unknown key ID: {0}")]
    UnknownKeyId(String),

    /// Failed to fetch JWKS from Cloudflare
    #[error("failed to fetch JWKS: {0}")]
    JwksFetchError(String),

    /// JWT decoding error from jsonwebtoken crate
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_error_display() {
        let err = AuthError::MissingToken;
        assert_eq!(err.to_string(), "no authentication token provided");
    }

    #[test]
    fn test_auth_error_unknown_key_id() {
        let err = AuthError::UnknownKeyId("abc123".to_string());
        assert_eq!(err.to_string(), "unknown key ID: abc123");
    }
}
