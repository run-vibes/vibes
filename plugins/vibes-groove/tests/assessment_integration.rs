//! Integration tests for the assessment framework.
//!
//! These tests validate the end-to-end assessment pipeline including:
//! - Event submission and processing
//! - Configuration-controlled behavior
//! - UUIDv7 event ID generation and lineage tracking

use std::sync::Arc;
use std::time::Duration;

use vibes_groove::assessment::{
    AssessmentConfig, AssessmentContext, AssessmentEvent, AssessmentLog, AssessmentProcessor,
    InMemoryAssessmentLog, LightweightEvent, SessionId,
};

/// Helper function to create a lightweight assessment event.
fn make_lightweight_event(session: &str, msg_idx: u32) -> AssessmentEvent {
    AssessmentEvent::Lightweight(LightweightEvent {
        context: AssessmentContext::new(session),
        message_idx: msg_idx,
        signals: vec![],
        frustration_ema: 0.0,
        success_ema: 1.0,
    })
}

#[tokio::test]
async fn assessment_pipeline_end_to_end() {
    // Setup processor with InMemoryAssessmentLog
    let log = Arc::new(InMemoryAssessmentLog::new());
    let config = AssessmentConfig::default();
    let processor = AssessmentProcessor::new(config, log.clone());

    // Subscribe before submitting
    let mut rx = processor.subscribe();

    // Submit 5 events
    let session_id = "e2e-test-session";
    for i in 0..5 {
        let event = make_lightweight_event(session_id, i);
        processor.submit(event);
    }

    // Give the background task time to process all events
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify events were logged
    let events = log.read_session(&SessionId::new(session_id)).await.unwrap();
    assert_eq!(events.len(), 5, "Expected 5 events in the log");

    // Verify message indices
    for (i, event) in events.iter().enumerate() {
        if let AssessmentEvent::Lightweight(e) = event {
            assert_eq!(e.message_idx, i as u32, "Event {} has wrong message_idx", i);
        } else {
            panic!("Expected Lightweight event");
        }
    }

    // Verify subscription received events
    let mut received_count = 0;
    while let Ok(Ok(event)) = tokio::time::timeout(Duration::from_millis(10), rx.recv()).await {
        assert_eq!(event.session_id().as_str(), session_id);
        received_count += 1;
    }
    assert_eq!(received_count, 5, "Expected 5 events from subscription");

    processor.shutdown();
}

#[tokio::test]
async fn assessment_config_controls_behavior() {
    // Setup with disabled config
    let log = Arc::new(InMemoryAssessmentLog::new());
    let config = AssessmentConfig {
        enabled: false,
        ..Default::default()
    };

    let processor = AssessmentProcessor::new(config, log.clone());

    assert!(
        !processor.is_enabled(),
        "Processor should report as disabled"
    );

    // Submit event
    let event = make_lightweight_event("disabled-session", 0);
    processor.submit(event);

    // Give time for any potential processing
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Verify log is empty
    assert!(
        log.is_empty(),
        "Log should be empty when assessment is disabled"
    );

    processor.shutdown();
}

#[tokio::test]
async fn assessment_events_have_correct_lineage() {
    // Create event
    let session_id = "lineage-test-session";
    let event = make_lightweight_event(session_id, 42);

    // Verify UUIDv7 event ID (get_version_num() == 7)
    let event_id = event.event_id();
    assert_eq!(
        event_id.as_uuid().get_version_num(),
        7,
        "Event ID should be UUIDv7"
    );

    // Verify timestamp can be extracted from UUIDv7
    let timestamp = event_id.timestamp();
    assert!(
        timestamp.is_some(),
        "UUIDv7 should have extractable timestamp"
    );

    // Verify session ID propagation
    assert_eq!(
        event.session_id().as_str(),
        session_id,
        "Session ID should propagate correctly"
    );

    // Verify context accessor works
    let ctx = event.context();
    assert_eq!(ctx.session_id.as_str(), session_id);
    assert_eq!(ctx.event_id, *event_id);
}

#[tokio::test]
async fn assessment_events_preserve_session_lineage_through_pipeline() {
    // Setup processor
    let log = Arc::new(InMemoryAssessmentLog::new());
    let config = AssessmentConfig::default();
    let processor = AssessmentProcessor::new(config, log.clone());

    // Create events for same session
    let session_id = "lineage-pipeline-session";
    let event1 = make_lightweight_event(session_id, 0);
    let event2 = make_lightweight_event(session_id, 1);

    // Capture event IDs before submission
    let event1_id = *event1.event_id();
    let event2_id = *event2.event_id();

    // Event IDs should be different (unique per event)
    assert_ne!(
        event1_id.as_uuid(),
        event2_id.as_uuid(),
        "Each event should have a unique ID"
    );

    // Submit events
    processor.submit(event1);
    processor.submit(event2);

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Read back from log
    let events = log.read_session(&SessionId::new(session_id)).await.unwrap();
    assert_eq!(events.len(), 2);

    // Verify all events share the same session ID
    for event in &events {
        assert_eq!(event.session_id().as_str(), session_id);
    }

    // Verify event IDs were preserved through the pipeline
    let logged_ids: Vec<_> = events.iter().map(|e| e.event_id().as_uuid()).collect();
    assert!(
        logged_ids.contains(&event1_id.as_uuid()),
        "First event ID should be preserved"
    );
    assert!(
        logged_ids.contains(&event2_id.as_uuid()),
        "Second event ID should be preserved"
    );

    processor.shutdown();
}

#[tokio::test]
async fn assessment_multiple_sessions_are_isolated() {
    let log = Arc::new(InMemoryAssessmentLog::new());
    let config = AssessmentConfig::default();
    let processor = AssessmentProcessor::new(config, log.clone());

    // Submit events for different sessions
    processor.submit(make_lightweight_event("session-a", 0));
    processor.submit(make_lightweight_event("session-b", 0));
    processor.submit(make_lightweight_event("session-a", 1));
    processor.submit(make_lightweight_event("session-c", 0));
    processor.submit(make_lightweight_event("session-b", 1));

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify session isolation
    let session_a = log
        .read_session(&SessionId::new("session-a"))
        .await
        .unwrap();
    let session_b = log
        .read_session(&SessionId::new("session-b"))
        .await
        .unwrap();
    let session_c = log
        .read_session(&SessionId::new("session-c"))
        .await
        .unwrap();

    assert_eq!(session_a.len(), 2, "Session A should have 2 events");
    assert_eq!(session_b.len(), 2, "Session B should have 2 events");
    assert_eq!(session_c.len(), 1, "Session C should have 1 event");

    // Verify correct events in each session
    for event in &session_a {
        assert_eq!(event.session_id().as_str(), "session-a");
    }
    for event in &session_b {
        assert_eq!(event.session_id().as_str(), "session-b");
    }
    for event in &session_c {
        assert_eq!(event.session_id().as_str(), "session-c");
    }

    processor.shutdown();
}
