//! Firehose event broadcasting integration tests
//!
//! Validates that VibesEvents are broadcast when:
//! - Clients connect/disconnect
//! - Sessions are created/removed
//!
//! The firehose endpoint subscribes to these events, so they must be
//! broadcast for the firehose to show any activity.

mod common;

use common::client::TestClient;
use std::time::Duration;
use vibes_core::VibesEvent;
use vibes_core::pty::PtyConfig;

/// Test that ClientConnected event is broadcast when WebSocket client connects
#[tokio::test]
async fn broadcasts_client_connected_on_websocket_connect() {
    let (state, addr) = common::create_test_server().await;

    // Subscribe to events BEFORE client connects
    let mut events_rx = state.subscribe_events();

    // Connect a test client
    let _client = TestClient::connect(addr).await;

    // Should receive ClientConnected event
    let event = tokio::time::timeout(Duration::from_secs(1), events_rx.recv())
        .await
        .expect("Timeout waiting for event")
        .expect("Channel closed");

    match event {
        (_offset, VibesEvent::ClientConnected { client_id }) => {
            assert!(!client_id.is_empty(), "client_id should not be empty");
        }
        other => panic!("Expected ClientConnected, got {:?}", other),
    }
}

/// Test that SessionCreated event is broadcast when session is created
#[tokio::test]
async fn broadcasts_session_created_on_attach() {
    // Use mock mode so this works in CI without Claude installed
    let pty_config = PtyConfig {
        mock_mode: true,
        ..Default::default()
    };
    let (state, addr) = common::create_test_server_with_pty_config(pty_config).await;

    // Subscribe to events BEFORE session is created
    let mut events_rx = state.subscribe_events();

    // Connect and create a session
    let mut client = TestClient::connect(addr).await;

    // Consume and verify ClientConnected event first
    let client_connected_event = tokio::time::timeout(Duration::from_secs(1), events_rx.recv())
        .await
        .expect("Timeout waiting for ClientConnected event")
        .expect("Channel closed before ClientConnected event");

    match client_connected_event {
        (_offset, VibesEvent::ClientConnected { client_id }) => {
            assert!(!client_id.is_empty(), "client_id should not be empty");
        }
        other => panic!(
            "Expected ClientConnected before SessionCreated, got {:?}",
            other
        ),
    }

    let session_id = client.create_session(Some("test-session")).await;

    // Should receive SessionCreated event
    let event = tokio::time::timeout(Duration::from_secs(1), events_rx.recv())
        .await
        .expect("Timeout waiting for SessionCreated event")
        .expect("Channel closed");

    match event {
        (
            _offset,
            VibesEvent::SessionCreated {
                session_id: event_session_id,
                name,
            },
        ) => {
            assert_eq!(event_session_id, session_id);
            assert_eq!(name, Some("test-session".to_string()));
        }
        other => panic!("Expected SessionCreated, got {:?}", other),
    }
}

/// Test that ClientDisconnected event is broadcast when WebSocket client disconnects
#[tokio::test]
async fn broadcasts_client_disconnected_on_websocket_close() {
    let (state, addr) = common::create_test_server().await;

    // Subscribe to events
    let mut events_rx = state.subscribe_events();

    // Connect a client
    let client = TestClient::connect(addr).await;

    // Consume ClientConnected event (if it exists)
    let _ = tokio::time::timeout(Duration::from_millis(100), events_rx.recv()).await;

    // Drop the client to close the connection
    drop(client);

    // Should receive ClientDisconnected event
    let event = tokio::time::timeout(Duration::from_secs(1), events_rx.recv())
        .await
        .expect("Timeout waiting for ClientDisconnected event")
        .expect("Channel closed");

    match event {
        (_offset, VibesEvent::ClientDisconnected { client_id }) => {
            assert!(!client_id.is_empty(), "client_id should not be empty");
        }
        other => panic!("Expected ClientDisconnected, got {:?}", other),
    }
}

/// Test that SessionRemoved event is broadcast when session is killed
#[tokio::test]
async fn broadcasts_session_removed_on_kill() {
    // Use mock mode
    let pty_config = PtyConfig {
        mock_mode: true,
        ..Default::default()
    };
    let (state, addr) = common::create_test_server_with_pty_config(pty_config).await;

    // Connect and create a session first
    let mut client = TestClient::connect(addr).await;
    let session_id = client.create_session(Some("test-session")).await;

    // Now subscribe to events
    let mut events_rx = state.subscribe_events();

    // Kill the session
    client
        .conn
        .send_json(&serde_json::json!({
            "type": "kill_session",
            "session_id": session_id,
        }))
        .await;

    // Consume the ServerMessage response
    let _ = client.recv().await;

    // Should receive SessionRemoved event
    let event = tokio::time::timeout(Duration::from_secs(1), events_rx.recv())
        .await
        .expect("Timeout waiting for SessionRemoved event")
        .expect("Channel closed");

    match event {
        (
            _offset,
            VibesEvent::SessionRemoved {
                session_id: event_session_id,
                reason,
            },
        ) => {
            assert_eq!(event_session_id, session_id);
            assert!(!reason.is_empty(), "reason should not be empty");
        }
        other => panic!("Expected SessionRemoved, got {:?}", other),
    }
}
