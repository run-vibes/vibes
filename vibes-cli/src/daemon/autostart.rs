//! Daemon auto-start functionality
//!
//! Ensures the vibes daemon is running before CLI operations.
//! Starts the daemon as a detached background process if needed.

use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result};
use tracing::{debug, info};

use super::state::{is_process_alive, read_daemon_state};
use crate::commands::serve::{DEFAULT_HOST, DEFAULT_PORT};

/// Default timeout for waiting for daemon to become ready
const DEFAULT_STARTUP_TIMEOUT: Duration = Duration::from_secs(10);

/// Interval between health check attempts
const HEALTH_CHECK_INTERVAL: Duration = Duration::from_millis(100);

/// Ensure the vibes daemon is running on the specified port.
///
/// If a daemon is already running on the port, returns Ok immediately.
/// Otherwise, starts a new daemon process and waits for it to become ready.
pub async fn ensure_daemon_running(port: u16) -> Result<()> {
    // Check if daemon is already running
    if let Some(state) = read_daemon_state()
        && state.port == port
        && is_process_alive(state.pid)
    {
        debug!(
            "Daemon already running on port {} (PID: {})",
            port, state.pid
        );
        return Ok(());
    }

    // Start the daemon
    info!("Starting vibes daemon on port {}...", port);
    start_daemon_process(port)?;

    // Wait for it to become ready
    wait_for_daemon_ready(port, DEFAULT_STARTUP_TIMEOUT).await?;

    info!("Daemon started successfully");
    Ok(())
}

/// Start the daemon as a detached background process.
///
/// On Unix, this creates a new session using setsid() so the daemon
/// survives the parent CLI process exiting.
#[cfg(unix)]
fn start_daemon_process(port: u16) -> Result<()> {
    use std::os::unix::process::CommandExt;

    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;

    // Spawn a detached process running `vibes serve`
    // Using std::process::Command which is safe (no shell interpretation)
    let mut cmd = std::process::Command::new(&current_exe);
    cmd.arg("serve")
        .arg("--port")
        .arg(port.to_string())
        .arg("--host")
        .arg(DEFAULT_HOST)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // Create a new session so the daemon is independent of the CLI
    // SAFETY: pre_exec is called after fork, before exec. setsid creates
    // a new session, making this process the session leader.
    unsafe {
        cmd.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }

    let child = cmd.spawn().context("Failed to spawn daemon process")?;

    debug!("Spawned daemon process with PID: {}", child.id());
    Ok(())
}

/// Start the daemon as a detached background process (Windows stub).
#[cfg(not(unix))]
fn start_daemon_process(port: u16) -> Result<()> {
    let _ = port;
    // TODO: Implement Windows daemon spawning
    anyhow::bail!("Daemon auto-start not yet implemented on Windows")
}

/// Wait for the daemon to become ready by polling the health endpoint.
async fn wait_for_daemon_ready(port: u16, timeout: Duration) -> Result<()> {
    let health_url = format!("http://{}:{}/api/health", DEFAULT_HOST, port);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(1))
        .build()
        .context("Failed to create HTTP client")?;

    let start = std::time::Instant::now();
    let mut attempts = 0;

    loop {
        attempts += 1;

        match client.get(&health_url).send().await {
            Ok(response) if response.status().is_success() => {
                debug!(
                    "Daemon ready after {} attempts ({:?})",
                    attempts,
                    start.elapsed()
                );
                return Ok(());
            }
            Ok(response) => {
                debug!(
                    "Health check returned status {}, retrying...",
                    response.status()
                );
            }
            Err(e) => {
                debug!("Health check failed: {}, retrying...", e);
            }
        }

        if start.elapsed() >= timeout {
            anyhow::bail!(
                "Daemon failed to start within {:?} (attempted {} health checks)",
                timeout,
                attempts
            );
        }

        tokio::time::sleep(HEALTH_CHECK_INTERVAL).await;
    }
}

/// Ensure daemon is running with default port.
/// Used by the claude command when auto-starting the daemon.
#[allow(dead_code)] // Will be used in Task 4.2
pub async fn ensure_daemon_running_default() -> Result<()> {
    ensure_daemon_running(DEFAULT_PORT).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_startup_timeout_is_10_seconds() {
        assert_eq!(DEFAULT_STARTUP_TIMEOUT, Duration::from_secs(10));
    }

    #[test]
    fn test_health_check_interval_is_100ms() {
        assert_eq!(HEALTH_CHECK_INTERVAL, Duration::from_millis(100));
    }

    // Integration tests for daemon auto-start would require actual process spawning
    // and are better suited for the integration test suite
}
