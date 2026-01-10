//! Capability gap detection from combined signals
//!
//! Identifies recurring failure patterns indicating system lacks knowledge or capability.
//! Combines failures, negative attribution, and low confidence to surface actionable gaps.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribution::{AttributionRecord, SessionOutcome};
    use crate::openworld::traits::NoOpOpenWorldStore;
    use crate::openworld::types::{GapCategory, GapSeverity};
    use chrono::Utc;
    use std::sync::Arc;
    use uuid::Uuid;

    fn test_detector() -> CapabilityGapDetector {
        let store = Arc::new(NoOpOpenWorldStore);
        CapabilityGapDetector::new(store, GapsConfig::default())
    }

    fn make_session_outcome(outcome: f64) -> SessionOutcome {
        SessionOutcome {
            session_id: "test-session".into(),
            outcome,
            timestamp: Utc::now(),
        }
    }

    fn make_attribution(learning_id: Uuid, attributed_value: f64) -> AttributionRecord {
        AttributionRecord {
            learning_id,
            session_id: "test-session".into(),
            timestamp: Utc::now(),
            was_activated: true,
            activation_confidence: 0.9,
            activation_signals: vec![],
            temporal_positive: 0.0,
            temporal_negative: 0.0,
            net_temporal: 0.0,
            was_withheld: false,
            session_outcome: 0.5,
            attributed_value,
        }
    }

    // =========================================================================
    // Config tests
    // =========================================================================

    #[test]
    fn test_config_defaults() {
        let config = GapsConfig::default();
        assert_eq!(config.min_failures_for_gap, 3);
        assert!(config.negative_attribution_threshold < 0.0);
        assert!(config.low_confidence_threshold > 0.0);
    }

    // =========================================================================
    // Failure detection tests
    // =========================================================================

    #[test]
    fn test_detect_failure_explicit_negative_feedback() {
        let detector = test_detector();
        // Outcome < 0 indicates negative feedback
        let session = make_session_outcome(-0.5);
        let attributions = vec![];

        let failure = detector.detect_failure(&session, &attributions, 12345);
        assert!(failure.is_some());
        let f = failure.unwrap();
        assert_eq!(f.failure_type, FailureType::ExplicitFeedback);
        assert_eq!(f.context_hash, 12345);
    }

    #[test]
    fn test_detect_failure_negative_attribution() {
        let detector = test_detector();
        let session = make_session_outcome(0.5); // Positive outcome
        let attributions = vec![make_attribution(Uuid::now_v7(), -0.5)]; // Negative attribution

        let failure = detector.detect_failure(&session, &attributions, 12345);
        assert!(failure.is_some());
        let f = failure.unwrap();
        assert_eq!(f.failure_type, FailureType::NegativeAttribution);
        assert!(!f.learning_ids.is_empty());
    }

    #[test]
    fn test_detect_failure_low_confidence() {
        let detector = test_detector();
        // Low positive outcome = low confidence
        let session = make_session_outcome(0.1);
        let attributions = vec![];

        let failure = detector.detect_failure(&session, &attributions, 12345);
        assert!(failure.is_some());
        let f = failure.unwrap();
        assert_eq!(f.failure_type, FailureType::LowConfidence);
    }

    #[test]
    fn test_detect_failure_no_failure_for_good_outcome() {
        let detector = test_detector();
        let session = make_session_outcome(0.8);
        let attributions = vec![make_attribution(Uuid::now_v7(), 0.5)];

        let failure = detector.detect_failure(&session, &attributions, 12345);
        assert!(failure.is_none());
    }

    // =========================================================================
    // Gap aggregation tests
    // =========================================================================

    #[tokio::test]
    async fn test_record_failure_tracks_by_context() {
        let mut detector = test_detector();
        let session = make_session_outcome(-0.5);

        // Record failure for context 12345
        detector
            .process_outcome(&session, &[], 12345)
            .await
            .unwrap();

        assert_eq!(detector.failure_count_for_context(12345), 1);
        assert_eq!(detector.failure_count_for_context(99999), 0);
    }

    #[tokio::test]
    async fn test_gap_created_after_threshold() {
        let mut detector = test_detector();

        // Record failures below threshold
        for _ in 0..2 {
            let session = make_session_outcome(-0.5);
            detector
                .process_outcome(&session, &[], 12345)
                .await
                .unwrap();
        }
        assert_eq!(detector.active_gap_count(), 0);

        // Third failure should create gap
        let session = make_session_outcome(-0.5);
        let result = detector
            .process_outcome(&session, &[], 12345)
            .await
            .unwrap();

        assert!(result.is_some());
        assert_eq!(detector.active_gap_count(), 1);
    }

    #[tokio::test]
    async fn test_existing_gap_updated_not_duplicated() {
        let mut detector = test_detector();

        // Create 5 failures for same context
        for _ in 0..5 {
            let session = make_session_outcome(-0.5);
            detector
                .process_outcome(&session, &[], 12345)
                .await
                .unwrap();
        }

        // Should still be just one gap
        assert_eq!(detector.active_gap_count(), 1);

        // Gap should have 5 failures
        let gaps = detector.get_active_gaps();
        assert_eq!(gaps[0].failure_count, 5);
    }

    // =========================================================================
    // Severity escalation tests
    // =========================================================================

    #[tokio::test]
    async fn test_severity_escalates_with_failures() {
        let mut detector = test_detector();

        // 3 failures = Low severity
        for _ in 0..3 {
            let session = make_session_outcome(-0.5);
            detector
                .process_outcome(&session, &[], 12345)
                .await
                .unwrap();
        }
        let gaps = detector.get_active_gaps();
        assert_eq!(gaps[0].severity, GapSeverity::Medium);

        // 11 failures = High severity
        for _ in 0..8 {
            let session = make_session_outcome(-0.5);
            detector
                .process_outcome(&session, &[], 12345)
                .await
                .unwrap();
        }
        let gaps = detector.get_active_gaps();
        assert_eq!(gaps[0].severity, GapSeverity::High);
    }

    // =========================================================================
    // Category classification tests
    // =========================================================================

    #[tokio::test]
    async fn test_category_incorrect_pattern_from_negative_attribution() {
        let mut detector = test_detector();
        let learning_id = Uuid::now_v7();

        // Record failures from negative attribution
        for _ in 0..3 {
            let session = make_session_outcome(0.5);
            let attributions = vec![make_attribution(learning_id, -0.5)];
            detector
                .process_outcome(&session, &attributions, 12345)
                .await
                .unwrap();
        }

        let gaps = detector.get_active_gaps();
        assert_eq!(gaps[0].category, GapCategory::IncorrectPattern);
    }

    #[tokio::test]
    async fn test_category_missing_knowledge_from_explicit_feedback() {
        let mut detector = test_detector();

        // Record failures from explicit negative feedback
        for _ in 0..3 {
            let session = make_session_outcome(-0.5);
            detector
                .process_outcome(&session, &[], 12345)
                .await
                .unwrap();
        }

        let gaps = detector.get_active_gaps();
        assert_eq!(gaps[0].category, GapCategory::MissingKnowledge);
    }
}

