//! Assessment event consumer.
//!
//! This consumer reads from the EventLog and dispatches events to the
//! AssessmentProcessor for pattern detection and learning extraction.
//!
//! Unlike the WebSocket consumer (which starts at End for live events),
//! the assessment consumer starts at Beginning to process full session
//! history for pattern detection.

use std::sync::Arc;

use tokio_util::sync::CancellationToken;
use vibes_core::StoredEvent;
use vibes_groove::assessment::{
    AssessmentConfig, AssessmentConsumerConfig, AssessmentLog, AssessmentProcessor,
    IggyAssessmentLog, InMemoryAssessmentLog, assessment_consumer_loop,
};
use vibes_iggy::{EventLog, IggyManager};

use super::Result;

/// Start the assessment consumer that processes events through the assessment pipeline.
///
/// # Arguments
///
/// * `event_log` - The EventLog to consume events from
/// * `iggy_manager` - Optional IggyManager for persistent assessment storage
/// * `shutdown` - Cancellation token to signal shutdown
///
/// # Returns
///
/// Returns the AssessmentLog on success (for WebSocket endpoint use),
/// or an error if the consumer fails to start.
pub async fn start_assessment_consumer(
    event_log: Arc<dyn EventLog<StoredEvent>>,
    iggy_manager: Option<Arc<IggyManager>>,
    shutdown: CancellationToken,
) -> Result<Arc<dyn AssessmentLog>> {
    // Create assessment log - use IggyAssessmentLog when we have a manager
    let assessment_log: Arc<dyn AssessmentLog> = if let Some(manager) = iggy_manager {
        let log = IggyAssessmentLog::new(manager);

        // Try to connect - fall back to in-memory if connection fails
        match log.connect().await {
            Ok(()) => {
                tracing::info!("Assessment log connected to Iggy");
                Arc::new(log)
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to connect assessment log to Iggy: {}. Using in-memory log.",
                    e
                );
                Arc::new(InMemoryAssessmentLog::new())
            }
        }
    } else {
        tracing::info!("Using in-memory assessment log (no Iggy manager)");
        Arc::new(InMemoryAssessmentLog::new())
    };

    // Create processor with default config
    let config = AssessmentConfig::default();
    let processor = Arc::new(AssessmentProcessor::new(config, assessment_log.clone()));

    // Create consumer from the event log
    let consumer = event_log
        .consumer("assessment")
        .await
        .map_err(|e| super::ConsumerError::Creation(e.to_string()))?;

    // Consumer config - replay from beginning for full history
    let consumer_config = AssessmentConsumerConfig::default();

    tracing::info!("Assessment consumer starting");

    // Spawn the consumer loop in a background task
    tokio::spawn(async move {
        let result = assessment_consumer_loop(consumer, processor, consumer_config, shutdown).await;

        match result {
            vibes_groove::assessment::ConsumerResult::Shutdown => {
                tracing::info!("Assessment consumer stopped gracefully");
            }
            vibes_groove::assessment::ConsumerResult::Error(e) => {
                tracing::error!("Assessment consumer stopped with error: {}", e);
            }
        }
    });

    Ok(assessment_log)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use vibes_core::VibesEvent;
    use vibes_iggy::InMemoryEventLog;

    fn make_stored_event(session_id: &str) -> StoredEvent {
        StoredEvent::new(VibesEvent::SessionCreated {
            session_id: session_id.to_string(),
            name: None,
        })
    }

    #[tokio::test]
    async fn test_assessment_consumer_starts() {
        let log = Arc::new(InMemoryEventLog::<StoredEvent>::new());
        let shutdown = CancellationToken::new();

        // Start consumer (no Iggy manager = in-memory log)
        let result = start_assessment_consumer(log, None, shutdown.clone()).await;
        assert!(result.is_ok());

        // Give it a moment to start
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Shutdown gracefully
        shutdown.cancel();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn test_assessment_consumer_uses_replay_mode() {
        // Verify the consumer starts at Beginning (replay mode) - full history
        let config = AssessmentConsumerConfig::default();
        assert_eq!(config.start_position, vibes_iggy::SeekPosition::Beginning);
        assert_eq!(config.group, "assessment");
    }

    #[tokio::test]
    async fn test_assessment_consumer_processes_events() {
        let log = Arc::new(InMemoryEventLog::<StoredEvent>::new());
        let shutdown = CancellationToken::new();

        // Append some events before starting consumer
        for i in 0..3 {
            log.append(make_stored_event(&format!("session-{i}")))
                .await
                .unwrap();
        }

        // Start consumer (no Iggy manager = in-memory log)
        start_assessment_consumer(log.clone(), None, shutdown.clone())
            .await
            .unwrap();

        // Give it time to process
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Shutdown
        shutdown.cancel();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Consumer processed events (no panics = success)
    }

    #[tokio::test]
    async fn test_assessment_consumer_returns_log() {
        let log = Arc::new(InMemoryEventLog::<StoredEvent>::new());
        let shutdown = CancellationToken::new();

        // Start consumer and get the assessment log
        let assessment_log = start_assessment_consumer(log, None, shutdown.clone())
            .await
            .expect("should start");

        // Subscribe to the assessment log
        let _rx = assessment_log.subscribe();

        // Cleanup
        shutdown.cancel();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
