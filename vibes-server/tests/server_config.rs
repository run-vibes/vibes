//! Tests that server is configured correctly with all features

mod common;

use vibes_core::EventBus;

#[tokio::test]
async fn server_with_all_features_has_history() {
    let (state, _addr) = common::create_test_server().await;
    assert!(state.history.is_some(), "History should be enabled");
}

#[tokio::test]
async fn server_state_has_event_bus() {
    let (state, _addr) = common::create_test_server().await;
    // EventBus is always present, verify it works
    let _rx = state.event_bus.subscribe();
}

#[tokio::test]
async fn server_state_has_session_manager() {
    let (state, _addr) = common::create_test_server().await;
    let sessions = state.session_manager.list_sessions().await;
    assert!(sessions.is_empty(), "No sessions initially");
}
