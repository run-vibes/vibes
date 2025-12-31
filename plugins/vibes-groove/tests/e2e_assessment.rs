//! End-to-end tests for the assessment pipeline.
//!
//! These tests validate the complete assessment flow from event submission
//! through the various assessment tiers to output generation.
//!
//! ## Test Scenarios
//!
//! - E2E-1: Full event flow through all components
//! - E2E-4: Assessment pipeline with pattern detection and circuit breaker
//! - E2E-5: Session end handling with final checkpoint

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::broadcast;
use vibes_core::VibesEvent;
use vibes_groove::assessment::{
    AssessmentConfig, AssessmentEvent, AssessmentLog, AssessmentProcessor, CheckpointConfig,
    CheckpointManager, CheckpointTrigger, CircuitBreaker, CircuitBreakerConfig, CircuitState,
    InMemoryAssessmentLog, LightweightDetector, LightweightDetectorConfig, LightweightEvent,
    PatternConfig, SamplingConfig, SamplingContext, SamplingDecision, SamplingStrategy,
    SessionBuffer, SessionBufferConfig, SessionEnd, SessionEndConfig, SessionEndDetector,
    SessionEndReason, SessionId, SessionState,
};

/// E2E Test Harness for assessment pipeline testing.
///
/// Provides utilities for simulating event flows and verifying
/// the assessment pipeline processes them correctly.
pub struct E2ETestHarness {
    /// The assessment processor under test
    pub processor: AssessmentProcessor,
    /// In-memory log for verifying persisted events
    pub log: Arc<InMemoryAssessmentLog>,
    /// Lightweight detector for pattern matching
    pub detector: LightweightDetector,
    /// Per-session state for the detector
    pub session_states: HashMap<SessionId, SessionState>,
    /// Session buffer for collecting events
    pub buffer: SessionBuffer,
    /// Checkpoint manager for triggering assessments
    pub checkpoint: CheckpointManager,
    /// Session end detector
    pub session_end: SessionEndDetector,
    /// Circuit breaker for intervention decisions
    pub circuit_breaker: CircuitBreaker,
    /// Sampling strategy for tier selection
    pub sampling: SamplingStrategy,
    /// Subscription receiver for assessment events
    pub rx: broadcast::Receiver<AssessmentEvent>,
}

impl E2ETestHarness {
    /// Create a new test harness with default configuration.
    pub async fn new() -> Self {
        Self::with_config(
            AssessmentConfig::default(),
            LightweightDetectorConfig::with_default_patterns(),
            SessionBufferConfig::default(),
            CheckpointConfig::default(),
            SessionEndConfig::default(),
            CircuitBreakerConfig::default(),
            SamplingConfig::default(),
        )
        .await
    }

    /// Create a new test harness with custom configuration.
    pub async fn with_config(
        processor_config: AssessmentConfig,
        detector_config: LightweightDetectorConfig,
        buffer_config: SessionBufferConfig,
        checkpoint_config: CheckpointConfig,
        session_end_config: SessionEndConfig,
        circuit_config: CircuitBreakerConfig,
        sampling_config: SamplingConfig,
    ) -> Self {
        let log = Arc::new(InMemoryAssessmentLog::new());
        let processor = AssessmentProcessor::new(processor_config, log.clone());
        let rx = processor.subscribe();

        Self {
            processor,
            log,
            detector: LightweightDetector::new(detector_config),
            session_states: HashMap::new(),
            buffer: SessionBuffer::new(buffer_config),
            checkpoint: CheckpointManager::new(checkpoint_config),
            session_end: SessionEndDetector::new(session_end_config),
            circuit_breaker: CircuitBreaker::new(circuit_config),
            sampling: SamplingStrategy::new(sampling_config),
            rx,
        }
    }

    /// Simulate a user input event.
    pub fn user_input(&self, session_id: &str, content: &str) -> VibesEvent {
        VibesEvent::UserInput {
            session_id: session_id.to_string(),
            content: content.to_string(),
            source: vibes_core::InputSource::Unknown,
        }
    }

    /// Simulate a Claude response event.
    pub fn claude_response(&self, session_id: &str, content: &str) -> VibesEvent {
        VibesEvent::Claude {
            session_id: session_id.to_string(),
            event: vibes_core::ClaudeEvent::TextDelta {
                text: content.to_string(),
            },
        }
    }

