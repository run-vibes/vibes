//! Novelty detection using embedding similarity
//!
//! Detects patterns the system hasn't seen before using a two-stage approach:
//! 1. Fast context hashing for pre-filtering known patterns
//! 2. Embedding similarity for semantic comparison

use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

use chrono::Utc;
use tracing::{debug, instrument};

use crate::error::Result;
use crate::extraction::embedder::{Embedder, cosine_similarity};
use crate::types::AdaptiveParam;

use super::traits::OpenWorldStore;
use super::types::{AnomalyCluster, ClusterId, NoveltyResult, PatternFingerprint};

/// Configuration for the novelty detector
#[derive(Debug, Clone)]
pub struct NoveltyConfig {
    /// Initial similarity threshold (0.0-1.0)
    pub initial_threshold: f64,
    /// Prior for adaptive threshold (alpha, beta)
    pub threshold_prior: (f64, f64),
    /// Maximum pending outliers before clustering
    pub max_pending_outliers: usize,
    /// Minimum cluster size for stability
    pub min_cluster_size: usize,
}

impl Default for NoveltyConfig {
    fn default() -> Self {
        Self {
            initial_threshold: 0.85,
            threshold_prior: (8.5, 1.5),
            max_pending_outliers: 50,
            min_cluster_size: 3,
        }
    }
}

/// Context for novelty detection
#[derive(Debug, Clone)]
pub struct NoveltyContext {
    /// Text content to analyze
    pub text: String,
    /// Optional summary
    pub summary: Option<String>,
}

impl NoveltyContext {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            summary: None,
        }
    }

    pub fn with_summary(text: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            summary: Some(summary.into()),
        }
    }

    pub fn to_text(&self) -> &str {
        &self.text
    }

    pub fn summary(&self) -> String {
        self.summary.clone().unwrap_or_else(|| {
            if self.text.len() > 100 {
                format!("{}...", &self.text[..100])
            } else {
                self.text.clone()
            }
        })
    }
}

/// Novelty detector using embedding similarity
pub struct NoveltyDetector {
    pub(crate) embedder: Arc<dyn Embedder>,
    store: Arc<dyn OpenWorldStore>,
    config: NoveltyConfig,
    known_hashes: HashSet<u64>,
    pending_outliers: Vec<PatternFingerprint>,
    pub(crate) clusters: Vec<AnomalyCluster>,
    similarity_threshold: AdaptiveParam,
}

impl NoveltyDetector {
    pub fn new(
        embedder: Arc<dyn Embedder>,
        store: Arc<dyn OpenWorldStore>,
        config: NoveltyConfig,
    ) -> Self {
        let similarity_threshold =
            AdaptiveParam::new_with_prior(config.threshold_prior.0, config.threshold_prior.1);

        Self {
            embedder,
            store,
            config,
            known_hashes: HashSet::new(),
            pending_outliers: Vec::new(),
            clusters: Vec::new(),
            similarity_threshold,
        }
    }

    #[instrument(skip(self))]
    pub async fn load_from_store(&mut self) -> Result<()> {
        let fingerprints = self.store.get_fingerprints().await?;
        debug!(count = fingerprints.len(), "Loaded fingerprints");
        for fp in fingerprints {
            self.known_hashes.insert(fp.hash);
        }

        let clusters = self.store.get_clusters().await?;
        debug!(count = clusters.len(), "Loaded clusters");
        self.clusters = clusters;

        Ok(())
    }

    #[instrument(skip(self, context), fields(hash))]
    pub async fn detect(&mut self, context: &NoveltyContext) -> Result<NoveltyResult> {
        let hash = self.hash_context(context);
        tracing::Span::current().record("hash", hash);

        // Step 1: Fast hash check
        if self.known_hashes.contains(&hash) {
            debug!("Pattern matched by hash");
            return Ok(NoveltyResult::Known {
                fingerprint: PatternFingerprint {
                    hash,
                    embedding: Vec::new(),
                    context_summary: context.summary(),
                    created_at: Utc::now(),
                },
            });
        }

        // Step 2: Compute embedding
        let embedding = self.embedder.embed(context.to_text()).await.map_err(|e| {
            crate::error::GrooveError::Embedding(format!("Embedding failed: {}", e))
        })?;

        // Step 3: Find nearest cluster
        if let Some((cluster_id, similarity)) = self.find_nearest_cluster(&embedding) {
            let threshold = self.similarity_threshold.value;
            if similarity > threshold {
                debug!(similarity, threshold, ?cluster_id, "Matched by embedding");

                let fingerprint = PatternFingerprint {
                    hash,
                    embedding: embedding.clone(),
                    context_summary: context.summary(),
                    created_at: Utc::now(),
                };

                self.known_hashes.insert(hash);
                return Ok(NoveltyResult::Known { fingerprint });
            }
        }

        // Step 4: Novel pattern
        debug!(pending = self.pending_outliers.len(), "Novel pattern");

        for pending in &self.pending_outliers {
            let sim = cosine_similarity(&embedding, &pending.embedding);
            if sim > self.similarity_threshold.value as f32 {
                return Ok(NoveltyResult::PendingClassification { embedding });
            }
        }

        Ok(NoveltyResult::Novel {
            cluster: None,
            embedding,
        })
    }

