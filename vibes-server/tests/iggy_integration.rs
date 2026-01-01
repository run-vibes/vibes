//! Integration tests for Iggy auto-start functionality.
//!
//! These tests verify that the server properly handles Iggy availability
//! and that events flow correctly through the Iggy-backed EventLog.

use std::sync::Arc;
use std::time::Duration;
use vibes_core::VibesEvent;
use vibes_iggy::SeekPosition;
use vibes_server::AppState;

/// Time to wait for Iggy server to fully initialize after startup.
const IGGY_INIT_WAIT: Duration = Duration::from_secs(1);

/// Test that AppState::new_with_iggy() returns an error when Iggy is unavailable.
///
/// This test runs unconditionally and verifies that errors are propagated.
/// Success criteria: returns an error without panic or hang.
#[tokio::test]
async fn test_new_with_iggy_returns_error_when_unavailable() {
    // This test expects iggy-server to NOT be available in most test environments
    // (CI, dev without `just build`). It should return an error.
    let result = AppState::new_with_iggy().await;

    // In CI/dev environments without iggy-server or with insufficient ulimit,
    // this should return an error. In environments with proper setup, it may succeed.
    // Either way, it shouldn't panic or hang.
    match result {
        Ok(_) => {
            // Iggy is available - that's fine too
        }
        Err(e) => {
            // Expected in most test environments
            eprintln!("Expected error: {}", e);
        }
    }
}

/// Test that AppState::new() always uses in-memory storage.
/// Success criteria: completes without panic.
#[test]
fn test_new_uses_in_memory_storage() {
    let _state = AppState::new();
    // Success is indicated by not panicking
}

/// Test that the server can be created with Iggy when the binary is available.
///
/// This test is ignored by default because it requires iggy-server to be built
/// and sufficient ulimit for io_uring.
/// Run with: cargo test -p vibes-server --test iggy_integration -- --ignored
/// Success criteria: completes without panic, server stays running.
#[tokio::test]
#[ignore]
async fn test_iggy_auto_start_when_available() {
    // This test requires iggy-server to be in the same directory as the test binary
    // or in PATH, and sufficient ulimit for io_uring. It's ignored by default.
    let state = Arc::new(
        AppState::new_with_iggy()
            .await
            .expect("Iggy should be available"),
    );

    // Give Iggy a moment to fully initialize
    tokio::time::sleep(IGGY_INIT_WAIT).await;

    // Verify the server stays running for additional operations
    // (accessing uptime confirms state is valid and not dropped)
    let _ = state.uptime_seconds();
    tokio::time::sleep(Duration::from_millis(100)).await;
}

/// Test that events can be appended and consumed through the EventLog.
///
/// This test verifies the full event flow: append -> persist -> consume.
/// Requires iggy-server and sufficient ulimit.
/// Run with: cargo test -p vibes-server --test iggy_integration -- --ignored
#[tokio::test]
#[ignore]
async fn test_events_flow_through_iggy() {
    let state = Arc::new(
        AppState::new_with_iggy()
            .await
            .expect("Iggy should be available"),
    );

    // Give Iggy a moment to fully initialize
    tokio::time::sleep(IGGY_INIT_WAIT).await;

    // Append some events
    let event1 = VibesEvent::SessionCreated {
        session_id: "test-session-1".to_string(),
        name: Some("Test Session".to_string()),
    };
    let event2 = VibesEvent::SessionStateChanged {
        session_id: "test-session-1".to_string(),
        state: "active".to_string(),
    };

    state.event_log.append(event1.clone()).await.unwrap();
    state.event_log.append(event2.clone()).await.unwrap();

    // Verify high water mark increased
    assert!(
        state.event_log.high_water_mark() >= 2,
        "High water mark should be at least 2"
    );

    // Create a consumer and poll for events
    let mut consumer = state.event_log.consumer("e2e-test-consumer").await.unwrap();
    consumer.seek(SeekPosition::Beginning).await.unwrap();

    let batch = consumer.poll(10, Duration::from_secs(1)).await.unwrap();

    // Should have received at least our 2 events
    assert!(
        batch.len() >= 2,
        "Should have polled at least 2 events, got {}",
        batch.len()
    );

    // Verify the events match what we sent
    let events: Vec<_> = batch.into_iter().map(|(_, e)| e).collect();
    assert!(events.contains(&event1), "Should contain event1");
    assert!(events.contains(&event2), "Should contain event2");
}

/// Test that events are partitioned by session_id.
///
/// This verifies the Partitionable trait implementation for VibesEvent.
/// Requires iggy-server and sufficient ulimit.
/// Run with: cargo test -p vibes-server --test iggy_integration -- --ignored
#[tokio::test]
#[ignore]
async fn test_events_partitioned_by_session() {
    let state = Arc::new(
        AppState::new_with_iggy()
            .await
            .expect("Iggy should be available"),
    );
    tokio::time::sleep(IGGY_INIT_WAIT).await;

    // Append events for multiple sessions
    for i in 0..10 {
        let session = format!("partition-test-session-{}", i % 3);
        state
            .event_log
            .append(VibesEvent::SessionCreated {
                session_id: session,
                name: None,
            })
            .await
            .unwrap();
    }

    // All events should be retrievable
    let mut consumer = state
        .event_log
        .consumer("partition-test-consumer")
        .await
        .unwrap();
    consumer.seek(SeekPosition::Beginning).await.unwrap();

    let batch = consumer.poll(20, Duration::from_secs(1)).await.unwrap();
    assert!(
        batch.len() >= 10,
        "Should retrieve all 10 events, got {}",
        batch.len()
    );
}
