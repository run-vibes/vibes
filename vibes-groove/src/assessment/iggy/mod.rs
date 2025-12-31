//! Iggy integration for assessment event log.
//!
//! Manages Iggy server subprocess and provides event log implementation.

pub mod log;
pub mod manager;

pub use log::IggyAssessmentLog;
pub use manager::{IggyConfig, IggyManager, IggyState};
