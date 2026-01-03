//! Integration tests requiring a running Iggy server.
//!
//! These tests validate the full Iggy SDK integration including:
//! - Connection and authentication
//! - Stream/topic creation
//! - Event append
//! - Consumer polling and offset commit
//!
//! # Running Tests
//!
//! These tests are in the `iggy-server` test group. Run them with:
//!
//! ```bash
//! # Run only iggy-server tests
//! cargo nextest run -E 'test-group(iggy-server)'
//!
//! # Run all tests EXCEPT iggy-server (for quick feedback)
//! cargo nextest run -E 'not test-group(iggy-server)'
//!
//! # With external server (faster, recommended for development):
//! IGGY_TEST_PORT=8090 cargo nextest run -E 'test-group(iggy-server)'
//! ```
//!
//! The tests will start their own Iggy server if `IGGY_TEST_PORT` is not set.
//! When starting an internal server, tests use an isolated temp directory to
//! avoid interference with production data.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// Port pool for test isolation.
///
/// Each test gets a unique pair of ports (TCP and HTTP) from this pool.
/// The pool provides 32 port pairs, which is more than enough for parallel tests.
mod port_pool {
    use super::*;

    /// Base port for the pool (in ephemeral range)
    const BASE_PORT: u16 = 50000;

    /// Number of port pairs available (each test needs 2 ports: TCP + HTTP)
    const POOL_SIZE: usize = 32;

    /// Port spacing between allocations (2 ports per test: TCP and HTTP)
    const PORT_SPACING: u16 = 2;

    /// Global counter for port allocation
    static NEXT_SLOT: AtomicUsize = AtomicUsize::new(0);

    /// Allocate a unique port pair for a test.
    ///
    /// Returns (tcp_port, http_port). The allocation wraps around after
    /// POOL_SIZE allocations, but tests complete before wrap-around in practice.
    pub fn allocate() -> (u16, u16) {
        let slot = NEXT_SLOT.fetch_add(1, Ordering::SeqCst) % POOL_SIZE;
        let tcp_port = BASE_PORT + (slot as u16 * PORT_SPACING);
        let http_port = tcp_port + 1;
        (tcp_port, http_port)
    }
}

use serde::{Deserialize, Serialize};
use tempfile::TempDir;
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

/// Test harness that holds the Iggy server and temp directory.
///
/// The temp directory is kept alive for the lifetime of the harness.
/// When the harness is dropped, the temp directory and all its contents
/// are cleaned up automatically.
struct TestHarness {
    /// The Iggy manager (only set when using internal server).
    _manager: Option<Arc<IggyManager>>,
    /// Temp directory for isolated test data (only set when using internal server).
    _temp_dir: Option<TempDir>,
}

/// Set up test environment.
///
/// If `IGGY_TEST_PORT` is set, connects to an externally running server.
/// Otherwise, starts a new server instance with an isolated temp directory.
async fn setup() -> (TestHarness, IggyEventLog<TestEvent>) {
    // Check if using external server
    if let Ok(port) = std::env::var("IGGY_TEST_PORT") {
        let port: u16 = port.parse().expect("Invalid IGGY_TEST_PORT");

        let config = IggyConfig::default().with_port(port);
        let manager = Arc::new(IggyManager::new(config));

        let log = IggyEventLog::new(Arc::clone(&manager));
        log.connect()
            .await
            .expect("Failed to connect to external server");

        let harness = TestHarness {
            _manager: None, // Don't own external server
            _temp_dir: None,
        };
        return (harness, log);
    }

    // Start our own server with isolated temp directory and unique port from pool
    let (tcp_port, http_port) = port_pool::allocate();
    eprintln!(
        "Starting internal Iggy server on port {} (pid={})...",
        tcp_port,
        std::process::id()
    );
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let config = IggyConfig::default()
        .with_data_dir(temp_dir.path())
        .with_port(tcp_port)
        .with_http_port(http_port);

    let manager = Arc::new(IggyManager::new(config));

    // Start Iggy server
    manager.start().await.expect("Failed to start Iggy");

    // Wait for startup (may need more time if memory-constrained)
    tokio::time::sleep(Duration::from_millis(2000)).await;

    let log = IggyEventLog::new(Arc::clone(&manager));
    log.connect().await.expect("Failed to connect");

    let harness = TestHarness {
        _manager: Some(manager),
        _temp_dir: Some(temp_dir),
    };
    (harness, log)
}

#[tokio::test]
async fn test_append_and_poll_roundtrip() {
    let (_harness, log) = setup().await;

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
async fn test_partition_by_session_id() {
    let (_harness, log) = setup().await;

    // Append events for different sessions (partition key is ignored with single partition)
    for i in 0..10 {
        log.append(TestEvent {
            session_id: format!("session-{}", i % 3), // 3 different sessions
            data: format!("Event {}", i),
        })
        .await
        .unwrap();
    }

    // Wait for Iggy to flush events to disk
    tokio::time::sleep(Duration::from_millis(500)).await;

    // All events should be retrievable from the single partition
    let mut consumer = log.consumer("partition-test").await.unwrap();
    consumer.seek(SeekPosition::Beginning).await.unwrap();

    let batch = consumer.poll(20, Duration::from_secs(1)).await.unwrap();
    assert!(batch.len() >= 10, "Should retrieve all 10 events");
}

#[tokio::test]
async fn test_consumer_offset_commit() {
    let (_harness, log) = setup().await;

    // Append some events
    for i in 0..5 {
        log.append(TestEvent {
            session_id: "commit-test".to_string(),
            data: format!("Event {}", i),
        })
        .await
        .unwrap();
    }

    // Wait for Iggy to flush events
    tokio::time::sleep(Duration::from_millis(100)).await;

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
async fn test_high_water_mark_increments() {
    let (_harness, log) = setup().await;

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
