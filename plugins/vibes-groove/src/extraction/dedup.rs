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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extraction::EmbedderResult;
    use crate::types::{LearningCategory, LearningContent, LearningId, LearningSource, Scope};
    use crate::{LearningRelation, RelationType, UsageStats};
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uuid::Uuid;

    /// Mock embedder that returns predetermined embeddings
    struct MockEmbedder {
        embeddings: Mutex<HashMap<String, Vec<f32>>>,
        default_embedding: Vec<f32>,
    }

    impl MockEmbedder {
        fn new() -> Self {
            Self {
                embeddings: Mutex::new(HashMap::new()),
                default_embedding: vec![1.0, 0.0, 0.0],
            }
        }
    }

    #[async_trait]
    impl Embedder for MockEmbedder {
        async fn embed(&self, text: &str) -> EmbedderResult<Vec<f32>> {
            let embeddings = self.embeddings.lock().unwrap();
            Ok(embeddings
                .get(text)
                .cloned()
                .unwrap_or_else(|| self.default_embedding.clone()))
        }

        fn dimensions(&self) -> usize {
            3
        }
    }

    /// Mock store that returns predetermined similar learnings
    struct MockStore {
        similar_results: Mutex<Vec<(Learning, f64)>>,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                similar_results: Mutex::new(Vec::new()),
            }
        }

        fn with_similar(self, learning: Learning, similarity: f64) -> Self {
            self.similar_results
                .lock()
                .unwrap()
                .push((learning, similarity));
            self
        }
    }

    #[async_trait]
    impl LearningStore for MockStore {
        async fn store(&self, _learning: &Learning) -> Result<LearningId, GrooveError> {
            Ok(Uuid::now_v7())
        }

        async fn get(&self, _id: LearningId) -> Result<Option<Learning>, GrooveError> {
            Ok(None)
        }

        async fn find_by_scope(&self, _scope: &Scope) -> Result<Vec<Learning>, GrooveError> {
            Ok(Vec::new())
        }

        async fn find_by_category(
            &self,
            _category: &LearningCategory,
        ) -> Result<Vec<Learning>, GrooveError> {
            Ok(Vec::new())
        }

        async fn semantic_search(
            &self,
            _embedding: &[f32],
            _limit: usize,
        ) -> Result<Vec<(Learning, f64)>, GrooveError> {
            Ok(Vec::new())
        }

        async fn update_usage(
            &self,
            _id: LearningId,
            _stats: &UsageStats,
        ) -> Result<(), GrooveError> {
            Ok(())
        }

        async fn find_related(
            &self,
            _id: LearningId,
            _relation_type: Option<&RelationType>,
        ) -> Result<Vec<Learning>, GrooveError> {
            Ok(Vec::new())
        }

        async fn store_relation(&self, _relation: &LearningRelation) -> Result<(), GrooveError> {
            Ok(())
        }

        async fn delete(&self, _id: LearningId) -> Result<bool, GrooveError> {
            Ok(true)
        }

        async fn count(&self) -> Result<u64, GrooveError> {
            Ok(0)
        }

        async fn update(&self, _learning: &Learning) -> Result<(), GrooveError> {
            Ok(())
        }

        async fn find_similar(
            &self,
            _embedding: &[f32],
            _threshold: f64,
            _limit: usize,
        ) -> Result<Vec<(Learning, f64)>, GrooveError> {
            Ok(self.similar_results.lock().unwrap().clone())
        }

        async fn find_for_injection(
            &self,
            _scope: &Scope,
            _context_embedding: Option<&[f32]>,
            _limit: usize,
        ) -> Result<Vec<Learning>, GrooveError> {
            Ok(Vec::new())
        }

        async fn count_by_scope(&self, _scope: &Scope) -> Result<u64, GrooveError> {
            Ok(0)
        }

        async fn count_by_category(
            &self,
            _category: &LearningCategory,
        ) -> Result<u64, GrooveError> {
            Ok(0)
        }
    }

    fn make_learning(description: &str, confidence: f64) -> Learning {
        let mut learning = Learning::new(
            Scope::User("test-user".to_string()),
            LearningCategory::Preference,
            LearningContent {
                description: description.to_string(),
                pattern: None,
                insight: "test insight".to_string(),
            },
            LearningSource::UserCreated,
        );
        learning.confidence = confidence;
        learning
    }

    // --- Threshold configuration tests ---

    #[test]
    fn test_default_threshold() {
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = SemanticDedup::new(embedder);
        assert!((dedup.threshold() - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_custom_threshold() {
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = SemanticDedup::with_threshold(embedder, 0.85);
        assert!((dedup.threshold() - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn test_threshold_clamped_high() {
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = SemanticDedup::with_threshold(embedder, 1.5);
        assert!((dedup.threshold() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_threshold_clamped_low() {
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = SemanticDedup::with_threshold(embedder, -0.5);
        assert!(dedup.threshold().abs() < f64::EPSILON);
    }

    // --- Merge behavior tests ---

    #[tokio::test]
    async fn test_merge_averages_confidence() {
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = SemanticDedup::new(embedder);

        let existing = make_learning("Use snake_case", 0.8);
        let duplicate = make_learning("Use snake_case for variables", 0.6);

        let merged = dedup.merge(&existing, &duplicate).await.unwrap();

        // Should average: (0.8 + 0.6) / 2 = 0.7
        assert!((merged.confidence - 0.7).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_merge_keeps_original_description() {
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = SemanticDedup::new(embedder);

        let existing = make_learning("Original description", 0.8);
        let duplicate = make_learning("Duplicate description", 0.6);

        let merged = dedup.merge(&existing, &duplicate).await.unwrap();

        assert_eq!(merged.content.description, "Original description");
    }

    #[tokio::test]
    async fn test_merge_keeps_original_id() {
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = SemanticDedup::new(embedder);

        let existing = make_learning("Test", 0.8);
        let duplicate = make_learning("Test", 0.6);

        let merged = dedup.merge(&existing, &duplicate).await.unwrap();

        assert_eq!(merged.id, existing.id);
    }

    #[tokio::test]
    async fn test_merge_updates_timestamp() {
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = SemanticDedup::new(embedder);

        let existing = make_learning("Test", 0.8);
        let original_updated_at = existing.updated_at;

        // Small delay to ensure timestamp changes
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let duplicate = make_learning("Test", 0.6);
        let merged = dedup.merge(&existing, &duplicate).await.unwrap();

        assert!(merged.updated_at > original_updated_at);
    }

    // --- Duplicate detection tests ---

    #[tokio::test]
    async fn test_find_duplicate_returns_match() {
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = SemanticDedup::new(embedder);

        let existing = make_learning("Existing learning", 0.8);
        let candidate = make_learning("Similar learning", 0.7);

        let store = MockStore::new().with_similar(existing.clone(), 0.95);

        let result = dedup.find_duplicate(&candidate, &store).await.unwrap();

        assert!(result.is_some());
        assert_eq!(result.unwrap().id, existing.id);
    }

    #[tokio::test]
    async fn test_find_duplicate_returns_none_when_no_match() {
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = SemanticDedup::new(embedder);

        let candidate = make_learning("Unique learning", 0.7);
        let store = MockStore::new(); // No similar results

        let result = dedup.find_duplicate(&candidate, &store).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_duplicate_excludes_self() {
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = SemanticDedup::new(embedder);

        let candidate = make_learning("Test learning", 0.7);

        // Store returns the candidate itself as a match (same ID)
        let store = MockStore::new().with_similar(candidate.clone(), 1.0);

        let result = dedup.find_duplicate(&candidate, &store).await.unwrap();

        // Should return None because the match is the candidate itself
        assert!(result.is_none());
    }
}
