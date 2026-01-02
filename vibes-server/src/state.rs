//! Shared application state for the vibes server

use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::{RwLock, broadcast};
use vibes_core::{
    AccessConfig, PluginHost, PluginHostConfig, StoredEvent, SubscriptionStore, TunnelConfig,
    TunnelManager, VapidKeyManager, VibesEvent,
    pty::{PtyConfig, PtyManager},
};
use vibes_iggy::{
    EventLog, IggyConfig, IggyEventLog, IggyManager, InMemoryEventLog, Offset, run_preflight_checks,
};

/// PTY output event for broadcasting to attached clients
#[derive(Clone, Debug)]
pub enum PtyEvent {
    /// Raw output from a PTY session (base64 encoded for binary safety)
    Output { session_id: String, data: String },
    /// PTY process has exited
    Exit {
        session_id: String,
        exit_code: Option<i32>,
    },
}

use crate::middleware::AuthLayer;

/// Default capacity for the event broadcast channel
const DEFAULT_BROADCAST_CAPACITY: usize = 1000;

/// Grace period for Iggy server startup before attempting connection.
/// Allows the server process to fully initialize its TCP listener.
const IGGY_STARTUP_GRACE_MS: u64 = 500;

/// Shared application state accessible by all handlers
#[derive(Clone)]
pub struct AppState {
    /// Plugin host for managing plugins
    pub plugin_host: Arc<RwLock<PluginHost>>,
    /// Event log for persistent event storage
    pub event_log: Arc<dyn EventLog<StoredEvent>>,
    /// Tunnel manager for remote access
    pub tunnel_manager: Arc<RwLock<TunnelManager>>,
    /// Authentication layer
    pub auth_layer: AuthLayer,
    /// When the server started
    pub started_at: DateTime<Utc>,
    /// Broadcast channel for WebSocket event distribution (offset, stored_event)
    event_broadcaster: broadcast::Sender<(Offset, StoredEvent)>,
    /// VAPID key manager for push notifications (optional)
    pub vapid: Option<Arc<VapidKeyManager>>,
    /// Push subscription store (optional)
    pub subscriptions: Option<Arc<SubscriptionStore>>,
    /// PTY session manager for terminal sessions
    pub pty_manager: Arc<RwLock<PtyManager>>,
    /// Broadcast channel for PTY output distribution
    pty_broadcaster: broadcast::Sender<PtyEvent>,
    /// Iggy manager for subprocess lifecycle (optional, only when using Iggy storage)
    iggy_manager: Option<Arc<IggyManager>>,
}

