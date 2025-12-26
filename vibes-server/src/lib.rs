//! vibes-server - HTTP and WebSocket server for vibes daemon
//!
//! This crate provides the server infrastructure that owns the SessionManager,
//! EventBus, and PluginHost. Both CLI and Web UI connect as WebSocket clients.

mod error;
pub mod http;
mod state;

pub use error::ServerError;
pub use http::create_router;
pub use state::AppState;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Host address to bind to
    pub host: String,
    /// Port to listen on
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 7432,
        }
    }
}

impl ServerConfig {
    /// Create a new ServerConfig with the specified host and port
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
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
}
