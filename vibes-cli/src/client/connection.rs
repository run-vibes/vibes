//! WebSocket connection handling for the vibes client

use std::time::Duration;

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, warn};
use vibes_server::ws::{ClientMessage, ServerMessage};

/// WebSocket client for communicating with the vibes daemon
pub struct VibesClient {
    /// Sender for outgoing messages
    tx: mpsc::Sender<ClientMessage>,
    /// Receiver for incoming messages
    rx: mpsc::Receiver<ServerMessage>,
}

impl VibesClient {
    /// Connect to the vibes daemon on the default port
    pub async fn connect() -> Result<Self> {
        Self::connect_default().await
    }

    /// Connect to the vibes daemon at the specified URL
    ///
    /// # Arguments
    /// * `url` - WebSocket URL (e.g., "ws://127.0.0.1:7743/ws")
    pub async fn connect_url(url: &str) -> Result<Self> {
        let (ws_stream, _response) = connect_async(url)
            .await
            .with_context(|| format!("Failed to connect to {}", url))?;

        let (ws_sender, ws_receiver) = ws_stream.split();

        // Create channels for bidirectional communication
        let (outgoing_tx, outgoing_rx) = mpsc::channel::<ClientMessage>(32);
        let (incoming_tx, incoming_rx) = mpsc::channel::<ServerMessage>(32);

        // Spawn task to forward outgoing messages to WebSocket
        let outgoing_task = Self::spawn_outgoing_task(outgoing_rx, ws_sender);

        // Spawn task to receive incoming messages from WebSocket
        let incoming_task = Self::spawn_incoming_task(ws_receiver, incoming_tx);

        // Spawn tasks in background
        tokio::spawn(outgoing_task);
        tokio::spawn(incoming_task);

        Ok(Self {
            tx: outgoing_tx,
            rx: incoming_rx,
        })
    }

    /// Connect to the vibes daemon on the default port
    pub async fn connect_default() -> Result<Self> {
        use crate::commands::serve::{DEFAULT_HOST, DEFAULT_PORT};
        let url = format!("ws://{}:{}/ws", DEFAULT_HOST, DEFAULT_PORT);
        Self::connect_url(&url).await
    }

    /// Send a message to the daemon
    pub async fn send(&self, msg: ClientMessage) -> Result<()> {
        self.tx
            .send(msg)
            .await
            .map_err(|_| anyhow::anyhow!("Failed to send message to daemon"))
    }

    /// Receive a message from the daemon
    ///
    /// Returns None if the connection is closed
    pub async fn recv(&mut self) -> Option<ServerMessage> {
        self.rx.recv().await
    }

    /// Receive a message with timeout
    pub async fn recv_timeout(&mut self, timeout: Duration) -> Result<Option<ServerMessage>> {
        match tokio::time::timeout(timeout, self.rx.recv()).await {
            Ok(msg) => Ok(msg),
            Err(_) => anyhow::bail!("Timeout waiting for message from daemon"),
        }
    }

    /// Create a new session and wait for confirmation
    ///
    /// Returns the new session ID
    ///
    /// Note: With PTY mode, sessions are created via PTY attachment instead.
    #[allow(dead_code)]
    pub async fn create_session(&mut self, name: Option<String>) -> Result<String> {
        let request_id = uuid::Uuid::new_v4().to_string();

        self.send(ClientMessage::CreateSession {
            name,
            request_id: request_id.clone(),
        })
        .await?;

        // Wait for response with matching request_id
        let timeout = Duration::from_secs(10);
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            match self.recv_timeout(Duration::from_secs(1)).await {
                Ok(Some(ServerMessage::SessionCreated {
                    request_id: resp_id,
                    session_id,
                    ..
                })) if resp_id == request_id => {
                    return Ok(session_id);
                }
                Ok(Some(ServerMessage::Error { message, .. })) => {
                    anyhow::bail!("Failed to create session: {}", message);
                }
                Ok(Some(_)) => {
                    // Not our response, continue waiting
                    continue;
                }
                Ok(None) => {
                    anyhow::bail!("Connection closed while waiting for session creation");
                }
                Err(_) => {
                    // Timeout on individual recv, continue loop
                    continue;
                }
            }
        }

