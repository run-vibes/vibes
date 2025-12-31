//! Error types for Iggy event log.

/// Error type for Iggy operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Placeholder error variant.
    #[error("placeholder")]
    Placeholder,
}

/// Result type alias for Iggy operations.
pub type Result<T> = std::result::Result<T, Error>;
