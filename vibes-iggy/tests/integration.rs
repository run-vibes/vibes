//! Integration tests requiring a running Iggy server.
//!
//! These tests validate the full Iggy SDK integration including:
//! - Connection and authentication
//! - Stream/topic creation
//! - Event append with partitioning
//! - Consumer polling and offset commit
//!
//! # Running Tests
//!
//! Option 1: Use an externally running server (recommended for development):
//! ```bash
//! # Start server manually first:
//! IGGY_TCP_ADDRESS=127.0.0.1:8091 IGGY_ROOT_USERNAME=iggy IGGY_ROOT_PASSWORD=iggy ./target/debug/iggy-server
//!
//! # Run tests:
//! IGGY_TEST_PORT=8091 cargo test -p vibes-iggy --test integration -- --ignored
//! ```
//!
//! Option 2: Let tests start the server (requires sufficient memory for 48 shards):
//! ```bash
//! cargo test -p vibes-iggy --test integration -- --ignored
//! ```

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use vibes_iggy::{EventLog, IggyConfig, IggyEventLog, IggyManager, Partitionable, SeekPosition};

/// Test event type for integration tests.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestEvent {
    session_id: String,
    data: String,
}

impl Partitionable for TestEvent {
    fn partition_key(&self) -> Option<&str> {
        Some(&self.session_id)
    }
}

/// Set up test environment.
///
/// If `IGGY_TEST_PORT` is set, connects to an externally running server.
/// Otherwise, starts a new server instance (requires sufficient memory).
async fn setup() -> (Option<Arc<IggyManager>>, IggyEventLog<TestEvent>) {
    // Check if using external server
    if let Ok(port) = std::env::var("IGGY_TEST_PORT") {
        let port: u16 = port.parse().expect("Invalid IGGY_TEST_PORT");

        let config = IggyConfig::default().with_port(port);
        let manager = Arc::new(IggyManager::new(config));

        let log = IggyEventLog::new(Arc::clone(&manager));
        log.connect()
            .await
            .expect("Failed to connect to external server");

        return (None, log); // Don't return manager to prevent shutdown
    }

    // Start our own server
    eprintln!("Starting internal Iggy server...");
    let config = IggyConfig::default();
    let manager = Arc::new(IggyManager::new(config));

    // Start Iggy server
    manager.start().await.expect("Failed to start Iggy");

    // Wait for startup (may need more time if memory-constrained)
    tokio::time::sleep(Duration::from_millis(2000)).await;

    let log = IggyEventLog::new(Arc::clone(&manager));
    log.connect().await.expect("Failed to connect");

    (Some(manager), log)
}

#[tokio::test]
#[ignore]
async fn test_append_and_poll_roundtrip() {
    let (_manager, log) = setup().await;

    let event = TestEvent {
        session_id: "integration-test".to_string(),
        data: "Integration Test Data".to_string(),
    };

    log.append(event.clone()).await.unwrap();

    let mut consumer = log.consumer("integration-consumer").await.unwrap();
    consumer.seek(SeekPosition::Beginning).await.unwrap();

    let batch = consumer.poll(10, Duration::from_secs(1)).await.unwrap();

    assert!(!batch.is_empty(), "Should have polled at least one event");
}

#[tokio::test]
#[ignore]
async fn test_partition_by_session_id() {
    let (_manager, log) = setup().await;

    // Append events for different sessions
    for i in 0..10 {
        log.append(TestEvent {
            session_id: format!("session-{}", i % 3), // 3 different sessions
            data: format!("Event {}", i),
        })
        .await
        .unwrap();
    }

    // All events should be retrievable
    let mut consumer = log.consumer("partition-test").await.unwrap();
    consumer.seek(SeekPosition::Beginning).await.unwrap();

    let batch = consumer.poll(20, Duration::from_secs(1)).await.unwrap();
    assert!(batch.len() >= 10, "Should retrieve all 10 events");
}

#[tokio::test]
#[ignore]
async fn test_consumer_offset_commit() {
    let (_manager, log) = setup().await;

    // Append some events
    for i in 0..5 {
        log.append(TestEvent {
            session_id: "commit-test".to_string(),
            data: format!("Event {}", i),
        })
        .await
        .unwrap();
    }

    // Poll and commit
    let mut consumer = log.consumer("commit-test-group").await.unwrap();
    consumer.seek(SeekPosition::Beginning).await.unwrap();

    let batch = consumer.poll(3, Duration::from_secs(1)).await.unwrap();
    if let Some(offset) = batch.last_offset() {
        consumer.commit(offset).await.unwrap();
    }

    // Committed offset should be updated
    assert!(
        consumer.committed_offset() > 0,
        "Should have committed offset"
    );
}

#[tokio::test]
#[ignore]
async fn test_high_water_mark_increments() {
    let (_manager, log) = setup().await;

    let initial = log.high_water_mark();

    log.append(TestEvent {
        session_id: "hwm-test".to_string(),
        data: "test".to_string(),
    })
    .await
    .unwrap();

    assert_eq!(
        log.high_water_mark(),
        initial + 1,
        "High water mark should increment"
    );
}
