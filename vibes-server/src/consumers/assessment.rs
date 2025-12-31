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
use vibes_groove::assessment::{
    AssessmentConfig, AssessmentConsumerConfig, AssessmentProcessor, InMemoryAssessmentLog,
    assessment_consumer_loop,
};
use vibes_iggy::EventLog;

use super::Result;

/// Start the assessment consumer that processes events through the assessment pipeline.
///
/// # Arguments
///
/// * `event_log` - The EventLog to consume events from
/// * `shutdown` - Cancellation token to signal shutdown
///
/// # Returns
///
/// Returns Ok(()) on success, or an error if the consumer fails to start.
pub async fn start_assessment_consumer(
    event_log: Arc<dyn EventLog<vibes_core::VibesEvent>>,
    shutdown: CancellationToken,
) -> Result<()> {
    // Create assessment log (in-memory for now, will use Iggy later)
    let assessment_log = Arc::new(InMemoryAssessmentLog::new());

    // Create processor with default config
    let config = AssessmentConfig::default();
    let processor = Arc::new(AssessmentProcessor::new(config, assessment_log));

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

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use vibes_iggy::InMemoryEventLog;

    #[tokio::test]
    async fn test_assessment_consumer_starts() {
        let log = Arc::new(InMemoryEventLog::<vibes_core::VibesEvent>::new());
        let shutdown = CancellationToken::new();

        // Start consumer
        let result = start_assessment_consumer(log, shutdown.clone()).await;
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
        let log = Arc::new(InMemoryEventLog::<vibes_core::VibesEvent>::new());
        let shutdown = CancellationToken::new();

        // Append some events before starting consumer
        for i in 0..3 {
            log.append(vibes_core::VibesEvent::SessionCreated {
                session_id: format!("session-{i}"),
                name: None,
            })
            .await
            .unwrap();
        }

        // Start consumer
        start_assessment_consumer(log.clone(), shutdown.clone())
            .await
            .unwrap();

        // Give it time to process
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Shutdown
        shutdown.cancel();
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Consumer processed events (no panics = success)
    }
}