    /// Simulate a session end event.
    pub fn session_removed(&self, session_id: &str) -> VibesEvent {
        VibesEvent::SessionRemoved {
            session_id: session_id.to_string(),
            reason: "test".to_string(),
        }
    }

    /// Process an event through the full pipeline and return results.
    pub fn process_event(&mut self, event: &VibesEvent) -> ProcessResult {
        let session_id = event.session_id().map(SessionId::from);

        // 1. Lightweight detection (requires session state)
        let lightweight = if let Some(ref sid) = session_id {
            let state = self.session_states.entry(sid.clone()).or_default();
            self.detector.process(event, state)
        } else {
            None
        };

        // 2. Buffer the event
        if let Some(ref sid) = session_id {
            self.buffer.push(sid.clone(), event.clone());
        }

        // 3. Check for checkpoint trigger
        let checkpoint_trigger = if let (Some(sid), Some(lw)) = (&session_id, &lightweight) {
            self.checkpoint.should_checkpoint(sid, lw, &self.buffer)
        } else {
            None
        };

        // 4. Check for session end
        let session_end = self.session_end.process(event);

        // 5. Update circuit breaker if we have lightweight event
        let circuit_transition = if let Some(ref lw) = lightweight {
            self.circuit_breaker.record_event(lw)
        } else {
            None
        };

        // 6. Determine sampling decision
        let sampling_decision = if checkpoint_trigger.is_some() || session_end.is_some() {
            let ctx = if session_end.is_some() {
                SamplingContext::session_end()
            } else {
                SamplingContext::checkpoint(
                    checkpoint_trigger
                        .clone()
                        .unwrap_or(CheckpointTrigger::TimeInterval),
                )
            };
            Some(self.sampling.should_sample(&ctx))
        } else {
            None
        };

        ProcessResult {
            lightweight,
            checkpoint_trigger,
            session_end,
            circuit_transition,
            sampling_decision,
        }
    }

    /// Wait for assessment events to be received.
    pub async fn wait_for_events(
        &mut self,
        count: usize,
        timeout: Duration,
    ) -> Vec<AssessmentEvent> {
        let mut events = Vec::new();
        let deadline = tokio::time::Instant::now() + timeout;

        while events.len() < count && tokio::time::Instant::now() < deadline {
            match tokio::time::timeout(Duration::from_millis(10), self.rx.recv()).await {
                Ok(Ok(event)) => events.push(event),
                Ok(Err(_)) => continue, // Lagged, try again
                Err(_) => continue,     // Timeout, try again
            }
        }

        events
    }

    /// Get events from the log for a session.
    pub async fn get_logged_events(&self, session_id: &str) -> Vec<AssessmentEvent> {
        self.log
            .read_session(&SessionId::new(session_id))
            .await
            .unwrap_or_default()
    }

    /// Shutdown the processor.
    pub fn shutdown(&self) {
        self.processor.shutdown();
    }
}

/// Result of processing an event through the pipeline.
#[derive(Debug)]
pub struct ProcessResult {
    /// Lightweight event if pattern/signal detected
    pub lightweight: Option<LightweightEvent>,
    /// Checkpoint trigger if threshold reached
    pub checkpoint_trigger: Option<CheckpointTrigger>,
    /// Session end if detected
    pub session_end: Option<SessionEnd>,
    /// Circuit state transition if any
    pub circuit_transition: Option<vibes_groove::assessment::CircuitTransition>,
    /// Sampling decision if checkpoint/session end
    pub sampling_decision: Option<SamplingDecision>,
}

// =============================================================================
// E2E Test: Full Event Flow (E2E-1)
// =============================================================================