    #[instrument(skip(self, context))]
    pub async fn mark_known(&mut self, context: &NoveltyContext) -> Result<()> {
        let hash = self.hash_context(context);
        self.known_hashes.insert(hash);

        let embedding = self.embedder.embed(context.to_text()).await.map_err(|e| {
            crate::error::GrooveError::Embedding(format!("Embedding failed: {}", e))
        })?;

        let fingerprint = PatternFingerprint {
            hash,
            embedding,
            context_summary: context.summary(),
            created_at: Utc::now(),
        };

        self.store.save_fingerprint(&fingerprint).await?;
        debug!(hash, "Marked pattern as known");
        Ok(())
    }

    pub async fn add_to_pending(&mut self, fingerprint: PatternFingerprint) -> Result<()> {
        self.pending_outliers.push(fingerprint);

        if self.pending_outliers.len() >= self.config.max_pending_outliers {
            self.cluster_pending().await?;
        }

        Ok(())
    }

    pub fn update_threshold(&mut self, outcome: f64, weight: f64) {
        self.similarity_threshold.update(outcome, weight);
        debug!(
            new_threshold = self.similarity_threshold.value,
            "Updated threshold"
        );
    }

    pub fn threshold(&self) -> f64 {
        self.similarity_threshold.value
    }

    pub fn known_count(&self) -> usize {
        self.known_hashes.len()
    }

    pub fn pending_count(&self) -> usize {
        self.pending_outliers.len()
    }

    pub fn cluster_count(&self) -> usize {
        self.clusters.len()
    }

    pub(crate) fn hash_context(&self, context: &NoveltyContext) -> u64 {
        let mut hasher = DefaultHasher::new();
        context.text.hash(&mut hasher);
        hasher.finish()
    }

    fn find_nearest_cluster(&self, embedding: &[f32]) -> Option<(ClusterId, f64)> {
        if self.clusters.is_empty() {
            return None;
        }

        let mut best: Option<(ClusterId, f64)> = None;
        for cluster in &self.clusters {
            let sim = cosine_similarity(embedding, &cluster.centroid) as f64;
            if best.is_none() || sim > best.as_ref().unwrap().1 {
                best = Some((cluster.id, sim));
            }
        }
        best
    }

    async fn cluster_pending(&mut self) -> Result<()> {
        if self.pending_outliers.is_empty() {
            return Ok(());
        }

        debug!(count = self.pending_outliers.len(), "Clustering outliers");

        let threshold = self.similarity_threshold.value as f32;
        let mut assigned: HashSet<usize> = HashSet::new();
        let mut new_clusters: Vec<AnomalyCluster> = Vec::new();

        for (i, outlier) in self.pending_outliers.iter().enumerate() {
            if assigned.contains(&i) {
                continue;
            }

            let mut members = vec![outlier.clone()];
            assigned.insert(i);

            for (j, other) in self.pending_outliers.iter().enumerate() {
                if assigned.contains(&j) {
                    continue;
                }

                let sim = cosine_similarity(&outlier.embedding, &other.embedding);
                if sim > threshold {
                    members.push(other.clone());
                    assigned.insert(j);
                }
            }

            if members.len() >= self.config.min_cluster_size {
                let cluster = AnomalyCluster {
                    id: uuid::Uuid::now_v7(),
                    centroid: self.compute_centroid(&members),
                    members,
                    created_at: Utc::now(),
                    last_seen: Utc::now(),
                };

                self.store.save_cluster(&cluster).await?;
                new_clusters.push(cluster);
            }
        }

        let remaining: Vec<_> = self
            .pending_outliers
            .iter()
            .enumerate()
            .filter(|(i, _)| !assigned.contains(i))
            .map(|(_, fp)| fp.clone())
            .collect();

        self.pending_outliers = remaining;
        self.clusters.extend(new_clusters);

        debug!(
            clusters = self.clusters.len(),
            remaining = self.pending_outliers.len(),
            "Clustering complete"
        );

        Ok(())
    }

    fn compute_centroid(&self, members: &[PatternFingerprint]) -> Vec<f32> {
        if members.is_empty() {
            return Vec::new();
        }

        let dim = members[0].embedding.len();
        let mut centroid = vec![0.0f32; dim];
        let count = members.len() as f32;

        for member in members {
            for (i, &val) in member.embedding.iter().enumerate() {
                centroid[i] += val;
            }
        }

        for val in &mut centroid {
            *val /= count;
        }

        centroid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openworld::traits::NoOpOpenWorldStore;
    use async_trait::async_trait;

    /// Mock embedder with deterministic embeddings
    struct MockEmbedder {
        dimensions: usize,
    }

    impl MockEmbedder {
        fn new(dimensions: usize) -> Self {
            Self { dimensions }
        }
    }

    #[async_trait]
    impl Embedder for MockEmbedder {
        async fn embed(&self, text: &str) -> crate::extraction::embedder::EmbedderResult<Vec<f32>> {
            let mut hasher = DefaultHasher::new();
            text.hash(&mut hasher);
            let hash = hasher.finish();

            let mut embedding = Vec::with_capacity(self.dimensions);
            let mut seed = hash;
            for _ in 0..self.dimensions {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let val = ((seed >> 32) as f32) / (u32::MAX as f32) * 2.0 - 1.0;
                embedding.push(val);
            }

            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for val in &mut embedding {
                    *val /= norm;
                }
            }

            Ok(embedding)
        }

        fn dimensions(&self) -> usize {
            self.dimensions
        }
    }

