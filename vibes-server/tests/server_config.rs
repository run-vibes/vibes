//! Tests that server is configured correctly with all features

mod common;

#[tokio::test]
async fn server_state_has_event_log() {
    let (state, _addr) = common::create_test_server().await;
    // EventLog is always present, verify it works
    let hwm = state.event_log.high_water_mark();
    assert_eq!(hwm, 0, "No events initially");
}

#[tokio::test]
async fn server_state_has_pty_manager() {
    let (state, _addr) = common::create_test_server().await;
    let pty_manager = state.pty_manager.read().await;
    let sessions = pty_manager.list_sessions();
    assert!(sessions.is_empty(), "No PTY sessions initially");
}
