//! Dashboard WebSocket types
//!
//! Types for the groove dashboard WebSocket API including topics,
//! messages, and data structures for each dashboard section.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{LearningCategory, LearningId, LearningStatus, Scope, strategy::InjectionStrategy};

/// Filter parameters for learnings queries
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct LearningsFilter {
    /// Filter by scope
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<Scope>,
    /// Filter by category
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<LearningCategory>,
    /// Filter by status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<LearningStatus>,
}

/// Time period for queries
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct Period {
    /// Number of days to include
    pub days: u32,
}

/// Dashboard WebSocket subscription topics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "topic", rename_all = "snake_case")]
pub enum DashboardTopic {
    /// Overview cards - all summary data
    Overview,
    /// Learnings list with filters
    Learnings {
        #[serde(default)]
        filters: LearningsFilter,
    },
    /// Single learning detail
    LearningDetail { id: LearningId },
    /// Attribution leaderboard and data
    Attribution {
        #[serde(default)]
        period: Period,
    },
    /// Session timeline for attribution view
    SessionTimeline {
        #[serde(default)]
        period: Period,
    },
    /// Strategy distributions by category
    StrategyDistributions,
    /// Learning-specific strategy overrides
    StrategyOverrides,
    /// System health metrics
    Health,
}

/// Server → Client messages
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DashboardMessage {
    /// Initial snapshot on subscribe
    Snapshot {
        topic: DashboardTopic,
        data: DashboardData,
    },
    /// Incremental update
    Update {
        topic: DashboardTopic,
        data: DashboardData,
    },
    /// Subscription confirmed
    Subscribed { topics: Vec<DashboardTopic> },
    /// Unsubscription confirmed
    Unsubscribed { topics: Vec<DashboardTopic> },
    /// Error response
    Error { message: String },
}

/// Client → Server messages
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DashboardRequest {
    /// Subscribe to topics
    Subscribe {
        topics: Vec<DashboardTopic>,
    },
    /// Unsubscribe from topics
    Unsubscribe {
        topics: Vec<DashboardTopic>,
    },
    /// Learning actions
    DisableLearning {
        id: LearningId,
    },
    EnableLearning {
        id: LearningId,
    },
    DeleteLearning {
        id: LearningId,
    },
}

/// Data payload for dashboard messages - one variant per topic
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "data_type", rename_all = "snake_case")]
pub enum DashboardData {
    Overview(OverviewData),
    Learnings(LearningsData),
    LearningDetail(LearningDetailData),
    Attribution(AttributionData),
    SessionTimeline(SessionTimelineData),
    StrategyDistributions(StrategyDistributionsData),
    StrategyOverrides(StrategyOverridesData),
    Health(HealthData),
}

// ============================================================
// Overview Data
// ============================================================

/// Overview page data - aggregated summaries
#[derive(Debug, Clone, Serialize, Default)]
pub struct OverviewData {
    pub trends: TrendSummary,
    pub learnings: LearningSummary,
    pub attribution: AttributionSummary,
    pub health: HealthSummary,
}

/// Session trend data for sparkline
#[derive(Debug, Clone, Serialize, Default)]
pub struct TrendSummary {
    /// Sparkline data points (scores over time)
    pub sparkline_data: Vec<f64>,
    /// Improvement percentage over period
    pub improvement_percent: f64,
    /// Trend direction indicator
    pub trend_direction: TrendDirection,
    /// Number of sessions in period
    pub session_count: u32,
    /// Period in days
    pub period_days: u32,
}

/// Trend direction indicator
#[derive(Debug, Clone, Copy, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    Rising,
    #[default]
    Stable,
    Falling,
}

/// Summary of learnings for overview card
#[derive(Debug, Clone, Serialize, Default)]
pub struct LearningSummary {
    /// Total number of learnings
    pub total: u32,
    /// Number of active learnings
    pub active: u32,
    /// Recent learnings (brief format)
    pub recent: Vec<LearningBrief>,
    /// Count by category
    pub by_category: HashMap<String, u32>,
}

/// Brief learning info for lists
#[derive(Debug, Clone, Serialize)]
pub struct LearningBrief {
    pub id: LearningId,
    pub content: String,
    pub category: LearningCategory,
    pub scope: Scope,
    pub status: LearningStatus,
    pub estimated_value: f64,
    pub created_at: DateTime<Utc>,
}

