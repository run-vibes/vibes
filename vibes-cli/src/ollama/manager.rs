//! Ollama server subprocess lifecycle management.
//!
//! Handles starting and stopping the Ollama server with automatic detection
//! of already-running instances and missing installations.

use std::process::{Child, Command, Stdio};
use std::sync::RwLock;
use std::time::Duration;

use tracing::{debug, info, warn};

use crate::config::OllamaConfigSection;

/// Timeout for readiness checks.
const READY_CHECK_TIMEOUT: Duration = Duration::from_secs(2);

/// Maximum time to wait for Ollama to become ready.
const READY_TIMEOUT: Duration = Duration::from_secs(30);

/// Interval between readiness check attempts.
const READY_CHECK_INTERVAL: Duration = Duration::from_millis(200);

/// Result type for Ollama operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during Ollama management.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Ollama binary not found in PATH.
    #[error("Ollama is not installed or not in PATH")]
    #[allow(dead_code)] // For future use - currently we warn instead of error
    NotInstalled,

    /// Failed to spawn Ollama process.
    #[error("Failed to spawn Ollama: {0}")]
    Spawn(#[from] std::io::Error),

    /// Ollama failed to become ready within timeout.
    #[error("Ollama failed to become ready within {0:?}")]
    Timeout(Duration),

    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    Http(String),
}

/// Manages the Ollama server subprocess lifecycle.
///
/// Handles starting, stopping, and detecting Ollama with graceful
/// handling of common scenarios:
/// - Ollama already running: detects and skips spawn
/// - Ollama not installed: warns but doesn't fail
/// - Clean shutdown: kills child process on drop
pub struct OllamaManager {
    /// Configuration for Ollama.
    config: OllamaConfigSection,

    /// The server subprocess handle (if we spawned it).
    process: RwLock<Option<Child>>,

    /// Whether we spawned the process (vs. it was already running).
    we_spawned: RwLock<bool>,
}

impl OllamaManager {
    /// Create a new Ollama manager with the given configuration.
    #[must_use]
    pub fn new(config: OllamaConfigSection) -> Self {
        Self {
            config,
            process: RwLock::new(None),
            we_spawned: RwLock::new(false),
        }
    }

