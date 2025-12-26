//! Shared application state for the vibes server

use std::sync::Arc;

use chrono::{DateTime, Utc};
use vibes_core::{
    BackendFactory, MemoryEventBus, PluginHost, PluginHostConfig, PrintModeBackendFactory,
    PrintModeConfig, SessionManager,
};

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
}

impl AppState {
    /// Create a new AppState with default components
    pub fn new() -> Self {
        let event_bus = Arc::new(MemoryEventBus::new(10_000));
        let factory: Arc<dyn BackendFactory> =
            Arc::new(PrintModeBackendFactory::new(PrintModeConfig::default()));
        let session_manager = Arc::new(SessionManager::new(factory, event_bus.clone()));
        let plugin_host = Arc::new(PluginHost::new(PluginHostConfig::default()));

        Self {
            session_manager,
            plugin_host,
            event_bus,
            started_at: Utc::now(),
        }
    }

    /// Create AppState with custom components (for testing)
    pub fn with_components(
        session_manager: Arc<SessionManager>,
        plugin_host: Arc<PluginHost>,
        event_bus: Arc<MemoryEventBus>,
    ) -> Self {
        Self {
            session_manager,
            plugin_host,
            event_bus,
            started_at: Utc::now(),
        }
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
}