/// Attribution summary for overview card
#[derive(Debug, Clone, Serialize, Default)]
pub struct AttributionSummary {
    /// Top contributing learnings
    pub top_contributors: Vec<ContributorBrief>,
    /// Number of learnings under review
    pub under_review_count: u32,
    /// Number of learnings with negative impact
    pub negative_count: u32,
}

/// Brief contributor info
#[derive(Debug, Clone, Serialize)]
pub struct ContributorBrief {
    pub learning_id: LearningId,
    pub content: String,
    pub estimated_value: f64,
    pub confidence: f64,
}

/// Health summary for overview card
#[derive(Debug, Clone, Serialize, Default)]
pub struct HealthSummary {
    /// Overall system status
    pub overall_status: SystemStatus,
    /// Assessment coverage percentage
    pub assessment_coverage: f64,
    /// Ablation coverage percentage
    pub ablation_coverage: f64,
    /// Last activity timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_activity: Option<DateTime<Utc>>,
}

/// System status indicator
#[derive(Debug, Clone, Copy, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SystemStatus {
    #[default]
    Ok,
    Degraded,
    Error,
}

// ============================================================
// Learnings Data
// ============================================================

/// Learnings list data
#[derive(Debug, Clone, Serialize, Default)]
pub struct LearningsData {
    /// List of learnings
    pub learnings: Vec<LearningBrief>,
    /// Total count (for pagination)
    pub total: u32,
}

/// Detailed learning data for detail panel
#[derive(Debug, Clone, Serialize)]
pub struct LearningDetailData {
    pub id: LearningId,
    pub content: String,
    pub category: LearningCategory,
    pub scope: Scope,
    pub status: LearningStatus,
    /// Metrics
    pub estimated_value: f64,
    pub confidence: f64,
    pub times_injected: u32,
    pub activation_rate: f64,
    pub session_count: u32,
    /// Source information
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_session: Option<String>,
    pub extraction_method: String,
}

// ============================================================
// Attribution Data
// ============================================================

/// Attribution page data
#[derive(Debug, Clone, Serialize, Default)]
pub struct AttributionData {
    /// Top positive contributors
    pub top_contributors: Vec<AttributionEntry>,
    /// Negative impact learnings
    pub negative_impact: Vec<AttributionEntry>,
    /// Ablation coverage stats
    pub ablation_coverage: AblationCoverage,
}

/// Single attribution entry
#[derive(Debug, Clone, Serialize)]
pub struct AttributionEntry {
    pub learning_id: LearningId,
    pub content: String,
    pub estimated_value: f64,
    pub confidence: f64,
    pub session_count: u32,
    pub status: LearningStatus,
}

/// Ablation experiment coverage
#[derive(Debug, Clone, Serialize, Default)]
pub struct AblationCoverage {
    /// Percentage of learnings ablation tested
    pub coverage_percent: f64,
    /// Completed experiments
    pub completed: u32,
    /// In-progress experiments
    pub in_progress: u32,
    /// Pending experiments
    pub pending: u32,
}

/// Session timeline data
#[derive(Debug, Clone, Serialize, Default)]
pub struct SessionTimelineData {
    /// Sessions grouped by day
    pub sessions: Vec<SessionTimelineEntry>,
}