    /// Check if Ollama is enabled in configuration.
    #[must_use]
    #[allow(dead_code)] // Public API for status checks
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Start Ollama if enabled and not already running.
    ///
    /// Returns `Ok(())` in all non-error cases:
    /// - Already running: logs and returns Ok
    /// - Not installed: warns and returns Ok
    /// - Started successfully: logs and returns Ok
    ///
    /// Only returns `Err` for unexpected failures during spawn.
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            debug!("Ollama autostart disabled, skipping");
            return Ok(());
        }

        // Check if already running
        if self.is_running().await {
            info!(
                host = %self.config.host_or_default(),
                "Ollama is already running, skipping spawn"
            );
            return Ok(());
        }

        // Check if installed
        if !Self::is_installed() {
            warn!(
                "Ollama is not installed or not in PATH. \
                 Install from https://ollama.ai to enable local models."
            );
            return Ok(());
        }

        // Spawn ollama serve
        info!(
            host = %self.config.host_or_default(),
            "Starting Ollama server"
        );

        let mut cmd = Command::new("ollama");
        cmd.arg("serve");

        // Set OLLAMA_HOST if custom host is configured
        if let Some(ref host) = self.config.host {
            cmd.env("OLLAMA_HOST", host);
        }

        // Redirect stdout/stderr to /dev/null to avoid noise
        // (Ollama logs to stderr by default)
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());

        let child = cmd.spawn()?;
        let pid = child.id();

        *self.process.write().unwrap() = Some(child);
        *self.we_spawned.write().unwrap() = true;

        info!(pid = pid, "Ollama process spawned");

        // Wait for it to be ready
        self.wait_for_ready().await?;

        info!("Ollama is ready");
        Ok(())
    }

    /// Stop Ollama if we spawned it.
    ///
    /// Does nothing if Ollama was already running when we started.
    #[allow(dead_code)] // Public API for explicit shutdown; Drop also handles cleanup
    pub fn stop(&self) {
        let we_spawned = *self.we_spawned.read().unwrap();
        if !we_spawned {
            debug!("We didn't spawn Ollama, not stopping");
            return;
        }

        let mut process_guard = self.process.write().unwrap();
        if let Some(mut child) = process_guard.take() {
            info!("Stopping Ollama server");

            if let Err(e) = child.kill() {
                // ESRCH (no such process) is fine - process already exited
                if e.kind() != std::io::ErrorKind::NotFound {
                    warn!(error = %e, "Failed to kill Ollama server");
                }
            }

            match child.wait() {
                Ok(status) => {
                    info!(status = ?status, "Ollama server stopped");
                }
                Err(e) => {
                    warn!(error = %e, "Error waiting for Ollama to exit");
                }
            }
        }

        *self.we_spawned.write().unwrap() = false;
    }

    /// Check if Ollama is running by pinging the API.
    pub async fn is_running(&self) -> bool {
        let url = format!("{}/api/tags", self.config.base_url());

        let client = match reqwest::Client::builder()
            .timeout(READY_CHECK_TIMEOUT)
            .build()
        {
            Ok(c) => c,
            Err(_) => return false,
        };

        client.get(&url).send().await.is_ok()
    }

    /// Check if the `ollama` binary is in PATH.
    #[must_use]
    pub fn is_installed() -> bool {
        which::which("ollama").is_ok()
    }

    /// Wait for Ollama to become ready.
    async fn wait_for_ready(&self) -> Result<()> {
        let start = std::time::Instant::now();
        let url = format!("{}/api/tags", self.config.base_url());

        let client = reqwest::Client::builder()
            .timeout(READY_CHECK_TIMEOUT)
            .build()
            .map_err(|e| Error::Http(e.to_string()))?;

        loop {
            match client.get(&url).send().await {
                Ok(response) if response.status().is_success() => {
                    debug!(
                        elapsed_ms = start.elapsed().as_millis() as u64,
                        "Ollama is ready"
                    );
                    return Ok(());
                }
                Ok(_) | Err(_) => {
                    // Not ready yet
                }
            }

            if start.elapsed() >= READY_TIMEOUT {
                return Err(Error::Timeout(READY_TIMEOUT));
            }

            // Check if process crashed
            {
                let mut process_guard = self.process.write().unwrap();
                if let Some(ref mut child) = *process_guard {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            warn!(status = ?status, "Ollama process exited unexpectedly");
                            return Err(Error::Spawn(std::io::Error::new(
                                std::io::ErrorKind::BrokenPipe,
                                "Ollama process exited before becoming ready",
                            )));
                        }
                        Ok(None) => {
                            // Still running, continue waiting
                        }
                        Err(e) => {
                            return Err(Error::Spawn(e));
                        }
                    }
                }
            }

            tokio::time::sleep(READY_CHECK_INTERVAL).await;
        }
    }
}

impl Drop for OllamaManager {
    fn drop(&mut self) {
        // Only stop if we spawned it
        let we_spawned = *self.we_spawned.read().unwrap();
        if !we_spawned {
            return;
        }

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
    fn manager_disabled_by_default() {
        let config = OllamaConfigSection::default();
        let manager = OllamaManager::new(config);
        assert!(!manager.is_enabled());
    }

    #[test]
    fn manager_enabled_when_configured() {
        let config = OllamaConfigSection {
            enabled: true,
            host: None,
        };
        let manager = OllamaManager::new(config);
        assert!(manager.is_enabled());
    }

    #[test]
    fn is_installed_returns_bool() {
        // Just test that it doesn't panic and returns a bool
        let _ = OllamaManager::is_installed();
    }

    #[tokio::test]
    async fn start_skips_when_disabled() {
        let config = OllamaConfigSection::default();
        let manager = OllamaManager::new(config);

        // Should succeed without doing anything
        manager.start().await.unwrap();

        // Should not have spawned anything
        assert!(!*manager.we_spawned.read().unwrap());
    }
}
