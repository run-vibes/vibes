//! Semantic deduplication for learnings
//!
//! Prevents storing duplicate learnings by using embedding similarity
//! to find and merge semantically equivalent content.

use async_trait::async_trait;

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
