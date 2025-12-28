//! WebSocket protocol integration tests
//!
//! These tests validate the WebSocket protocol behavior including:
//! - Session creation and management
//! - Event broadcasting to multiple clients

mod common;

use common::client::TestClient;

#[tokio::test]
async fn create_session_returns_session_id() {
    let (_state, addr) = common::create_test_server().await;
    let mut client = TestClient::connect(addr).await;

    let session_id = client.create_session(Some("test-session")).await;

    assert!(!session_id.is_empty());
}

#[tokio::test]
async fn list_sessions_returns_empty_initially() {
    let (_state, addr) = common::create_test_server().await;
    let mut client = TestClient::connect(addr).await;

    let sessions = client.list_sessions().await;
    assert!(sessions.is_empty(), "No sessions initially");
}
