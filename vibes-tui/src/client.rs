//! WebSocket client for connecting to the vibes daemon.
//!
//! Provides a TuiClient with non-blocking message receiving for the TUI event loop.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, info, warn};
use vibes_server::ws::{ClientMessage, ServerMessage};

/// WebSocket client for the TUI.
///
/// Provides both async send and non-blocking try_recv for use in the TUI event loop.
pub struct TuiClient {
    /// Sender for outgoing messages
    tx: mpsc::Sender<ClientMessage>,
    /// Receiver for incoming messages
    rx: mpsc::Receiver<ServerMessage>,
    /// Connection status flag (true = connected)
    connected: Arc<AtomicBool>,
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

        // Connection status flag
        let connected = Arc::new(AtomicBool::new(true));
        let connected_clone = Arc::clone(&connected);

        // Spawn task to forward outgoing messages to WebSocket
        tokio::spawn(Self::outgoing_task(outgoing_rx, ws_sender));

        // Spawn task to receive incoming messages from WebSocket
        tokio::spawn(Self::incoming_task(
            ws_receiver,
            incoming_tx,
            connected_clone,
        ));

        Ok(Self {
            tx: outgoing_tx,
            rx: incoming_rx,
            connected,
        })
    }

    /// Returns true if the connection is still active.
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
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
    async fn incoming_task<S>(
        mut ws_receiver: S,
        tx: mpsc::Sender<ServerMessage>,
        connected: Arc<AtomicBool>,
    ) where
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
        // Mark connection as closed when task exits
        connected.store(false, Ordering::Relaxed);
        info!("WebSocket connection closed");
    }
}

/// Exponential backoff configuration for reconnection.
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// Initial delay between reconnection attempts.
    pub initial_delay: Duration,
    /// Maximum delay between attempts.
    pub max_delay: Duration,
    /// Multiplier for exponential backoff.
    pub multiplier: f64,
    /// Maximum number of attempts (None = unlimited).
    pub max_attempts: Option<u32>,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            max_attempts: None,
        }
    }
}

impl ReconnectConfig {
    /// Calculate the delay for a given attempt number (0-indexed).
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay_ms = self.initial_delay.as_millis() as f64 * self.multiplier.powi(attempt as i32);
        Duration::from_millis(delay_ms.min(self.max_delay.as_millis() as f64) as u64)
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

    // ReconnectConfig tests
    #[test]
    fn reconnect_config_default_values() {
        let config = ReconnectConfig::default();
        assert_eq!(config.initial_delay, Duration::from_millis(500));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert_eq!(config.multiplier, 2.0);
        assert!(config.max_attempts.is_none());
    }

    #[test]
    fn reconnect_config_first_attempt_uses_initial_delay() {
        let config = ReconnectConfig::default();
        let delay = config.delay_for_attempt(0);
        assert_eq!(delay, Duration::from_millis(500));
    }

    #[test]
    fn reconnect_config_delay_increases_exponentially() {
        let config = ReconnectConfig::default();
        // attempt 0: 500ms
        // attempt 1: 1000ms
        // attempt 2: 2000ms
        assert_eq!(config.delay_for_attempt(0), Duration::from_millis(500));
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(1000));
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(2000));
        assert_eq!(config.delay_for_attempt(3), Duration::from_millis(4000));
    }

    #[test]
    fn reconnect_config_delay_capped_at_max() {
        let config = ReconnectConfig {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(10),
            multiplier: 10.0,
            max_attempts: None,
        };
        // attempt 0: 1s, attempt 1: 10s, attempt 2: would be 100s but capped to 10s
        assert_eq!(config.delay_for_attempt(0), Duration::from_secs(1));
        assert_eq!(config.delay_for_attempt(1), Duration::from_secs(10));
        assert_eq!(config.delay_for_attempt(2), Duration::from_secs(10));
    }
}
