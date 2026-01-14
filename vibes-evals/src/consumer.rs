//! Event projection consumer for keeping the read model in sync.
//!
//! The consumer reads events from the event log and applies them to the
//! projection (Turso database) to maintain the read-side state.

use std::sync::Arc;
use std::time::Duration;

use tracing::{debug, info, instrument};
use vibes_iggy::EventLog;
use vibes_iggy::traits::SeekPosition;

use crate::events::StoredEvalEvent;
use crate::storage::{EvalProjection, Result};

/// Consumes events from the event log and applies them to the projection.
///
/// This keeps the Turso read model in sync with the event log source of truth.
pub struct EvalProjectionConsumer {
    event_log: Arc<dyn EventLog<StoredEvalEvent>>,
    projection: Arc<dyn EvalProjection>,
    consumer_group: String,
}

impl EvalProjectionConsumer {
    /// Create a new projection consumer.
    pub fn new(
        event_log: Arc<dyn EventLog<StoredEvalEvent>>,
        projection: Arc<dyn EvalProjection>,
        consumer_group: impl Into<String>,
    ) -> Self {
        Self {
            event_log,
            projection,
            consumer_group: consumer_group.into(),
        }
    }

    /// Run the consumer loop, processing events as they arrive.
    ///
    /// This method runs indefinitely until an error occurs.
    #[instrument(skip(self), fields(group = %self.consumer_group))]
    pub async fn run(&self) -> Result<()> {
        info!("starting projection consumer");
        let mut consumer = self.event_log.consumer(&self.consumer_group).await?;

        loop {
            let batch = consumer.poll(100, Duration::from_secs(1)).await?;

            if batch.is_empty() {
                continue;
            }

            debug!(count = batch.len(), "processing event batch");

            for (offset, event) in batch {
                self.projection.apply(&event).await?;
                consumer.commit(offset).await?;
            }
        }
    }

    /// Rebuild the projection from the beginning of the event log.
    ///
    /// This clears the projection and replays all events.
    #[instrument(skip(self))]
    pub async fn rebuild(&self) -> Result<()> {
        info!("rebuilding projection from event log");

        // Clear existing data
        self.projection.clear().await?;

        // Create a new consumer starting from the beginning
        let mut consumer = self.event_log.consumer("eval-rebuild").await?;
        consumer.seek(SeekPosition::Beginning).await?;

        let mut count = 0u64;

        loop {
            let batch = consumer.poll(1000, Duration::from_millis(100)).await?;

            if batch.is_empty() {
                break;
            }

            for (offset, event) in batch {
                self.projection.apply(&event).await?;
                consumer.commit(offset).await?;
                count += 1;
            }
        }

        info!(events = count, "projection rebuild complete");
        Ok(())
    }

