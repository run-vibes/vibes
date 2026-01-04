//! Assessment processor for fire-and-forget event handling.
//!
//! The `AssessmentProcessor` provides a non-blocking interface for submitting
//! assessment events to the log. Events are queued and written by a background
//! task, ensuring that the main processing path is never blocked by I/O.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc};
use tracing::{debug, error, trace};

use super::lightweight::{LightweightDetector, LightweightDetectorConfig, SessionState};
use super::types::SessionId;
use super::{AssessmentConfig, AssessmentEvent, AssessmentLog};

/// Messages sent to the background writer task.
#[derive(Debug)]
pub enum WriterMessage {
    /// An assessment event to write to the log.
    ///
    /// Boxed to reduce enum size variance (AssessmentEvent is large).
    Event(Box<AssessmentEvent>),
    /// Shutdown signal for the writer task.
    Shutdown,
}

/// Processor for assessment events with fire-and-forget semantics.
///
/// The processor spawns a background task that handles event persistence,
/// allowing callers to submit events without blocking on I/O operations.
/// Events are delivered to subscribers via a broadcast channel.
pub struct AssessmentProcessor {
    /// Configuration for assessment behavior.
    config: AssessmentConfig,
    /// Channel sender for the background writer.
    writer_tx: mpsc::UnboundedSender<WriterMessage>,
    /// Broadcast sender for real-time event subscribers.
    broadcast_tx: broadcast::Sender<AssessmentEvent>,
    /// Lightweight detector for per-message signal detection.
    detector: LightweightDetector,
    /// Per-session state for EMA computation (protected by RwLock for async access).
    session_states: Arc<RwLock<HashMap<SessionId, SessionState>>>,
}

impl AssessmentProcessor {
    /// Create a new assessment processor.
    ///
    /// This spawns a background task that writes events to the provided log.
    /// The background task runs until `shutdown()` is called or the processor
    /// is dropped.
    #[must_use]
    pub fn new(config: AssessmentConfig, log: Arc<dyn AssessmentLog>) -> Self {
        let (writer_tx, writer_rx) = mpsc::unbounded_channel();
        let (broadcast_tx, _) = broadcast::channel(1024);

        // Clone broadcast_tx for the writer task
        let task_broadcast_tx = broadcast_tx.clone();

        // Spawn background writer task
        tokio::spawn(async move {
            Self::writer_task(log, writer_rx, task_broadcast_tx).await;
        });

        // Create detector from pattern config
        let detector_config = LightweightDetectorConfig::from_pattern_config(&config.patterns);
        let detector = LightweightDetector::new(detector_config);

        Self {
            config,
            writer_tx,
            broadcast_tx,
            detector,
            session_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Background task that processes queued events.
    ///
    /// This task runs in a loop, receiving events from the channel and
    /// writing them to the assessment log. It exits when a Shutdown message
    /// is received or the channel is closed.
    async fn writer_task(
        log: Arc<dyn AssessmentLog>,
        mut rx: mpsc::UnboundedReceiver<WriterMessage>,
        broadcast_tx: broadcast::Sender<AssessmentEvent>,
    ) {
        debug!("Assessment writer task started");

        while let Some(msg) = rx.recv().await {
            match msg {
                WriterMessage::Event(event) => {
                    trace!(session_id = %event.session_id(), "Writing assessment event");

                    // Write to log (fire-and-forget semantics - log errors but don't fail)
                    if let Err(e) = log.append((*event).clone()).await {
                        error!(error = %e, "Failed to write assessment event to log");
                    }

                    // Broadcast to subscribers (ignore if no subscribers)
                    let _ = broadcast_tx.send(*event);
                }
                WriterMessage::Shutdown => {
                    debug!("Assessment writer task received shutdown signal");
                    break;
                }
            }
        }

        debug!("Assessment writer task stopped");
    }

    /// Check if assessment is enabled.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Submit an event for processing.
    ///
    /// This is non-blocking - the event is queued for the background writer.
    /// If the writer channel is closed, the event is silently dropped.
    pub fn submit(&self, event: AssessmentEvent) {
        if !self.is_enabled() {
            trace!("Assessment disabled, dropping event");
            return;
        }

        if let Err(e) = self.writer_tx.send(WriterMessage::Event(Box::new(event))) {
            error!(error = %e, "Failed to queue assessment event - writer channel closed");
        }
    }

    /// Subscribe to real-time assessment events.
    ///
    /// Returns a broadcast receiver that will receive all events submitted
    /// to this processor.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<AssessmentEvent> {
        self.broadcast_tx.subscribe()
    }

    /// Signal the background writer to shut down.
    ///
    /// This sends a shutdown message to the writer task. The task will
    /// finish processing any queued events before exiting.
    pub fn shutdown(&self) {
        let _ = self.writer_tx.send(WriterMessage::Shutdown);
    }

    /// Get a reference to the configuration.
    #[must_use]
    pub fn config(&self) -> &AssessmentConfig {
        &self.config
    }

    /// Process a VibesEvent from the EventLog.
    ///
    /// This is the main entry point for the assessment consumer. It analyzes
    /// incoming events and potentially emits assessment events based on
    /// detected patterns and signals.
    ///
    /// The processing pipeline includes:
    /// - LightweightDetector for per-message signal detection (B1)
    /// - CircuitBreaker for intervention decisions (B2)
    /// - SessionBuffer for event collection (B3)
    /// - CheckpointManager for checkpoint triggers (B4)
    pub async fn process_event(&self, event: &vibes_core::VibesEvent) {
        if !self.is_enabled() {
            trace!("Assessment disabled, skipping event processing");
            return;
        }

        trace!(event = ?event, "Processing VibesEvent for assessment");

        // B1: Route to LightweightDetector for signal detection
        // Get session ID to look up/create session state
        let session_id = match Self::extract_session_id(event) {
            Some(id) => id,
            None => {
                trace!("Event has no session_id, skipping lightweight detection");
                return;
            }
        };

        // Get or create session state (using write lock to potentially insert)
        let mut states = self.session_states.write().await;
        let state = states.entry(session_id).or_insert_with(SessionState::new);

        // Run the detector
        if let Some(lightweight_event) = self.detector.process(event, state) {
            trace!(
                session_id = %lightweight_event.context.session_id,
                message_idx = lightweight_event.message_idx,
                signals = lightweight_event.signals.len(),
                "Emitting LightweightEvent"
            );
            self.submit(AssessmentEvent::Lightweight(lightweight_event));
        }

        // TODO(B2): Route signals to CircuitBreaker for state management
        // TODO(B3): Buffer events per session
        // TODO(B4): Check for checkpoint triggers
    }

    /// Extract session_id from a VibesEvent, if present.
    fn extract_session_id(event: &vibes_core::VibesEvent) -> Option<SessionId> {
        // Use the built-in session_id() method which handles all variants
        event.session_id().map(SessionId::from)
    }
}

impl Drop for AssessmentProcessor {
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl std::fmt::Debug for AssessmentProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssessmentProcessor")
            .field("config", &self.config)
            .field("enabled", &self.is_enabled())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assessment::{AssessmentContext, InMemoryAssessmentLog, LightweightEvent};
    use std::time::Duration;
    use vibes_core::{ClaudeEvent, VibesEvent};

