//! Attribution types for tracking learning value over time
//!
//! This module defines the core types used by the attribution engine to
//! track how learnings influence sessions and their measured value.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::assessment::SessionId;
use crate::types::LearningId;

/// Per-session attribution record stored in CozoDB
///
/// Captures the output of all attribution layers for a single
/// learning-session pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionRecord {
    pub learning_id: LearningId,
    pub session_id: SessionId,
    pub timestamp: DateTime<Utc>,

    // Layer 1: Activation detection
    pub was_activated: bool,
    pub activation_confidence: f64,
    pub activation_signals: Vec<ActivationSignal>,

    // Layer 2: Temporal correlation
    pub temporal_positive: f64,
    pub temporal_negative: f64,
    pub net_temporal: f64,

    // Layer 3: Ablation
    pub was_withheld: bool,

    // Final attribution
    pub session_outcome: f64,
    pub attributed_value: f64,
}

/// Signal indicating how a learning was activated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActivationSignal {
    /// Learning content matched response via embedding similarity
    EmbeddingSimilarity { score: f64, message_idx: u32 },
    /// Learning was explicitly referenced by pattern
    ExplicitReference { pattern: String, message_idx: u32 },
}

/// Aggregated lifetime value for a learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningValue {
    pub learning_id: LearningId,
    pub estimated_value: f64,
    pub confidence: f64,
    pub session_count: u32,
    pub activation_rate: f64,

    // Per-source breakdown
    pub temporal_value: f64,
    pub temporal_confidence: f64,
    pub ablation_value: Option<f64>,
    pub ablation_confidence: Option<f64>,

    pub status: LearningStatus,
    pub updated_at: DateTime<Utc>,
}

/// Status of a learning in the attribution system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LearningStatus {
    /// Learning is actively being used
    Active,
    /// Learning has been deprecated (auto or manual)
    Deprecated { reason: String },
    /// Learning is under evaluation
    Experimental,
}

impl LearningStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Deprecated { .. } => "deprecated",
            Self::Experimental => "experimental",
        }
    }

    pub fn from_str_with_reason(s: &str, reason: Option<String>) -> Self {
        match s {
            "active" => Self::Active,
            "deprecated" => Self::Deprecated {
                reason: reason.unwrap_or_default(),
            },
            "experimental" => Self::Experimental,
            _ => Self::Active, // Default fallback
        }
    }
}

/// Tracks an A/B experiment for a learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AblationExperiment {
    pub learning_id: LearningId,
    pub started_at: DateTime<Utc>,
    pub sessions_with: Vec<SessionOutcome>,
    pub sessions_without: Vec<SessionOutcome>,
    pub result: Option<AblationResult>,
}

/// Outcome of a session for ablation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionOutcome {
    pub session_id: SessionId,
    pub outcome: f64,
    pub timestamp: DateTime<Utc>,
}

/// Result of an ablation experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AblationResult {
    pub marginal_value: f64,
    pub confidence: f64,
    pub is_significant: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_learning_status_roundtrip() {
        assert_eq!(LearningStatus::Active.as_str(), "active");
        assert_eq!(
            LearningStatus::Deprecated {
                reason: "low value".into()
            }
            .as_str(),
            "deprecated"
        );
        assert_eq!(LearningStatus::Experimental.as_str(), "experimental");

        assert_eq!(
            LearningStatus::from_str_with_reason("active", None),
            LearningStatus::Active
        );
        assert_eq!(
            LearningStatus::from_str_with_reason("deprecated", Some("test".into())),
            LearningStatus::Deprecated {
                reason: "test".into()
            }
        );
    }

    #[test]
    fn test_activation_signal_serialization() {
        let signal = ActivationSignal::EmbeddingSimilarity {
            score: 0.85,
            message_idx: 5,
        };
        let json = serde_json::to_string(&signal).unwrap();
        let parsed: ActivationSignal = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, signal);

        let signal = ActivationSignal::ExplicitReference {
            pattern: "use Result".into(),
            message_idx: 3,
        };
        let json = serde_json::to_string(&signal).unwrap();
        let parsed: ActivationSignal = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, signal);
    }

    #[test]
    fn test_attribution_record_creation() {
        let record = AttributionRecord {
            learning_id: Uuid::now_v7(),
            session_id: SessionId::from("test-session"),
            timestamp: Utc::now(),
            was_activated: true,
            activation_confidence: 0.9,
            activation_signals: vec![ActivationSignal::EmbeddingSimilarity {
                score: 0.85,
                message_idx: 2,
            }],
            temporal_positive: 0.7,
            temporal_negative: 0.1,
            net_temporal: 0.6,
            was_withheld: false,
            session_outcome: 0.8,
            attributed_value: 0.72,
        };

        assert!(record.was_activated);
        assert_eq!(record.activation_signals.len(), 1);
    }

    #[test]
    fn test_learning_value_creation() {
        let value = LearningValue {
            learning_id: Uuid::now_v7(),
            estimated_value: 0.65,
            confidence: 0.8,
            session_count: 10,
            activation_rate: 0.4,
            temporal_value: 0.6,
            temporal_confidence: 0.75,
            ablation_value: Some(0.7),
            ablation_confidence: Some(0.85),
            status: LearningStatus::Active,
            updated_at: Utc::now(),
        };

        assert_eq!(value.session_count, 10);
        assert!(value.ablation_value.is_some());
    }
}
