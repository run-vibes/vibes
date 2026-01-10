//! Core types for open-world adaptation
//!
//! Defines types for novelty detection, capability gaps, and graduated response.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::assessment::SessionId;
use crate::strategy::StrategyVariant;
use crate::types::{LearningCategory, LearningId};

// ============================================================================
// ID Types
// ============================================================================

/// Unique identifier for anomaly clusters
pub type ClusterId = Uuid;

/// Unique identifier for capability gaps
pub type GapId = Uuid;

/// Unique identifier for failure records
pub type FailureId = Uuid;

// ============================================================================
// Pattern Types (Task 1)
// ============================================================================

/// Fingerprint of a pattern for novelty detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternFingerprint {
    /// Fast pre-filter hash for context matching
    pub hash: u64,

    /// Embedding vector from gte-small (384 dims)
    pub embedding: Vec<f32>,

    /// Human-readable summary of the context
    pub context_summary: String,

    /// When this fingerprint was created
    pub created_at: DateTime<Utc>,
}

impl PatternFingerprint {
    /// Create a new pattern fingerprint
    pub fn new(hash: u64, embedding: Vec<f32>, context_summary: String) -> Self {
        Self {
            hash,
            embedding,
            context_summary,
            created_at: Utc::now(),
        }
    }
}

/// Cluster of similar novel patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyCluster {
    /// Unique cluster identifier
    pub id: ClusterId,

    /// Cluster center embedding
    pub centroid: Vec<f32>,

    /// Member fingerprints in this cluster
    pub members: Vec<PatternFingerprint>,

    /// When this cluster was first created
    pub created_at: DateTime<Utc>,

    /// When this cluster was last updated
    pub last_seen: DateTime<Utc>,
}

impl AnomalyCluster {
    /// Create a new anomaly cluster
    pub fn new(centroid: Vec<f32>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            centroid,
            members: Vec::new(),
            created_at: now,
            last_seen: now,
        }
    }

    /// Add a member to the cluster
    pub fn add_member(&mut self, fingerprint: PatternFingerprint) {
        self.members.push(fingerprint);
        self.last_seen = Utc::now();
    }

    /// Get the number of members in the cluster
    pub fn size(&self) -> usize {
        self.members.len()
    }
}

// ============================================================================
// Novelty Types (Task 2)
// ============================================================================

/// Result of novelty detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NoveltyResult {
    /// Pattern matches a known fingerprint
    Known {
        /// The matching fingerprint
        fingerprint: PatternFingerprint,
    },

    /// Pattern is novel, assigned to existing or new cluster
    Novel {
        /// Cluster ID if assigned (None for outliers)
        cluster: Option<ClusterId>,
        /// The embedding of the novel pattern
        embedding: Vec<f32>,
    },

    /// Pattern is pending classification (not enough data)
    PendingClassification {
        /// The embedding awaiting classification
        embedding: Vec<f32>,
    },
}

// ============================================================================
// Capability Gap Types (Task 2)
// ============================================================================

/// A capability gap identified by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityGap {
    /// Unique gap identifier
    pub id: GapId,

    /// Category of the gap
    pub category: GapCategory,

    /// Severity level
    pub severity: GapSeverity,

    /// Current status
    pub status: GapStatus,

    /// Pattern context that triggers this gap
    pub context_pattern: String,

    /// Number of failures contributing to this gap
    pub failure_count: u32,

    /// When this gap was first detected
    pub first_seen: DateTime<Utc>,

    /// When this gap was last observed
    pub last_seen: DateTime<Utc>,

    /// Suggested solutions for this gap
    pub suggested_solutions: Vec<SuggestedSolution>,
}

impl CapabilityGap {
    /// Create a new capability gap
    pub fn new(category: GapCategory, context_pattern: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            category,
            severity: GapSeverity::Low,
            status: GapStatus::Detected,
            context_pattern,
            failure_count: 1,
            first_seen: now,
            last_seen: now,
            suggested_solutions: Vec::new(),
        }
    }

    /// Record another failure for this gap
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_seen = Utc::now();
        self.update_severity();
    }

    /// Update severity based on failure count
    fn update_severity(&mut self) {
        self.severity = match self.failure_count {
            0..=2 => GapSeverity::Low,
            3..=10 => GapSeverity::Medium,
            _ => GapSeverity::High,
        };
    }
}