    fn make_lightweight_event(session: &str) -> AssessmentEvent {
        AssessmentEvent::Lightweight(LightweightEvent {
            context: AssessmentContext::new(session),
            message_idx: 0,
            signals: vec![],
            frustration_ema: 0.0,
            success_ema: 1.0,
        })
    }

    #[tokio::test]
    async fn processor_submits_events() {
        let log = Arc::new(InMemoryAssessmentLog::new());
        let config = AssessmentConfig::default();
        let processor = AssessmentProcessor::new(config, log.clone());

        let event = make_lightweight_event("test-session");
        processor.submit(event);

        // Give the background task time to process
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Verify event was written
        let events = log.read_session(&"test-session".into()).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].session_id().as_str(), "test-session");

        processor.shutdown();
    }

    #[tokio::test]
    async fn processor_respects_disabled_config() {
        let log = Arc::new(InMemoryAssessmentLog::new());
        let config = AssessmentConfig {
            enabled: false,
            ..Default::default()
        };

        let processor = AssessmentProcessor::new(config, log.clone());

        assert!(!processor.is_enabled());

        // Submit event - should be dropped
        let event = make_lightweight_event("disabled-session");
        processor.submit(event);

        // Give time for any potential processing
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Verify no event was written
        let events = log.read_session(&"disabled-session".into()).await.unwrap();
        assert!(events.is_empty());

        processor.shutdown();
    }

    #[tokio::test]
    async fn processor_subscribe_receives_events() {
        let log = Arc::new(InMemoryAssessmentLog::new());
        let config = AssessmentConfig::default();
        let processor = AssessmentProcessor::new(config, log.clone());

        // Subscribe before submitting
        let mut rx = processor.subscribe();

        let event = make_lightweight_event("broadcast-session");
        processor.submit(event);

        // Receive the broadcasted event
        let received = tokio::time::timeout(Duration::from_millis(100), rx.recv())
            .await
            .expect("should receive within timeout")
            .expect("should receive event");

        assert_eq!(received.session_id().as_str(), "broadcast-session");

        processor.shutdown();
    }

    #[tokio::test]
    async fn processor_config_accessor() {
        let log = Arc::new(InMemoryAssessmentLog::new());
        let mut config = AssessmentConfig::default();
        config.sampling.base_rate = 0.5;

        let processor = AssessmentProcessor::new(config, log);

        assert_eq!(processor.config().sampling.base_rate, 0.5);
        assert!(processor.config().enabled);

        processor.shutdown();
    }

    #[tokio::test]
    async fn processor_handles_multiple_events() {
        let log = Arc::new(InMemoryAssessmentLog::new());
        let config = AssessmentConfig::default();
        let processor = AssessmentProcessor::new(config, log.clone());

        // Submit multiple events
        for i in 0..5 {
            let event = make_lightweight_event(&format!("multi-session-{i}"));
            processor.submit(event);
        }

        // Give the background task time to process all events
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verify all events were written
        assert_eq!(log.len(), 5);

        processor.shutdown();
    }

    #[tokio::test]
    async fn processor_shutdown_stops_writer() {
        let log = Arc::new(InMemoryAssessmentLog::new());
        let config = AssessmentConfig::default();
        let processor = AssessmentProcessor::new(config, log.clone());

        // Submit an event and wait for it
        processor.submit(make_lightweight_event("before-shutdown"));
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Shutdown the processor
        processor.shutdown();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Events submitted after shutdown may not be processed
        // (channel may be closed or task stopped)
        let events = log.read_session(&"before-shutdown".into()).await.unwrap();
        assert_eq!(events.len(), 1);
    }

    // ==================== process_event Tests ====================

    fn make_text_delta(session_id: &str, text: &str) -> VibesEvent {
        VibesEvent::Claude {
            session_id: session_id.to_string(),
            event: ClaudeEvent::TextDelta {
                text: text.to_string(),
            },
        }
    }

    #[tokio::test]
    async fn process_event_emits_lightweight_event() {
        let log = Arc::new(InMemoryAssessmentLog::new());
        let config = AssessmentConfig::default();
        let processor = AssessmentProcessor::new(config, log.clone());

        // Create a VibesEvent that the detector should process
        let event = make_text_delta("test-session", "Hello, this is a test message");

        // Process the event
        processor.process_event(&event).await;

        // Give the background task time to write
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Verify a LightweightEvent was emitted
        let events = log.read_session(&"test-session".into()).await.unwrap();
        assert_eq!(events.len(), 1, "Should emit one LightweightEvent");

        match &events[0] {
            AssessmentEvent::Lightweight(le) => {
                assert_eq!(le.context.session_id.as_str(), "test-session");
                assert_eq!(le.message_idx, 0, "First message should have index 0");
            }
            other => panic!("Expected LightweightEvent, got {:?}", other),
        }

        processor.shutdown();
    }

    #[tokio::test]
    async fn process_event_maintains_session_state() {
        let log = Arc::new(InMemoryAssessmentLog::new());
        let config = AssessmentConfig::default();
        let processor = AssessmentProcessor::new(config, log.clone());

        // Process multiple events for the same session
        let events = vec![
            make_text_delta("stateful-session", "First message"),
            make_text_delta("stateful-session", "Second message"),
            make_text_delta("stateful-session", "Third message"),
        ];

        for event in &events {
            processor.process_event(event).await;
        }

        // Give background task time to write
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verify message indices increment
        let logged_events = log.read_session(&"stateful-session".into()).await.unwrap();
        assert_eq!(logged_events.len(), 3);

        for (i, event) in logged_events.iter().enumerate() {
            match event {
                AssessmentEvent::Lightweight(le) => {
                    assert_eq!(
                        le.message_idx, i as u32,
                        "Message {} should have index {}",
                        i, i
                    );
                }
                other => panic!("Expected LightweightEvent, got {:?}", other),
            }
        }

        processor.shutdown();
    }

    #[tokio::test]
    async fn process_event_detects_frustration_patterns() {
        let log = Arc::new(InMemoryAssessmentLog::new());
        let config = AssessmentConfig::default();
        let processor = AssessmentProcessor::new(config, log.clone());

        // Send a frustrating message
        let event = make_text_delta("frustration-session", "This is broken and doesn't work!");
        processor.process_event(&event).await;

        tokio::time::sleep(Duration::from_millis(50)).await;

        let events = log
            .read_session(&"frustration-session".into())
            .await
            .unwrap();
        assert_eq!(events.len(), 1);

        match &events[0] {
            AssessmentEvent::Lightweight(le) => {
                assert!(
                    !le.signals.is_empty(),
                    "Should detect negative signals in frustrating message"
                );
                assert!(
                    le.frustration_ema > 0.0,
                    "Frustration EMA should be positive"
                );
            }
            other => panic!("Expected LightweightEvent, got {:?}", other),
        }

        processor.shutdown();
    }
}
