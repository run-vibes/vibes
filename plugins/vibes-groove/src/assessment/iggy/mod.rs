//! Iggy integration for assessment event log.
//!
//! Re-exports IggyManager and types from vibes-iggy crate.

pub mod log;

// Re-export from vibes-iggy
pub use log::IggyAssessmentLog;
pub use vibes_iggy::{IggyConfig, IggyManager, IggyState};
