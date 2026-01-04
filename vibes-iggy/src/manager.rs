//! Iggy server subprocess lifecycle management.
//!
//! Manages starting, stopping, and supervising the Iggy server process
//! with automatic health checks and restart capabilities.

use std::io::{BufRead, BufReader};
use std::process::{Child, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tokio::sync::RwLock;
use tracing::{debug, error, info, trace, warn};

use crate::config::IggyConfig;
use crate::error::{Error, Result};

/// Timeout for readiness checks (per request).
const READY_CHECK_TIMEOUT: Duration = Duration::from_secs(1);

/// Maximum time to wait for Iggy to become fully ready.
const READY_TIMEOUT: Duration = Duration::from_secs(30);

/// Interval between readiness check attempts.
const READY_CHECK_INTERVAL: Duration = Duration::from_millis(100);

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
    /// Spawns the iggy-server binary with environment variables for configuration.
    /// Iggy uses env vars (IGGY_TCP_ADDRESS, IGGY_HTTP_ADDRESS, IGGY_SYSTEM_PATH)
    /// rather than CLI flags.
    pub async fn start(&self) -> Result<()> {
        let current_state = self.state().await;
        if current_state == IggyState::Running {
            debug!("Iggy server already running, skipping start");
            return Ok(());
        }

        // Set state to starting
        *self.state.write().await = IggyState::Starting;

        // Find the binary
        let binary_path = self.config.find_binary().ok_or(Error::BinaryNotFound)?;

        let env_vars = self.config.env_vars();
        info!(
            binary = %binary_path.display(),
            data_dir = %self.config.data_dir.display(),
            tcp_port = self.config.port,
            http_port = self.config.http_port,
            "Starting Iggy server"
        );

        // Ensure data directory exists
        std::fs::create_dir_all(&self.config.data_dir)?;

        // Spawn the server process with environment variables
        // Iggy uses env vars for configuration, not CLI flags
        // Capture stderr for debugging (stdout goes to /dev/null to avoid noise)
        let mut child = std::process::Command::new(&binary_path)
            .envs(&env_vars)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                Error::Io(std::io::Error::new(
                    e.kind(),
                    format!("Failed to spawn iggy-server: {}", e),
                ))
            })?;

        let pid = child.id();

        // Spawn a thread to log stderr output for debugging
        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            // Check for log level indicators at word boundaries
                            // to avoid false positives like "The error handler"
                            let is_error = line.contains(" ERROR ")
                                || line.starts_with("ERROR ")
                                || line.contains("[ERROR]");
                            let is_warn = line.contains(" WARN ")
                                || line.starts_with("WARN ")
                                || line.contains("[WARN]");

                            if is_error {
                                error!(target: "iggy", "{}", line);
                            } else if is_warn {
                                warn!(target: "iggy", "{}", line);
                            } else {
                                debug!(target: "iggy", "{}", line);
                            }
                        }
                        Err(e) => {
                            debug!("Error reading iggy stderr: {}", e);
                            break;
                        }
                    }
                }
            });
        }
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
                Ok(None) => true,     // Still running
                Ok(Some(_)) => false, // Process exited
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

    /// Signal the supervisor to stop.
    pub fn signal_shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }

    /// Get the connection address for clients.
    #[must_use]
    pub fn connection_address(&self) -> String {
        self.config.connection_address()
    }

    /// Get the configuration.
    #[must_use]
    pub fn config(&self) -> &IggyConfig {
        &self.config
    }

    /// Wait for Iggy to be fully ready to accept connections.
    ///
    /// This polls both the HTTP and TCP endpoints until they respond successfully.
    /// This is crucial because:
    /// - CLI hooks use the HTTP API to send events
    /// - The Rust SDK uses TCP for event streaming
    ///
    /// Without waiting for both protocols, there's a race condition where:
    /// 1. Iggy starts and TCP becomes ready
    /// 2. We declare Iggy ready and start accepting requests
    /// 3. CLI hooks fire events via HTTP
    /// 4. HTTP server isn't ready yet → events lost!
    pub async fn wait_for_ready(&self) -> Result<()> {
        let start = std::time::Instant::now();

        info!(
            tcp_port = self.config.port,
            http_port = self.config.http_port,
            "Waiting for Iggy to become ready (HTTP + TCP)"
        );

        // Check if process is even running first
        if !self.is_running().await && self.state().await != IggyState::Starting {
            return Err(Error::Connection("Iggy server is not running".to_string()));
        }

        // Wait for both HTTP and TCP concurrently
        let http_ready = self.wait_for_http_ready(start);
        let tcp_ready = self.wait_for_tcp_ready(start);

        // Both must succeed
        let (http_result, tcp_result) = tokio::join!(http_ready, tcp_ready);

        http_result?;
        tcp_result?;

        debug!(
            elapsed_ms = start.elapsed().as_millis() as u64,
            "Iggy is fully ready (HTTP + TCP)"
        );

        Ok(())
    }

    /// Wait for the HTTP endpoint to become ready.
    async fn wait_for_http_ready(&self, start: std::time::Instant) -> Result<()> {
        let http_url = format!("http://127.0.0.1:{}/", self.config.http_port);

        let client = reqwest::Client::builder()
            .timeout(READY_CHECK_TIMEOUT)
            .build()
            .map_err(|e| Error::Connection(format!("Failed to create HTTP client: {}", e)))?;

        let mut attempts = 0;

        loop {
            attempts += 1;

            match client.get(&http_url).send().await {
                Ok(response) => {
                    // Any response (even 404) means HTTP server is up
                    debug!(
                        status = %response.status(),
                        attempts = attempts,
                        elapsed_ms = start.elapsed().as_millis() as u64,
                        "Iggy HTTP API is ready"
                    );
                    return Ok(());
                }
                Err(e) => {
                    trace!(
                        attempt = attempts,
                        error = %e,
                        "HTTP check failed, retrying..."
                    );
                }
            }

            if start.elapsed() >= READY_TIMEOUT {
                return Err(Error::Connection(format!(
                    "Iggy HTTP API failed to become ready within {:?} ({} attempts)",
                    READY_TIMEOUT, attempts
                )));
            }

            // Check if server crashed while waiting
            if !self.is_running().await {
                return Err(Error::Connection(
                    "Iggy server exited while waiting for HTTP readiness".to_string(),
                ));
            }

            tokio::time::sleep(READY_CHECK_INTERVAL).await;
        }
    }

    /// Wait for the TCP endpoint to become ready.
    async fn wait_for_tcp_ready(&self, start: std::time::Instant) -> Result<()> {
        let tcp_addr = format!("127.0.0.1:{}", self.config.port);
        let mut attempts = 0;

        loop {
            attempts += 1;

            match tokio::time::timeout(
                READY_CHECK_TIMEOUT,
                tokio::net::TcpStream::connect(&tcp_addr),
            )
            .await
            {
                Ok(Ok(_stream)) => {
                    // Connection succeeded
                    debug!(
                        attempts = attempts,
                        elapsed_ms = start.elapsed().as_millis() as u64,
                        "Iggy TCP API is ready"
                    );
                    return Ok(());
                }
                Ok(Err(e)) => {
                    trace!(
                        attempt = attempts,
                        error = %e,
                        "TCP check failed, retrying..."
                    );
                }
                Err(_) => {
                    trace!(attempt = attempts, "TCP check timed out, retrying...");
                }
            }

            if start.elapsed() >= READY_TIMEOUT {
                return Err(Error::Connection(format!(
                    "Iggy TCP API failed to become ready within {:?} ({} attempts)",
                    READY_TIMEOUT, attempts
                )));
            }

            // Check if server crashed while waiting
            if !self.is_running().await {
                return Err(Error::Connection(
                    "Iggy server exited while waiting for TCP readiness".to_string(),
                ));
            }

            tokio::time::sleep(READY_CHECK_INTERVAL).await;
        }
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

    #[tokio::test]
    async fn manager_initial_state_is_stopped() {
        let config = IggyConfig::default();
        let manager = IggyManager::new(config);

        assert_eq!(manager.state().await, IggyState::Stopped);
    }

    #[test]
    fn manager_connection_address() {
        let config = IggyConfig::default().with_port(8091);
        let manager = IggyManager::new(config);

        assert_eq!(manager.connection_address(), "127.0.0.1:8091");
    }

    #[test]
    fn state_display() {
        assert_eq!(format!("{}", IggyState::Stopped), "stopped");
        assert_eq!(format!("{}", IggyState::Starting), "starting");
        assert_eq!(format!("{}", IggyState::Running), "running");
        assert_eq!(format!("{}", IggyState::Restarting), "restarting");
        assert_eq!(format!("{}", IggyState::Failed), "failed");
    }

    #[tokio::test]
    async fn manager_is_running_returns_false_when_stopped() {
        let config = IggyConfig::default();
        let manager = IggyManager::new(config);

        assert!(!manager.is_running().await);
    }

    #[test]
    fn manager_config_accessor() {
        let config = IggyConfig::default().with_port(9999);
        let manager = IggyManager::new(config);

        assert_eq!(manager.config().port, 9999);
    }
}
