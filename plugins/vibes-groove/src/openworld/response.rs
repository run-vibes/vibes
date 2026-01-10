//! Graduated response system for progressive novelty handling
//!
//! Determines how to respond to novel patterns based on observation count:
//! - Monitor: Just track, no action
//! - Cluster: Ensure clustering is running
//! - AutoAdjust: Increase exploration for uncertain contexts
//! - Surface: Create capability gap and notify user

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openworld::types::{AnomalyCluster, PatternFingerprint};
    use chrono::Utc;

    fn test_responder() -> GraduatedResponse {
        GraduatedResponse::new(ResponseConfig::default())
    }

    fn make_cluster(member_count: usize) -> AnomalyCluster {
        let mut cluster = AnomalyCluster {
            id: uuid::Uuid::now_v7(),
            centroid: vec![0.5, 0.5, 0.5],
            members: Vec::new(),
            created_at: Utc::now(),
            last_seen: Utc::now(),
        };

        for i in 0..member_count {
            cluster.members.push(PatternFingerprint {
                hash: i as u64,
                embedding: vec![0.1 * i as f32, 0.2, 0.3],
                context_summary: format!("member_{}", i),
                created_at: Utc::now(),
            });
        }

        cluster
    }

    // =========================================================================
    // Config tests
    // =========================================================================

    #[test]
    fn test_config_defaults() {
        let config = ResponseConfig::default();
        assert_eq!(config.monitor_threshold, 3);
        assert_eq!(config.cluster_threshold, 10);
        assert_eq!(config.auto_adjust_threshold, 25);
        assert!(config.exploration_adjustment > 0.0);
        assert!(config.max_exploration_bonus > 0.0);
    }

    // =========================================================================
    // Stage determination tests
    // =========================================================================

    #[test]
    fn test_determine_stage_monitor() {
        let responder = test_responder();
        let cluster = make_cluster(2);
        assert_eq!(responder.determine_stage(&cluster), ResponseStage::Monitor);
    }

    #[test]
    fn test_determine_stage_cluster() {
        let responder = test_responder();
        let cluster = make_cluster(5);
        assert_eq!(responder.determine_stage(&cluster), ResponseStage::Cluster);
    }

    #[test]
    fn test_determine_stage_auto_adjust() {
        let responder = test_responder();
        let cluster = make_cluster(15);
        assert_eq!(
            responder.determine_stage(&cluster),
            ResponseStage::AutoAdjust
        );
    }

    #[test]
    fn test_determine_stage_surface() {
        let responder = test_responder();
        let cluster = make_cluster(30);
        assert_eq!(responder.determine_stage(&cluster), ResponseStage::Surface);
    }

    // =========================================================================
    // Response action tests
    // =========================================================================

    #[test]
    fn test_respond_monitor_returns_none() {
        let responder = test_responder();
        let cluster = make_cluster(2);
        let action = responder.respond_to_cluster(&cluster);
        assert!(matches!(action, ResponseAction::None));
    }

    #[test]
    fn test_respond_cluster_returns_none() {
        let responder = test_responder();
        let cluster = make_cluster(5);
        let action = responder.respond_to_cluster(&cluster);
        assert!(matches!(action, ResponseAction::None));
    }

    #[test]
    fn test_respond_auto_adjust_returns_exploration() {
        let responder = test_responder();
        let cluster = make_cluster(15);
        let action = responder.respond_to_cluster(&cluster);
        if let ResponseAction::AdjustExploration(bonus) = action {
            assert!(bonus > 0.0);
            assert!(bonus <= responder.config.max_exploration_bonus);
        } else {
            panic!("Expected AdjustExploration, got {:?}", action);
        }
    }

    #[test]
    fn test_respond_surface_creates_gap() {
        let responder = test_responder();
        let cluster = make_cluster(30);
        let action = responder.respond_to_cluster(&cluster);
        assert!(matches!(action, ResponseAction::CreateGap(_)));
    }

    // =========================================================================
    // Exploration calculation tests
    // =========================================================================

    #[test]
    fn test_calculate_exploration_bonus_scales_with_size() {
        let responder = test_responder();

        let small = make_cluster(10);
        let large = make_cluster(20);

        let small_bonus = responder.calculate_exploration_bonus(&small);
        let large_bonus = responder.calculate_exploration_bonus(&large);

        assert!(large_bonus > small_bonus);
    }

    #[test]
    fn test_calculate_exploration_bonus_capped() {
        let responder = test_responder();
        let huge = make_cluster(1000);

        let bonus = responder.calculate_exploration_bonus(&huge);
        assert!(bonus <= responder.config.max_exploration_bonus);
    }

    // =========================================================================
    // Gap creation tests
    // =========================================================================

    #[test]
    fn test_create_gap_from_cluster() {
        let responder = test_responder();
        let cluster = make_cluster(30);

        let gap = responder.create_gap_from_cluster(&cluster);

        assert_eq!(gap.failure_count, 30);
        assert!(gap.context_pattern.contains("cluster:"));
    }

    // =========================================================================
    // Event emission tests
    // =========================================================================

    #[test]
    fn test_respond_emits_events() {
        let responder = test_responder();
        let cluster = make_cluster(30);

        let (action, events) = responder.respond_with_events(&cluster);

        assert!(matches!(action, ResponseAction::CreateGap(_)));
        assert!(!events.is_empty());
    }
}

// =============================================================================
// Implementation
// =============================================================================

