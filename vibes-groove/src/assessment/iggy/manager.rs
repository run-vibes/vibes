//! Iggy server subprocess lifecycle management.
//!
//! This module manages the Iggy server as a subprocess, handling:
//! - Server startup and shutdown
//! - Health monitoring with automatic restarts
//! - Graceful shutdown on drop

use std::path::PathBuf;
use std::process::Child;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::error::{GrooveError, Result};
use crate::paths::GroovePaths;

/// Configuration for the Iggy server subprocess.
#[derive(Debug, Clone)]
pub struct IggyConfig {
    /// Path to the iggy-server binary.
    pub binary_path: PathBuf,

    /// Directory where Iggy stores its data.
    pub data_dir: PathBuf,

    /// TCP port for Iggy server.
    pub port: u16,

    /// Interval between health checks.
    pub health_check_interval: Duration,

    /// Maximum number of restart attempts before giving up.
    pub max_restart_attempts: u32,
}

impl Default for IggyConfig {
    fn default() -> Self {
        // Use vibes plugin data directory for iggy data
        let data_dir = GroovePaths::default_data_dir()
            .map(|d| d.join("iggy"))
            .unwrap_or_else(|| PathBuf::from("/tmp/vibes-groove/iggy"));

        Self {
            binary_path: PathBuf::from("iggy-server"),
            data_dir,
            port: 8090,
            health_check_interval: Duration::from_secs(5),
            max_restart_attempts: 3,
        }
    }
}

impl IggyConfig {
    /// Create a new config with a custom binary path.
    #[must_use]
    pub fn with_binary_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.binary_path = path.into();
        self
    }

    /// Create a new config with a custom data directory.
    #[must_use]
    pub fn with_data_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.data_dir = path.into();
        self
    }

    /// Create a new config with a custom port.
    #[must_use]
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Create a new config with a custom health check interval.
    #[must_use]
    pub fn with_health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = interval;
        self
    }

    /// Create a new config with a custom max restart attempts.
    #[must_use]
    pub fn with_max_restart_attempts(mut self, attempts: u32) -> Self {
        self.max_restart_attempts = attempts;
        self
    }
}

/// State of the Iggy server subprocess.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IggyState {
    /// Server is stopped.
    Stopped,
    /// Server is starting up.
    Starting,
    /// Server is running and healthy.
    Running,
    /// Server is restarting after a failure.
    Restarting,
    /// Server failed and max restart attempts exhausted.
    Failed,
}

