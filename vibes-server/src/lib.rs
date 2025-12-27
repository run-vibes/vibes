//! vibes-server - HTTP and WebSocket server for vibes daemon
//!
//! This crate provides the server infrastructure that owns the SessionManager,
//! EventBus, and PluginHost. Both CLI and Web UI connect as WebSocket clients.

mod error;
pub mod http;
pub mod middleware;
mod state;
pub mod ws;

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;
use vibes_core::EventBus;

pub use error::ServerError;
pub use http::create_router;
pub use middleware::{AuthLayer, auth_middleware};
pub use state::AppState;

/// The main vibes server
pub struct VibesServer {
    config: ServerConfig,
    state: Arc<AppState>,
}

impl VibesServer {
    /// Create a new server with default state
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            state: Arc::new(AppState::new()),
        }
    }

    /// Create a server with custom state (for testing)
    pub fn with_state(config: ServerConfig, state: Arc<AppState>) -> Self {
        Self { config, state }
    }

    /// Get the server configuration
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Get the shared application state
    pub fn state(&self) -> Arc<AppState> {
        Arc::clone(&self.state)
    }

    /// Run the server, binding to the configured address
    pub async fn run(self) -> Result<(), ServerError> {
        let addr = self.config.addr();
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| ServerError::Bind {
                addr: addr.clone(),
                source: e,
            })?;

        tracing::info!("vibes server listening on {}", addr);

        // Start event forwarding from event_bus to WebSocket broadcaster
        self.start_event_forwarding();

        let router = create_router(self.state);
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

        Ok(())
    }

    /// Start a background task that forwards events from EventBus to WebSocket broadcaster
    fn start_event_forwarding(&self) {
        let state = Arc::clone(&self.state);
        let mut rx = state.event_bus.subscribe();

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok((_seq, event)) => {
                        let count = state.broadcast_event(event);
                        tracing::trace!("Broadcast event to {} WebSocket clients", count);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        tracing::warn!("Event bus channel closed");
                        break;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(count)) => {
                        tracing::warn!("Event forwarding lagged by {} events", count);
                        // Continue receiving
                    }
                }
            }
        });
    }
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Host address to bind to
    pub host: String,
    /// Port to listen on
    pub port: u16,
    /// Enable tunnel
    pub tunnel_enabled: bool,
    /// Use quick tunnel mode (temporary URL)
    pub tunnel_quick: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 7432,
            tunnel_enabled: false,
            tunnel_quick: false,
        }
    }
}

impl ServerConfig {
    /// Create a new ServerConfig with the specified host and port
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            tunnel_enabled: false,
            tunnel_quick: false,
        }
    }

    /// Returns the socket address string (e.g., "0.0.0.0:7432")
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 7432);
    }

    #[test]
    fn test_server_config_addr() {
        let config = ServerConfig::new("127.0.0.1", 8080);
        assert_eq!(config.addr(), "127.0.0.1:8080");
    }

    #[test]
    fn test_vibes_server_new() {
        let config = ServerConfig::default();
        let server = VibesServer::new(config.clone());
        assert_eq!(server.config().addr(), config.addr());
    }

    #[test]
    fn test_vibes_server_with_state() {
        let config = ServerConfig::new("127.0.0.1", 9000);
        let state = Arc::new(AppState::new());
        let server = VibesServer::with_state(config.clone(), state);
        assert_eq!(server.config().port, 9000);
    }
}
