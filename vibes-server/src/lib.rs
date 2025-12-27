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
use std::path::PathBuf;
use std::sync::Arc;

use tokio::net::TcpListener;
use vibes_core::{
    EventBus, NotificationConfig, NotificationService, SubscriptionStore, VapidKeyManager,
};

pub use error::ServerError;
pub use http::create_router;
pub use middleware::{AuthLayer, auth_middleware};
pub use state::AppState;

/// The main vibes server
pub struct VibesServer {
    config: ServerConfig,
    state: Arc<AppState>,
    notification_service: Option<Arc<NotificationService>>,
}

impl VibesServer {
    /// Create a new server with default state
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            state: Arc::new(AppState::new()),
            notification_service: None,
        }
    }

    /// Create a new server with push notifications enabled
    pub async fn with_notifications(config: ServerConfig) -> Result<Self, ServerError> {
        let config_dir = get_vibes_config_dir()?;

        // Initialize VAPID keys
        let vapid = VapidKeyManager::load_or_generate(&config_dir)
            .await
            .map_err(|e| {
                ServerError::Internal(format!("Failed to initialize VAPID keys: {}", e))
            })?;
        let vapid = Arc::new(vapid);

        // Initialize subscription store
        let subscriptions = SubscriptionStore::load(&config_dir)
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to load subscriptions: {}", e)))?;
        let subscriptions = Arc::new(subscriptions);

        // Create state with push notification components
        let state = Arc::new(AppState::new().with_push(vapid.clone(), subscriptions.clone()));

        // Create notification service
        let notification_config = NotificationConfig::default();
        let notification_service = Arc::new(NotificationService::new(
            vapid,
            subscriptions,
            notification_config,
        ));

        tracing::info!("Push notifications initialized");

        Ok(Self {
            config,
            state,
            notification_service: Some(notification_service),
        })
    }

    /// Create a server with custom state (for testing)
    pub fn with_state(config: ServerConfig, state: Arc<AppState>) -> Self {
        Self {
            config,
            state,
            notification_service: None,
        }
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

        // Start notification service if enabled
        if let Some(notification_service) = &self.notification_service {
            self.start_notification_service(notification_service.clone());
        }

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

    /// Start the notification service in a background task
    fn start_notification_service(&self, notification_service: Arc<NotificationService>) {
        let state = Arc::clone(&self.state);
        let rx = state.event_bus.subscribe();

        tokio::spawn(async move {
            notification_service.run(rx).await;
        });

        tracing::info!("Notification service started");
    }
}

/// Get the vibes configuration directory
fn get_vibes_config_dir() -> Result<PathBuf, ServerError> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| ServerError::Internal("Could not determine config directory".to_string()))?
        .join("vibes");

    // Ensure the directory exists
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| ServerError::Internal(format!("Failed to create config directory: {}", e)))?;

    Ok(config_dir)
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
    /// Enable push notifications
    pub notify_enabled: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 7432,
            tunnel_enabled: false,
            tunnel_quick: false,
            notify_enabled: false,
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
            notify_enabled: false,
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