use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use super::types::{
    AnomalyCluster, CapabilityGap, GapCategory, GapSeverity, OpenWorldEvent, ResponseAction,
    ResponseStage,
};

/// Configuration for graduated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseConfig {
    /// Threshold for Monitor stage (< this = Monitor)
    pub monitor_threshold: u32,
    /// Threshold for Cluster stage (< this = Cluster)
    pub cluster_threshold: u32,
    /// Threshold for AutoAdjust stage (< this = AutoAdjust)
    pub auto_adjust_threshold: u32,

    /// Base exploration adjustment
    pub exploration_adjustment: f64,
    /// Maximum exploration bonus
    pub max_exploration_bonus: f64,
}

impl Default for ResponseConfig {
    fn default() -> Self {
        Self {
            monitor_threshold: 3,
            cluster_threshold: 10,
            auto_adjust_threshold: 25,
            exploration_adjustment: 0.1,
            max_exploration_bonus: 0.5,
        }
    }
}

/// Graduated response system for progressive novelty handling
pub struct GraduatedResponse {
    config: ResponseConfig,
}

impl GraduatedResponse {
    /// Create a new graduated response handler
    pub fn new(config: ResponseConfig) -> Self {
        Self { config }
    }

    /// Determine response stage based on cluster size
    #[instrument(skip(self, cluster), fields(cluster_id = %cluster.id, size = cluster.members.len()))]
    pub fn determine_stage(&self, cluster: &AnomalyCluster) -> ResponseStage {
        let count = cluster.members.len() as u32;
        match count {
            n if n < self.config.monitor_threshold => ResponseStage::Monitor,
            n if n < self.config.cluster_threshold => ResponseStage::Cluster,
            n if n < self.config.auto_adjust_threshold => ResponseStage::AutoAdjust,
            _ => ResponseStage::Surface,
        }
    }

    /// Respond to a cluster based on its stage
    #[instrument(skip(self, cluster), fields(cluster_id = %cluster.id))]
    pub fn respond_to_cluster(&self, cluster: &AnomalyCluster) -> ResponseAction {
        let stage = self.determine_stage(cluster);
        debug!(?stage, "Responding to cluster");

        match stage {
            ResponseStage::Monitor => {
                // Just track, no action needed
                ResponseAction::None
            }

            ResponseStage::Cluster => {
                // Clustering is already happening, no additional action
                ResponseAction::None
            }

            ResponseStage::AutoAdjust => {
                // Increase exploration for uncertain contexts
                let bonus = self.calculate_exploration_bonus(cluster);
                debug!(bonus, "Adjusting exploration");
                ResponseAction::AdjustExploration(bonus)
            }

            ResponseStage::Surface => {
                // Create capability gap for persistent novelty
                let gap = self.create_gap_from_cluster(cluster);
                debug!(gap_id = %gap.id, "Creating gap from cluster");
                ResponseAction::CreateGap(gap)
            }
        }
    }

    /// Respond to cluster and emit events
    pub fn respond_with_events(
        &self,
        cluster: &AnomalyCluster,
    ) -> (ResponseAction, Vec<OpenWorldEvent>) {
        let action = self.respond_to_cluster(cluster);
        let mut events = Vec::new();

        match &action {
            ResponseAction::CreateGap(gap) => {
                events.push(OpenWorldEvent::GapCreated { gap: gap.clone() });
            }
            ResponseAction::AdjustExploration(bonus) => {
                // Could emit strategy feedback events here
                debug!(bonus, "Exploration adjustment event");
            }
            _ => {}
        }

        (action, events)
    }

    /// Calculate exploration bonus based on cluster size
    pub fn calculate_exploration_bonus(&self, cluster: &AnomalyCluster) -> f64 {
        let size = cluster.members.len() as f64;
        let base_bonus = self.config.exploration_adjustment * (size / 10.0).ln_1p();
        base_bonus.min(self.config.max_exploration_bonus)
    }

    /// Create a capability gap from a persistent cluster
    pub fn create_gap_from_cluster(&self, cluster: &AnomalyCluster) -> CapabilityGap {
        let context_pattern = format!("cluster:{}", cluster.id);
        let category = self.classify_cluster_category(cluster);

        let mut gap = CapabilityGap::new(category, context_pattern);
        gap.failure_count = cluster.members.len() as u32;
        gap.severity = self.severity_for_cluster_size(cluster.members.len());

        gap
    }

    /// Classify gap category from cluster patterns
    fn classify_cluster_category(&self, cluster: &AnomalyCluster) -> GapCategory {
        // For now, use a simple heuristic based on cluster characteristics
        // Could be enhanced with embedding analysis
        if cluster.members.is_empty() {
            return GapCategory::MissingKnowledge;
        }

        // Check if members have similar context summaries (same problem recurring)
        let first_summary = &cluster.members[0].context_summary;
        let all_similar = cluster.members.iter().all(|m| {
            m.context_summary
                .starts_with(&first_summary[..first_summary.len().min(20)])
        });

        if all_similar {
            GapCategory::ContextMismatch
        } else {
            GapCategory::MissingKnowledge
        }
    }

    /// Determine severity based on cluster size
    fn severity_for_cluster_size(&self, size: usize) -> GapSeverity {
        match size {
            0..=10 => GapSeverity::Low,
            11..=25 => GapSeverity::Medium,
            26..=50 => GapSeverity::High,
            _ => GapSeverity::Critical,
        }
    }

    /// Get the current config
    pub fn config(&self) -> &ResponseConfig {
        &self.config
    }
}
