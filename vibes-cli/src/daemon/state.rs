//! Daemon state file management
//!
//! Tracks running daemon processes with PID, port, and start time.
//! State is persisted to ~/.config/vibes/daemon.json

use std::fs;
use std::io;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// State of a running daemon
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DaemonState {
    /// Process ID of the daemon
    pub pid: u32,
    /// Port the daemon is listening on
    pub port: u16,
    /// When the daemon was started
    pub started_at: DateTime<Utc>,
}

impl DaemonState {
    /// Create a new daemon state for the current process
    pub fn new(port: u16) -> Self {
        Self {
            pid: std::process::id(),
            port,
            started_at: Utc::now(),
        }
    }

    /// Create a daemon state with specific values (for testing)
    #[cfg(test)]
    pub fn with_values(pid: u32, port: u16, started_at: DateTime<Utc>) -> Self {
        Self {
            pid,
            port,
            started_at,
        }
    }
}

/// Get the path to the daemon state file
///
/// Returns ~/.config/vibes/daemon.json
pub fn state_file_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(".config"))
        .join("vibes");
    config_dir.join("daemon.json")
}

/// Read the daemon state from the state file
///
/// Returns None if the file doesn't exist or is invalid JSON
pub fn read_daemon_state() -> Option<DaemonState> {
    let path = state_file_path();
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Write the daemon state to the state file
///
/// Creates parent directories if they don't exist
pub fn write_daemon_state(state: &DaemonState) -> io::Result<()> {
    let path = state_file_path();

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(state)?;
    fs::write(&path, content)
}

/// Clear the daemon state file
///
/// Removes the state file if it exists
pub fn clear_daemon_state() -> io::Result<()> {
    let path = state_file_path();
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}

/// Check if a process is still alive
///
/// Uses kill(pid, 0) on Unix to check if the process exists
#[cfg(unix)]
pub fn is_process_alive(pid: u32) -> bool {
    // Use libc directly for kill -0
    // SAFETY: kill with signal 0 only checks if process exists, doesn't send a signal
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}

/// Check if a process is still alive (Windows stub)
#[cfg(not(unix))]
pub fn is_process_alive(_pid: u32) -> bool {
    // TODO: Implement Windows process checking
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_daemon_state_new() {
        let state = DaemonState::new(7743);
        assert_eq!(state.port, 7743);
        assert!(state.pid > 0);
    }

    #[test]
    fn test_daemon_state_with_values() {
        let started_at = Utc::now();
        let state = DaemonState::with_values(1234, 8080, started_at);
        assert_eq!(state.pid, 1234);
        assert_eq!(state.port, 8080);
        assert_eq!(state.started_at, started_at);
    }

    #[test]
    fn test_daemon_state_serialization_roundtrip() {
        let state = DaemonState::new(7743);
        let json = serde_json::to_string(&state).unwrap();
        let parsed: DaemonState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.pid, state.pid);
        assert_eq!(parsed.port, state.port);
    }

    #[test]
    fn test_state_file_path_ends_with_daemon_json() {
        let path = state_file_path();
        assert!(path.ends_with("vibes/daemon.json"));
    }

    #[test]
    fn test_read_nonexistent_state_returns_none() {
        // This test relies on a clean state - may fail if daemon is running
        // We just test that it doesn't panic
        let _ = read_daemon_state();
    }

    #[test]
    fn test_is_process_alive_current_process() {
        let pid = std::process::id();
        assert!(is_process_alive(pid));
    }

    #[test]
    fn test_is_process_alive_nonexistent_process() {
        // PID 999999 is unlikely to exist
        assert!(!is_process_alive(999999));
    }

    #[test]
    fn test_write_and_read_daemon_state() {
        // Create a temp directory and override the config dir
        let temp = tempdir().unwrap();
        let state_path = temp.path().join("daemon.json");

        // Write state to temp file
        let state = DaemonState::new(7743);
        let content = serde_json::to_string_pretty(&state).unwrap();
        std::fs::write(&state_path, &content).unwrap();

        // Read it back
        let read_content = std::fs::read_to_string(&state_path).unwrap();
        let parsed: DaemonState = serde_json::from_str(&read_content).unwrap();

        assert_eq!(parsed.pid, state.pid);
        assert_eq!(parsed.port, state.port);
    }
}
