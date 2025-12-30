//! Shared application state for the vibes server

use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::{RwLock, broadcast};
use vibes_core::{
    AccessConfig, MemoryEventBus, PluginHost, PluginHostConfig, SubscriptionStore, TunnelConfig,
    TunnelManager, VapidKeyManager, VibesEvent,
    pty::{PtyConfig, PtyManager},
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

/// Shared application state accessible by all handlers
#[derive(Clone)]
pub struct AppState {
    /// Plugin host for managing plugins
    pub plugin_host: Arc<RwLock<PluginHost>>,
    /// Event bus for publishing/subscribing to events
    pub event_bus: Arc<MemoryEventBus>,
    /// Tunnel manager for remote access
    pub tunnel_manager: Arc<RwLock<TunnelManager>>,
    /// Authentication layer
    pub auth_layer: AuthLayer,
    /// When the server started
    pub started_at: DateTime<Utc>,
    /// Broadcast channel for WebSocket event distribution
    event_broadcaster: broadcast::Sender<VibesEvent>,
    /// VAPID key manager for push notifications (optional)
    pub vapid: Option<Arc<VapidKeyManager>>,
    /// Push subscription store (optional)
    pub subscriptions: Option<Arc<SubscriptionStore>>,
    /// PTY session manager for terminal sessions
    pub pty_manager: Arc<RwLock<PtyManager>>,
    /// Broadcast channel for PTY output distribution
    pty_broadcaster: broadcast::Sender<PtyEvent>,
}

impl AppState {
    /// Create a new AppState with default components
    pub fn new() -> Self {
        let event_bus = Arc::new(MemoryEventBus::new(10_000));
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
            event_bus,
            tunnel_manager,
            auth_layer: AuthLayer::disabled(),
            started_at: Utc::now(),
            event_broadcaster,
            pty_broadcaster,
            vapid: None,
            subscriptions: None,
            pty_manager,
        }
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
        event_bus: Arc<MemoryEventBus>,
        tunnel_manager: Arc<RwLock<TunnelManager>>,
    ) -> Self {
        let (event_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let (pty_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);
        let pty_manager = Arc::new(RwLock::new(PtyManager::new(PtyConfig::default())));

        Self {
            plugin_host,
            event_bus,
            tunnel_manager,
            auth_layer: AuthLayer::disabled(),
            started_at: Utc::now(),
            event_broadcaster,
            pty_broadcaster,
            vapid: None,
            subscriptions: None,
            pty_manager,
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

    /// Subscribe to events broadcast to WebSocket clients
    ///
    /// Returns a receiver that will receive all VibesEvents published
    /// through the event broadcaster.
    pub fn subscribe_events(&self) -> broadcast::Receiver<VibesEvent> {
        self.event_broadcaster.subscribe()
    }

    /// Publish an event to all subscribed WebSocket clients
    ///
    /// Returns the number of receivers that received the event.
    /// Returns 0 if there are no active subscribers.
    pub fn broadcast_event(&self, event: VibesEvent) -> usize {
        self.event_broadcaster.send(event).unwrap_or(0)
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
        let event_bus = Arc::new(MemoryEventBus::new(100));
        let plugin_host = Arc::new(RwLock::new(PluginHost::new(PluginHostConfig::default())));
        let tunnel_manager = Arc::new(RwLock::new(TunnelManager::new(
            TunnelConfig::default(),
            7432,
        )));

        let state = AppState::with_components(plugin_host, event_bus, tunnel_manager);
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
        let event = VibesEvent::SessionCreated {
            session_id: "sess-1".to_string(),
            name: None,
        };
        let count = state.broadcast_event(event);
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_broadcast_event_to_subscriber() {
        let state = AppState::new();
        let mut receiver = state.subscribe_events();

        let event = VibesEvent::Claude {
            session_id: "sess-1".to_string(),
            event: ClaudeEvent::TextDelta {
                text: "Hello".to_string(),
            },
        };

        let count = state.broadcast_event(event.clone());
        assert_eq!(count, 1);

        let received = receiver.recv().await.unwrap();
        match received {
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

        let event = VibesEvent::SessionStateChanged {
            session_id: "sess-1".to_string(),
            state: "processing".to_string(),
        };

        let count = state.broadcast_event(event);
        assert_eq!(count, 2);

        // Both receivers should get the event
        let received1 = receiver1.recv().await.unwrap();
        let received2 = receiver2.recv().await.unwrap();

        match received1 {
            VibesEvent::SessionStateChanged { session_id, state } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(state, "processing");
            }
            _ => panic!("Expected SessionStateChanged"),
        }

        match received2 {
            VibesEvent::SessionStateChanged { session_id, state } => {
                assert_eq!(session_id, "sess-1");
                assert_eq!(state, "processing");
            }
            _ => panic!("Expected SessionStateChanged"),
        }
    }
}