/// Single session in timeline
#[derive(Debug, Clone, Serialize)]
pub struct SessionTimelineEntry {
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub score: f64,
    pub learnings_activated: u32,
    /// Top contributing learnings
    pub contributions: Vec<SessionContribution>,
    /// Warning flags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

/// Learning contribution in a session
#[derive(Debug, Clone, Serialize)]
pub struct SessionContribution {
    pub learning_id: LearningId,
    pub content: String,
    pub contribution: f64,
}

// ============================================================
// Strategy Data
// ============================================================

/// Strategy distributions data
#[derive(Debug, Clone, Serialize, Default)]
pub struct StrategyDistributionsData {
    /// Distributions by category key
    pub distributions: Vec<CategoryDistribution>,
    /// Summary stats
    pub specialized_count: u32,
    pub total_learnings: u32,
}

/// Distribution for a single category
#[derive(Debug, Clone, Serialize)]
pub struct CategoryDistribution {
    /// Category identifier (e.g., "correction_interactive")
    pub category_key: String,
    /// Human-readable label
    pub label: String,
    /// Session count for this category
    pub session_count: u32,
    /// Weights by strategy variant
    pub weights: Vec<StrategyWeight>,
}

/// Weight for a single strategy variant
#[derive(Debug, Clone, Serialize)]
pub struct StrategyWeight {
    pub strategy: InjectionStrategy,
    pub weight: f64,
}

/// Strategy overrides data
#[derive(Debug, Clone, Serialize, Default)]
pub struct StrategyOverridesData {
    /// Learning-specific overrides
    pub overrides: Vec<LearningOverrideEntry>,
}

/// Single learning override entry
#[derive(Debug, Clone, Serialize)]
pub struct LearningOverrideEntry {
    pub learning_id: LearningId,
    pub content: String,
    pub session_count: u32,
    pub is_specialized: bool,
    /// Base category
    pub base_category: String,
    /// Override weights (if specialized)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_weights: Option<Vec<StrategyWeight>>,
    /// Sessions needed to specialize (if not specialized)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sessions_to_specialize: Option<u32>,
}

// ============================================================
// Health Data
// ============================================================

/// System health data
#[derive(Debug, Clone, Serialize, Default)]
pub struct HealthData {
    /// Overall status
    pub overall_status: SystemStatus,
    /// Component statuses
    pub assessment: ComponentHealth,
    pub extraction: ComponentHealth,
    pub attribution: ComponentHealth,
    /// Adaptive parameters
    pub adaptive_params: Vec<AdaptiveParamStatus>,
    /// Recent activity log
    pub recent_activity: Vec<ActivityEntry>,
}

/// Health status for a component
#[derive(Debug, Clone, Serialize, Default)]
pub struct ComponentHealth {
    pub status: SystemStatus,
    pub coverage: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_activity: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_count: Option<u32>,
}

/// Adaptive parameter status
#[derive(Debug, Clone, Serialize)]
pub struct AdaptiveParamStatus {
    pub name: String,
    pub current_value: f64,
    pub confidence: f64,
    pub trend: TrendDirection,
}

/// Recent activity entry
#[derive(Debug, Clone, Serialize)]
pub struct ActivityEntry {
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub activity_type: ActivityType,
}

/// Activity type for health log
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    Assessment,
    Extraction,
    Attribution,
    Strategy,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_topic_serializes_correctly() {
        let topic = DashboardTopic::Overview;
        let json = serde_json::to_string(&topic).unwrap();
        assert!(json.contains("overview"));

        let topic = DashboardTopic::Learnings {
            filters: LearningsFilter::default(),
        };
        let json = serde_json::to_string(&topic).unwrap();
        assert!(json.contains("learnings"));
    }

    #[test]
    fn dashboard_request_deserializes_subscribe() {
        let json = r#"{"type":"subscribe","topics":[{"topic":"overview"},{"topic":"health"}]}"#;
        let req: DashboardRequest = serde_json::from_str(json).unwrap();
        match req {
            DashboardRequest::Subscribe { topics } => {
                assert_eq!(topics.len(), 2);
                assert_eq!(topics[0], DashboardTopic::Overview);
                assert_eq!(topics[1], DashboardTopic::Health);
            }
            _ => panic!("Expected Subscribe"),
        }
    }

    #[test]
    fn dashboard_request_deserializes_learning_action() {
        let json = r#"{"type":"disable_learning","id":"01936f8a-1234-7000-8000-000000000001"}"#;
        let req: DashboardRequest = serde_json::from_str(json).unwrap();
        match req {
            DashboardRequest::DisableLearning { id } => {
                assert!(!id.is_nil());
            }
            _ => panic!("Expected DisableLearning"),
        }
    }

    #[test]
    fn dashboard_message_serializes_snapshot() {
        let msg = DashboardMessage::Snapshot {
            topic: DashboardTopic::Overview,
            data: DashboardData::Overview(OverviewData::default()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("snapshot"));
        assert!(json.contains("overview"));
    }

    #[test]
    fn learnings_filter_with_all_fields() {
        let json = r#"{"topic":"learnings","filters":{"scope":{"Project":"myproject"},"category":"Correction","status":"Active"}}"#;
        let topic: DashboardTopic = serde_json::from_str(json).unwrap();
        match topic {
            DashboardTopic::Learnings { filters } => {
                assert!(filters.scope.is_some());
                assert!(filters.category.is_some());
                assert!(filters.status.is_some());
            }
            _ => panic!("Expected Learnings topic"),
        }
    }
}