/// Category of capability gap
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GapCategory {
    /// System lacks knowledge in this area
    MissingKnowledge,

    /// Learned pattern is incorrect
    IncorrectPattern,

    /// Context doesn't match learned patterns
    ContextMismatch,

    /// Missing tool or capability
    ToolGap,
}

impl GapCategory {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MissingKnowledge => "missing_knowledge",
            Self::IncorrectPattern => "incorrect_pattern",
            Self::ContextMismatch => "context_mismatch",
            Self::ToolGap => "tool_gap",
        }
    }
}

/// Severity level of a capability gap
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GapSeverity {
    /// Less than 3 failures
    Low,

    /// 3-10 failures
    Medium,

    /// More than 10 failures
    High,

    /// User explicitly escalated
    Critical,
}

impl GapSeverity {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

/// Status of a capability gap
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GapStatus {
    /// Gap has been detected but not yet confirmed
    Detected,

    /// Gap has been confirmed as persistent
    Confirmed,

    /// Solution is being applied
    InProgress,

    /// Gap has been resolved
    Resolved,

    /// Gap was dismissed (false positive)
    Dismissed,
}

impl GapStatus {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Detected => "detected",
            Self::Confirmed => "confirmed",
            Self::InProgress => "in_progress",
            Self::Resolved => "resolved",
            Self::Dismissed => "dismissed",
        }
    }
}

// ============================================================================
// Failure Types (Task 3)
// ============================================================================

/// Record of a failure contributing to gap detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureRecord {
    /// Unique failure identifier
    pub id: FailureId,

    /// Session where the failure occurred
    pub session_id: SessionId,

    /// Type of failure
    pub failure_type: FailureType,

    /// Hash of the context where failure occurred
    pub context_hash: u64,

    /// Learning IDs involved (if any)
    pub learning_ids: Vec<LearningId>,

    /// When this failure occurred
    pub timestamp: DateTime<Utc>,
}

impl FailureRecord {
    /// Create a new failure record
    pub fn new(
        session_id: SessionId,
        failure_type: FailureType,
        context_hash: u64,
        learning_ids: Vec<LearningId>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            session_id,
            failure_type,
            context_hash,
            learning_ids,
            timestamp: Utc::now(),
        }
    }
}

/// Type of failure detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FailureType {
    /// Learning should have been activated but wasn't
    LearningNotActivated,

    /// Learning was activated but had negative attribution
    NegativeAttribution,

    /// Outcome confidence was too low
    LowConfidence,

    /// User explicitly marked as wrong
    ExplicitFeedback,
}

impl FailureType {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LearningNotActivated => "learning_not_activated",
            Self::NegativeAttribution => "negative_attribution",
            Self::LowConfidence => "low_confidence",
            Self::ExplicitFeedback => "explicit_feedback",
        }
    }
}

// ============================================================================
// Solution Types (Task 3)
// ============================================================================

/// A suggested solution for a capability gap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedSolution {
    /// The action to take
    pub action: SolutionAction,

    /// Where this solution came from
    pub source: SolutionSource,

    /// Confidence in this solution (0.0-1.0)
    pub confidence: f64,

    /// Whether this solution has been applied
    pub applied: bool,
}

impl SuggestedSolution {
    /// Create a new suggested solution
    pub fn new(action: SolutionAction, source: SolutionSource, confidence: f64) -> Self {
        Self {
            action,
            source,
            confidence,
            applied: false,
        }
    }

    /// Mark the solution as applied
    pub fn mark_applied(&mut self) {
        self.applied = true;
    }
}

/// Action to take to address a capability gap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SolutionAction {
    /// Create a new learning
    CreateLearning {
        /// Content for the new learning
        content: String,
        /// Category for the new learning
        category: LearningCategory,
    },

    /// Modify an existing learning
    ModifyLearning {
        /// ID of the learning to modify
        id: LearningId,
        /// Description of the change
        change: String,
    },

    /// Disable a problematic learning
    DisableLearning {
        /// ID of the learning to disable
        id: LearningId,
    },

    /// Adjust strategy parameters for a category
    AdjustStrategy {
        /// Category to adjust
        category: LearningCategory,
        /// The strategy change to apply
        change: StrategyChange,
    },

    /// Request human input to resolve the gap
    RequestHumanInput {
        /// Question to ask the human
        question: String,
    },
}

