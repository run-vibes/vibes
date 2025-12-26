//! Shared application state for the vibes server

use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::broadcast;
use vibes_core::{
    BackendFactory, MemoryEventBus, PluginHost, PluginHostConfig, PrintModeBackendFactory,
    PrintModeConfig, SessionManager, VibesEvent,
};

/// Default capacity for the event broadcast channel
const DEFAULT_BROADCAST_CAPACITY: usize = 1000;

/// Shared application state accessible by all handlers
#[derive(Clone)]
pub struct AppState {
    /// Session manager for Claude sessions
    pub session_manager: Arc<SessionManager>,
    /// Plugin host for managing plugins
    pub plugin_host: Arc<PluginHost>,
    /// Event bus for publishing/subscribing to events
    pub event_bus: Arc<MemoryEventBus>,
    /// When the server started
    pub started_at: DateTime<Utc>,
    /// Broadcast channel for WebSocket event distribution
    event_broadcaster: broadcast::Sender<VibesEvent>,
}

impl AppState {
    /// Create a new AppState with default components
    pub fn new() -> Self {
        let event_bus = Arc::new(MemoryEventBus::new(10_000));
        let factory: Arc<dyn BackendFactory> =
            Arc::new(PrintModeBackendFactory::new(PrintModeConfig::default()));
        let session_manager = Arc::new(SessionManager::new(factory, event_bus.clone()));
        let plugin_host = Arc::new(PluginHost::new(PluginHostConfig::default()));
        let (event_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);

        Self {
            session_manager,
            plugin_host,
            event_bus,
            started_at: Utc::now(),
            event_broadcaster,
        }
    }

    /// Create AppState with custom components (for testing)
    pub fn with_components(
        session_manager: Arc<SessionManager>,
        plugin_host: Arc<PluginHost>,
        event_bus: Arc<MemoryEventBus>,
    ) -> Self {
        let (event_broadcaster, _) = broadcast::channel(DEFAULT_BROADCAST_CAPACITY);

        Self {
            session_manager,
            plugin_host,
            event_bus,
            started_at: Utc::now(),
            event_broadcaster,
        }
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
        let factory: Arc<dyn BackendFactory> =
            Arc::new(PrintModeBackendFactory::new(PrintModeConfig::default()));
        let session_manager = Arc::new(SessionManager::new(factory, event_bus.clone()));
        let plugin_host = Arc::new(PluginHost::new(PluginHostConfig::default()));

        let state = AppState::with_components(session_manager, plugin_host, event_bus);
        assert!(state.uptime_seconds() >= 0);
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
