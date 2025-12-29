//! Security error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    #[error("Content scan failed: {0}")]
    ScanFailed(String),

    #[error("Quarantined learning: {0}")]
    Quarantined(uuid::Uuid),

    #[error("Trust level insufficient: requires {required:?}, has {actual:?}")]
    InsufficientTrust {
        required: super::TrustLevel,
        actual: super::TrustLevel,
    },

    #[error("Policy load error: {0}")]
    PolicyLoad(String),

    #[error("Audit log error: {0}")]
    AuditLog(String),

    #[error("Provenance verification failed: {0}")]
    ProvenanceFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type SecurityResult<T> = Result<T, SecurityError>;
