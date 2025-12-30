//! Iggy integration for assessment event log.
//!
//! Manages Iggy server subprocess and provides event log implementation.

pub mod manager;

pub use manager::{IggyConfig, IggyManager, IggyState};
