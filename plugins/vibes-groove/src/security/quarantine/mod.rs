//! Quarantine management
//!
//! Provides quarantine workflow and management for suspicious learnings.

mod manager;
mod types;

pub use manager::QuarantineManager;
pub use types::{QuarantineReason, QuarantineStatus, ReviewOutcome};
