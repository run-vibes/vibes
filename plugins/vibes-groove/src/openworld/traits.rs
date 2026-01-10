//! Traits for open-world adaptation storage
//!
//! Defines the storage interface for novelty detection and capability gap data.

use async_trait::async_trait;

use crate::error::Result;

use super::types::{
    AnomalyCluster, CapabilityGap, ClusterId, FailureRecord, GapId, GapStatus, OpenWorldEvent,
    PatternFingerprint, SuggestedSolution,
};

/// Storage interface for open-world adaptation data
#[async_trait]
pub trait OpenWorldStore: Send + Sync {
    // ========================================================================
    // Pattern Fingerprints
    // ========================================================================

    /// Save a pattern fingerprint
    async fn save_fingerprint(&self, fingerprint: &PatternFingerprint) -> Result<()>;

    /// Get all known fingerprints
    async fn get_fingerprints(&self) -> Result<Vec<PatternFingerprint>>;

    /// Find fingerprints by context hash (for fast pre-filtering)
    async fn find_fingerprints_by_hash(&self, hash: u64) -> Result<Vec<PatternFingerprint>>;

    // ========================================================================
    // Anomaly Clusters
    // ========================================================================

    /// Save an anomaly cluster
    async fn save_cluster(&self, cluster: &AnomalyCluster) -> Result<()>;

    /// Get a cluster by ID
    async fn get_cluster(&self, id: ClusterId) -> Result<Option<AnomalyCluster>>;

    /// Get all anomaly clusters
    async fn get_clusters(&self) -> Result<Vec<AnomalyCluster>>;

    /// Delete a cluster (when it's been classified as known)
    async fn delete_cluster(&self, id: ClusterId) -> Result<()>;

    // ========================================================================
    // Capability Gaps
    // ========================================================================

    /// Save a capability gap
    async fn save_gap(&self, gap: &CapabilityGap) -> Result<()>;

    /// Get a gap by ID
    async fn get_gap(&self, id: GapId) -> Result<Option<CapabilityGap>>;

    /// Get all gaps, optionally filtered by status
    async fn get_gaps(&self, status: Option<GapStatus>) -> Result<Vec<CapabilityGap>>;

    /// Update gap status
    async fn update_gap_status(&self, id: GapId, status: GapStatus) -> Result<()>;

    /// Add solutions to a gap
    async fn add_gap_solutions(&self, id: GapId, solutions: Vec<SuggestedSolution>) -> Result<()>;

    // ========================================================================
    // Failure Records
    // ========================================================================

    /// Save a failure record
    async fn save_failure(&self, record: &FailureRecord) -> Result<()>;

    /// Get failures by context hash
    async fn get_failures_by_context(&self, context_hash: u64) -> Result<Vec<FailureRecord>>;

    /// Get recent failures (for gap analysis)
    async fn get_recent_failures(&self, limit: usize) -> Result<Vec<FailureRecord>>;

    // ========================================================================
    // Events
    // ========================================================================

    /// Emit an open-world event (for Iggy streaming)
    async fn emit_event(&self, event: OpenWorldEvent) -> Result<()>;

    /// Get recent events
    async fn get_recent_events(&self, limit: usize) -> Result<Vec<OpenWorldEvent>>;
}

/// No-op implementation for testing
#[derive(Debug, Default)]
pub struct NoOpOpenWorldStore;

#[async_trait]
impl OpenWorldStore for NoOpOpenWorldStore {
    async fn save_fingerprint(&self, _fingerprint: &PatternFingerprint) -> Result<()> {
        Ok(())
    }

    async fn get_fingerprints(&self) -> Result<Vec<PatternFingerprint>> {
        Ok(Vec::new())
    }

    async fn find_fingerprints_by_hash(&self, _hash: u64) -> Result<Vec<PatternFingerprint>> {
        Ok(Vec::new())
    }

    async fn save_cluster(&self, _cluster: &AnomalyCluster) -> Result<()> {
        Ok(())
    }

    async fn get_cluster(&self, _id: ClusterId) -> Result<Option<AnomalyCluster>> {
        Ok(None)
    }

    async fn get_clusters(&self) -> Result<Vec<AnomalyCluster>> {
        Ok(Vec::new())
    }

    async fn delete_cluster(&self, _id: ClusterId) -> Result<()> {
        Ok(())
    }

    async fn save_gap(&self, _gap: &CapabilityGap) -> Result<()> {
        Ok(())
    }

    async fn get_gap(&self, _id: GapId) -> Result<Option<CapabilityGap>> {
        Ok(None)
    }

    async fn get_gaps(&self, _status: Option<GapStatus>) -> Result<Vec<CapabilityGap>> {
        Ok(Vec::new())
    }

    async fn update_gap_status(&self, _id: GapId, _status: GapStatus) -> Result<()> {
        Ok(())
    }

    async fn add_gap_solutions(
        &self,
        _id: GapId,
        _solutions: Vec<SuggestedSolution>,
    ) -> Result<()> {
        Ok(())
    }

    async fn save_failure(&self, _record: &FailureRecord) -> Result<()> {
        Ok(())
    }

    async fn get_failures_by_context(&self, _context_hash: u64) -> Result<Vec<FailureRecord>> {
        Ok(Vec::new())
    }

    async fn get_recent_failures(&self, _limit: usize) -> Result<Vec<FailureRecord>> {
        Ok(Vec::new())
    }

    async fn emit_event(&self, _event: OpenWorldEvent) -> Result<()> {
        Ok(())
    }

    async fn get_recent_events(&self, _limit: usize) -> Result<Vec<OpenWorldEvent>> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_store_returns_empty() {
        let store = NoOpOpenWorldStore;

        assert!(store.get_fingerprints().await.unwrap().is_empty());
        assert!(store.get_clusters().await.unwrap().is_empty());
        assert!(store.get_gaps(None).await.unwrap().is_empty());
        assert!(store.get_recent_failures(10).await.unwrap().is_empty());
        assert!(store.get_recent_events(10).await.unwrap().is_empty());
    }
}
