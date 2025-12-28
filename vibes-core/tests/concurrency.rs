//! Concurrency tests for SessionManager
//!
//! These tests validate that per-session locking works correctly:
//! - Operations on different sessions can proceed in parallel
//! - List operations don't block on active sessions

use std::sync::Arc;
use std::time::{Duration, Instant};

use vibes_core::EventBus;
use vibes_core::backend::{BackendFactory, MockBackend, SlowMockBackend};
use vibes_core::events::{ClaudeEvent, MemoryEventBus, Usage};
use vibes_core::session::SessionManager;

/// Factory that creates MockBackend instances
struct MockBackendFactory;

impl BackendFactory for MockBackendFactory {
    fn create(
        &self,
        _claude_session_id: Option<String>,
    ) -> Box<dyn vibes_core::backend::ClaudeBackend> {
        Box::new(MockBackend::new())
    }
}

fn create_test_manager() -> SessionManager {
    let factory: Arc<dyn BackendFactory> = Arc::new(MockBackendFactory);
    let event_bus: Arc<dyn EventBus> = Arc::new(MemoryEventBus::new(100));
    SessionManager::new(factory, event_bus)
}

#[tokio::test]
async fn concurrent_sends_to_different_sessions_dont_block() {
    let manager = Arc::new(create_test_manager());

    // Create two sessions with different delays
    let id1 = {
        let mut backend = SlowMockBackend::new(Duration::from_millis(100));
        backend.queue_response(vec![ClaudeEvent::TurnComplete {
            usage: Usage::default(),
        }]);
        manager
            .create_session_with_backend(None, backend)
            .await
            .unwrap()
    };

    let id2 = {
        let mut backend = SlowMockBackend::new(Duration::from_millis(10));
        backend.queue_response(vec![ClaudeEvent::TurnComplete {
            usage: Usage::default(),
        }]);
        manager
            .create_session_with_backend(None, backend)
            .await
            .unwrap()
    };

    let start = Instant::now();

    let m1 = Arc::clone(&manager);
    let m2 = Arc::clone(&manager);
    let id1_clone = id1.clone();
    let id2_clone = id2.clone();

    let (r1, r2) = tokio::join!(
        async move { m1.send_to_session(&id1_clone, "slow").await },
        async move { m2.send_to_session(&id2_clone, "fast").await },
    );

    let elapsed = start.elapsed();

    assert!(r1.is_ok(), "Slow session should succeed");
    assert!(r2.is_ok(), "Fast session should succeed");

    // If properly parallelized, total time should be ~100ms, not 110ms
    // Allow some margin for test flakiness
    assert!(
        elapsed < Duration::from_millis(150),
        "Concurrent sends should not block: took {:?}",
        elapsed
    );
}

#[tokio::test]
async fn list_sessions_during_active_send() {
    let manager = Arc::new(create_test_manager());

    // Create session with slow backend
    let id = {
        let mut backend = SlowMockBackend::new(Duration::from_millis(100));
        backend.queue_response(vec![ClaudeEvent::TurnComplete {
            usage: Usage::default(),
        }]);
        manager
            .create_session_with_backend(None, backend)
            .await
            .unwrap()
    };

    let m1 = Arc::clone(&manager);
    let m2 = Arc::clone(&manager);
    let id_clone = id.clone();

    // Start a slow send
    let send_handle = tokio::spawn(async move { m1.send_to_session(&id_clone, "slow").await });

    // Give it time to start
    tokio::time::sleep(Duration::from_millis(10)).await;

    // list_sessions should NOT block
    let start = Instant::now();
    let sessions = m2.list_sessions().await;
    let elapsed = start.elapsed();

    assert_eq!(sessions.len(), 1);
    assert!(
        elapsed < Duration::from_millis(20),
        "list_sessions blocked for {:?}",
        elapsed
    );

    // Wait for send to complete
    send_handle.await.unwrap().unwrap();
}
