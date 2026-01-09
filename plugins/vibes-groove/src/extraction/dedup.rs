//! Semantic deduplication for learnings
//!
//! Prevents storing duplicate learnings by using embedding similarity
//! to find and merge semantically equivalent content.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;

use crate::extraction::Embedder;
use crate::store::LearningStore;
use crate::{GrooveError, Learning};

/// Strategy for detecting and handling duplicate learnings
#[async_trait]
pub trait DeduplicationStrategy: Send + Sync {
    /// Check if a candidate learning is a duplicate of an existing one
    ///
    /// Returns the existing learning if a duplicate is found, None otherwise.
    async fn find_duplicate(
        &self,
        candidate: &Learning,
        store: &dyn LearningStore,
    ) -> Result<Option<Learning>, GrooveError>;

    /// Merge a duplicate into an existing learning
    ///
    /// Returns a new learning that combines information from both,
    /// preserving the original's description while updating metadata.
    async fn merge(
        &self,
        existing: &Learning,
        duplicate: &Learning,
    ) -> Result<Learning, GrooveError>;
}

/// Default similarity threshold for semantic deduplication
pub const DEFAULT_SIMILARITY_THRESHOLD: f64 = 0.9;

/// Semantic deduplication using embedding similarity
///
/// Uses cosine similarity between embeddings to detect duplicates.
/// Learnings with similarity above the threshold are considered duplicates.
pub struct SemanticDedup {
    /// Similarity threshold (0.0-1.0), defaults to 0.9
    similarity_threshold: f64,
    /// Embedder for generating text embeddings
    embedder: Arc<dyn Embedder>,
}

impl SemanticDedup {
    /// Create a new SemanticDedup with the given embedder and default threshold
    pub fn new(embedder: Arc<dyn Embedder>) -> Self {
        Self {
            similarity_threshold: DEFAULT_SIMILARITY_THRESHOLD,
            embedder,
        }
    }

    /// Create with a custom similarity threshold
    pub fn with_threshold(embedder: Arc<dyn Embedder>, threshold: f64) -> Self {
        Self {
            similarity_threshold: threshold.clamp(0.0, 1.0),
            embedder,
        }
    }

    /// Get the current similarity threshold
    pub fn threshold(&self) -> f64 {
        self.similarity_threshold
    }
}

#[async_trait]
impl DeduplicationStrategy for SemanticDedup {
    async fn find_duplicate(
        &self,
        candidate: &Learning,
        store: &dyn LearningStore,
    ) -> Result<Option<Learning>, GrooveError> {
        // Generate embedding for candidate's description
        let embedding = self
            .embedder
            .embed(&candidate.content.description)
            .await
            .map_err(|e| GrooveError::Embedding(format!("Failed to embed candidate: {}", e)))?;

        // Search for similar learnings above threshold
        let similar = store
            .find_similar(&embedding, self.similarity_threshold, 1)
            .await?;

        // Return the first match if any (and it's not the candidate itself)
        Ok(similar
            .into_iter()
            .find(|(learning, _)| learning.id != candidate.id)
            .map(|(learning, _)| learning))
    }

    async fn merge(
        &self,
        existing: &Learning,
        duplicate: &Learning,
    ) -> Result<Learning, GrooveError> {
        // Create merged learning:
        // - Keep existing ID and description
        // - Average confidence scores
        // - Update timestamp to latest
        // - Keep existing source (we don't have multi-source tracking yet)
        let mut merged = existing.clone();

        // Average confidence scores
        merged.confidence = (existing.confidence + duplicate.confidence) / 2.0;

        // Update to latest timestamp
        merged.updated_at = Utc::now();

        Ok(merged)
    }
}
