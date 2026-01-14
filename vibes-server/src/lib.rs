//! vibes-server - HTTP and WebSocket server for vibes daemon
//!
//! This crate provides the server infrastructure that owns the EventLog
//! and PluginHost. Both CLI and Web UI connect as WebSocket clients.

mod agent_registry;
pub mod consumers;
mod error;
pub mod http;
pub mod middleware;
mod state;
pub mod ws;

pub use agent_registry::ServerAgentRegistry;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::net::TcpListener;
use vibes_core::{
    HookInstaller, HookInstallerConfig, NotificationConfig, NotificationService, SubscriptionStore,
    TunnelConfig, TunnelEvent, VapidKeyManager,
};

use consumers::{
    ConsumerManager, notification::start_notification_consumer,
    plugin::start_plugin_event_consumer, websocket::start_websocket_consumer,
};

pub use error::ServerError;
pub use http::create_router;
pub use middleware::{AuthLayer, auth_middleware};
pub use state::{AppState, PtyEvent};

/// Create a future that resolves when a shutdown signal is received.
///
/// On Unix, this listens for SIGTERM and SIGINT (Ctrl-C).
/// On other platforms, this only listens for Ctrl-C.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {
            tracing::info!("Received Ctrl-C, initiating graceful shutdown");
        }
        () = terminate => {
            tracing::info!("Received SIGTERM, initiating graceful shutdown");
        }
    }
}

/// The main vibes server
pub struct VibesServer {
    config: ServerConfig,
    state: Arc<AppState>,
    notification_service: Option<Arc<NotificationService>>,
}

