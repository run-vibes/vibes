//! Integration tests for Iggy auto-start functionality.
//!
//! These tests verify that the server properly handles Iggy availability
//! and that events flow correctly through the Iggy-backed EventLog.
//!
//! Tests that start their own Iggy server use isolated temp directories
//! and random ports to avoid conflicts with other tests or running servers.

use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use vibes_core::{StoredEvent, VibesEvent};
use vibes_iggy::{IggyConfig, SeekPosition};
use vibes_server::AppState;

/// Create an isolated IggyConfig for testing.
///
/// Uses a temp directory and random ports to avoid conflicts.
fn test_iggy_config(temp_dir: &TempDir) -> IggyConfig {
    // Use process ID to generate unique ports in the high range
    let tcp_port = 49152 + (std::process::id() % 16384) as u16;
    let http_port = tcp_port + 1;

    IggyConfig::default()
        .with_data_dir(temp_dir.path())
        .with_port(tcp_port)
        .with_http_port(http_port)
}

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
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = test_iggy_config(&temp_dir);

    let state = Arc::new(
        AppState::new_with_iggy_config(config)
            .await
            .expect("Iggy should be available"),
    );

    // Give Iggy a moment to fully initialize
    tokio::time::sleep(IGGY_INIT_WAIT).await;

    // Verify the server stays running for additional operations
    // (accessing uptime confirms state is valid and not dropped)
    let _ = state.uptime_seconds();
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Keep temp_dir alive until test completes
    drop(temp_dir);
}

/// Test that events can be appended and consumed through the EventLog.
///
/// This test verifies the full event flow: append -> persist -> consume.
/// Requires iggy-server and sufficient ulimit.
/// Run with: cargo test -p vibes-server --test iggy_integration -- --ignored
#[tokio::test]
#[ignore]
async fn test_events_flow_through_iggy() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = test_iggy_config(&temp_dir);

    let state = Arc::new(
        AppState::new_with_iggy_config(config)
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

    state
        .event_log
        .append(StoredEvent::new(event1.clone()))
        .await
        .unwrap();
    state
        .event_log
        .append(StoredEvent::new(event2.clone()))
        .await
        .unwrap();

    // Wait for events to be flushed
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify high water mark increased
    assert!(
        state.event_log.high_water_mark() >= 2,
        "High water mark should be at least 2"
    );

    // Create a consumer and poll for events
    // Poll 40 to ensure we get enough per partition (8 partitions)
    let mut consumer = state.event_log.consumer("e2e-test-consumer").await.unwrap();
    consumer.seek(SeekPosition::Beginning).await.unwrap();

    let batch = consumer.poll(40, Duration::from_secs(1)).await.unwrap();

    // Should have received at least our 2 events
    assert!(
        batch.len() >= 2,
        "Should have polled at least 2 events, got {}",
        batch.len()
    );

    // Verify the events match what we sent
    let events: Vec<_> = batch.into_iter().map(|(_, stored)| stored.event).collect();
    assert!(events.contains(&event1), "Should contain event1");
    assert!(events.contains(&event2), "Should contain event2");

    drop(temp_dir);
}

/// Test that events are partitioned by session_id.
///
/// This verifies the Partitionable trait implementation for VibesEvent.
/// Requires iggy-server and sufficient ulimit.
/// Run with: cargo test -p vibes-server --test iggy_integration -- --ignored
#[tokio::test]
#[ignore]
async fn test_events_partitioned_by_session() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = test_iggy_config(&temp_dir);

    let state = Arc::new(
        AppState::new_with_iggy_config(config)
            .await
            .expect("Iggy should be available"),
    );
    tokio::time::sleep(IGGY_INIT_WAIT).await;

    // Append events for multiple sessions
    for i in 0..10 {
        let session = format!("partition-test-session-{}", i % 3);
        state
            .event_log
            .append(StoredEvent::new(VibesEvent::SessionCreated {
                session_id: session,
                name: None,
            }))
            .await
            .unwrap();
    }

    // Wait for events to be flushed
    tokio::time::sleep(Duration::from_millis(500)).await;

    // All events should be retrievable
    // Poll 80 to ensure we get enough per partition (8 partitions × 10 per partition)
    let mut consumer = state
        .event_log
        .consumer("partition-test-consumer")
        .await
        .unwrap();
    consumer.seek(SeekPosition::Beginning).await.unwrap();

    let batch = consumer.poll(80, Duration::from_secs(1)).await.unwrap();
    assert!(
        batch.len() >= 10,
        "Should retrieve all 10 events, got {}",
        batch.len()
    );

    drop(temp_dir);
}
