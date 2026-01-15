//! WebSocket client for connecting to the vibes daemon.
//!
//! Provides a TuiClient with non-blocking message receiving for the TUI event loop.

use std::time::Duration;

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, warn};
use vibes_server::ws::{ClientMessage, ServerMessage};

/// WebSocket client for the TUI.
///
/// Provides both async send and non-blocking try_recv for use in the TUI event loop.
pub struct TuiClient {
    /// Sender for outgoing messages
    tx: mpsc::Sender<ClientMessage>,
    /// Receiver for incoming messages
    rx: mpsc::Receiver<ServerMessage>,
}

impl std::fmt::Debug for TuiClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TuiClient").finish_non_exhaustive()
    }
}

impl TuiClient {
    /// Connect to the vibes daemon at the specified URL.
    pub async fn connect(url: &str) -> Result<Self> {
        let (ws_stream, _response) = connect_async(url)
            .await
            .with_context(|| format!("Failed to connect to {}", url))?;

        let (ws_sender, ws_receiver) = ws_stream.split();

        // Create channels for bidirectional communication
        let (outgoing_tx, outgoing_rx) = mpsc::channel::<ClientMessage>(32);
        let (incoming_tx, incoming_rx) = mpsc::channel::<ServerMessage>(32);

        // Spawn task to forward outgoing messages to WebSocket
        tokio::spawn(Self::outgoing_task(outgoing_rx, ws_sender));

        // Spawn task to receive incoming messages from WebSocket
        tokio::spawn(Self::incoming_task(ws_receiver, incoming_tx));

        Ok(Self {
            tx: outgoing_tx,
            rx: incoming_rx,
        })
    }

    /// Send a message to the daemon.
    pub async fn send(&self, msg: ClientMessage) -> Result<()> {
        self.tx
            .send(msg)
            .await
            .map_err(|_| anyhow::anyhow!("Failed to send message to daemon"))
    }

    /// Try to receive a message without blocking.
    ///
    /// Returns None if no message is available.
    pub fn try_recv(&mut self) -> Option<ServerMessage> {
        self.rx.try_recv().ok()
    }

    /// Receive a message, blocking until one is available.
    ///
    /// Returns None if the connection is closed.
    pub async fn recv(&mut self) -> Option<ServerMessage> {
        self.rx.recv().await
    }

    /// Receive a message with timeout.
    pub async fn recv_timeout(&mut self, timeout: Duration) -> Result<Option<ServerMessage>> {
        match tokio::time::timeout(timeout, self.rx.recv()).await {
            Ok(msg) => Ok(msg),
            Err(_) => anyhow::bail!("Timeout waiting for message from daemon"),
        }
    }

    /// Request list of active sessions.
    pub async fn list_sessions(&self, request_id: &str) -> Result<()> {
        self.send(ClientMessage::ListSessions {
            request_id: request_id.to_string(),
        })
        .await
    }

    /// Request list of agents.
    pub async fn list_agents(&self, request_id: &str) -> Result<()> {
        self.send(ClientMessage::ListAgents {
            request_id: request_id.to_string(),
        })
        .await
    }

    /// Task that forwards outgoing messages to the WebSocket.
    async fn outgoing_task<S>(mut rx: mpsc::Receiver<ClientMessage>, mut ws_sender: S)
    where
        S: SinkExt<Message> + Unpin,
        S::Error: std::fmt::Debug,
    {
        while let Some(msg) = rx.recv().await {
            match serde_json::to_string(&msg) {
                Ok(json) => {
                    debug!("TUI sending: {}", json);
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

    /// Task that receives incoming messages from the WebSocket.
    async fn incoming_task<S>(mut ws_receiver: S, tx: mpsc::Sender<ServerMessage>)
    where
        S: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
    {
        while let Some(result) = ws_receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    let text_str: &str = &text;
                    debug!("TUI received: {}", text_str);
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
                Ok(
                    Message::Ping(_) | Message::Pong(_) | Message::Binary(_) | Message::Frame(_),
                ) => {
                    // Ignore these message types
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
    fn tui_client_implements_debug() {
        // Compile-time test that TuiClient implements Debug
        fn assert_debug<T: std::fmt::Debug>() {}
        assert_debug::<TuiClient>();
    }
}