/// A change to strategy parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyChange {
    /// Variant to adjust (if specific)
    pub variant: Option<StrategyVariant>,

    /// Weight adjustment (-1.0 to 1.0)
    pub weight_delta: f64,

    /// Exploration bonus adjustment
    pub exploration_delta: f64,
}

impl StrategyChange {
    /// Create a new strategy change
    pub fn new(
        variant: Option<StrategyVariant>,
        weight_delta: f64,
        exploration_delta: f64,
    ) -> Self {
        Self {
            variant,
            weight_delta,
            exploration_delta,
        }
    }
}

/// Source of a suggested solution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SolutionSource {
    /// From predefined templates
    Template,

    /// From analysis of similar contexts
    PatternAnalysis,

    /// From user suggestion/feedback
    UserSuggestion,
}

impl SolutionSource {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Template => "template",
            Self::PatternAnalysis => "pattern_analysis",
            Self::UserSuggestion => "user_suggestion",
        }
    }
}

// ============================================================================
// Response Types (Task 4)
// ============================================================================

/// Action to take in response to novelty or gap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseAction {
    /// Just monitor, no action needed
    None,

    /// Adjust exploration rate
    AdjustExploration(f64),

    /// Create or update a capability gap
    CreateGap(CapabilityGap),

    /// Suggest a solution to the user
    SuggestSolution(SuggestedSolution),

    /// Notify the user via dashboard
    NotifyUser(String),
}

/// Stage of graduated response
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ResponseStage {
    /// Less than 3 observations - just monitor
    Monitor,

    /// 3-10 observations - cluster patterns
    Cluster,

    /// 10-25 observations - auto-adjust thresholds
    AutoAdjust,

    /// More than 25 observations - surface to user
    Surface,
}

impl ResponseStage {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Monitor => "monitor",
            Self::Cluster => "cluster",
            Self::AutoAdjust => "auto_adjust",
            Self::Surface => "surface",
        }
    }

    /// Determine response stage based on observation count
    pub fn from_count(count: u32) -> Self {
        match count {
            0..=2 => Self::Monitor,
            3..=9 => Self::Cluster,
            10..=24 => Self::AutoAdjust,
            _ => Self::Surface,
        }
    }
}

// ============================================================================
// Event Types (Task 4)
// ============================================================================

