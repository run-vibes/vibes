//! Study types for longitudinal evaluations.
//!
//! A study tracks AI performance over time using periodic checkpoints.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::metrics::LongitudinalMetrics;
use crate::types::{CheckpointId, StudyId};

/// Current status of a study.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StudyStatus {
    /// Created but not yet started
    Pending,
    /// Actively collecting data
    Running,
    /// Temporarily paused
    Paused,
    /// Finished (terminal state)
    Stopped,
}

impl StudyStatus {
    /// Convert to database string representation.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Stopped => "stopped",
        }
    }

    /// Parse from database string.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "running" => Some(Self::Running),
            "paused" => Some(Self::Paused),
            "stopped" => Some(Self::Stopped),
            _ => None,
        }
    }
}

/// Time period type for study duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeriodType {
    /// Hourly checkpoints
    Hourly,
    /// Daily checkpoints
    Daily,
    /// Weekly checkpoints
    Weekly,
    /// Monthly checkpoints
    Monthly,
}

impl PeriodType {
    /// Convert to database string representation.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hourly => "hourly",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
        }
    }

    /// Parse from database string.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "hourly" => Some(Self::Hourly),
            "daily" => Some(Self::Daily),
            "weekly" => Some(Self::Weekly),
            "monthly" => Some(Self::Monthly),
            _ => None,
        }
    }
}

/// Configuration options for a study.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StudyConfig {
    /// Optional description of the study
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Tags for categorization
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Arbitrary key-value metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

/// A longitudinal evaluation study.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Study {
    /// Unique identifier
    pub id: StudyId,

    /// Human-readable name
    pub name: String,

    /// Current status
    pub status: StudyStatus,

    /// Period type for checkpoints
    pub period_type: PeriodType,

    /// Number of periods (e.g., 2 weeks)
    pub period_value: Option<u32>,

    /// Study configuration
    pub config: StudyConfig,

    /// When the study was created
    pub created_at: DateTime<Utc>,

    /// When the study was started (None if pending)
    pub started_at: Option<DateTime<Utc>>,

    /// When the study was stopped (None if not stopped)
    pub stopped_at: Option<DateTime<Utc>>,
}

impl Study {
    /// Create a new pending study.
    #[must_use]
    pub fn new(
        id: StudyId,
        name: String,
        period_type: PeriodType,
        period_value: Option<u32>,
        config: StudyConfig,
    ) -> Self {
        Self {
            id,
            name,
            status: StudyStatus::Pending,
            period_type,
            period_value,
            config,
            created_at: Utc::now(),
            started_at: None,
            stopped_at: None,
        }
    }

    /// Check if the study is currently active (running or paused).
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(self.status, StudyStatus::Running | StudyStatus::Paused)
    }

    /// Check if the study is in a terminal state.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        self.status == StudyStatus::Stopped
    }
}

