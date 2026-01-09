//! Learning extraction pipeline
//!
//! This module orchestrates the extraction of learnings from assessed sessions.
//! It processes heavy assessment events, runs pattern detection, deduplicates
//! learnings, and persists them to the learning store.

pub mod embedder;
pub mod types;

pub use embedder::{Embedder, EmbedderError, EmbedderResult, cosine_similarity};
pub use types::*;