impl AppState {
    /// Create a new AppState with default components
    pub fn new() -> Self {
        let event_log: Arc<dyn EventLog<StoredEvent>> =
            Arc::new(InMemoryEventLog::<StoredEvent>::new());
        let plugin_host = Arc::new(RwLock::new(PluginHost::new(PluginHostConfig::default())));
        let tunnel_manager = Arc::new(RwLock::new(TunnelManager::new(
            TunnelConfig::default(),
            7432,
        )));
        let (event_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (pty_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let pty_manager = Arc::new(RwLock::new(PtyManager::new(PtyConfig::default())));

        Self {
            plugin_host,
            event_log,
            tunnel_manager,
            auth_layer: AuthLayer::disabled(),
            started_at: Utc::now(),
            event_broadcaster,
            pty_broadcaster,
            vapid: None,
            subscriptions: None,
            pty_manager,
            iggy_manager: None,
        }
    }

    /// Create a new AppState with Iggy-backed persistent storage.
    ///
    /// Attempts to start and connect to the bundled Iggy server.
    ///
    /// # Errors
    ///
    /// Returns an error if Iggy cannot be started (missing binary, insufficient
    /// system resources like ulimit, connection failure, etc.)
    pub async fn new_with_iggy() -> Result<Self, vibes_iggy::Error> {
        let (log, manager) = Self::try_start_iggy().await?;
        tracing::info!("Using Iggy for persistent event storage");
        let event_log: Arc<dyn EventLog<StoredEvent>> = Arc::new(log);
        let iggy_manager = Some(manager);

        let plugin_host = Arc::new(RwLock::new(PluginHost::new(PluginHostConfig::default())));
        let tunnel_manager = Arc::new(RwLock::new(TunnelManager::new(
            TunnelConfig::default(),
            7432,
        )));
        let (event_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (pty_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let pty_manager = Arc::new(RwLock::new(PtyManager::new(PtyConfig::default())));

        Ok(Self {
            plugin_host,
            event_log,
            tunnel_manager,
            auth_layer: AuthLayer::disabled(),
            started_at: Utc::now(),
            event_broadcaster,
            pty_broadcaster,
            vapid: None,
            subscriptions: None,
            pty_manager,
            iggy_manager,
        })
    }

    /// Create AppState with Iggy using custom configuration.
    ///
    /// This is useful for tests that need isolated ports and data directories.
    #[doc(hidden)]
    pub async fn new_with_iggy_config(iggy_config: IggyConfig) -> Result<Self, vibes_iggy::Error> {
        let (log, manager) = Self::try_start_iggy_with_config(iggy_config).await?;
        tracing::info!("Using Iggy for persistent event storage");
        let event_log: Arc<dyn EventLog<StoredEvent>> = Arc::new(log);
        let iggy_manager = Some(manager);

        let plugin_host = Arc::new(RwLock::new(PluginHost::new(PluginHostConfig::default())));
        let tunnel_manager = Arc::new(RwLock::new(TunnelManager::new(
            TunnelConfig::default(),
            7432,
        )));
        let (event_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (pty_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let pty_manager = Arc::new(RwLock::new(PtyManager::new(PtyConfig::default())));

        Ok(Self {
            plugin_host,
            event_log,
            tunnel_manager,
            auth_layer: AuthLayer::disabled(),
            started_at: Utc::now(),
            event_broadcaster,
            pty_broadcaster,
            vapid: None,
            subscriptions: None,
            pty_manager,
            iggy_manager,
        })
    }

    /// Try to start the Iggy server and create an event log.
    ///
    /// Returns both the event log and a reference to the manager for shutdown.
    async fn try_start_iggy()
    -> Result<(IggyEventLog<StoredEvent>, Arc<IggyManager>), vibes_iggy::Error> {
        Self::try_start_iggy_with_config(IggyConfig::default()).await
    }

    /// Try to start the Iggy server with custom configuration.
    async fn try_start_iggy_with_config(
        config: IggyConfig,
    ) -> Result<(IggyEventLog<StoredEvent>, Arc<IggyManager>), vibes_iggy::Error> {
        // Check if binary is available before trying to start
        if config.find_binary().is_none() {
            return Err(vibes_iggy::Error::BinaryNotFound);
        }

        // Check system requirements (ulimit for io_uring)
        run_preflight_checks()?;

        let manager = Arc::new(IggyManager::new(config));
        manager.start().await?;

        // Spawn the supervisor loop to monitor the process and handle restarts
        // The supervisor calls try_wait() which reaps zombie processes
        let supervisor_manager = Arc::clone(&manager);
        tokio::spawn(async move {
            if let Err(e) = supervisor_manager.supervise().await {
                tracing::error!("Iggy supervisor exited with error: {}", e);
            }
        });

        // Give the server a moment to become ready
        tokio::time::sleep(std::time::Duration::from_millis(IGGY_STARTUP_GRACE_MS)).await;

        // Create event log from the manager (cloning the Arc)
        let event_log = IggyEventLog::new(Arc::clone(&manager));
        event_log.connect().await?;

        Ok((event_log, manager))
    }

    /// Configure authentication for this state
    pub fn with_auth(mut self, config: AccessConfig) -> Self {
        self.auth_layer = AuthLayer::new(config);
        self
    }

    /// Configure push notifications for this state
    pub fn with_push(
        mut self,
        vapid: Arc<VapidKeyManager>,
        subscriptions: Arc<SubscriptionStore>,
    ) -> Self {
        self.vapid = Some(vapid);
        self.subscriptions = Some(subscriptions);
        self
    }

    /// Configure PTY settings for this state
    pub fn with_pty_config(mut self, config: PtyConfig) -> Self {
        self.pty_manager = Arc::new(RwLock::new(PtyManager::new(config)));
        self
    }

    /// Create AppState with custom components (for testing)
    pub fn with_components(
        plugin_host: Arc<RwLock<PluginHost>>,
        tunnel_manager: Arc<RwLock<TunnelManager>>,
    ) -> Self {
        let event_log: Arc<dyn EventLog<StoredEvent>> =
            Arc::new(InMemoryEventLog::<StoredEvent>::new());
        let (event_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (pty_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let pty_manager = Arc::new(RwLock::new(PtyManager::new(PtyConfig::default())));

        Self {
            plugin_host,
            event_log,
            tunnel_manager,
            auth_layer: AuthLayer::disabled(),
            started_at: Utc::now(),
            event_broadcaster,
            pty_broadcaster,
            vapid: None,
            subscriptions: None,
            pty_manager,
            iggy_manager: None,
        }
    }

    /// Set the Iggy manager for shutdown coordination
    #[must_use]
    pub fn with_iggy_manager(mut self, manager: Arc<IggyManager>) -> Self {
        self.iggy_manager = Some(manager);
        self
    }

    /// Gracefully shutdown the server, stopping all managed subprocesses.
    ///
    /// This should be called before the server exits to ensure clean termination
    /// of the Iggy server subprocess.
    pub async fn shutdown(&self) {
        if let Some(manager) = &self.iggy_manager {
            tracing::info!("Stopping Iggy server subprocess");
            if let Err(e) = manager.stop().await {
                tracing::error!("Error stopping Iggy server: {}", e);
            }
        }
    }

    /// Get a reference to the tunnel manager
    pub fn tunnel_manager(&self) -> &Arc<RwLock<TunnelManager>> {
        &self.tunnel_manager
    }

    /// Get a reference to the plugin host
    pub fn plugin_host(&self) -> &Arc<RwLock<PluginHost>> {
        &self.plugin_host
    }

    /// Get the event broadcaster sender for consumer integration.
    ///
    /// This is used by EventLog consumers to broadcast events to WebSocket clients.
    pub fn event_broadcaster(&self) -> broadcast::Sender<(Offset, StoredEvent)> {
        self.event_broadcaster.clone()
    }

    /// Subscribe to events broadcast to WebSocket clients
    ///
    /// Returns a receiver that will receive (offset, stored_event) tuples published
    /// through the event broadcaster.
    pub fn subscribe_events(&self) -> broadcast::Receiver<(Offset, StoredEvent)> {
        self.event_broadcaster.subscribe()
    }

    /// Append an event to the EventLog.
    ///
    /// This is the primary way to publish events. The event will be:
    /// 1. Wrapped in a StoredEvent with a unique UUIDv7 event_id
    /// 2. Persisted to the EventLog (Iggy when available, in-memory otherwise)
    /// 3. Picked up by consumers (WebSocket, Notification, Assessment)
    /// 4. Broadcast to connected clients via the WebSocket consumer
    ///
    /// This method spawns a task to avoid blocking the caller.
    /// If persistence fails, the error is logged but not propagated.
    pub fn append_event(&self, event: VibesEvent) {
        let event_log = Arc::clone(&self.event_log);
        let stored = StoredEvent::new(event);
        tokio::spawn(async move {
            if let Err(e) = event_log.append(stored).await {
                tracing::warn!("Failed to append event to EventLog: {}", e);
            }
        });
    }

    /// Broadcast a stored event with its offset to all subscribed WebSocket clients.
    ///
    /// **Internal API:** Event producers should NOT call this directly.
    /// Use [`append_event`] instead, which writes to the EventLog.
    /// The WebSocket consumer will then call this method after reading
    /// from the log.
    ///
    /// Returns the number of receivers that received the event.
    /// Returns 0 if there are no active subscribers.
    pub fn broadcast_event(&self, offset: Offset, stored: StoredEvent) -> usize {
        self.event_broadcaster.send((offset, stored)).unwrap_or(0)
    }

    /// Subscribe to PTY events broadcast to WebSocket clients
    ///
    /// Returns a receiver that will receive all PtyEvents published
    /// through the PTY broadcaster.
    pub fn subscribe_pty_events(&self) -> broadcast::Receiver<PtyEvent> {
        self.pty_broadcaster.subscribe()
    }

    /// Publish a PTY event to all subscribed WebSocket clients
    ///
    /// Returns the number of receivers that received the event.
    /// Returns 0 if there are no active subscribers.
    pub fn broadcast_pty_event(&self, event: PtyEvent) -> usize {
        self.pty_broadcaster.send(event).unwrap_or(0)
    }

    /// Returns how long the server has been running
    pub fn uptime_seconds(&self) -> i64 {
        (Utc::now() - self.started_at).num_seconds()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vibes_core::ClaudeEvent;

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert!(state.uptime_seconds() >= 0);
    }

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert!(state.uptime_seconds() >= 0);
    }

    #[test]
    fn test_app_state_with_components() {
        let plugin_host = Arc::new(RwLock::new(PluginHost::new(PluginHostConfig::default())));
        let tunnel_manager = Arc::new(RwLock::new(TunnelManager::new(
            TunnelConfig::default(),
            7432,
        )));

        let state = AppState::with_components(plugin_host, tunnel_manager);
        assert!(state.uptime_seconds() >= 0);
    }

    #[tokio::test]
    async fn test_app_state_has_tunnel_manager() {
        let state = AppState::new();
        let tunnel = state.tunnel_manager.read().await;
        assert!(!tunnel.is_enabled());
    }

    // ==================== Event Broadcasting Tests ====================

    #[tokio::test]
    async fn test_subscribe_events_returns_receiver() {
        let state = AppState::new();
        let _receiver = state.subscribe_events();
        // Receiver created successfully
    }

    #[tokio::test]
    async fn test_broadcast_event_with_no_subscribers_returns_zero() {
        let state = AppState::new();
        // No subscribers, should return 0
        let stored = StoredEvent::new(VibesEvent::SessionCreated {
            session_id: "sess-1".to_string(),
            name: None,
        });
        let count = state.broadcast_event(0, stored);
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_broadcast_event_to_subscriber() {
        let state = AppState::new();
        let mut receiver = state.subscribe_events();

        let stored = StoredEvent::new(VibesEvent::Claude {
            session_id: "sess-1".to_string(),
            event: ClaudeEvent::TextDelta {
                text: "Hello".to_string(),
            },
        });
        let expected_event_id = stored.event_id;

        let count = state.broadcast_event(42, stored);
        assert_eq!(count, 1);

        let (offset, received) = receiver.recv().await.unwrap();
        assert_eq!(offset, 42);
        assert_eq!(received.event_id, expected_event_id);
        match &received.event {
            VibesEvent::Claude { session_id, event } => {
                assert_eq!(session_id, "sess-1");
                match event {
                    ClaudeEvent::TextDelta { text } => assert_eq!(text, "Hello"),
                    _ => panic!("Expected TextDelta"),
                }
            }
            _ => panic!("Expected Claude event"),
        }
    }

    #[tokio::test]
    async fn test_broadcast_to_multiple_subscribers() {
        let state = AppState::new();
        let mut receiver1 = state.subscribe_events();
        let mut receiver2 = state.subscribe_events();

        let stored = StoredEvent::new(VibesEvent::SessionStateChanged {
            session_id: "sess-1".to_string(),
            state: "processing".to_string(),
        });

        let count = state.broadcast_event(99, stored);
        assert_eq!(count, 2);

        // Both receivers should get the event with offset
        let (offset1, received1) = receiver1.recv().await.unwrap();
        let (offset2, received2) = receiver2.recv().await.unwrap();

        assert_eq!(offset1, 99);
        assert_eq!(offset2, 99);

        match &received1.event {
            VibesEvent::SessionStateChanged { session_id, state } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(state, "processing");
            }
            _ => panic!("Expected SessionStateChanged"),
        }

        match &received2.event {
            VibesEvent::SessionStateChanged { session_id, state } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(state, "processing");
            }
            _ => panic!("Expected SessionStateChanged"),
        }
    }

    // ==================== Shutdown Tests ====================

    #[tokio::test]
    async fn test_shutdown_stops_iggy_manager() {
        // Create an IggyManager without starting a process
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));

        // Create AppState with the manager
        let state = AppState::new().with_iggy_manager(manager.clone());

        // Call shutdown
        state.shutdown().await;

        // Verify the manager's shutdown signal was set
        assert_eq!(manager.state().await, vibes_iggy::IggyState::Stopped);
    }

    #[tokio::test]
    async fn test_shutdown_without_iggy_is_safe() {
        // AppState without Iggy should not panic on shutdown
        let state = AppState::new();
        state.shutdown().await;
        // No panic = success
    }

    // ==================== Event Appending Tests ====================

    #[tokio::test]
    async fn test_append_event_writes_to_log() {
        use vibes_iggy::SeekPosition;

        let state = AppState::new();

        let event = VibesEvent::SessionCreated {
            session_id: "test-session".to_string(),
            name: Some("Test".to_string()),
        };

        state.append_event(event);

        // Give the spawned task time to complete
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Create a consumer and verify the event is in the log
        let mut consumer = state.event_log.consumer("test-reader").await.unwrap();
        consumer.seek(SeekPosition::Beginning).await.unwrap();

        let batch = consumer
            .poll(10, std::time::Duration::from_millis(100))
            .await
            .unwrap();
        assert_eq!(batch.events.len(), 1);

        let stored = &batch.events[0].1;

        // Verify the event has a valid UUIDv7 event_id
        assert_eq!(stored.event_id.get_version(), Some(uuid::Version::SortRand));

        // Verify the inner event data
        match &stored.event {
            VibesEvent::SessionCreated { session_id, .. } => {
                assert_eq!(session_id, "test-session");
            }
            _ => panic!("Expected SessionCreated event"),
        }
    }
}
