//! Assessment event consumer loop.
//!
//! This consumer reads from the EventLog and dispatches events to the
//! AssessmentProcessor for analysis. Unlike the WebSocket consumer (which
//! starts at End for live events), the assessment consumer starts at
//! Beginning to process full session history for pattern detection.

use std::sync::Arc;
use std::time::Duration;

use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace};
use vibes_core::VibesEvent;
use vibes_iggy::{EventConsumer, Offset, SeekPosition};

use super::AssessmentProcessor;

/// Default poll timeout for assessment consumer.
const DEFAULT_POLL_TIMEOUT: Duration = Duration::from_secs(1);

/// Default batch size for assessment consumer.
const DEFAULT_BATCH_SIZE: usize = 100;

/// Configuration for the assessment consumer.
#[derive(Debug, Clone)]
pub struct AssessmentConsumerConfig {
    /// Consumer group name.
    pub group: String,
    /// Where to start reading from.
    pub start_position: SeekPosition,
    /// Maximum events per poll.
    pub batch_size: usize,
    /// Poll timeout.
    pub poll_timeout: Duration,
}

impl Default for AssessmentConsumerConfig {
    fn default() -> Self {
        Self {
            group: "assessment".to_string(),
            start_position: SeekPosition::Beginning,
            batch_size: DEFAULT_BATCH_SIZE,
            poll_timeout: DEFAULT_POLL_TIMEOUT,
        }
    }
}

impl AssessmentConsumerConfig {
    /// Create a new configuration with the default assessment settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the poll timeout.
    #[must_use]
    pub fn with_poll_timeout(mut self, timeout: Duration) -> Self {
        self.poll_timeout = timeout;
        self
    }

    /// Set the batch size.
    #[must_use]
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }
}

/// Result of running the assessment consumer loop.
#[derive(Debug)]
pub enum ConsumerResult {
    /// Consumer stopped due to shutdown signal.
    Shutdown,
    /// Consumer stopped due to an error.
    Error(String),
}

