//! Command types for CQRS study lifecycle management.
//!
//! Commands represent intentions to change state. They are processed by
//! [`StudyManager`] which validates them and emits corresponding events.

use crate::metrics::LongitudinalMetrics;
use crate::study::{PeriodType, StudyConfig};
use crate::types::StudyId;

/// Command to create a new study.
#[derive(Debug, Clone)]
pub struct CreateStudy {
    /// Human-readable name for the study.
    pub name: String,

    /// Period type for checkpoints.
    pub period_type: PeriodType,

    /// Number of periods (e.g., 2 for "2 weeks").
    pub period_value: Option<u32>,

    /// Study configuration.
    pub config: StudyConfig,
}

/// Command to record a checkpoint in a study.
#[derive(Debug, Clone)]
pub struct RecordCheckpoint {
    /// The study this checkpoint belongs to.
    pub study_id: StudyId,

    /// Metrics collected for this checkpoint.
    pub metrics: LongitudinalMetrics,

    /// Number of events analyzed.
    pub events_analyzed: u64,

    /// Session IDs included in this checkpoint.
    pub sessions_included: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::TimePeriod;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn create_study_command_stores_all_fields() {
        let cmd = CreateStudy {
            name: "weekly-eval".to_string(),
            period_type: PeriodType::Weekly,
            period_value: Some(4),
            config: StudyConfig {
                description: Some("Test study".to_string()),
                ..Default::default()
            },
        };

        assert_eq!(cmd.name, "weekly-eval");
        assert_eq!(cmd.period_type, PeriodType::Weekly);
        assert_eq!(cmd.period_value, Some(4));
        assert!(cmd.config.description.is_some());
    }

    #[test]
    fn record_checkpoint_command_stores_all_fields() {
        let study_id = StudyId(Uuid::nil());
        let metrics = LongitudinalMetrics {
            period: TimePeriod {
                start: Utc::now(),
                end: Utc::now(),
            },
            sessions_completed: 5,
            ..Default::default()
        };

        let cmd = RecordCheckpoint {
            study_id,
            metrics: metrics.clone(),
            events_analyzed: 100,
            sessions_included: vec!["sess-1".to_string()],
        };

        assert_eq!(cmd.study_id, study_id);
        assert_eq!(cmd.events_analyzed, 100);
        assert_eq!(cmd.sessions_included.len(), 1);
    }
}
