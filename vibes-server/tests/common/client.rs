//! WebSocket test client for protocol testing
//!
//! Provides both low-level WsConnection and high-level TestClient.
//!
//! Note: Some methods may appear unused because they're only used in specific
//! test files and clippy checks each test independently.

use std::net::SocketAddr;
use std::time::Duration;

use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use serde::{Serialize, de::DeserializeOwned};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use uuid::Uuid;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Low-level WebSocket connection
pub struct WsConnection {
    sink: SplitSink<WsStream, Message>,
    stream: SplitStream<WsStream>,
}

impl WsConnection {
    /// Connect to WebSocket endpoint
    pub async fn connect(addr: SocketAddr) -> Self {
        let url = format!("ws://{}/ws", addr);
        let (ws, _) = tokio_tungstenite::connect_async(&url)
            .await
            .expect("Failed to connect");
        let (sink, stream) = ws.split();
        Self { sink, stream }
    }

    /// Send raw text message
    pub async fn send_raw(&mut self, msg: &str) {
        self.sink
            .send(Message::Text(msg.to_string().into()))
            .await
            .unwrap();
    }

    /// Send JSON message
    pub async fn send_json<T: Serialize>(&mut self, msg: &T) {
        let json = serde_json::to_string(msg).unwrap();
        self.send_raw(&json).await;
    }

    /// Receive raw text message
    pub async fn recv_raw(&mut self) -> String {
        loop {
            match self.stream.next().await {
                Some(Ok(Message::Text(text))) => return text.to_string(),
                Some(Ok(Message::Ping(_))) => continue,
                Some(Ok(_)) => continue,
                Some(Err(e)) => panic!("WebSocket error: {}", e),
                None => panic!("WebSocket closed"),
            }
        }
    }

    /// Receive and deserialize JSON message
    pub async fn recv_json<T: DeserializeOwned>(&mut self) -> T {
        let text = self.recv_raw().await;
        serde_json::from_str(&text).expect("Failed to parse JSON")
    }

    /// Receive with timeout, returns None if timeout
    pub async fn recv_timeout(&mut self, duration: Duration) -> Option<String> {
        tokio::time::timeout(duration, self.recv_raw()).await.ok()
    }
}

/// High-level test client with helper methods
pub struct TestClient {
    pub conn: WsConnection,
}

impl TestClient {
    /// Connect to server (consumes initial auth_context message)
    #[allow(dead_code)]
    pub async fn connect(addr: SocketAddr) -> Self {
        let mut conn = WsConnection::connect(addr).await;

        // Server sends auth_context on connect, consume it
        let auth_msg: serde_json::Value = conn.recv_json().await;
        assert_eq!(
            auth_msg["type"], "auth_context",
            "Expected auth_context message on connect"
        );

        Self { conn }
    }

    /// Create a new session, returns session ID
    #[allow(dead_code)]
    pub async fn create_session(&mut self, name: Option<&str>) -> String {
        let request_id = Uuid::new_v4().to_string();
        self.conn
            .send_json(&serde_json::json!({
                "type": "create_session",
                "name": name,
                "request_id": request_id,
            }))
            .await;

        let response: serde_json::Value = self.conn.recv_json().await;
        assert_eq!(
            response["type"], "session_created",
            "Expected session_created but got: {}",
            response
        );
        response["session_id"].as_str().unwrap().to_string()
    }

    /// Subscribe to sessions
    ///
    /// Note: The server only sends SubscribeAck when catch_up=true.
    /// When catch_up=false, subscription happens silently.
    #[allow(dead_code)]
    pub async fn subscribe(&mut self, session_ids: &[&str], catch_up: bool) {
        self.conn
            .send_json(&serde_json::json!({
                "type": "subscribe",
                "session_ids": session_ids,
                "catch_up": catch_up,
            }))
            .await;

        // Server only sends SubscribeAck when catch_up is true
        if catch_up {
            let response: serde_json::Value = self.conn.recv_json().await;
            assert_eq!(
                response["type"], "subscribe_ack",
                "Expected subscribe_ack but got: {}",
                response
            );
        }
        // When catch_up=false, subscription happens silently - no response expected
    }

    /// Send input to session
    #[allow(dead_code)]
    pub async fn send_input(&mut self, session_id: &str, content: &str) {
        self.conn
            .send_json(&serde_json::json!({
                "type": "input",
                "session_id": session_id,
                "content": content,
            }))
            .await;
    }

    /// Receive next message
    #[allow(dead_code)]
    pub async fn recv(&mut self) -> serde_json::Value {
        self.conn.recv_json().await
    }

    /// Assert no message received within duration
    #[allow(dead_code)]
    pub async fn expect_no_message(&mut self, duration: Duration) {
        assert!(
            self.conn.recv_timeout(duration).await.is_none(),
            "Expected no message but received one"
        );
    }
}
