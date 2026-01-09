//! HTTP/CLI API response types for assessment.
//!
//! These types are used for serializing assessment data to external consumers
//! (HTTP API responses, CLI output). They're intentionally separate from internal
//! domain types to:
//!
//! 1. **API Stability** - External interfaces can evolve independently
//! 2. **Clarity** - Clear boundary between internal processing and external representation
//! 3. **Flexibility** - Response types can include computed fields, omit internal details
//!
//! # Type Categories
//!
//! - **Status Types** - Current state of assessment system (`AssessmentStatusResponse`)
//! - **History Types** - Past assessment data (`AssessmentHistoryResponse`)
//! - **Stats Types** - Aggregated metrics (`AssessmentStatsResponse`)

use serde::{Deserialize, Serialize};

// ============================================================================
// Assessment Status Response Types
// ============================================================================

/// Assessment system status for HTTP API and CLI.
///
/// Provides a snapshot of the assessment system's current configuration
/// and activity state.
#[derive(Debug, Serialize, Deserialize)]
pub struct AssessmentStatusResponse {
    /// Circuit breaker configuration and state.
    pub circuit_breaker: CircuitBreakerStatus,
    /// Sampling configuration.
    pub sampling: SamplingStatus,
    /// Current activity metrics.
    pub activity: ActivityStatus,
}

/// Circuit breaker configuration status.
///
/// Shows the current circuit breaker settings that control intervention throttling.
#[derive(Debug, Serialize, Deserialize)]
pub struct CircuitBreakerStatus {
    /// Whether the circuit breaker is enabled.
    pub enabled: bool,
    /// Cooldown period in seconds after an intervention.
    pub cooldown_seconds: u32,
    /// Maximum interventions allowed per session.
    pub max_interventions_per_session: u32,
}

/// Sampling configuration status.
///
/// Shows the current sampling settings for assessment collection.
#[derive(Debug, Serialize, Deserialize)]
pub struct SamplingStatus {
    /// Base rate for sampling assessments (0.0 to 1.0).
    pub base_rate: f64,
    /// Number of sessions before sampling applies.
    pub burnin_sessions: u32,
}

/// Current assessment activity metrics.
///
/// Provides real-time view of assessment system activity.
#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityStatus {
    /// Number of currently active sessions being assessed.
    pub active_sessions: usize,
    /// Total number of assessment events stored.
    pub events_stored: usize,
    /// List of active session IDs.
    pub sessions: Vec<String>,
    /// Total number of interventions triggered across all sessions.
    #[serde(default)]
    pub intervention_count: u32,
}

// ============================================================================
// Assessment History Response Types
// ============================================================================

/// Assessment history response for HTTP API and CLI.
///
/// Returns paginated session history with assessment summaries.
#[derive(Debug, Serialize, Deserialize)]
pub struct AssessmentHistoryResponse {
    /// List of session summaries with assessment data.
    pub sessions: Vec<SessionHistoryItem>,
    /// Whether there are more results beyond this page.
    pub has_more: bool,
    /// Current page number (1-indexed).
    pub page: usize,
    /// Number of items per page.
    pub per_page: usize,
    /// Total number of sessions.
    pub total: usize,
    /// Total number of pages.
    pub total_pages: usize,
}

/// Summary of a single session's assessment history.
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionHistoryItem {
    /// The session identifier.
    pub session_id: String,
    /// Number of assessment events for this session.
    pub event_count: usize,
    /// Types of assessments recorded (e.g., "lightweight", "medium", "heavy").
    pub result_types: Vec<String>,
}

// ============================================================================
// Assessment Statistics Response Types
// ============================================================================

/// Assessment statistics response for HTTP API and CLI.
///
/// Provides aggregated metrics about assessment distribution and activity.
#[derive(Debug, Serialize, Deserialize)]
pub struct AssessmentStatsResponse {
    /// Distribution of assessments by tier.
    pub tier_distribution: TierDistribution,
    /// Total number of assessments across all tiers.
    pub total_assessments: usize,
    /// Sessions with highest assessment activity.
    pub top_sessions: Vec<SessionStats>,
}

/// Count of assessments by tier.
///
/// Shows how assessments are distributed across the three-tier system.
#[derive(Debug, Serialize, Deserialize)]
pub struct TierDistribution {
    /// Number of lightweight (per-message) assessments.
    pub lightweight: usize,
    /// Number of medium (checkpoint) assessments.
    pub medium: usize,
    /// Number of heavy (full-session) assessments.
    pub heavy: usize,
    /// Number of checkpoint events (separate from medium for backwards compat).
    pub checkpoint: usize,
}

