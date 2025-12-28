//! Hook receiver - listens for hook events from Claude Code
//!
//! On Unix systems, uses a Unix domain socket for fast IPC.
//! On Windows, falls back to a TCP socket on localhost.

use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use super::HookEvent;

/// Configuration for the hook receiver
#[derive(Debug, Clone)]
pub struct HookReceiverConfig {
    /// Socket path for Unix systems
    pub socket_path: PathBuf,
    /// TCP port for Windows (fallback)
    pub tcp_port: u16,
}

impl Default for HookReceiverConfig {
    fn default() -> Self {
        Self {
            socket_path: PathBuf::from("/tmp/vibes-hooks.sock"),
            tcp_port: 7744,
        }
    }
}

/// Receiver for hook events from Claude Code
pub struct HookReceiver {
    config: HookReceiverConfig,
    event_tx: mpsc::Sender<HookEvent>,
}

impl HookReceiver {
    /// Create a new hook receiver
    pub fn new(config: HookReceiverConfig, event_tx: mpsc::Sender<HookEvent>) -> Self {
        Self { config, event_tx }
    }

    /// Start listening for hook events
    ///
    /// This spawns a background task that accepts connections and
    /// reads JSON-encoded hook events, one per line.
    #[cfg(unix)]
    pub async fn start(&self) -> std::io::Result<()> {
        use tokio::net::UnixListener;

        // Remove existing socket file if present
        if self.config.socket_path.exists() {
            std::fs::remove_file(&self.config.socket_path)?;
        }

        let listener = UnixListener::bind(&self.config.socket_path)?;
        info!("Hook receiver listening on {:?}", self.config.socket_path);

        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _addr)) => {
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            let reader = BufReader::new(stream);
                            let mut lines = reader.lines();

                            while let Ok(Some(line)) = lines.next_line().await {
                                match serde_json::from_str::<HookEvent>(&line) {
                                    Ok(event) => {
                                        debug!("Received hook event: {:?}", event.hook_type());
                                        if tx.send(event).await.is_err() {
                                            warn!("Hook event channel closed");
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to parse hook event: {}", e);
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept hook connection: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Start listening on TCP (for Windows or fallback)
    #[cfg(not(unix))]
    pub async fn start(&self) -> std::io::Result<()> {
        use tokio::net::TcpListener;

        let addr = format!("127.0.0.1:{}", self.config.tcp_port);
        let listener = TcpListener::bind(&addr).await?;
        info!("Hook receiver listening on {}", addr);

        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _addr)) => {
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            let reader = BufReader::new(stream);
                            let mut lines = reader.lines();

                            while let Ok(Some(line)) = lines.next_line().await {
                                match serde_json::from_str::<HookEvent>(&line) {
                                    Ok(event) => {
                                        debug!("Received hook event: {:?}", event.hook_type());
                                        if tx.send(event).await.is_err() {
                                            warn!("Hook event channel closed");
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to parse hook event: {}", e);
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept hook connection: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Get the socket path (Unix) or address (Windows)
    pub fn address(&self) -> String {
        #[cfg(unix)]
        {
            self.config.socket_path.to_string_lossy().to_string()
        }
        #[cfg(not(unix))]
        {
            format!("127.0.0.1:{}", self.config.tcp_port)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = HookReceiverConfig::default();
        assert_eq!(
            config.socket_path.to_str().unwrap(),
            "/tmp/vibes-hooks.sock"
        );
        assert_eq!(config.tcp_port, 7744);
    }

    #[tokio::test]
    async fn test_receiver_creation() {
        let (tx, _rx) = mpsc::channel(10);
        let config = HookReceiverConfig::default();
        let receiver = HookReceiver::new(config, tx);
        assert!(receiver.address().contains("vibes-hooks"));
    }
}
