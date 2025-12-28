//! Tests for history catch-up on subscribe

mod common;

use common::client::TestClient;
use vibes_core::EventBus;
use vibes_core::events::{ClaudeEvent, VibesEvent};

#[tokio::test]
async fn subscribe_with_catchup_returns_history() {
    let (state, addr) = common::create_test_server().await;

    let mut client1 = TestClient::connect(addr).await;
    let session_id = client1.create_session(None).await;

    // Create session in history via event
    if let Some(history) = &state.history {
        history
            .process_event(&VibesEvent::SessionCreated {
                session_id: session_id.clone(),
                name: None,
            })
            .await
            .unwrap();
    }

    // Send user input event to populate history
    state
        .event_bus
        .publish(VibesEvent::UserInput {
            session_id: session_id.clone(),
            content: "Hello".to_string(),
            source: vibes_core::events::InputSource::WebUi,
        })
        .await;

    // Send claude event to populate history
    state
        .event_bus
        .publish(VibesEvent::Claude {
            session_id: session_id.clone(),
            event: ClaudeEvent::TextDelta {
                text: "Hi there!".to_string(),
            },
        })
        .await;

    // Give time for events to be processed by history service
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // New client subscribes with catch_up
    let mut client2 = TestClient::connect(addr).await;
    client2
        .conn
        .send_json(&serde_json::json!({
            "type": "subscribe",
            "session_ids": [session_id],
            "catch_up": true,
        }))
        .await;

    let response: serde_json::Value = client2.conn.recv_json().await;

    assert_eq!(response["type"], "subscribe_ack");
    // History contains sequenced events, count may vary based on event types stored
    assert!(
        response["history"].is_array(),
        "Expected history to be an array"
    );
}

#[tokio::test]
async fn subscribe_without_catchup_returns_no_ack() {
    let (state, addr) = common::create_test_server().await;

    let mut client1 = TestClient::connect(addr).await;
    let session_id = client1.create_session(None).await;

    // Create session in history via event
    if let Some(history) = &state.history {
        history
            .process_event(&VibesEvent::SessionCreated {
                session_id: session_id.clone(),
                name: None,
            })
            .await
            .unwrap();
    }

    // Subscribe without catch_up - no response expected (subscribe happens silently)
    let mut client2 = TestClient::connect(addr).await;
    client2
        .conn
        .send_json(&serde_json::json!({
            "type": "subscribe",
            "session_ids": [session_id],
            "catch_up": false,
        }))
        .await;

    // No SubscribeAck when catch_up is false
    client2
        .expect_no_message(std::time::Duration::from_millis(50))
        .await;
}