    /// Process a single batch of events (for testing).
    ///
    /// Returns the number of events processed.
    pub async fn process_batch(&self, max_count: usize) -> Result<usize> {
        let mut consumer = self.event_log.consumer(&self.consumer_group).await?;
        let batch = consumer.poll(max_count, Duration::from_millis(100)).await?;
        let count = batch.len();

        for (offset, event) in batch {
            self.projection.apply(&event).await?;
            consumer.commit(offset).await?;
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::EvalEvent;
    use crate::storage::{EvalStorage, TursoEvalProjection, TursoEvalStorage};
    use crate::study::{PeriodType, StudyConfig, StudyStatus};
    use crate::types::StudyId;
    use std::sync::Mutex;
    use vibes_iggy::traits::{EventBatch, EventConsumer, Offset};

    /// Test event log that records events and plays them back.
    struct TestEventLog {
        events: Mutex<Vec<StoredEvalEvent>>,
    }

    impl TestEventLog {
        fn new() -> Self {
            Self {
                events: Mutex::new(Vec::new()),
            }
        }

        fn add_event(&self, event: StoredEvalEvent) {
            self.events.lock().unwrap().push(event);
        }
    }

    struct TestConsumer {
        log: Arc<TestEventLog>,
        cursor: usize,
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
            let len = events.len();
            stored.extend(events);
            Ok((stored.len() - len) as Offset)
        }

        async fn consumer(
            &self,
            _group: &str,
        ) -> vibes_iggy::error::Result<Box<dyn EventConsumer<StoredEvalEvent>>> {
            Ok(Box::new(TestConsumer {
                log: Arc::new(TestEventLog {
                    events: Mutex::new(self.events.lock().unwrap().clone()),
                }),
                cursor: 0,
            }))
        }

        fn high_water_mark(&self) -> Offset {
            self.events.lock().unwrap().len() as Offset
        }
    }

    #[async_trait::async_trait]
    impl EventConsumer<StoredEvalEvent> for TestConsumer {
        async fn poll(
            &mut self,
            max: usize,
            _timeout: Duration,
        ) -> vibes_iggy::error::Result<EventBatch<StoredEvalEvent>> {
            let events = self.log.events.lock().unwrap();
            let remaining = events.len() - self.cursor;
            let count = remaining.min(max);

            let batch: Vec<_> = events[self.cursor..self.cursor + count]
                .iter()
                .enumerate()
                .map(|(i, e)| ((self.cursor + i) as Offset, e.clone()))
                .collect();

            self.cursor += count;
            Ok(EventBatch::new(batch))
        }

        async fn commit(&mut self, _offset: Offset) -> vibes_iggy::error::Result<()> {
            Ok(())
        }

        async fn seek(
            &mut self,
            position: vibes_iggy::traits::SeekPosition,
        ) -> vibes_iggy::error::Result<()> {
            match position {
                vibes_iggy::traits::SeekPosition::Beginning => self.cursor = 0,
                vibes_iggy::traits::SeekPosition::End => {
                    self.cursor = self.log.events.lock().unwrap().len()
                }
                vibes_iggy::traits::SeekPosition::Offset(o) => self.cursor = o as usize,
                vibes_iggy::traits::SeekPosition::FromEnd(n) => {
                    let len = self.log.events.lock().unwrap().len();
                    self.cursor = len.saturating_sub(n as usize);
                }
            }
            Ok(())
        }

        fn committed_offset(&self) -> Offset {
            self.cursor as Offset
        }

        fn group(&self) -> &str {
            "test"
        }
    }

    async fn create_test_consumer() -> (EvalProjectionConsumer, Arc<TestEventLog>, TursoEvalStorage)
    {
        let event_log = Arc::new(TestEventLog::new());
        let storage = TursoEvalStorage::new_memory().await.unwrap();
        let projection = Arc::new(TursoEvalProjection::new(storage.clone()));
        let consumer = EvalProjectionConsumer::new(event_log.clone(), projection, "test-consumer");
        (consumer, event_log, storage)
    }

    #[tokio::test]
    async fn consumer_applies_study_created_event() {
        let (consumer, event_log, storage) = create_test_consumer().await;
        let study_id = StudyId::new();

        // Add event to log
        event_log.add_event(StoredEvalEvent::new(EvalEvent::StudyCreated {
            id: study_id,
            name: "test-study".to_string(),
            period_type: PeriodType::Weekly,
            period_value: Some(2),
            config: StudyConfig::default(),
        }));

        // Process events
        let count = consumer.process_batch(10).await.unwrap();
        assert_eq!(count, 1);

        // Verify projection was updated
        let study = storage.get_study(study_id).await.unwrap().unwrap();
        assert_eq!(study.name, "test-study");
        assert_eq!(study.status, StudyStatus::Pending);
    }

    #[tokio::test]
    async fn consumer_applies_lifecycle_events_in_order() {
        let (consumer, event_log, storage) = create_test_consumer().await;
        let study_id = StudyId::new();

        // Add events: create -> start -> pause -> resume -> stop
        event_log.add_event(StoredEvalEvent::new(EvalEvent::StudyCreated {
            id: study_id,
            name: "lifecycle-test".to_string(),
            period_type: PeriodType::Daily,
            period_value: None,
            config: StudyConfig::default(),
        }));
        event_log.add_event(StoredEvalEvent::new(EvalEvent::StudyStarted {
            id: study_id,
        }));
        event_log.add_event(StoredEvalEvent::new(EvalEvent::StudyPaused {
            id: study_id,
        }));
        event_log.add_event(StoredEvalEvent::new(EvalEvent::StudyResumed {
            id: study_id,
        }));
        event_log.add_event(StoredEvalEvent::new(EvalEvent::StudyStopped {
            id: study_id,
        }));

        // Process all events
        let count = consumer.process_batch(10).await.unwrap();
        assert_eq!(count, 5);

        // Verify final state
        let study = storage.get_study(study_id).await.unwrap().unwrap();
        assert_eq!(study.status, StudyStatus::Stopped);
        assert!(study.started_at.is_some());
        assert!(study.stopped_at.is_some());
    }

    #[tokio::test]
    async fn rebuild_replays_all_events() {
        let (consumer, event_log, storage) = create_test_consumer().await;

        // Create multiple studies
        for i in 0..3 {
            let study_id = StudyId::new();
            event_log.add_event(StoredEvalEvent::new(EvalEvent::StudyCreated {
                id: study_id,
                name: format!("study-{}", i),
                period_type: PeriodType::Daily,
                period_value: None,
                config: StudyConfig::default(),
            }));
        }

        // Rebuild from scratch
        consumer.rebuild().await.unwrap();

        // Verify all studies exist
        let studies = storage.list_studies().await.unwrap();
        assert_eq!(studies.len(), 3);
    }
}
