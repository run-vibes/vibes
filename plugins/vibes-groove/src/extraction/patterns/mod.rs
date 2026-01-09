//! Pattern detection for learning extraction
//!
//! This module provides pattern detectors that scan transcripts for specific
//! patterns indicating learnable behavior (corrections, error recovery, etc.).

pub mod correction;

pub use correction::{CorrectionConfig, CorrectionDetector};