        anyhow::bail!("Timeout waiting for session creation response")
    }

    /// Send input to a session
    ///
    /// Note: With PTY mode, input is sent via PTY data messages instead.
    #[allow(dead_code)]
    pub async fn send_input(&self, session_id: &str, content: &str) -> Result<()> {
        self.send(ClientMessage::Input {
            session_id: session_id.to_string(),
            content: content.to_string(),
        })
        .await
    }

    /// Respond to a permission request
    #[allow(dead_code)]
    pub async fn respond_permission(
        &self,
        session_id: &str,
        request_id: &str,
        approved: bool,
    ) -> Result<()> {
        self.send(ClientMessage::PermissionResponse {
            session_id: session_id.to_string(),
            request_id: request_id.to_string(),
            approved,
        })
        .await
    }

    /// Request list of active sessions
    pub async fn send_list_sessions(&self, request_id: &str) -> Result<()> {
        self.send(ClientMessage::ListSessions {
            request_id: request_id.to_string(),
        })
        .await
    }

    /// Kill a session
    pub async fn send_kill_session(&self, session_id: &str) -> Result<()> {
        self.send(ClientMessage::KillSession {
            session_id: session_id.to_string(),
        })
        .await
    }

    // === PTY Methods ===

    /// Attach to a PTY session to receive output
    pub async fn attach(&self, session_id: &str) -> Result<()> {
        self.send(ClientMessage::Attach {
            session_id: session_id.to_string(),
        })
        .await
    }

    /// Detach from a PTY session
    pub async fn detach(&self, session_id: &str) -> Result<()> {
        self.send(ClientMessage::Detach {
            session_id: session_id.to_string(),
        })
        .await
    }

    /// Send input to a PTY session (base64 encoded)
    pub async fn pty_input(&self, session_id: &str, data: &str) -> Result<()> {
        self.send(ClientMessage::PtyInput {
            session_id: session_id.to_string(),
            data: data.to_string(),
        })
        .await
    }

    /// Resize a PTY session
    pub async fn pty_resize(&self, session_id: &str, cols: u16, rows: u16) -> Result<()> {
        self.send(ClientMessage::PtyResize {
            session_id: session_id.to_string(),
            cols,
            rows,
        })
        .await
    }

    /// Spawn a task that forwards outgoing messages to the WebSocket
    async fn spawn_outgoing_task<S>(mut rx: mpsc::Receiver<ClientMessage>, mut ws_sender: S)
    where
        S: SinkExt<Message> + Unpin,
        S::Error: std::fmt::Debug,
    {
        while let Some(msg) = rx.recv().await {
            match serde_json::to_string(&msg) {
                Ok(json) => {
                    debug!("Sending: {}", json);
                    if let Err(e) = ws_sender.send(Message::Text(json.into())).await {
                        warn!("Failed to send WebSocket message: {:?}", e);
                        break;
                    }
                }
                Err(e) => {
                    warn!("Failed to serialize message: {}", e);
                }
            }
        }
    }

    /// Spawn a task that receives incoming messages from the WebSocket
    async fn spawn_incoming_task<S>(mut ws_receiver: S, tx: mpsc::Sender<ServerMessage>)
    where
        S: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
    {
        while let Some(result) = ws_receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    let text_str: &str = &text;
                    debug!("Received: {}", text_str);
                    match serde_json::from_str::<ServerMessage>(text_str) {
                        Ok(msg) => {
                            if tx.send(msg).await.is_err() {
                                debug!("Receiver dropped, stopping incoming task");
                                break;
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse server message: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    debug!("Received close message");
                    break;
                }
                Ok(Message::Ping(data)) => {
                    debug!("Received ping: {:?}", data);
                    // Pong is automatically sent by tungstenite
                }
                Ok(Message::Pong(_)) => {
                    // Ignore pong messages
                }
                Ok(Message::Binary(_)) => {
                    debug!("Ignoring binary message");
                }
                Ok(Message::Frame(_)) => {
                    // Raw frame, ignore
                }
                Err(e) => {
                    warn!("WebSocket error: {}", e);
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_serialization() {
        let msg = ClientMessage::CreateSession {
            name: Some("test".to_string()),
            request_id: "req-1".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("create_session"));
        assert!(json.contains("test"));
    }

    #[test]
    fn test_server_message_deserialization() {
        let json = r#"{"type":"session_created","request_id":"req-1","session_id":"sess-1","name":"test"}"#;
        let msg: ServerMessage = serde_json::from_str(json).unwrap();
        match msg {
            ServerMessage::SessionCreated {
                request_id,
                session_id,
                name,
            } => {
                assert_eq!(request_id, "req-1");
                assert_eq!(session_id, "sess-1");
                assert_eq!(name, Some("test".to_string()));
            }
            _ => panic!("Expected SessionCreated"),
        }
    }
}
