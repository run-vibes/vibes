//! Tests that server is configured correctly with all features

mod common;

use vibes_core::EventBus;

#[tokio::test]
async fn server_state_has_event_bus() {
    let (state, _addr) = common::create_test_server().await;
    // EventBus is always present, verify it works
    let _rx = state.event_bus.subscribe();
}

#[tokio::test]
async fn server_state_has_pty_manager() {
    let (state, _addr) = common::create_test_server().await;
    let pty_manager = state.pty_manager.read().await;
    let sessions = pty_manager.list_sessions();
    assert!(sessions.is_empty(), "No PTY sessions initially");
}
