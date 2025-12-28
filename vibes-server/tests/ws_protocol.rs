//! WebSocket protocol integration tests
//!
//! These tests validate the WebSocket protocol behavior including:
//! - Session creation and management
//! - Subscribe/unsubscribe flows
//! - Event broadcasting to multiple clients

mod common;

use common::client::TestClient;
use vibes_core::EventBus;
use vibes_core::events::{ClaudeEvent, VibesEvent};

#[tokio::test]
async fn create_session_returns_session_id() {
    let (_state, addr) = common::create_test_server().await;
    let mut client = TestClient::connect(addr).await;

    let session_id = client.create_session(Some("test-session")).await;

    assert!(!session_id.is_empty());
}

#[tokio::test]
async fn subscribe_receives_ack() {
    let (_state, addr) = common::create_test_server().await;
    let mut client = TestClient::connect(addr).await;

    let session_id = client.create_session(None).await;
    client.subscribe(&[&session_id], false).await;

    // subscribe() already asserts SubscribeAck
}

#[tokio::test]
async fn multiple_clients_receive_same_events() {
    let (state, addr) = common::create_test_server().await;

    let mut client1 = TestClient::connect(addr).await;
    let mut client2 = TestClient::connect(addr).await;

    let session_id = client1.create_session(None).await;

    client1.subscribe(&[&session_id], false).await;
    client2.subscribe(&[&session_id], false).await;

    // Small delay to ensure subscriptions are processed
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    // Publish event directly to event bus
    state
        .event_bus
        .publish(VibesEvent::Claude {
            session_id: session_id.clone(),
            event: ClaudeEvent::TextDelta {
                text: "Hello".to_string(),
            },
        })
        .await;

    // Both clients should receive it
    let msg1 = client1.recv().await;
    let msg2 = client2.recv().await;

    assert_eq!(msg1["type"], "claude", "Expected claude event: {}", msg1);
    assert_eq!(msg2["type"], "claude", "Expected claude event: {}", msg2);
}

#[tokio::test]
async fn unsubscribed_client_receives_no_events() {
    let (state, addr) = common::create_test_server().await;

    // Use a different client to create the session (so this client doesn't own it)
    let mut creator = TestClient::connect(addr).await;
    let session_id = creator.create_session(None).await;

    // Connect a new client that doesn't subscribe
    let mut client = TestClient::connect(addr).await;

    // Publish event
    state
        .event_bus
        .publish(VibesEvent::Claude {
            session_id: session_id.clone(),
            event: ClaudeEvent::TextDelta {
                text: "Hello".to_string(),
            },
        })
        .await;

    // Client should not receive anything (it didn't subscribe)
    client
        .expect_no_message(std::time::Duration::from_millis(50))
        .await;
}
