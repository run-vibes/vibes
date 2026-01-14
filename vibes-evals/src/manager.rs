//! Study lifecycle management with CQRS pattern.
//!
//! [`StudyManager`] coordinates commands (event emission) and queries (projection reads).

use std::sync::Arc;

use chrono::Utc;
use vibes_iggy::EventLog;

use crate::commands::{CreateStudy, RecordCheckpoint};
use crate::events::{EvalEvent, StoredEvalEvent};
use crate::storage::{EvalStorage, Result};
use crate::study::{Checkpoint, Study};
use crate::types::{CheckpointId, StudyId};

/// Manages study lifecycle using CQRS pattern.
///
/// Commands emit events to the event log (write side).
/// Queries read from the projection (read side).
pub struct StudyManager {
    event_log: Arc<dyn EventLog<StoredEvalEvent>>,
    storage: Arc<dyn EvalStorage>,
}

impl StudyManager {
    /// Create a new study manager.
    pub fn new(
        event_log: Arc<dyn EventLog<StoredEvalEvent>>,
        storage: Arc<dyn EvalStorage>,
    ) -> Self {
        Self { event_log, storage }
    }

    // === Commands (emit events) ===

    /// Create a new study.
    ///
    /// Emits a `StudyCreated` event and returns the new study ID.
    pub async fn create_study(&self, cmd: CreateStudy) -> Result<StudyId> {
        let id = StudyId::new();
        let event = StoredEvalEvent::new(EvalEvent::StudyCreated {
            id,
            name: cmd.name,
            period_type: cmd.period_type,
            period_value: cmd.period_value,
            config: cmd.config,
        });
        self.event_log.append(event).await?;
        Ok(id)
    }

    /// Start a pending study.
    ///
    /// Emits a `StudyStarted` event.
    pub async fn start_study(&self, id: StudyId) -> Result<()> {
        let event = StoredEvalEvent::new(EvalEvent::StudyStarted { id });
        self.event_log.append(event).await?;
        Ok(())
    }

    /// Pause a running study.
    ///
    /// Emits a `StudyPaused` event.
    pub async fn pause_study(&self, id: StudyId) -> Result<()> {
        let event = StoredEvalEvent::new(EvalEvent::StudyPaused { id });
        self.event_log.append(event).await?;
        Ok(())
    }

    /// Resume a paused study.
    ///
    /// Emits a `StudyResumed` event.
    pub async fn resume_study(&self, id: StudyId) -> Result<()> {
        let event = StoredEvalEvent::new(EvalEvent::StudyResumed { id });
        self.event_log.append(event).await?;
        Ok(())
    }

    /// Stop a study (terminal state).
    ///
    /// Emits a `StudyStopped` event.
    pub async fn stop_study(&self, id: StudyId) -> Result<()> {
        let event = StoredEvalEvent::new(EvalEvent::StudyStopped { id });
        self.event_log.append(event).await?;
        Ok(())
    }

    /// Record a checkpoint with metrics.
    ///
    /// Emits a `CheckpointRecorded` event.
    pub async fn record_checkpoint(&self, cmd: RecordCheckpoint) -> Result<CheckpointId> {
        let id = CheckpointId::new();
        let event = StoredEvalEvent::new(EvalEvent::CheckpointRecorded {
            id,
            study_id: cmd.study_id,
            timestamp: Utc::now(),
            metrics: cmd.metrics,
            events_analyzed: cmd.events_analyzed,
            sessions_included: cmd.sessions_included,
        });
        self.event_log.append(event).await?;
        Ok(id)
    }

    // === Queries (read from projection) ===

    /// Get a study by ID.
    pub async fn get_study(&self, id: StudyId) -> Result<Option<Study>> {
        self.storage.get_study(id).await
    }

    /// List all studies.
    pub async fn list_studies(&self) -> Result<Vec<Study>> {
        self.storage.list_studies().await
    }

    /// Get all checkpoints for a study.
    pub async fn get_checkpoints(&self, study_id: StudyId) -> Result<Vec<Checkpoint>> {
        self.storage.get_checkpoints(study_id).await
    }

