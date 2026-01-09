//! Pattern detection for learning extraction
//!
//! This module provides pattern detectors that scan transcripts for specific
//! patterns indicating learnable behavior (corrections, error recovery, etc.).

pub mod correction;
pub mod error_recovery;

pub use correction::{CorrectionConfig, CorrectionDetector};
pub use error_recovery::{ErrorRecoveryConfig, ErrorRecoveryDetector, ErrorType};