/// Events emitted by the open-world adaptation system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpenWorldEvent {
    /// A novel pattern was detected
    NoveltyDetected {
        /// The fingerprint of the novel pattern
        fingerprint: PatternFingerprint,
        /// Cluster assignment (if any)
        cluster: Option<ClusterId>,
    },

    /// An anomaly cluster was updated
    ClusterUpdated {
        /// The updated cluster
        cluster: AnomalyCluster,
    },

    /// A new capability gap was created
    GapCreated {
        /// The new gap
        gap: CapabilityGap,
    },

    /// A gap's status changed
    GapStatusChanged {
        /// ID of the gap
        gap_id: GapId,
        /// Previous status
        old: GapStatus,
        /// New status
        new: GapStatus,
    },

    /// Solutions were generated for a gap
    SolutionGenerated {
        /// ID of the gap
        gap_id: GapId,
        /// The generated solution
        solution: SuggestedSolution,
    },

    /// Strategy feedback was applied
    StrategyFeedback {
        /// ID of the learning affected
        learning_id: LearningId,
        /// The adjustment applied
        adjustment: f64,
    },
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_fingerprint_creation() {
        let fp = PatternFingerprint::new(12345, vec![0.1, 0.2, 0.3], "test context".to_string());

        assert_eq!(fp.hash, 12345);
        assert_eq!(fp.embedding.len(), 3);
        assert_eq!(fp.context_summary, "test context");
    }

    #[test]
    fn test_anomaly_cluster_operations() {
        let mut cluster = AnomalyCluster::new(vec![0.5, 0.5, 0.5]);

        assert_eq!(cluster.size(), 0);

        let fp = PatternFingerprint::new(1, vec![0.1], "member".to_string());
        cluster.add_member(fp);

        assert_eq!(cluster.size(), 1);
    }

    #[test]
    fn test_capability_gap_severity_escalation() {
        let mut gap = CapabilityGap::new(GapCategory::MissingKnowledge, "test pattern".to_string());

        assert_eq!(gap.severity, GapSeverity::Low);
        assert_eq!(gap.failure_count, 1);

        // Record more failures to escalate severity
        gap.record_failure();
        gap.record_failure();
        assert_eq!(gap.severity, GapSeverity::Medium);

        for _ in 0..8 {
            gap.record_failure();
        }
        assert_eq!(gap.severity, GapSeverity::High);
    }

    #[test]
    fn test_response_stage_from_count() {
        assert_eq!(ResponseStage::from_count(0), ResponseStage::Monitor);
        assert_eq!(ResponseStage::from_count(2), ResponseStage::Monitor);
        assert_eq!(ResponseStage::from_count(3), ResponseStage::Cluster);
        assert_eq!(ResponseStage::from_count(9), ResponseStage::Cluster);
        assert_eq!(ResponseStage::from_count(10), ResponseStage::AutoAdjust);
        assert_eq!(ResponseStage::from_count(24), ResponseStage::AutoAdjust);
        assert_eq!(ResponseStage::from_count(25), ResponseStage::Surface);
        assert_eq!(ResponseStage::from_count(100), ResponseStage::Surface);
    }

    #[test]
    fn test_gap_category_as_str() {
        assert_eq!(GapCategory::MissingKnowledge.as_str(), "missing_knowledge");
        assert_eq!(GapCategory::IncorrectPattern.as_str(), "incorrect_pattern");
        assert_eq!(GapCategory::ContextMismatch.as_str(), "context_mismatch");
        assert_eq!(GapCategory::ToolGap.as_str(), "tool_gap");
    }

    #[test]
    fn test_failure_type_as_str() {
        assert_eq!(
            FailureType::LearningNotActivated.as_str(),
            "learning_not_activated"
        );
        assert_eq!(
            FailureType::NegativeAttribution.as_str(),
            "negative_attribution"
        );
        assert_eq!(FailureType::LowConfidence.as_str(), "low_confidence");
        assert_eq!(FailureType::ExplicitFeedback.as_str(), "explicit_feedback");
    }

    #[test]
    fn test_solution_source_as_str() {
        assert_eq!(SolutionSource::Template.as_str(), "template");
        assert_eq!(SolutionSource::PatternAnalysis.as_str(), "pattern_analysis");
        assert_eq!(SolutionSource::UserSuggestion.as_str(), "user_suggestion");
    }

    #[test]
    fn test_suggested_solution_mark_applied() {
        let mut solution = SuggestedSolution::new(
            SolutionAction::DisableLearning { id: Uuid::nil() },
            SolutionSource::Template,
            0.9,
        );

        assert!(!solution.applied);
        solution.mark_applied();
        assert!(solution.applied);
    }

    #[test]
    fn test_novelty_result_serialization() {
        let result = NoveltyResult::Known {
            fingerprint: PatternFingerprint::new(1, vec![0.1], "test".to_string()),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: NoveltyResult = serde_json::from_str(&json).unwrap();

        if let NoveltyResult::Known { fingerprint } = deserialized {
            assert_eq!(fingerprint.hash, 1);
        } else {
            panic!("Expected Known variant");
        }
    }

    #[test]
    fn test_open_world_event_serialization() {
        let event = OpenWorldEvent::GapStatusChanged {
            gap_id: Uuid::now_v7(),
            old: GapStatus::Detected,
            new: GapStatus::Confirmed,
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: OpenWorldEvent = serde_json::from_str(&json).unwrap();

        if let OpenWorldEvent::GapStatusChanged { old, new, .. } = deserialized {
            assert_eq!(old, GapStatus::Detected);
            assert_eq!(new, GapStatus::Confirmed);
        } else {
            panic!("Expected GapStatusChanged variant");
        }
    }
}