    /// Get the latest checkpoint for a study.
    pub async fn get_latest_checkpoint(&self, study_id: StudyId) -> Result<Option<Checkpoint>> {
        self.storage.get_latest_checkpoint(study_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EvalEvent;
    use crate::metrics::LongitudinalMetrics;
    use crate::storage::TursoEvalStorage;
    use crate::study::{PeriodType, StudyConfig};
    use std::sync::Mutex;
    use vibes_iggy::traits::{EventConsumer, Offset};

    /// In-memory event log for testing that captures emitted events.
    struct TestEventLog {
        events: Mutex<Vec<StoredEvalEvent>>,
    }

    impl TestEventLog {
        fn new() -> Self {
            Self {
                events: Mutex::new(Vec::new()),
            }
        }

        fn events(&self) -> Vec<StoredEvalEvent> {
            self.events.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl EventLog<StoredEvalEvent> for TestEventLog {
        async fn append(&self, event: StoredEvalEvent) -> vibes_iggy::error::Result<Offset> {
            let mut events = self.events.lock().unwrap();
            let offset = events.len() as Offset;
            events.push(event);
            Ok(offset)
        }

        async fn append_batch(
            &self,
            events: Vec<StoredEvalEvent>,
        ) -> vibes_iggy::error::Result<Offset> {
            let mut stored = self.events.lock().unwrap();
            let start = stored.len() as Offset;
            stored.extend(events);
            Ok(start + stored.len() as Offset - 1)
        }

        async fn consumer(
            &self,
            _group: &str,
        ) -> vibes_iggy::error::Result<Box<dyn EventConsumer<StoredEvalEvent>>> {
            unimplemented!("not needed for manager tests")
        }

        fn high_water_mark(&self) -> Offset {
            self.events.lock().unwrap().len() as Offset
        }
    }

    async fn create_test_manager() -> (StudyManager, Arc<TestEventLog>, Arc<TursoEvalStorage>) {
        let event_log = Arc::new(TestEventLog::new());
        let storage = Arc::new(TursoEvalStorage::new_memory().await.unwrap());
        let manager = StudyManager::new(event_log.clone(), storage.clone());
        (manager, event_log, storage)
    }

    // === Command Tests ===

    #[tokio::test]
    async fn create_study_emits_study_created_event() {
        let (manager, event_log, _storage) = create_test_manager().await;

        let cmd = CreateStudy {
            name: "test-study".to_string(),
            period_type: PeriodType::Weekly,
            period_value: Some(2),
            config: StudyConfig::default(),
        };

        let study_id = manager.create_study(cmd).await.unwrap();

        // Verify event was emitted
        let events = event_log.events();
        assert_eq!(events.len(), 1);

        match &events[0].event {
            EvalEvent::StudyCreated {
                id,
                name,
                period_type,
                period_value,
                ..
            } => {
                assert_eq!(*id, study_id);
                assert_eq!(name, "test-study");
                assert_eq!(*period_type, PeriodType::Weekly);
                assert_eq!(*period_value, Some(2));
            }
            _ => panic!("expected StudyCreated event"),
        }
    }

    #[tokio::test]
    async fn start_study_emits_study_started_event() {
        let (manager, event_log, _storage) = create_test_manager().await;

        // First create a study
        let study_id = manager
            .create_study(CreateStudy {
                name: "test".to_string(),
                period_type: PeriodType::Daily,
                period_value: None,
                config: StudyConfig::default(),
            })
            .await
            .unwrap();

        // Then start it
        manager.start_study(study_id).await.unwrap();

        let events = event_log.events();
        assert_eq!(events.len(), 2);

        match &events[1].event {
            EvalEvent::StudyStarted { id } => {
                assert_eq!(*id, study_id);
            }
            _ => panic!("expected StudyStarted event"),
        }
    }

    #[tokio::test]
    async fn pause_study_emits_study_paused_event() {
        let (manager, event_log, _storage) = create_test_manager().await;

        let study_id = manager
            .create_study(CreateStudy {
                name: "test".to_string(),
                period_type: PeriodType::Daily,
                period_value: None,
                config: StudyConfig::default(),
            })
            .await
            .unwrap();
        manager.start_study(study_id).await.unwrap();

        manager.pause_study(study_id).await.unwrap();

        let events = event_log.events();
        assert_eq!(events.len(), 3);
        assert!(matches!(events[2].event, EvalEvent::StudyPaused { .. }));
    }

    #[tokio::test]
    async fn resume_study_emits_study_resumed_event() {
        let (manager, event_log, _storage) = create_test_manager().await;

        let study_id = manager
            .create_study(CreateStudy {
                name: "test".to_string(),
                period_type: PeriodType::Daily,
                period_value: None,
                config: StudyConfig::default(),
            })
            .await
            .unwrap();
        manager.start_study(study_id).await.unwrap();
        manager.pause_study(study_id).await.unwrap();

        manager.resume_study(study_id).await.unwrap();

        let events = event_log.events();
        assert_eq!(events.len(), 4);
        assert!(matches!(events[3].event, EvalEvent::StudyResumed { .. }));
    }

    #[tokio::test]
    async fn stop_study_emits_study_stopped_event() {
        let (manager, event_log, _storage) = create_test_manager().await;

        let study_id = manager
            .create_study(CreateStudy {
                name: "test".to_string(),
                period_type: PeriodType::Daily,
                period_value: None,
                config: StudyConfig::default(),
            })
            .await
            .unwrap();
        manager.start_study(study_id).await.unwrap();

        manager.stop_study(study_id).await.unwrap();

        let events = event_log.events();
        assert_eq!(events.len(), 3);
        assert!(matches!(events[2].event, EvalEvent::StudyStopped { .. }));
    }

    #[tokio::test]
    async fn record_checkpoint_emits_checkpoint_recorded_event() {
        let (manager, event_log, _storage) = create_test_manager().await;

        let study_id = manager
            .create_study(CreateStudy {
                name: "test".to_string(),
                period_type: PeriodType::Daily,
                period_value: None,
                config: StudyConfig::default(),
            })
            .await
            .unwrap();

        let cmd = RecordCheckpoint {
            study_id,
            metrics: LongitudinalMetrics::default(),
            events_analyzed: 500,
            sessions_included: vec!["sess-1".to_string()],
        };

        let checkpoint_id = manager.record_checkpoint(cmd).await.unwrap();

        let events = event_log.events();
        assert_eq!(events.len(), 2);

        match &events[1].event {
            EvalEvent::CheckpointRecorded {
                id,
                study_id: sid,
                events_analyzed,
                ..
            } => {
                assert_eq!(*id, checkpoint_id);
                assert_eq!(*sid, study_id);
                assert_eq!(*events_analyzed, 500);
            }
            _ => panic!("expected CheckpointRecorded event"),
        }
    }
}