// =============================================================================
// Implementation
// =============================================================================

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use crate::attribution::{AttributionRecord, SessionOutcome};
use crate::error::Result;

use super::traits::OpenWorldStore;
use super::types::{CapabilityGap, FailureRecord, FailureType, GapCategory, GapId, GapSeverity};

/// Configuration for gap detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapsConfig {
    /// Minimum failures before creating a gap
    pub min_failures_for_gap: usize,
    /// Threshold below which attribution is considered negative
    pub negative_attribution_threshold: f64,
    /// Threshold below which confidence is considered low
    pub low_confidence_threshold: f64,
}

impl Default for GapsConfig {
    fn default() -> Self {
        Self {
            min_failures_for_gap: 3,
            negative_attribution_threshold: -0.3,
            low_confidence_threshold: 0.3,
        }
    }
}

/// Detects capability gaps from combined signals
pub struct CapabilityGapDetector {
    store: Arc<dyn OpenWorldStore>,
    config: GapsConfig,

    // In-memory tracking
    failure_counts: HashMap<u64, Vec<FailureRecord>>,
    active_gaps: HashMap<GapId, CapabilityGap>,
    context_to_gap: HashMap<u64, GapId>,
}

impl CapabilityGapDetector {
    /// Create a new gap detector
    pub fn new(store: Arc<dyn OpenWorldStore>, config: GapsConfig) -> Self {
        Self {
            store,
            config,
            failure_counts: HashMap::new(),
            active_gaps: HashMap::new(),
            context_to_gap: HashMap::new(),
        }
    }