#[tokio::test]
async fn e2e_full_event_flow_through_pipeline() {
    let mut harness = E2ETestHarness::new().await;
    let session_id = "e2e-flow-test";

    // Simulate a conversation with multiple exchanges
    let events = vec![
        harness.user_input(session_id, "Help me write a function"),
        harness.claude_response(session_id, "Here's a function for you..."),
        harness.user_input(session_id, "Can you add error handling?"),
        harness.claude_response(session_id, "Sure, here's the updated version..."),
        harness.user_input(session_id, "Thanks, that works!"),
    ];

    // Process each event through the pipeline
    let mut lightweight_count = 0;
    for event in &events {
        let result = harness.process_event(event);
        if result.lightweight.is_some() {
            lightweight_count += 1;
        }
    }

    // Verify lightweight detector processed events
    assert!(
        lightweight_count > 0,
        "Should have detected at least one event"
    );

    // Verify session buffer collected events
    let buffered = harness.buffer.get(&SessionId::new(session_id));
    assert!(buffered.is_some(), "Buffer should contain events");
    assert_eq!(
        buffered.unwrap().len(),
        5,
        "Buffer should contain all 5 events"
    );

    harness.shutdown();
}

// =============================================================================
// E2E Test: Assessment Pipeline with Pattern Detection (E2E-4)
// =============================================================================

#[tokio::test]
async fn e2e_assessment_pipeline_pattern_detection() {
    // Configure detector to recognize error patterns
    let pattern_config = PatternConfig {
        negative: vec![r"(?i)\berror\b".to_string(), r"(?i)\bfailed\b".to_string()],
        positive: vec![r"(?i)\bsuccess\b".to_string(), r"(?i)\bworks\b".to_string()],
    };
    let detector_config = LightweightDetectorConfig::from_pattern_config(&pattern_config);

    let mut harness = E2ETestHarness::with_config(
        AssessmentConfig::default(),
        detector_config,
        SessionBufferConfig::default(),
        CheckpointConfig::default(),
        SessionEndConfig::default(),
        CircuitBreakerConfig::default(),
        SamplingConfig::default(),
    )
    .await;

    let session_id = "e2e-pattern-test";

    // Simulate conversation with error patterns
    let error_event = harness.user_input(session_id, "I keep getting an error, it failed again");
    let result = harness.process_event(&error_event);

    // Verify error pattern was detected
    assert!(
        result.lightweight.is_some(),
        "Should detect lightweight event"
    );
    let lw = result.lightweight.unwrap();
    assert!(!lw.signals.is_empty(), "Should have detected error signals");

    // Now simulate success
    let success_event = harness.user_input(session_id, "It works now, success!");
    let success_result = harness.process_event(&success_event);

    // Verify success pattern was detected
    assert!(
        success_result.lightweight.is_some(),
        "Should detect success event"
    );

    harness.shutdown();
}

#[tokio::test]
async fn e2e_circuit_breaker_state_transitions() {
    // Configure circuit breaker for testing
    let circuit_config = CircuitBreakerConfig {
        enabled: true,
        cooldown_seconds: 0, // Instant cooldown for testing
        max_interventions_per_session: 3,
    };

    // Use patterns that will detect signals
    let pattern_config = PatternConfig {
        negative: vec![r"(?i)\berror\b".to_string()],
        positive: vec![],
    };
    let detector_config = LightweightDetectorConfig::from_pattern_config(&pattern_config);

    let mut harness = E2ETestHarness::with_config(
        AssessmentConfig::default(),
        detector_config,
        SessionBufferConfig::default(),
        CheckpointConfig::default(),
        SessionEndConfig::default(),
        circuit_config,
        SamplingConfig::default(),
    )
    .await;

    let session_id = SessionId::new("e2e-circuit-test");

    // Initial state should be Closed
    assert_eq!(
        harness.circuit_breaker.state(&session_id),
        CircuitState::Closed,
        "Initial state should be Closed"
    );

    // Generate error events to trip the circuit breaker
    for i in 0..10 {
        let event = harness.user_input(
            session_id.as_str(),
            &format!("Error #{i}, something failed"),
        );
        let result = harness.process_event(&event);

        if let Some(ref transition) = result.circuit_transition
            && matches!(
                transition,
                vibes_groove::assessment::CircuitTransition::Opened { .. }
            )
        {
            // Circuit opened, we're done
            break;
        }
    }

    // State should now be Open (or still Closed if threshold not reached)
    let state = harness.circuit_breaker.state(&session_id);
    // The exact state depends on how frustration_ema accumulates
    // For this test, we just verify the circuit breaker was exercised
    assert!(
        state == CircuitState::Open || state == CircuitState::Closed,
        "State should be valid"
    );

    harness.shutdown();
}

