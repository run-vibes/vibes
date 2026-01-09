//! Learning extraction pipeline
//!
//! This module orchestrates the extraction of learnings from assessed sessions.
//! It processes heavy assessment events, runs pattern detection, deduplicates
//! learnings, and persists them to the learning store.

pub mod consumer;
pub mod dedup;
pub mod embedder;
pub mod patterns;
pub mod types;

pub use consumer::{ExtractionConfig, ExtractionConsumer, ExtractionResult, TranscriptFetcher};
pub use dedup::{DEFAULT_SIMILARITY_THRESHOLD, DeduplicationStrategy, SemanticDedup};
pub use embedder::{
    Embedder, EmbedderError, EmbedderResult, GTE_SMALL_DIMENSIONS, LocalEmbedder, ModelInfo,
    cosine_similarity, default_cache_dir,
};
pub use types::*;
