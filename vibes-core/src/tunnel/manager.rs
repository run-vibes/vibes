//! Tunnel manager for cloudflared process lifecycle

use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info};

use super::cloudflared;
use super::config::TunnelConfig;
use super::restart::RestartPolicy;
use super::state::{TunnelEvent, TunnelState};

/// Manages the cloudflared tunnel process
pub struct TunnelManager {
    config: TunnelConfig,
    local_port: u16,
    state: Arc<RwLock<TunnelState>>,
    event_tx: broadcast::Sender<TunnelEvent>,
    process: Option<Child>,
    restart_policy: RestartPolicy,
}

impl TunnelManager {
    /// Create a new tunnel manager
    pub fn new(config: TunnelConfig, local_port: u16) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        Self {
            config,
            local_port,
            state: Arc::new(RwLock::new(TunnelState::Disabled)),
            event_tx,
            process: None,
            restart_policy: RestartPolicy::default_policy(),
        }
    }

    /// Get current tunnel state
    pub async fn state(&self) -> TunnelState {
        self.state.read().await.clone()
    }

    /// Subscribe to tunnel events
    pub fn subscribe(&self) -> broadcast::Receiver<TunnelEvent> {
        self.event_tx.subscribe()
    }

    /// Check if tunnel is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the tunnel configuration
    pub fn config(&self) -> &TunnelConfig {
        &self.config
    }

    /// Start the tunnel
    pub async fn start(&mut self) -> Result<(), TunnelError> {
        if !self.config.enabled {
            return Err(TunnelError::NotEnabled);
        }

        // Check cloudflared is installed
        let info = cloudflared::check_installation()
            .await
            .ok_or(TunnelError::CloudflaredNotInstalled)?;

        info!("Using cloudflared {} at {}", info.version, info.path);

        self.set_state(TunnelState::Starting).await;
        self.emit(TunnelEvent::Starting);

        self.spawn_process().await
    }

    /// Stop the tunnel
    pub async fn stop(&mut self) -> Result<(), TunnelError> {
        if let Some(mut child) = self.process.take() {
            info!("Stopping tunnel");
            child.kill().await.ok();
            self.set_state(TunnelState::Stopped).await;
            self.emit(TunnelEvent::Stopped);
        }
        Ok(())
    }

    /// Spawn the cloudflared process
    async fn spawn_process(&mut self) -> Result<(), TunnelError> {
        let child =
            cloudflared::spawn_tunnel(&self.config.mode, self.local_port).map_err(|e| {
                error!("Failed to spawn cloudflared: {}", e);
                TunnelError::SpawnFailed(e.to_string())
            })?;

        self.process = Some(child);
        self.restart_policy.reset();

        // TODO: Start output monitoring task
        // For now, mark as connected after spawn
        // Real implementation will parse output for URL

        Ok(())
    }

    /// Set state and log change
    async fn set_state(&self, new_state: TunnelState) {
        let mut state = self.state.write().await;
        debug!("Tunnel state: {:?} -> {:?}", *state, new_state);
        *state = new_state;
    }

    /// Emit an event
    fn emit(&self, event: TunnelEvent) {
        let _ = self.event_tx.send(event);
    }
}

/// Errors from tunnel operations
#[derive(Debug, thiserror::Error)]
pub enum TunnelError {
    #[error("Tunnel is not enabled in configuration")]
    NotEnabled,

    #[error("cloudflared is not installed")]
    CloudflaredNotInstalled,

    #[error("Failed to spawn cloudflared: {0}")]
    SpawnFailed(String),

    #[error("Tunnel failed: {0}")]
    Failed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tunnel_manager_disabled_by_default() {
        let config = TunnelConfig::default();
        let manager = TunnelManager::new(config, 7432);
        assert!(!manager.is_enabled());
    }

    #[tokio::test]
    async fn tunnel_manager_start_when_disabled_returns_error() {
        let config = TunnelConfig::default();
        let mut manager = TunnelManager::new(config, 7432);
        let result = manager.start().await;
        assert!(matches!(result, Err(TunnelError::NotEnabled)));
    }

    #[tokio::test]
    async fn tunnel_manager_initial_state_is_disabled() {
        let config = TunnelConfig::default();
        let manager = TunnelManager::new(config, 7432);
        let state = manager.state().await;
        assert!(matches!(state, TunnelState::Disabled));
    }

    #[tokio::test]
    async fn tunnel_manager_subscribe_returns_receiver() {
        let config = TunnelConfig::quick();
        let manager = TunnelManager::new(config, 7432);
        let _rx = manager.subscribe();
        // Should not panic
    }
}