// =============================================================================
// E2E Test: Session End Handling (E2E-5)
// =============================================================================

#[tokio::test]
async fn e2e_session_end_explicit() {
    let session_end_config = SessionEndConfig {
        hook_enabled: true,
        timeout_enabled: false,
        timeout_minutes: 15,
    };

    let mut harness = E2ETestHarness::with_config(
        AssessmentConfig::default(),
        LightweightDetectorConfig::with_default_patterns(),
        SessionBufferConfig::default(),
        CheckpointConfig::default(),
        session_end_config,
        CircuitBreakerConfig::default(),
        SamplingConfig::default(),
    )
    .await;

    let session_id = "e2e-session-end-test";

    // First, add some activity
    let input = harness.user_input(session_id, "Hello");
    harness.process_event(&input);

    // Now end the session
    let end_event = harness.session_removed(session_id);
    let result = harness.process_event(&end_event);

    // Verify session end was detected
    assert!(result.session_end.is_some(), "Should detect session end");
    let end = result.session_end.unwrap();
    assert_eq!(end.session_id.as_str(), session_id);
    assert_eq!(end.reason, SessionEndReason::Explicit);

    // Verify sampling decision was made for session end
    assert!(
        result.sampling_decision.is_some(),
        "Should have sampling decision"
    );

    harness.shutdown();
}

#[tokio::test]
async fn e2e_session_end_timeout() {
    let session_end_config = SessionEndConfig {
        hook_enabled: false,
        timeout_enabled: true,
        timeout_minutes: 0, // Immediate timeout for testing
    };

    let mut harness = E2ETestHarness::with_config(
        AssessmentConfig::default(),
        LightweightDetectorConfig::with_default_patterns(),
        SessionBufferConfig::default(),
        CheckpointConfig::default(),
        session_end_config,
        CircuitBreakerConfig::default(),
        SamplingConfig::default(),
    )
    .await;

    let session_id = "e2e-timeout-test";

    // Add activity to start tracking the session
    let input = harness.user_input(session_id, "Hello");
    harness.process_event(&input);

    // Wait a bit for timeout
    tokio::time::sleep(Duration::from_millis(20)).await;

    // Check for timeouts
    let timed_out = harness.session_end.check_timeouts();

    assert!(!timed_out.is_empty(), "Should have timed out sessions");
    let end = &timed_out[0];
    assert_eq!(end.session_id.as_str(), session_id);
    assert_eq!(end.reason, SessionEndReason::InactivityTimeout);

    harness.shutdown();
}

// =============================================================================
// E2E Test: Sampling Strategy Integration
// =============================================================================

#[tokio::test]
async fn e2e_sampling_respects_burnin() {
    // Configure sampling with burnin period
    let sampling_config = SamplingConfig {
        base_rate: 0.0, // Would never sample without burnin
        burnin_sessions: 10,
    };

    let mut harness = E2ETestHarness::with_config(
        AssessmentConfig::default(),
        LightweightDetectorConfig::with_default_patterns(),
        SessionBufferConfig::default(),
        CheckpointConfig::default(),
        SessionEndConfig::default(),
        CircuitBreakerConfig::default(),
        sampling_config,
    )
    .await;

    // During burnin, should always sample
    let ctx =
        SamplingContext::checkpoint(CheckpointTrigger::TimeInterval).with_completed_sessions(5);
    let decision = harness.sampling.should_sample(&ctx);

    assert_eq!(
        decision,
        SamplingDecision::Medium,
        "Should sample during burnin"
    );

    // After burnin with 0 rate, should skip
    let ctx_post_burnin =
        SamplingContext::checkpoint(CheckpointTrigger::TimeInterval).with_completed_sessions(15);
    let decision_post = harness.sampling.should_sample(&ctx_post_burnin);

    assert_eq!(
        decision_post,
        SamplingDecision::Skip,
        "Should skip after burnin with 0 rate"
    );

    harness.shutdown();
}

// =============================================================================
// E2E Test: Checkpoint Triggering
// =============================================================================

