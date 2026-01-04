//! WebSocket assessment endpoint integration tests
//!
//! These tests validate the /ws/assessment WebSocket endpoint behavior including:
//! - Connection and initial batch request
//! - Session filtering
//! - Fetch older pagination

mod common;

use std::net::SocketAddr;
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Connect to the assessment WebSocket endpoint
async fn connect_assessment(addr: SocketAddr) -> WsStream {
    let url = format!("ws://{}/ws/assessment", addr);
    let (ws, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("Failed to connect to assessment WebSocket");
    ws
}

/// Send JSON message to WebSocket
async fn send_json(ws: &mut WsStream, msg: &serde_json::Value) {
    let json = serde_json::to_string(msg).unwrap();
    ws.send(Message::Text(json.into())).await.unwrap();
}

/// Receive JSON message from WebSocket with timeout
async fn recv_json_timeout(ws: &mut WsStream, timeout: Duration) -> Option<serde_json::Value> {
    let result = tokio::time::timeout(timeout, async {
        loop {
            match ws.next().await {
                Some(Ok(Message::Text(text))) => {
                    return serde_json::from_str::<serde_json::Value>(&text).ok();
                }
                Some(Ok(Message::Ping(_))) => continue,
                Some(Ok(_)) => continue,
                Some(Err(_)) | None => return None,
            }
        }
    })
    .await;

    result.ok().flatten()
}

#[tokio::test]
async fn assessment_ws_connects_successfully() {
    let (_state, addr) = common::create_test_server().await;
    let mut ws = connect_assessment(addr).await;

    // Connection should succeed - verify by sending a message
    send_json(
        &mut ws,
        &serde_json::json!({
            "type": "set_filters"
        }),
    )
    .await;

    // Should get a response
    let response = recv_json_timeout(&mut ws, Duration::from_secs(2)).await;
    assert!(
        response.is_some(),
        "Should receive response after connecting"
    );
}

#[tokio::test]
async fn assessment_ws_responds_to_set_filters() {
    let (_state, addr) = common::create_test_server().await;
    let mut ws = connect_assessment(addr).await;

    // Send set_filters message to request initial events
    send_json(
        &mut ws,
        &serde_json::json!({
            "type": "set_filters"
        }),
    )
    .await;

    // Should receive a batch response
    let response = recv_json_timeout(&mut ws, Duration::from_secs(2)).await;
    assert!(response.is_some(), "Should receive response to set_filters");

    let msg = response.unwrap();
    assert_eq!(
        msg["type"], "assessment_events_batch",
        "Response should be assessment_events_batch"
    );
    assert!(
        msg["events"].is_array(),
        "Batch should contain events array"
    );
}

#[tokio::test]
async fn assessment_ws_set_filters_with_session() {
    let (_state, addr) = common::create_test_server().await;
    let mut ws = connect_assessment(addr).await;

    // Send set_filters with session filter
    send_json(
        &mut ws,
        &serde_json::json!({
            "type": "set_filters",
            "session": "test-session-id"
        }),
    )
    .await;

    // Should receive a batch response (empty, since no events exist for this session)
    let response = recv_json_timeout(&mut ws, Duration::from_secs(2)).await;
    assert!(
        response.is_some(),
        "Should receive response to set_filters with session"
    );

    let msg = response.unwrap();
    assert_eq!(msg["type"], "assessment_events_batch");

    // Events array should be empty since we have no events for this session
    let events = msg["events"].as_array().unwrap();
    assert!(events.is_empty(), "No events should exist for fake session");
}

#[tokio::test]
async fn assessment_ws_set_filters_clears_session() {
    let (_state, addr) = common::create_test_server().await;
    let mut ws = connect_assessment(addr).await;

    // First, set a session filter
    send_json(
        &mut ws,
        &serde_json::json!({
            "type": "set_filters",
            "session": "test-session"
        }),
    )
    .await;

    let _ = recv_json_timeout(&mut ws, Duration::from_secs(2)).await;

    // Now clear the session filter by sending null
    send_json(
        &mut ws,
        &serde_json::json!({
            "type": "set_filters",
            "session": null
        }),
    )
    .await;

    // Should receive a new batch (clearing session filter triggers refresh)
    let response = recv_json_timeout(&mut ws, Duration::from_secs(2)).await;
    assert!(
        response.is_some(),
        "Should receive response when clearing session filter"
    );

    let msg = response.unwrap();
    assert_eq!(msg["type"], "assessment_events_batch");
}

#[tokio::test]
async fn assessment_ws_fetch_older_requires_event_id() {
    let (_state, addr) = common::create_test_server().await;
    let mut ws = connect_assessment(addr).await;

    // First request initial events
    send_json(
        &mut ws,
        &serde_json::json!({
            "type": "set_filters"
        }),
    )
    .await;
    let _ = recv_json_timeout(&mut ws, Duration::from_secs(2)).await;

    // Send fetch_older with a valid UUID
    send_json(
        &mut ws,
        &serde_json::json!({
            "type": "fetch_older",
            "before_event_id": "01936f8a-1234-7000-8000-000000000001"
        }),
    )
    .await;

    // Should receive a batch response
    let response = recv_json_timeout(&mut ws, Duration::from_secs(2)).await;
    assert!(response.is_some(), "Should receive response to fetch_older");

    let msg = response.unwrap();
    assert_eq!(msg["type"], "assessment_events_batch");
}

#[tokio::test]
async fn assessment_ws_batch_contains_pagination_info() {
    let (_state, addr) = common::create_test_server().await;
    let mut ws = connect_assessment(addr).await;

    // Request initial events
    send_json(
        &mut ws,
        &serde_json::json!({
            "type": "set_filters"
        }),
    )
    .await;

    let response = recv_json_timeout(&mut ws, Duration::from_secs(2)).await;
    assert!(response.is_some());

    let msg = response.unwrap();

    // Batch should contain pagination fields
    assert!(
        msg.get("oldest_event_id").is_some(),
        "Should have oldest_event_id field"
    );
    assert!(msg.get("has_more").is_some(), "Should have has_more field");
}