/// Session with assessment count.
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStats {
    /// The session identifier.
    pub session_id: String,
    /// Total number of assessments for this session.
    pub assessment_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assessment_status_response_serialization_roundtrip() {
        let response = AssessmentStatusResponse {
            circuit_breaker: CircuitBreakerStatus {
                enabled: true,
                cooldown_seconds: 120,
                max_interventions_per_session: 3,
            },
            sampling: SamplingStatus {
                base_rate: 0.2,
                burnin_sessions: 10,
            },
            activity: ActivityStatus {
                active_sessions: 2,
                events_stored: 100,
                sessions: vec!["sess-1".to_string(), "sess-2".to_string()],
                intervention_count: 0,
            },
        };

        let json = serde_json::to_string(&response).expect("should serialize");
        let parsed: AssessmentStatusResponse =
            serde_json::from_str(&json).expect("should deserialize");

        assert!(parsed.circuit_breaker.enabled);
        assert_eq!(parsed.circuit_breaker.cooldown_seconds, 120);
        assert_eq!(parsed.sampling.base_rate, 0.2);
        assert_eq!(parsed.activity.active_sessions, 2);
    }

    #[test]
    fn assessment_history_response_serialization_roundtrip() {
        let response = AssessmentHistoryResponse {
            sessions: vec![
                SessionHistoryItem {
                    session_id: "sess-123".to_string(),
                    event_count: 15,
                    result_types: vec!["lightweight".to_string(), "medium".to_string()],
                },
                SessionHistoryItem {
                    session_id: "sess-456".to_string(),
                    event_count: 8,
                    result_types: vec!["lightweight".to_string()],
                },
            ],
            has_more: true,
            page: 1,
            per_page: 20,
            total: 2,
            total_pages: 1,
        };

        let json = serde_json::to_string(&response).expect("should serialize");
        let parsed: AssessmentHistoryResponse =
            serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(parsed.sessions.len(), 2);
        assert_eq!(parsed.sessions[0].session_id, "sess-123");
        assert_eq!(parsed.sessions[0].event_count, 15);
        assert!(parsed.has_more);
    }

    #[test]
    fn assessment_stats_response_serialization_roundtrip() {
        let response = AssessmentStatsResponse {
            tier_distribution: TierDistribution {
                lightweight: 1000,
                medium: 100,
                heavy: 10,
                checkpoint: 50,
            },
            total_assessments: 1160,
            top_sessions: vec![SessionStats {
                session_id: "sess-top".to_string(),
                assessment_count: 250,
            }],
        };

        let json = serde_json::to_string(&response).expect("should serialize");
        let parsed: AssessmentStatsResponse =
            serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(parsed.tier_distribution.lightweight, 1000);
        assert_eq!(parsed.tier_distribution.medium, 100);
        assert_eq!(parsed.tier_distribution.heavy, 10);
        assert_eq!(parsed.total_assessments, 1160);
        assert_eq!(parsed.top_sessions.len(), 1);
    }

    #[test]
    fn tier_distribution_default_values() {
        let json = r#"{"lightweight":0,"medium":0,"heavy":0,"checkpoint":0}"#;
        let parsed: TierDistribution = serde_json::from_str(json).expect("should deserialize");

        assert_eq!(parsed.lightweight, 0);
        assert_eq!(parsed.medium, 0);
        assert_eq!(parsed.heavy, 0);
        assert_eq!(parsed.checkpoint, 0);
    }

    #[test]
    fn assessment_history_response_includes_pagination_metadata() {
        let response = AssessmentHistoryResponse {
            sessions: vec![SessionHistoryItem {
                session_id: "sess-123".to_string(),
                event_count: 15,
                result_types: vec!["lightweight".to_string()],
            }],
            has_more: true,
            page: 2,
            per_page: 20,
            total: 45,
            total_pages: 3,
        };

        let json = serde_json::to_string(&response).expect("should serialize");
        let parsed: AssessmentHistoryResponse =
            serde_json::from_str(&json).expect("should deserialize");

        assert_eq!(parsed.page, 2);
        assert_eq!(parsed.per_page, 20);
        assert_eq!(parsed.total, 45);
        assert_eq!(parsed.total_pages, 3);
        assert!(parsed.has_more);
    }
}