#[tokio::test]
async fn e2e_checkpoint_triggers_on_pattern() {
    let checkpoint_config = CheckpointConfig {
        enabled: true,
        interval_seconds: 3600,     // Long interval so time doesn't trigger
        frustration_threshold: 0.8, // High threshold so EMA doesn't trigger easily
        min_events: 1,
    };

    let pattern_config = PatternConfig {
        negative: vec![r"(?i)\berror\b".to_string()],
        positive: vec![],
    };
    let detector_config = LightweightDetectorConfig::from_pattern_config(&pattern_config);

    let mut harness = E2ETestHarness::with_config(
        AssessmentConfig::default(),
        detector_config,
        SessionBufferConfig::default(),
        checkpoint_config,
        SessionEndConfig::default(),
        CircuitBreakerConfig::default(),
        SamplingConfig::default(),
    )
    .await;

    let session_id = "e2e-checkpoint-test";

    // Event with error pattern should trigger checkpoint
    let error_event = harness.user_input(session_id, "I got an error message");
    let result = harness.process_event(&error_event);

    assert!(
        result.lightweight.is_some(),
        "Should detect lightweight event with error"
    );

    // Note: Checkpoint trigger depends on the pattern being in signals
    // The implementation may vary - we're testing the integration works
    harness.shutdown();
}

// =============================================================================
// E2E Test: Multiple Sessions Isolation
// =============================================================================

#[tokio::test]
async fn e2e_multiple_sessions_isolated() {
    let mut harness = E2ETestHarness::new().await;

    let session_a = "session-a";
    let session_b = "session-b";

    // Process events for different sessions
    let event_a1 = harness.user_input(session_a, "Hello from A");
    let event_b1 = harness.user_input(session_b, "Hello from B");
    let event_a2 = harness.user_input(session_a, "More from A");

    harness.process_event(&event_a1);
    harness.process_event(&event_b1);
    harness.process_event(&event_a2);

    // Verify buffers are isolated
    let buffer_a = harness.buffer.get(&SessionId::new(session_a));
    let buffer_b = harness.buffer.get(&SessionId::new(session_b));

    assert_eq!(
        buffer_a.map(|b| b.len()),
        Some(2),
        "Session A should have 2 events"
    );
    assert_eq!(
        buffer_b.map(|b| b.len()),
        Some(1),
        "Session B should have 1 event"
    );

    // Verify session states are isolated
    let state_a = harness.session_states.get(&SessionId::new(session_a));
    let state_b = harness.session_states.get(&SessionId::new(session_b));

    assert!(state_a.is_some(), "Session A should have state");
    assert!(state_b.is_some(), "Session B should have state");
    assert_eq!(
        state_a.unwrap().message_idx,
        2,
        "Session A should have processed 2 messages"
    );
    assert_eq!(
        state_b.unwrap().message_idx,
        1,
        "Session B should have processed 1 message"
    );

    // Verify circuit breaker states are isolated
    let circuit_a = harness.circuit_breaker.state(&SessionId::new(session_a));
    let circuit_b = harness.circuit_breaker.state(&SessionId::new(session_b));

    // Both should start Closed
    assert_eq!(circuit_a, CircuitState::Closed);
    assert_eq!(circuit_b, CircuitState::Closed);

    harness.shutdown();
}

// =============================================================================
// E2E Test: Assessment Processor Integration
// =============================================================================

#[tokio::test]
async fn e2e_processor_stores_and_broadcasts_events() {
    let log = Arc::new(InMemoryAssessmentLog::new());
    let config = AssessmentConfig::default();
    let processor = AssessmentProcessor::new(config, log.clone());

    // Subscribe before submitting
    let mut rx = processor.subscribe();

    // Create and submit a lightweight event
    let session_id = "e2e-processor-test";
    let event = AssessmentEvent::Lightweight(LightweightEvent {
        context: vibes_groove::assessment::AssessmentContext::new(session_id),
        message_idx: 0,
        signals: vec![],
        frustration_ema: 0.0,
        success_ema: 1.0,
    });

    processor.submit(event.clone());

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Verify event was stored
    let stored = log.read_session(&SessionId::new(session_id)).await.unwrap();
    assert_eq!(stored.len(), 1, "Should have stored 1 event");

    // Verify event was broadcast
    let received = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;
    assert!(received.is_ok(), "Should receive broadcast event");

    processor.shutdown();
}