/// Run the assessment consumer loop.
///
/// This function processes events from the EventLog and dispatches them to the
/// AssessmentProcessor. It runs until the shutdown token is cancelled or an
/// error occurs.
///
/// # Arguments
///
/// * `consumer` - The event consumer to poll from.
/// * `processor` - The assessment processor to dispatch events to.
/// * `config` - Configuration for the consumer loop.
/// * `shutdown` - Cancellation token to signal shutdown.
///
/// # Returns
///
/// Returns `ConsumerResult::Shutdown` on graceful shutdown, or
/// `ConsumerResult::Error` if an unrecoverable error occurred.
pub async fn assessment_consumer_loop(
    mut consumer: Box<dyn EventConsumer<VibesEvent>>,
    processor: Arc<AssessmentProcessor>,
    config: AssessmentConsumerConfig,
    shutdown: CancellationToken,
) -> ConsumerResult {
    info!(group = %config.group, "Assessment consumer starting");

    // Seek to the configured start position
    if let Err(e) = consumer.seek(config.start_position).await {
        error!(error = %e, "Failed to seek to start position");
        return ConsumerResult::Error(format!("Seek failed: {e}"));
    }

    loop {
        tokio::select! {
            biased;

            _ = shutdown.cancelled() => {
                info!(group = %config.group, "Assessment consumer received shutdown signal");
                return ConsumerResult::Shutdown;
            }

            result = consumer.poll(config.batch_size, config.poll_timeout) => {
                match result {
                    Ok(batch) => {
                        if batch.is_empty() {
                            trace!(group = %config.group, "Empty batch, continuing");
                            continue;
                        }

                        debug!(group = %config.group, count = batch.len(), "Processing batch");

                        let mut last_offset: Option<Offset> = None;
                        for (offset, event) in batch {
                            // Dispatch to processor for analysis
                            processor.process_event(&event).await;
                            last_offset = Some(offset);
                        }

                        // Commit after processing batch
                        if let Some(offset) = last_offset
                            && let Err(e) = consumer.commit(offset).await
                        {
                            error!(group = %config.group, error = %e, "Failed to commit offset");
                            // Continue processing - commit failure is not fatal
                        }
                    }
                    Err(e) => {
                        error!(group = %config.group, error = %e, "Poll failed");
                        // Back off on error
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use vibes_iggy::{EventLog, InMemoryEventLog};

    fn make_event(session_id: &str) -> VibesEvent {
        VibesEvent::SessionCreated {
            session_id: session_id.to_string(),
            name: None,
        }
    }

    #[tokio::test]
    async fn test_assessment_consumer_processes_events() {
        // Setup: EventLog with some events
        let log = Arc::new(InMemoryEventLog::<VibesEvent>::new());

        // Append events before starting consumer
        for i in 0..5 {
            log.append(make_event(&format!("session-{i}")))
                .await
                .unwrap();
        }

        // Track processed count
        let processed_count = Arc::new(AtomicUsize::new(0));
        let processed_count_clone = Arc::clone(&processed_count);

        // Spawn consumer task that polls and counts events
        let handle = tokio::spawn(async move {
            let mut consumer = log.consumer("assessment-test").await.unwrap();

            // Seek to beginning
            consumer.seek(SeekPosition::Beginning).await.unwrap();

            // Process events
            let batch = consumer
                .poll(100, Duration::from_millis(100))
                .await
                .unwrap();
            for (_offset, _event) in batch {
                processed_count_clone.fetch_add(1, Ordering::SeqCst);
            }
        });

        // Wait for processing
        tokio::time::timeout(Duration::from_secs(1), handle)
            .await
            .expect("should complete within timeout")
            .expect("task should not panic");

        // Verify events were processed
        assert_eq!(processed_count.load(Ordering::SeqCst), 5);
    }

    #[tokio::test]
    async fn test_assessment_consumer_commits_after_batch() {
        // Setup: EventLog with events
        let log = Arc::new(InMemoryEventLog::<VibesEvent>::new());

        // Append events
        for i in 0..3 {
            log.append(make_event(&format!("commit-test-{i}")))
                .await
                .unwrap();
        }

        // Create consumer and process events
        let mut consumer = log.consumer("commit-test-group").await.unwrap();
        consumer.seek(SeekPosition::Beginning).await.unwrap();

        // Initial committed offset should be 0
        assert_eq!(consumer.committed_offset(), 0);

        // Poll and process
        let batch = consumer
            .poll(100, Duration::from_millis(100))
            .await
            .unwrap();
        assert_eq!(batch.len(), 3);

        // Commit the last offset
        let last_offset = batch.last_offset().unwrap();
        consumer.commit(last_offset).await.unwrap();

        // Committed offset should now be updated
        assert_eq!(consumer.committed_offset(), last_offset);

        // Create a new consumer in the same group - should resume from committed offset
        let consumer2 = log.consumer("commit-test-group").await.unwrap();
        assert_eq!(consumer2.committed_offset(), last_offset);
    }

    #[tokio::test]
    async fn test_assessment_consumer_respects_shutdown() {
        // Setup: EventLog
        let log = Arc::new(InMemoryEventLog::<VibesEvent>::new());

        // Append one event
        log.append(make_event("shutdown-test")).await.unwrap();

        // Create consumer
        let consumer = log.consumer("shutdown-test-group").await.unwrap();

        // Create shutdown token
        let shutdown = CancellationToken::new();
        let shutdown_clone = shutdown.clone();

        // Create a simple processor using InMemoryAssessmentLog
        let assessment_log = Arc::new(crate::assessment::InMemoryAssessmentLog::new());
        let processor = Arc::new(AssessmentProcessor::new(
            crate::assessment::AssessmentConfig::default(),
            assessment_log,
        ));

        // Spawn consumer loop
        let handle = tokio::spawn(async move {
            assessment_consumer_loop(
                consumer,
                processor,
                AssessmentConsumerConfig::default().with_poll_timeout(Duration::from_millis(50)),
                shutdown_clone,
            )
            .await
        });

        // Give it time to start processing
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Signal shutdown
        shutdown.cancel();

        // Wait for consumer to stop
        let result = tokio::time::timeout(Duration::from_secs(1), handle)
            .await
            .expect("should complete within timeout")
            .expect("task should not panic");

        // Should have shut down gracefully
        assert!(matches!(result, ConsumerResult::Shutdown));
    }

    #[tokio::test]
    async fn test_assessment_consumer_config_defaults() {
        let config = AssessmentConsumerConfig::default();

        assert_eq!(config.group, "assessment");
        assert_eq!(config.start_position, SeekPosition::Beginning);
        assert_eq!(config.batch_size, DEFAULT_BATCH_SIZE);
        assert_eq!(config.poll_timeout, DEFAULT_POLL_TIMEOUT);
    }

    #[tokio::test]
    async fn test_assessment_consumer_config_builder() {
        let config = AssessmentConsumerConfig::new()
            .with_poll_timeout(Duration::from_millis(500))
            .with_batch_size(50);

        assert_eq!(config.poll_timeout, Duration::from_millis(500));
        assert_eq!(config.batch_size, 50);
    }

    #[tokio::test]
    async fn test_assessment_consumer_starts_from_beginning() {
        // Verify that assessment consumer uses Beginning position (not End like WebSocket)
        let config = AssessmentConsumerConfig::default();
        assert_eq!(config.start_position, SeekPosition::Beginning);

        // This is important: assessment needs full history for pattern detection,
        // unlike WebSocket which only needs live events
    }

    #[tokio::test]
    async fn test_assessment_consumer_processes_all_historical_events() {
        // Setup: EventLog with events added BEFORE consumer starts
        let log = Arc::new(InMemoryEventLog::<VibesEvent>::new());

        // Append events before creating consumer
        for i in 0..10 {
            log.append(make_event(&format!("historical-{i}")))
                .await
                .unwrap();
        }

        // Create consumer (should start from Beginning)
        let mut consumer = log.consumer("historical-test").await.unwrap();
        consumer.seek(SeekPosition::Beginning).await.unwrap();

        // Poll for events
        let batch = consumer
            .poll(100, Duration::from_millis(100))
            .await
            .unwrap();

        // Should receive all 10 historical events
        assert_eq!(batch.len(), 10);
    }
}