impl std::fmt::Display for IggyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stopped => write!(f, "stopped"),
            Self::Starting => write!(f, "starting"),
            Self::Running => write!(f, "running"),
            Self::Restarting => write!(f, "restarting"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Manages the Iggy server subprocess lifecycle.
///
/// Handles starting, stopping, and supervising the Iggy server process
/// with automatic health checks and restart capabilities.
pub struct IggyManager {
    /// Configuration for the server.
    config: IggyConfig,

    /// The server subprocess handle.
    process: RwLock<Option<Child>>,

    /// Current state of the server.
    state: RwLock<IggyState>,

    /// Signal to stop the supervisor loop.
    shutdown: Arc<AtomicBool>,

    /// Current restart attempt count.
    restart_count: RwLock<u32>,
}

impl IggyManager {
    /// Create a new Iggy manager with the given configuration.
    #[must_use]
    pub fn new(config: IggyConfig) -> Self {
        Self {
            config,
            process: RwLock::new(None),
            state: RwLock::new(IggyState::Stopped),
            shutdown: Arc::new(AtomicBool::new(false)),
            restart_count: RwLock::new(0),
        }
    }

    /// Get the current state of the server.
    pub async fn state(&self) -> IggyState {
        *self.state.read().await
    }

    /// Start the Iggy server subprocess.
    ///
    /// Spawns the iggy-server binary with appropriate arguments for
    /// data directory and TCP port configuration.
    pub async fn start(&self) -> Result<()> {
        let current_state = self.state().await;
        if current_state == IggyState::Running {
            debug!("Iggy server already running, skipping start");
            return Ok(());
        }

        // Set state to starting
        *self.state.write().await = IggyState::Starting;
        info!(
            binary = %self.config.binary_path.display(),
            data_dir = %self.config.data_dir.display(),
            port = self.config.port,
            "Starting Iggy server"
        );

        // Ensure data directory exists
        std::fs::create_dir_all(&self.config.data_dir)?;

        // Spawn the server process
        let child = std::process::Command::new(&self.config.binary_path)
            .arg("--data-dir")
            .arg(&self.config.data_dir)
            .arg("--tcp-port")
            .arg(self.config.port.to_string())
            .spawn()
            .map_err(|e| {
                GrooveError::Io(std::io::Error::new(
                    e.kind(),
                    format!("Failed to spawn iggy-server: {}", e),
                ))
            })?;

        let pid = child.id();
        *self.process.write().await = Some(child);
        *self.state.write().await = IggyState::Running;
        *self.restart_count.write().await = 0;

        info!(pid = pid, "Iggy server started successfully");
        Ok(())
    }

    /// Stop the Iggy server subprocess.
    ///
    /// Terminates the process and waits for it to exit.
    pub async fn stop(&self) -> Result<()> {
        let mut process_guard = self.process.write().await;

        if let Some(mut child) = process_guard.take() {
            info!("Stopping Iggy server");

            // Kill the process
            if let Err(e) = child.kill() {
                // ESRCH (no such process) is fine - process already exited
                if e.kind() != std::io::ErrorKind::NotFound {
                    error!(error = %e, "Failed to kill Iggy server");
                }
            }

            // Wait for the process to exit
            match child.wait() {
                Ok(status) => {
                    info!(status = ?status, "Iggy server stopped");
                }
                Err(e) => {
                    error!(error = %e, "Error waiting for Iggy server to exit");
                }
            }
        }

        *self.state.write().await = IggyState::Stopped;
        self.shutdown.store(true, Ordering::SeqCst);

        Ok(())
    }

    /// Check if the server process is currently running.
    pub async fn is_running(&self) -> bool {
        let mut process_guard = self.process.write().await;

        if let Some(ref mut child) = *process_guard {
            match child.try_wait() {
                Ok(None) => true, // Still running
                Ok(Some(_)) => {
                    // Process exited
                    false
                }
                Err(e) => {
                    error!(error = %e, "Error checking process status");
                    false
                }
            }
        } else {
            false
        }
    }

    /// Run the supervisor loop that monitors the server and restarts it if needed.
    ///
    /// Uses exponential backoff for restart attempts up to the configured maximum.
    pub async fn supervise(&self) -> Result<()> {
        info!("Starting Iggy server supervisor loop");

        while !self.shutdown.load(Ordering::SeqCst) {
            tokio::time::sleep(self.config.health_check_interval).await;

            if self.shutdown.load(Ordering::SeqCst) {
                break;
            }

            let is_running = self.is_running().await;
            let current_state = self.state().await;

            if !is_running && current_state == IggyState::Running {
                // Server crashed, attempt restart
                let restart_count = *self.restart_count.read().await;

                if restart_count >= self.config.max_restart_attempts {
                    error!(
                        attempts = restart_count,
                        max = self.config.max_restart_attempts,
                        "Max restart attempts reached, marking as failed"
                    );
                    *self.state.write().await = IggyState::Failed;
                    break;
                }

                warn!(
                    attempt = restart_count + 1,
                    max = self.config.max_restart_attempts,
                    "Iggy server crashed, attempting restart"
                );

                *self.state.write().await = IggyState::Restarting;
                *self.restart_count.write().await = restart_count + 1;

                // Exponential backoff: 1s, 2s, 4s, etc.
                let backoff = Duration::from_secs(1 << restart_count);
                debug!(backoff_secs = backoff.as_secs(), "Waiting before restart");
                tokio::time::sleep(backoff).await;

                if let Err(e) = self.start().await {
                    error!(error = %e, "Failed to restart Iggy server");
                }
            }
        }

        info!("Iggy server supervisor loop exited");
        Ok(())
    }

    /// Get the connection address for clients to connect to the server.
    #[must_use]
    pub fn connection_address(&self) -> String {
        format!("127.0.0.1:{}", self.config.port)
    }

    /// Get the configuration.
    #[must_use]
    pub fn config(&self) -> &IggyConfig {
        &self.config
    }
}

impl Drop for IggyManager {
    fn drop(&mut self) {
        // Signal shutdown to stop the supervisor loop
        self.shutdown.store(true, Ordering::SeqCst);

        // Attempt to stop the process synchronously
        if let Ok(mut guard) = self.process.try_write()
            && let Some(mut child) = guard.take()
        {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iggy_config_default_has_sensible_values() {
        let config = IggyConfig::default();

        // Binary path should be just the executable name (relies on PATH)
        assert_eq!(config.binary_path, PathBuf::from("iggy-server"));

        // Port should be 8090
        assert_eq!(config.port, 8090);

        // Max restart attempts should be 3
        assert_eq!(config.max_restart_attempts, 3);

        // Health check interval should be 5 seconds
        assert_eq!(config.health_check_interval, Duration::from_secs(5));

        // Data dir should contain "iggy" somewhere
        assert!(
            config.data_dir.to_string_lossy().contains("iggy"),
            "Data dir should contain 'iggy': {:?}",
            config.data_dir
        );
    }

    #[test]
    fn iggy_config_builder_pattern() {
        let config = IggyConfig::default()
            .with_binary_path("/usr/local/bin/iggy-server")
            .with_data_dir("/var/lib/iggy")
            .with_port(9090)
            .with_health_check_interval(Duration::from_secs(10))
            .with_max_restart_attempts(5);

        assert_eq!(
            config.binary_path,
            PathBuf::from("/usr/local/bin/iggy-server")
        );
        assert_eq!(config.data_dir, PathBuf::from("/var/lib/iggy"));
        assert_eq!(config.port, 9090);
        assert_eq!(config.health_check_interval, Duration::from_secs(10));
        assert_eq!(config.max_restart_attempts, 5);
    }

    #[tokio::test]
    async fn iggy_manager_initial_state_is_stopped() {
        let config = IggyConfig::default();
        let manager = IggyManager::new(config);

        assert_eq!(manager.state().await, IggyState::Stopped);
    }

    #[test]
    fn iggy_manager_connection_address() {
        let config = IggyConfig::default().with_port(8091);
        let manager = IggyManager::new(config);

        assert_eq!(manager.connection_address(), "127.0.0.1:8091");
    }

    #[test]
    fn iggy_state_display() {
        assert_eq!(format!("{}", IggyState::Stopped), "stopped");
        assert_eq!(format!("{}", IggyState::Starting), "starting");
        assert_eq!(format!("{}", IggyState::Running), "running");
        assert_eq!(format!("{}", IggyState::Restarting), "restarting");
        assert_eq!(format!("{}", IggyState::Failed), "failed");
    }

    #[tokio::test]
    async fn iggy_manager_is_running_returns_false_when_stopped() {
        let config = IggyConfig::default();
        let manager = IggyManager::new(config);

        // No process started, should return false
        assert!(!manager.is_running().await);
    }

    #[test]
    fn iggy_manager_config_accessor() {
        let config = IggyConfig::default().with_port(9999);
        let manager = IggyManager::new(config);

        assert_eq!(manager.config().port, 9999);
    }
}