/// A checkpoint in a study containing metrics for a time period.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Unique identifier
    pub id: CheckpointId,

    /// Study this checkpoint belongs to
    pub study_id: StudyId,

    /// When the checkpoint was recorded
    pub timestamp: DateTime<Utc>,

    /// Metrics for this checkpoint period
    pub metrics: LongitudinalMetrics,

    /// Number of events analyzed for this checkpoint
    pub events_analyzed: u64,

    /// Session IDs included in this checkpoint
    pub sessions_included: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn sample_study_id() -> StudyId {
        StudyId(Uuid::nil())
    }

    // ==================== StudyStatus Tests ====================

    #[test]
    fn study_status_as_str_returns_correct_values() {
        assert_eq!(StudyStatus::Pending.as_str(), "pending");
        assert_eq!(StudyStatus::Running.as_str(), "running");
        assert_eq!(StudyStatus::Paused.as_str(), "paused");
        assert_eq!(StudyStatus::Stopped.as_str(), "stopped");
    }

    #[test]
    fn study_status_parse_returns_correct_variants() {
        assert_eq!(StudyStatus::parse("pending"), Some(StudyStatus::Pending));
        assert_eq!(StudyStatus::parse("running"), Some(StudyStatus::Running));
        assert_eq!(StudyStatus::parse("paused"), Some(StudyStatus::Paused));
        assert_eq!(StudyStatus::parse("stopped"), Some(StudyStatus::Stopped));
        assert_eq!(StudyStatus::parse("invalid"), None);
    }

    #[test]
    fn study_status_serialization_roundtrip() {
        for status in [
            StudyStatus::Pending,
            StudyStatus::Running,
            StudyStatus::Paused,
            StudyStatus::Stopped,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let parsed: StudyStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, parsed);
        }
    }

    // ==================== PeriodType Tests ====================

    #[test]
    fn period_type_as_str_returns_correct_values() {
        assert_eq!(PeriodType::Hourly.as_str(), "hourly");
        assert_eq!(PeriodType::Daily.as_str(), "daily");
        assert_eq!(PeriodType::Weekly.as_str(), "weekly");
        assert_eq!(PeriodType::Monthly.as_str(), "monthly");
    }

    #[test]
    fn period_type_parse_returns_correct_variants() {
        assert_eq!(PeriodType::parse("hourly"), Some(PeriodType::Hourly));
        assert_eq!(PeriodType::parse("daily"), Some(PeriodType::Daily));
        assert_eq!(PeriodType::parse("weekly"), Some(PeriodType::Weekly));
        assert_eq!(PeriodType::parse("monthly"), Some(PeriodType::Monthly));
        assert_eq!(PeriodType::parse("invalid"), None);
    }

    #[test]
    fn period_type_serialization_roundtrip() {
        for period in [
            PeriodType::Hourly,
            PeriodType::Daily,
            PeriodType::Weekly,
            PeriodType::Monthly,
        ] {
            let json = serde_json::to_string(&period).unwrap();
            let parsed: PeriodType = serde_json::from_str(&json).unwrap();
            assert_eq!(period, parsed);
        }
    }

    // ==================== StudyConfig Tests ====================

    #[test]
    fn study_config_default_is_empty() {
        let config = StudyConfig::default();
        assert!(config.description.is_none());
        assert!(config.tags.is_empty());
        assert!(config.metadata.is_empty());
    }

    #[test]
    fn study_config_serialization_roundtrip() {
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), "value".to_string());

        let config = StudyConfig {
            description: Some("A test study".to_string()),
            tags: vec!["test".to_string(), "eval".to_string()],
            metadata,
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: StudyConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config, parsed);
    }

    #[test]
    fn study_config_empty_fields_not_serialized() {
        let config = StudyConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, "{}");
    }

    // ==================== Study Tests ====================

    #[test]
    fn study_new_creates_pending_study() {
        let study = Study::new(
            sample_study_id(),
            "test-study".to_string(),
            PeriodType::Weekly,
            Some(2),
            StudyConfig::default(),
        );

        assert_eq!(study.status, StudyStatus::Pending);
        assert!(study.started_at.is_none());
        assert!(study.stopped_at.is_none());
    }

    #[test]
    fn study_is_active_for_running_status() {
        let mut study = Study::new(
            sample_study_id(),
            "test".to_string(),
            PeriodType::Daily,
            None,
            StudyConfig::default(),
        );
        study.status = StudyStatus::Running;

        assert!(study.is_active());
    }

    #[test]
    fn study_is_active_for_paused_status() {
        let mut study = Study::new(
            sample_study_id(),
            "test".to_string(),
            PeriodType::Daily,
            None,
            StudyConfig::default(),
        );
        study.status = StudyStatus::Paused;

        assert!(study.is_active());
    }

    #[test]
    fn study_is_not_active_for_pending_status() {
        let study = Study::new(
            sample_study_id(),
            "test".to_string(),
            PeriodType::Daily,
            None,
            StudyConfig::default(),
        );

        assert!(!study.is_active());
    }

    #[test]
    fn study_is_not_active_for_stopped_status() {
        let mut study = Study::new(
            sample_study_id(),
            "test".to_string(),
            PeriodType::Daily,
            None,
            StudyConfig::default(),
        );
        study.status = StudyStatus::Stopped;

        assert!(!study.is_active());
    }

    #[test]
    fn study_is_terminal_only_for_stopped_status() {
        let mut study = Study::new(
            sample_study_id(),
            "test".to_string(),
            PeriodType::Daily,
            None,
            StudyConfig::default(),
        );

        assert!(!study.is_terminal());

        study.status = StudyStatus::Running;
        assert!(!study.is_terminal());

        study.status = StudyStatus::Paused;
        assert!(!study.is_terminal());

        study.status = StudyStatus::Stopped;
        assert!(study.is_terminal());
    }

    #[test]
    fn study_serialization_roundtrip() {
        let study = Study::new(
            sample_study_id(),
            "test-study".to_string(),
            PeriodType::Weekly,
            Some(2),
            StudyConfig::default(),
        );

        let json = serde_json::to_string(&study).unwrap();
        let parsed: Study = serde_json::from_str(&json).unwrap();

        assert_eq!(study.id, parsed.id);
        assert_eq!(study.name, parsed.name);
        assert_eq!(study.status, parsed.status);
        assert_eq!(study.period_type, parsed.period_type);
    }

    // ==================== Checkpoint Tests ====================

    #[test]
    fn checkpoint_serialization_roundtrip() {
        let checkpoint = Checkpoint {
            id: crate::types::CheckpointId(Uuid::nil()),
            study_id: sample_study_id(),
            timestamp: Utc::now(),
            metrics: crate::metrics::LongitudinalMetrics::default(),
            events_analyzed: 500,
            sessions_included: vec!["sess-1".to_string()],
        };

        let json = serde_json::to_string(&checkpoint).unwrap();
        let parsed: Checkpoint = serde_json::from_str(&json).unwrap();

        assert_eq!(checkpoint.id, parsed.id);
        assert_eq!(checkpoint.study_id, parsed.study_id);
        assert_eq!(checkpoint.events_analyzed, parsed.events_analyzed);
    }
}
