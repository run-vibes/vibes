//! vibes-server - HTTP and WebSocket server for vibes daemon
//!
//! This crate provides the server infrastructure that owns the EventLog
//! and PluginHost. Both CLI and Web UI connect as WebSocket clients.

pub mod consumers;
mod error;
pub mod http;
pub mod middleware;
mod state;
pub mod ws;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use vibes_core::{
    HookInstaller, HookInstallerConfig, NotificationConfig, NotificationService, SubscriptionStore,
    VapidKeyManager,
};

use consumers::{
    ConsumerManager, assessment::start_assessment_consumer,
    notification::start_notification_consumer, websocket::start_websocket_consumer,
};

pub use error::ServerError;
pub use http::create_router;
pub use middleware::{AuthLayer, auth_middleware};
pub use state::{AppState, PtyEvent};

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

        self.run_with_listener(listener).await
    }

    /// Run the server with a pre-bound listener
    ///
    /// This is useful for testing where you want to bind to port 0
    /// and get the actual address before starting the server.
    pub async fn run_with_listener(self, listener: TcpListener) -> Result<(), ServerError> {
        let addr = listener
            .local_addr()
            .map_err(|e| ServerError::Internal(e.to_string()))?;

        tracing::info!("vibes server listening on {}", addr);

        // Install Claude Code hooks
        self.install_hooks();

        // Start EventLog consumer-based event processing
        // This includes WebSocket consumer and notification consumer
        self.start_event_log_consumers(self.notification_service.clone())
            .await;

        let router = create_router(self.state);
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

        Ok(())
    }

    /// Start EventLog consumers for event processing.
    ///
    /// This is the new consumer-based architecture that reads from the EventLog.
    async fn start_event_log_consumers(
        &self,
        notification_service: Option<Arc<NotificationService>>,
    ) {
        let event_log = Arc::clone(&self.state.event_log);
        let broadcaster = self.state.event_broadcaster();

        // Create shutdown token for consumers that need it
        let shutdown = CancellationToken::new();

        let mut manager = ConsumerManager::new(Arc::clone(&event_log));

        // Start WebSocket consumer
        if let Err(e) = start_websocket_consumer(&mut manager, broadcaster).await {
            tracing::error!("Failed to start WebSocket consumer: {}", e);
            return;
        }

        // Start notification consumer if service is available
        if let Some(service) = notification_service {
            if let Err(e) = start_notification_consumer(&mut manager, service).await {
                tracing::error!("Failed to start notification consumer: {}", e);
                // Continue without notifications - not fatal
            } else {
                tracing::info!("Notification consumer started");
            }
        }

        // Start assessment consumer for pattern detection and learning
        if let Err(e) = start_assessment_consumer(event_log, shutdown.clone()).await {
            tracing::error!("Failed to start assessment consumer: {}", e);
            // Continue without assessment - not fatal
        } else {
            tracing::info!("Assessment consumer started");
        }

        tracing::info!("EventLog consumers started");

        // The manager is moved into the spawned task to keep it alive
        tokio::spawn(async move {
            // Keep the manager alive until server shutdown
            // In the future, this will be coordinated with server shutdown
            std::future::pending::<()>().await;
            shutdown.cancel();
            manager.shutdown();
            manager.wait_for_shutdown().await;
        });
    }

    /// Install Claude Code hooks for structured event capture
    fn install_hooks(&self) {
        let installer = HookInstaller::new(HookInstallerConfig::default());

        match installer.install() {
            Ok(()) => {
                tracing::info!("Claude Code hooks installed successfully");
            }
            Err(e) => {
                // Log error but don't fail startup - hooks are optional
                tracing::warn!("Failed to install Claude Code hooks: {}", e);
            }
        }
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