    /// Detect failure from session outcome and attributions
    pub fn detect_failure(
        &self,
        session: &SessionOutcome,
        attributions: &[AttributionRecord],
        context_hash: u64,
    ) -> Option<FailureRecord> {
        // Priority 1: Explicit negative feedback (outcome < 0)
        if session.outcome < 0.0 {
            return Some(FailureRecord::new(
                session.session_id.clone(),
                FailureType::ExplicitFeedback,
                context_hash,
                vec![],
            ));
        }

        // Priority 2: Negative attribution
        for attr in attributions {
            if attr.attributed_value < self.config.negative_attribution_threshold {
                return Some(FailureRecord::new(
                    session.session_id.clone(),
                    FailureType::NegativeAttribution,
                    context_hash,
                    vec![attr.learning_id],
                ));
            }
        }

        // Priority 3: Low confidence (outcome between 0 and threshold)
        if session.outcome < self.config.low_confidence_threshold {
            return Some(FailureRecord::new(
                session.session_id.clone(),
                FailureType::LowConfidence,
                context_hash,
                vec![],
            ));
        }

        None
    }

    /// Process an outcome and update gap tracking
    #[instrument(skip(self, session, attributions))]
    pub async fn process_outcome(
        &mut self,
        session: &SessionOutcome,
        attributions: &[AttributionRecord],
        context_hash: u64,
    ) -> Result<Option<CapabilityGap>> {
        let failure = match self.detect_failure(session, attributions, context_hash) {
            Some(f) => f,
            None => return Ok(None),
        };

        self.record_failure(failure).await
    }

    /// Record a failure and check for gap creation
    async fn record_failure(&mut self, failure: FailureRecord) -> Result<Option<CapabilityGap>> {
        let context_hash = failure.context_hash;
        let failure_type = failure.failure_type;

        // Track failure
        self.failure_counts
            .entry(context_hash)
            .or_default()
            .push(failure.clone());

        // Persist failure
        self.store.save_failure(&failure).await?;

        // Check if we should create/update gap
        let failures = &self.failure_counts[&context_hash];
        let failure_count = failures.len() as u32;
        if failure_count >= self.config.min_failures_for_gap as u32 {
            let gap = self.get_or_create_gap(context_hash, failure_type, failure_count)?;
            return Ok(Some(gap.clone()));
        }

        Ok(None)
    }

    /// Get or create a gap for the context
    fn get_or_create_gap(
        &mut self,
        context_hash: u64,
        failure_type: FailureType,
        failure_count: u32,
    ) -> Result<&mut CapabilityGap> {
        if let Some(&gap_id) = self.context_to_gap.get(&context_hash) {
            // Update existing gap
            let gap = self.active_gaps.get_mut(&gap_id).unwrap();
            gap.record_failure();
            debug!(gap_id = %gap.id, failures = gap.failure_count, "Updated existing gap");
            Ok(gap)
        } else {
            // Create new gap with current failure count
            let category = Self::classify_category(failure_type);
            let context_pattern = format!("context:{}", context_hash);
            let mut gap = CapabilityGap::new(category, context_pattern);
            gap.failure_count = failure_count;
            gap.severity = Self::severity_for_count(failure_count);

            debug!(gap_id = %gap.id, category = ?category, failures = failure_count, "Created new capability gap");

            let gap_id = gap.id;
            self.active_gaps.insert(gap_id, gap);
            self.context_to_gap.insert(context_hash, gap_id);

            Ok(self.active_gaps.get_mut(&gap_id).unwrap())
        }
    }

    /// Determine severity based on failure count
    fn severity_for_count(count: u32) -> GapSeverity {
        match count {
            0..=2 => GapSeverity::Low,
            3..=10 => GapSeverity::Medium,
            11..=50 => GapSeverity::High,
            _ => GapSeverity::Critical,
        }
    }

    /// Classify gap category from failure type
    fn classify_category(failure_type: FailureType) -> GapCategory {
        match failure_type {
            FailureType::NegativeAttribution => GapCategory::IncorrectPattern,
            FailureType::ExplicitFeedback => GapCategory::MissingKnowledge,
            FailureType::LowConfidence => GapCategory::ContextMismatch,
            FailureType::LearningNotActivated => GapCategory::MissingKnowledge,
        }
    }

    /// Get failure count for a context
    pub fn failure_count_for_context(&self, context_hash: u64) -> usize {
        self.failure_counts
            .get(&context_hash)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Get number of active gaps
    pub fn active_gap_count(&self) -> usize {
        self.active_gaps.len()
    }

    /// Get all active gaps
    pub fn get_active_gaps(&self) -> Vec<&CapabilityGap> {
        self.active_gaps.values().collect()
    }
}