impl VibesServer {
    /// Create a new server with default state (in-memory storage)
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            state: Arc::new(AppState::new()),
            notification_service: None,
        }
    }

    /// Create a new server with Iggy-backed persistent storage.
    ///
    /// Attempts to start the bundled Iggy server for persistent event storage.
    ///
    /// # Errors
    ///
    /// Returns an error if Iggy cannot be started (missing binary, insufficient
    /// system resources like ulimit, connection failure, etc.)
    pub async fn new_with_iggy(config: ServerConfig) -> Result<Self, ServerError> {
        let state = AppState::new_with_iggy()
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to start Iggy: {}", e)))?;
        Ok(Self {
            config,
            state: Arc::new(state),
            notification_service: None,
        })
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

        // Create state with Iggy and push notification components
        let state = AppState::new_with_iggy()
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to start Iggy: {}", e)))?
            .with_push(vapid.clone(), subscriptions.clone());
        let state = Arc::new(state);

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
        self.run_with_graceful_shutdown(listener, shutdown_signal())
            .await
    }

    /// Run the server with a pre-bound listener and custom shutdown signal.
    ///
    /// The server will stop accepting new connections when the shutdown signal
    /// resolves, then clean up resources (including stopping Iggy) before returning.
    pub async fn run_with_graceful_shutdown<F>(
        self,
        listener: TcpListener,
        shutdown_signal: F,
    ) -> Result<(), ServerError>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let addr = listener
            .local_addr()
            .map_err(|e| ServerError::Internal(e.to_string()))?;

        tracing::info!("vibes server listening on {}", addr);

        // Install Claude Code hooks
        self.install_hooks();

        // Load plugins
        self.load_plugins().await;

        // Register model providers (Ollama, etc.)
        self.register_model_providers().await;

        // Start tunnel if enabled
        self.start_tunnel().await;

        // Start EventLog consumer-based event processing
        // This includes WebSocket consumer and notification consumer
        self.start_event_log_consumers(self.notification_service.clone())
            .await;

        let state = Arc::clone(&self.state);
        let router = create_router(self.state);

        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal)
        .await
        .map_err(|e| ServerError::Internal(e.to_string()))?;

        // Clean up resources after server stops
        tracing::info!("Server shutdown initiated, cleaning up resources");
        state.shutdown().await;

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

        // Get shutdown token from AppState - cancelled when server shuts down
        let shutdown = self.state.consumer_shutdown_token();

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

        tracing::info!("EventLog consumers started");

        // Notify plugins that the runtime is ready
        // This allows plugins to start their own background consumers
        // Note: We double-wrap the event_log Arc so it can be stored as Any
        // Plugins retrieve it via: ctx.event_log::<Arc<dyn EventLog<StoredEvent>>>()
        {
            let event_log_any: Arc<dyn std::any::Any + Send + Sync> =
                Arc::new(Arc::clone(&self.state.event_log));
            let mut plugin_host = self.state.plugin_host().write().await;
            plugin_host.notify_ready(event_log_any, shutdown.clone(), self.state.iggy_manager());
        }

        // Start plugin event consumer
        // This consumer polls events and routes them to plugins via dispatch_raw_event
        // Results are broadcast via AppState for WebSocket clients
        {
            let plugin_host = Arc::clone(self.state.plugin_host());
            let state = Arc::clone(&self.state);

            if let Err(e) =
                start_plugin_event_consumer(&mut manager, plugin_host, state, shutdown.clone())
                    .await
            {
                tracing::error!("Failed to start plugin event consumer: {}", e);
            } else {
                tracing::info!("Plugin event consumer started");
            }
        }

        // The manager is moved into the spawned task to keep it alive.
        // When shutdown signal is received, gracefully stop consumers.
        tokio::spawn(async move {
            // Wait for shutdown signal
            shutdown.cancelled().await;

            // Signal all consumers to stop
            tracing::info!("Shutdown signal received, stopping plugin consumers");
            manager.shutdown();

            // Wait for consumers to complete in-flight work
            manager.wait_for_shutdown().await;
            tracing::info!("Plugin consumers stopped");
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

    /// Load all available plugins
    async fn load_plugins(&self) {
        let mut plugin_host = self.state.plugin_host().write().await;

        match plugin_host.load_all() {
            Ok(()) => {
                let count = plugin_host.list_plugins(false).len();
                tracing::info!("Loaded {} plugins", count);
            }
            Err(e) => {
                // Log error but don't fail startup - plugins are optional
                tracing::warn!("Failed to load plugins: {}", e);
            }
        }
    }

    /// Register model providers (Ollama, etc.)
    ///
    /// Attempts to connect to configured providers and register them in the
    /// model registry. Failures are logged but don't prevent server startup.
    async fn register_model_providers(&self) {
        use vibes_models::providers::OllamaProvider;

        // Only try to register Ollama if a base URL is configured
        let Some(ollama_url) = &self.config.ollama_base_url else {
            tracing::debug!("Ollama not configured, skipping model provider registration");
            return;
        };

        let provider = OllamaProvider::with_base_url(ollama_url);

        match provider.refresh_models().await {
            Ok(()) => {
                let model_count = provider.models().len();
                let mut registry = self.state.model_registry.write().await;
                registry.register_provider(std::sync::Arc::new(provider));
                tracing::info!(
                    "Registered Ollama provider at {} with {} models",
                    ollama_url,
                    model_count
                );
            }
            Err(e) => {
                // Ollama not available - this is fine, just log it
                tracing::debug!(
                    "Ollama not available at {}: {} (this is OK if not using local models)",
                    ollama_url,
                    e
                );
            }
        }
    }

    /// Start the tunnel if enabled in config
    async fn start_tunnel(&self) {
        if !self.config.tunnel_enabled && !self.config.tunnel_quick {
            return;
        }

        // Configure tunnel based on flags
        let tunnel_config = if self.config.tunnel_quick {
            TunnelConfig::quick()
        } else if let (Some(name), Some(hostname)) =
            (&self.config.tunnel_name, &self.config.tunnel_hostname)
        {
            // Use named tunnel from config
            TunnelConfig::named(name.clone(), hostname.clone())
        } else {
            // Fallback to quick tunnel if named config is incomplete
            tracing::warn!(
                "Named tunnel requested but config incomplete, falling back to quick tunnel"
            );
            TunnelConfig::quick()
        };

        // Update tunnel manager with config and start
        {
            let mut manager = self.state.tunnel_manager.write().await;
            *manager = vibes_core::TunnelManager::new(tunnel_config, self.config.port);

            // Subscribe to events before starting
            let mut event_rx = manager.subscribe();

            // Start the tunnel
            if let Err(e) = manager.start().await {
                tracing::error!("Failed to start tunnel: {}", e);
                return;
            }

            tracing::info!("Tunnel starting...");

            // Spawn task to handle tunnel events and print URL
            tokio::spawn(async move {
                while let Ok(event) = event_rx.recv().await {
                    match event {
                        TunnelEvent::Connected { url } => {
                            // Print URL prominently
                            println!();
                            println!(
                                "╔════════════════════════════════════════════════════════════╗"
                            );
                            println!("║  🌐 Tunnel URL: {:<43} ║", url);
                            println!(
                                "╚════════════════════════════════════════════════════════════╝"
                            );
                            println!();
                        }
                        TunnelEvent::Disconnected { reason } => {
                            tracing::warn!("Tunnel disconnected: {}", reason);
                        }
                        TunnelEvent::Failed { error } => {
                            tracing::error!("Tunnel failed: {}", error);
                        }
                        _ => {}
                    }
                }
            });
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
    /// Named tunnel name (from config)
    pub tunnel_name: Option<String>,
    /// Named tunnel hostname (from config)
    pub tunnel_hostname: Option<String>,
    /// Enable push notifications
    pub notify_enabled: bool,
    /// Ollama base URL (e.g., "http://localhost:11434")
    pub ollama_base_url: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 7432,
            tunnel_enabled: false,
            tunnel_quick: false,
            tunnel_name: None,
            tunnel_hostname: None,
            notify_enabled: false,
            ollama_base_url: None,
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
            tunnel_name: None,
            tunnel_hostname: None,
            notify_enabled: false,
            ollama_base_url: None,
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

    #[tokio::test]
    async fn test_graceful_shutdown_stops_iggy_manager() {
        use vibes_iggy::{IggyConfig, IggyManager, IggyState};

        // Create a manager (without starting a real process)
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));

        // Create state with the manager (use new_for_testing to avoid loading external plugins)
        let state = Arc::new(AppState::new_for_testing().with_iggy_manager(manager.clone()));
        let server_config = ServerConfig::new("127.0.0.1", 0); // Port 0 = random available
        let server = VibesServer::with_state(server_config, Arc::clone(&state));

        // Create a shutdown trigger we can control
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        // Bind to a random port
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

        // Run server in background, passing our shutdown signal
        let server_handle = tokio::spawn(async move {
            server
                .run_with_graceful_shutdown(listener, async {
                    let _ = shutdown_rx.await;
                })
                .await
        });

        // Give server a moment to start
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Trigger shutdown
        shutdown_tx.send(()).unwrap();

        // Wait for server to finish
        let result = server_handle.await.unwrap();
        assert!(result.is_ok());

        // Verify the manager was stopped
        assert_eq!(manager.state().await, IggyState::Stopped);
    }
}