    fn test_detector() -> NoveltyDetector {
        let embedder = Arc::new(MockEmbedder::new(64));
        let store = Arc::new(NoOpOpenWorldStore);
        NoveltyDetector::new(embedder, store, NoveltyConfig::default())
    }

    #[tokio::test]
    async fn test_novel_pattern_detected() {
        let mut detector = test_detector();

        let context = NoveltyContext::new("This is a completely new pattern");
        let result = detector.detect(&context).await.unwrap();

        assert!(matches!(result, NoveltyResult::Novel { .. }));
    }

    #[tokio::test]
    async fn test_known_pattern_via_hash() {
        let mut detector = test_detector();

        let context = NoveltyContext::new("This pattern will be known");
        detector.mark_known(&context).await.unwrap();

        let result = detector.detect(&context).await.unwrap();
        assert!(matches!(result, NoveltyResult::Known { .. }));
    }

    #[tokio::test]
    async fn test_similar_patterns_match_via_embedding() {
        let mut detector = test_detector();

        let known_context = NoveltyContext::new("The quick brown fox jumps");
        detector.mark_known(&known_context).await.unwrap();

        let embedding = detector
            .embedder
            .embed(known_context.to_text())
            .await
            .unwrap();

        let cluster = AnomalyCluster {
            id: uuid::Uuid::now_v7(),
            centroid: embedding,
            members: vec![],
            created_at: Utc::now(),
            last_seen: Utc::now(),
        };
        detector.clusters.push(cluster);

        let result = detector.detect(&known_context).await.unwrap();
        assert!(matches!(result, NoveltyResult::Known { .. }));
    }

    #[tokio::test]
    async fn test_threshold_adaptation() {
        let mut detector = test_detector();

        let initial = detector.threshold();

        detector.update_threshold(1.0, 1.0);
        let after_positive = detector.threshold();
        assert!(after_positive > initial);

        detector.update_threshold(0.0, 1.0);
        let after_negative = detector.threshold();
        assert!(after_negative < after_positive);
    }

    #[tokio::test]
    async fn test_add_to_pending() {
        let mut detector = test_detector();

        let fingerprint = PatternFingerprint {
            hash: 12345,
            embedding: vec![0.1, 0.2, 0.3],
            context_summary: "test".to_string(),
            created_at: Utc::now(),
        };

        detector.add_to_pending(fingerprint).await.unwrap();
        assert_eq!(detector.pending_count(), 1);
    }

    #[test]
    fn test_novelty_context_summary() {
        let ctx = NoveltyContext::new("short text");
        assert_eq!(ctx.summary(), "short text");

        let long_text = "a".repeat(200);
        let ctx = NoveltyContext::new(&long_text);
        assert!(ctx.summary().ends_with("..."));
        assert!(ctx.summary().len() < 110);

        let ctx = NoveltyContext::with_summary("text", "custom summary");
        assert_eq!(ctx.summary(), "custom summary");
    }

    #[test]
    fn test_config_defaults() {
        let config = NoveltyConfig::default();
        assert!((config.initial_threshold - 0.85).abs() < 0.01);
        assert_eq!(config.max_pending_outliers, 50);
        assert_eq!(config.min_cluster_size, 3);
    }

    #[test]
    fn test_hash_is_deterministic() {
        let embedder = Arc::new(MockEmbedder::new(64));
        let store = Arc::new(NoOpOpenWorldStore);
        let detector = NoveltyDetector::new(embedder, store, NoveltyConfig::default());

        let ctx1 = NoveltyContext::new("test content");
        let ctx2 = NoveltyContext::new("test content");
        let ctx3 = NoveltyContext::new("different content");

        let hash1 = detector.hash_context(&ctx1);
        let hash2 = detector.hash_context(&ctx2);
        let hash3 = detector.hash_context(&ctx3);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_compute_centroid() {
        let embedder = Arc::new(MockEmbedder::new(3));
        let store = Arc::new(NoOpOpenWorldStore);
        let detector = NoveltyDetector::new(embedder, store, NoveltyConfig::default());

        let members = vec![
            PatternFingerprint {
                hash: 1,
                embedding: vec![1.0, 2.0, 3.0],
                context_summary: "a".to_string(),
                created_at: Utc::now(),
            },
            PatternFingerprint {
                hash: 2,
                embedding: vec![3.0, 4.0, 5.0],
                context_summary: "b".to_string(),
                created_at: Utc::now(),
            },
        ];

        let centroid = detector.compute_centroid(&members);
        assert_eq!(centroid, vec![2.0, 3.0, 4.0]);
    }
}
