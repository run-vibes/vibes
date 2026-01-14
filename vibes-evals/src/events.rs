//! Event types for the evaluation system.
//!
//! These events are the source of truth for study lifecycle and checkpoint data.
//! Events are appended to the Iggy "evals" stream and projected to Turso for queries.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vibes_iggy::Partitionable;

use crate::metrics::LongitudinalMetrics;
use crate::study::{PeriodType, StudyConfig};
use crate::types::{CheckpointId, StudyId};

/// Events for the evaluation system.
///
/// All state changes are captured as events and stored in Iggy.
/// The Turso projection is derived by replaying these events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EvalEvent {
    // Study lifecycle events
    /// A new study was created (but not yet started)
    StudyCreated {
        id: StudyId,
        name: String,
        period_type: PeriodType,
        period_value: Option<u32>,
        config: StudyConfig,
    },

    /// Study was started
    StudyStarted { id: StudyId },

    /// Study was paused
    StudyPaused { id: StudyId },

    /// Study was resumed from paused state
    StudyResumed { id: StudyId },

    /// Study was stopped (terminal state)
    StudyStopped { id: StudyId },

    // Data capture events
    /// A checkpoint was recorded with metrics
    CheckpointRecorded {
        id: CheckpointId,
        study_id: StudyId,
        timestamp: DateTime<Utc>,
        metrics: LongitudinalMetrics,
        events_analyzed: u64,
        sessions_included: Vec<String>,
    },
}

impl EvalEvent {
    /// Extract the study ID from this event.
    #[must_use]
    pub fn study_id(&self) -> StudyId {
        match self {
            EvalEvent::StudyCreated { id, .. } => *id,
            EvalEvent::StudyStarted { id } => *id,
            EvalEvent::StudyPaused { id } => *id,
            EvalEvent::StudyResumed { id } => *id,
            EvalEvent::StudyStopped { id } => *id,
            EvalEvent::CheckpointRecorded { study_id, .. } => *study_id,
        }
    }
}

impl Partitionable for EvalEvent {
    fn partition_key(&self) -> Option<&str> {
        // Events are partitioned by study ID for ordered processing per study
        None // TODO: Return study ID string when we add a method for that
    }
}

/// An EvalEvent with a globally unique UUIDv7 identifier.
///
/// This wrapper is used for storage in the EventLog. The event_id provides
/// a globally unique, time-ordered identifier that works across Iggy partitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredEvalEvent {
    /// Globally unique, time-ordered event identifier (UUIDv7)
    pub event_id: Uuid,
    /// The event payload
    pub event: EvalEvent,
}

impl StoredEvalEvent {
    /// Create a new StoredEvalEvent with a fresh UUIDv7 identifier.
    #[must_use]
    pub fn new(event: EvalEvent) -> Self {
        Self {
            event_id: Uuid::now_v7(),
            event,
        }
    }

    /// Get the study ID from the inner event.
    #[must_use]
    pub fn study_id(&self) -> StudyId {
        self.event.study_id()
    }
}

impl Partitionable for StoredEvalEvent {
    fn partition_key(&self) -> Option<&str> {
        self.event.partition_key()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::TimePeriod;
    use chrono::TimeZone;

    fn sample_study_id() -> StudyId {
        StudyId(Uuid::nil())
    }

    fn sample_checkpoint_id() -> CheckpointId {
        CheckpointId(Uuid::nil())
    }

    #[test]
    fn eval_event_study_created_serialization_roundtrip() {
        let event = EvalEvent::StudyCreated {
            id: sample_study_id(),
            name: "test-study".to_string(),
            period_type: PeriodType::Weekly,
            period_value: Some(2),
            config: StudyConfig::default(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: EvalEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event, parsed);
    }

    #[test]
    fn eval_event_study_started_serialization_roundtrip() {
        let event = EvalEvent::StudyStarted {
            id: sample_study_id(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: EvalEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event, parsed);
    }

    #[test]
    fn eval_event_study_paused_serialization_roundtrip() {
        let event = EvalEvent::StudyPaused {
            id: sample_study_id(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: EvalEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event, parsed);
    }

    #[test]
    fn eval_event_study_resumed_serialization_roundtrip() {
        let event = EvalEvent::StudyResumed {
            id: sample_study_id(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: EvalEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event, parsed);
    }

    #[test]
    fn eval_event_study_stopped_serialization_roundtrip() {
        let event = EvalEvent::StudyStopped {
            id: sample_study_id(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: EvalEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event, parsed);
    }

    #[test]
    fn eval_event_checkpoint_recorded_serialization_roundtrip() {
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap();

        let metrics = LongitudinalMetrics {
            period: TimePeriod { start, end },
            sessions_completed: 10,
            ..Default::default()
        };

        let event = EvalEvent::CheckpointRecorded {
            id: sample_checkpoint_id(),
            study_id: sample_study_id(),
            timestamp: Utc::now(),
            metrics,
            events_analyzed: 1000,
            sessions_included: vec!["sess-1".to_string(), "sess-2".to_string()],
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: EvalEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event, parsed);
    }

    #[test]
    fn eval_event_study_id_returns_correct_id_for_all_variants() {
        let study_id = sample_study_id();

        let events = vec![
            EvalEvent::StudyCreated {
                id: study_id,
                name: "test".to_string(),
                period_type: PeriodType::Daily,
                period_value: None,
                config: StudyConfig::default(),
            },
            EvalEvent::StudyStarted { id: study_id },
            EvalEvent::StudyPaused { id: study_id },
            EvalEvent::StudyResumed { id: study_id },
            EvalEvent::StudyStopped { id: study_id },
            EvalEvent::CheckpointRecorded {
                id: sample_checkpoint_id(),
                study_id,
                timestamp: Utc::now(),
                metrics: LongitudinalMetrics::default(),
                events_analyzed: 0,
                sessions_included: vec![],
            },
        ];

        for event in events {
            assert_eq!(event.study_id(), study_id);
        }
    }

    #[test]
    fn stored_eval_event_new_generates_uuidv7() {
        let event = EvalEvent::StudyStarted {
            id: sample_study_id(),
        };
        let stored = StoredEvalEvent::new(event);

        // UUIDv7 has version 7 in bits 48-51
        assert_eq!(stored.event_id.get_version_num(), 7);
    }

    #[test]
    fn stored_eval_event_ids_are_unique() {
        let event = EvalEvent::StudyStarted {
            id: sample_study_id(),
        };
        let stored1 = StoredEvalEvent::new(event.clone());
        let stored2 = StoredEvalEvent::new(event);

        assert_ne!(stored1.event_id, stored2.event_id);
    }

    #[test]
    fn stored_eval_event_serialization_roundtrip() {
        let event = EvalEvent::StudyCreated {
            id: sample_study_id(),
            name: "test-study".to_string(),
            period_type: PeriodType::Monthly,
            period_value: Some(1),
            config: StudyConfig::default(),
        };
        let stored = StoredEvalEvent::new(event);

        let json = serde_json::to_string(&stored).unwrap();
        let parsed: StoredEvalEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(stored.event_id, parsed.event_id);
        assert_eq!(stored.event, parsed.event);
    }

    #[test]
    fn stored_eval_event_study_id_delegates_to_inner() {
        let study_id = sample_study_id();
        let event = EvalEvent::StudyStarted { id: study_id };
        let stored = StoredEvalEvent::new(event);

        assert_eq!(stored.study_id(), study_id);
    }
}
