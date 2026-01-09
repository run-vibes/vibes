//! Learning extraction pipeline
//!
//! This module orchestrates the extraction of learnings from assessed sessions.
//! It processes heavy assessment events, runs pattern detection, deduplicates
//! learnings, and persists them to the learning store.

pub mod types;

pub use types::*;
