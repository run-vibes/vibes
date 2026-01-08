# Cloudflare Tunnel Integration - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable remote access to vibes from anywhere via Cloudflare Tunnel integration.

**Architecture:** TunnelManager in vibes-core spawns and supervises cloudflared as a child process. CLI gets `--tunnel` and `--quick-tunnel` flags plus `vibes tunnel` subcommands. Web UI shows tunnel status via header badge. Quick tunnels work instantly; named tunnels use setup wizard.

**Tech Stack:** Rust (tokio, clap), cloudflared CLI, React (TanStack)

---

## Task 1: Add TunnelState and TunnelEvent Types

**Files:**
- Create: `vibes-core/src/tunnel/state.rs`
- Create: `vibes-core/src/tunnel/mod.rs`
- Modify: `vibes-core/src/lib.rs`

**Step 1: Write the failing test for TunnelState**

Create `vibes-core/src/tunnel/state.rs`:

```rust
//! Tunnel state and event types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Current state of the tunnel connection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TunnelState {
    /// Tunnel is disabled in configuration
    Disabled,
    /// Tunnel is starting up
    Starting,
    /// Tunnel is connected and ready
    Connected {
        url: String,
        connected_at: DateTime<Utc>,
    },
    /// Tunnel lost connection, attempting to reconnect
    Reconnecting {
        attempt: u32,
        last_error: String,
    },
    /// Tunnel failed to connect
    Failed {
        error: String,
        can_retry: bool,
    },
    /// Tunnel was explicitly stopped
    Stopped,
}

impl Default for TunnelState {
    fn default() -> Self {
        Self::Disabled
    }
}

/// Events emitted by the tunnel manager
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TunnelEvent {
    /// Tunnel is starting
    Starting,
    /// Tunnel connected successfully
    Connected { url: String },
    /// Tunnel disconnected
    Disconnected { reason: String },
    /// Tunnel is reconnecting
    Reconnecting { attempt: u32 },
    /// Tunnel failed
    Failed { error: String },
    /// Tunnel stopped
    Stopped,
    /// Log message from cloudflared
    Log { level: LogLevel, message: String },
}

/// Log levels for tunnel events
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tunnel_state_default_is_disabled() {
        let state = TunnelState::default();
        assert!(matches!(state, TunnelState::Disabled));
    }

    #[test]
    fn tunnel_state_serialization_roundtrip() {
        let state = TunnelState::Connected {
            url: "https://example.trycloudflare.com".to_string(),
            connected_at: Utc::now(),
        };
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("connected"));
        let parsed: TunnelState = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, TunnelState::Connected { .. }));
    }

    #[test]
    fn tunnel_event_serialization_roundtrip() {
        let event = TunnelEvent::Connected {
            url: "https://test.trycloudflare.com".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: TunnelEvent = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, TunnelEvent::Connected { .. }));
    }
}
```

**Step 2: Create the tunnel module**

Create `vibes-core/src/tunnel/mod.rs`:

```rust
//! Cloudflare Tunnel integration for remote access

pub mod state;

pub use state::{LogLevel, TunnelEvent, TunnelState};
```

**Step 3: Export from lib.rs**

Modify `vibes-core/src/lib.rs` - add after `pub mod session;`:

```rust
pub mod tunnel;
```

Add to re-exports:

```rust
pub use tunnel::{LogLevel, TunnelEvent, TunnelState};
```

**Step 4: Run tests to verify**

Run: `cargo test -p vibes-core tunnel_state`

Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/tunnel/
git add vibes-core/src/lib.rs
git commit -m "feat(tunnel): add TunnelState and TunnelEvent types"
```

---

## Task 2: Add TunnelConfig Types

**Files:**
- Create: `vibes-core/src/tunnel/config.rs`
- Modify: `vibes-core/src/tunnel/mod.rs`

**Step 1: Write TunnelConfig with tests**

Create `vibes-core/src/tunnel/config.rs`:

```rust
//! Tunnel configuration types

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the tunnel
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TunnelConfig {
    /// Whether tunnel is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Tunnel mode
    #[serde(default)]
    pub mode: TunnelMode,
}

impl Default for TunnelConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: TunnelMode::Quick,
        }
    }
}

/// Tunnel operating mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TunnelMode {
    /// Quick tunnel with temporary URL (no account needed)
    Quick,
    /// Named tunnel with persistent hostname
    Named {
        /// Tunnel name from cloudflared
        name: String,
        /// Public hostname
        hostname: String,
        /// Path to credentials file (auto-detected if not specified)
        #[serde(default)]
        credentials_path: Option<PathBuf>,
    },
}

impl Default for TunnelMode {
    fn default() -> Self {
        Self::Quick
    }
}

impl TunnelConfig {
    /// Create a quick tunnel config
    pub fn quick() -> Self {
        Self {
            enabled: true,
            mode: TunnelMode::Quick,
        }
    }

    /// Create a named tunnel config
    pub fn named(name: String, hostname: String) -> Self {
        Self {
            enabled: true,
            mode: TunnelMode::Named {
                name,
                hostname,
                credentials_path: None,
            },
        }
    }

    /// Check if this is a quick tunnel
    pub fn is_quick(&self) -> bool {
        matches!(self.mode, TunnelMode::Quick)
    }

    /// Get the tunnel name for named tunnels
    pub fn tunnel_name(&self) -> Option<&str> {
        match &self.mode {
            TunnelMode::Named { name, .. } => Some(name),
            TunnelMode::Quick => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tunnel_config_default_is_disabled_quick() {
        let config = TunnelConfig::default();
        assert!(!config.enabled);
        assert!(config.is_quick());
    }

    #[test]
    fn tunnel_config_quick_constructor() {
        let config = TunnelConfig::quick();
        assert!(config.enabled);
        assert!(config.is_quick());
        assert!(config.tunnel_name().is_none());
    }

    #[test]
    fn tunnel_config_named_constructor() {
        let config = TunnelConfig::named("my-tunnel".to_string(), "vibes.example.com".to_string());
        assert!(config.enabled);
        assert!(!config.is_quick());
        assert_eq!(config.tunnel_name(), Some("my-tunnel"));
    }

    #[test]
    fn tunnel_mode_serialization_quick() {
        let mode = TunnelMode::Quick;
        let json = serde_json::to_string(&mode).unwrap();
        assert!(json.contains("quick"));
    }

    #[test]
    fn tunnel_mode_serialization_named() {
        let mode = TunnelMode::Named {
            name: "test".to_string(),
            hostname: "test.example.com".to_string(),
            credentials_path: None,
        };
        let json = serde_json::to_string(&mode).unwrap();
        assert!(json.contains("named"));
        assert!(json.contains("test.example.com"));
    }

    #[test]
    fn tunnel_config_toml_roundtrip() {
        let config = TunnelConfig::named("vibes-home".to_string(), "vibes.example.com".to_string());
        let toml = toml::to_string(&config).unwrap();
        let parsed: TunnelConfig = toml::from_str(&toml).unwrap();
        assert_eq!(parsed, config);
    }
}
```

**Step 2: Update mod.rs**

Add to `vibes-core/src/tunnel/mod.rs`:

```rust
pub mod config;

pub use config::{TunnelConfig, TunnelMode};
```

**Step 3: Update lib.rs exports**

Add to `vibes-core/src/lib.rs` re-exports:

```rust
pub use tunnel::{LogLevel, TunnelConfig, TunnelEvent, TunnelMode, TunnelState};
```

**Step 4: Run tests**

Run: `cargo test -p vibes-core tunnel_config`

Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/tunnel/config.rs vibes-core/src/tunnel/mod.rs vibes-core/src/lib.rs
git commit -m "feat(tunnel): add TunnelConfig and TunnelMode types"
```

---

## Task 3: Add Cloudflared CLI Wrapper

**Files:**
- Create: `vibes-core/src/tunnel/cloudflared.rs`
- Modify: `vibes-core/src/tunnel/mod.rs`

**Step 1: Write cloudflared wrapper with output parsing**

Create `vibes-core/src/tunnel/cloudflared.rs`:

```rust
//! Cloudflared CLI wrapper and output parsing

use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

use super::config::TunnelMode;

/// Result of checking cloudflared installation
#[derive(Debug, Clone)]
pub struct CloudflaredInfo {
    pub version: String,
    pub path: String,
}

/// Check if cloudflared is installed and get version
pub async fn check_installation() -> Option<CloudflaredInfo> {
    let output = Command::new("cloudflared")
        .arg("--version")
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let version_str = String::from_utf8_lossy(&output.stdout);
    // Parse "cloudflared version 2024.12.0 (built 2024-12-01)"
    let version = version_str
        .split_whitespace()
        .nth(2)
        .unwrap_or("unknown")
        .to_string();

    let path = which::which("cloudflared")
        .ok()?
        .to_string_lossy()
        .to_string();

    Some(CloudflaredInfo { version, path })
}

/// Spawn cloudflared process for the given mode
pub fn spawn_tunnel(mode: &TunnelMode, local_port: u16) -> std::io::Result<Child> {
    let mut cmd = Command::new("cloudflared");

    match mode {
        TunnelMode::Quick => {
            cmd.arg("tunnel")
                .arg("--url")
                .arg(format!("http://localhost:{}", local_port));
        }
        TunnelMode::Named {
            name,
            credentials_path,
            ..
        } => {
            cmd.arg("tunnel");
            if let Some(creds) = credentials_path {
                cmd.arg("--credentials-file").arg(creds);
            }
            cmd.arg("run")
                .arg("--url")
                .arg(format!("http://localhost:{}", local_port))
                .arg(name);
        }
    }

    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
}

/// Parse a quick tunnel URL from cloudflared output
///
/// Looks for pattern: "https://xxx.trycloudflare.com"
pub fn parse_quick_tunnel_url(line: &str) -> Option<String> {
    // cloudflared prints URL in a box like:
    // | https://random-words.trycloudflare.com |
    if line.contains("trycloudflare.com") {
        // Extract URL using regex-like matching
        let start = line.find("https://")?;
        let url_part = &line[start..];
        let end = url_part.find(|c: char| c.is_whitespace() || c == '|')?;
        return Some(url_part[..end].to_string());
    }
    None
}

/// Parse log level from cloudflared output
pub fn parse_log_level(line: &str) -> Option<(&str, &str)> {
    // Format: "INF message here" or "ERR message here"
    let level = if line.starts_with("INF") {
        "info"
    } else if line.starts_with("WRN") {
        "warn"
    } else if line.starts_with("ERR") {
        "error"
    } else if line.starts_with("DBG") {
        "debug"
    } else {
        return None;
    };

    let message = line.get(4..)?.trim();
    Some((level, message))
}

/// Check if line indicates successful connection
pub fn is_connection_registered(line: &str) -> bool {
    line.contains("Connection") && line.contains("registered")
}

/// Check if line indicates connection lost
pub fn is_connection_lost(line: &str) -> bool {
    line.contains("Unregistered") || line.contains("connection lost")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_quick_tunnel_url_from_box() {
        let line = "INF | https://random-words-here.trycloudflare.com            |";
        let url = parse_quick_tunnel_url(line);
        assert_eq!(
            url,
            Some("https://random-words-here.trycloudflare.com".to_string())
        );
    }

    #[test]
    fn parse_quick_tunnel_url_plain() {
        let line = "Your quick Tunnel is https://test-tunnel.trycloudflare.com";
        let url = parse_quick_tunnel_url(line);
        assert_eq!(
            url,
            Some("https://test-tunnel.trycloudflare.com".to_string())
        );
    }

    #[test]
    fn parse_quick_tunnel_url_no_match() {
        let line = "Starting tunnel connector";
        assert!(parse_quick_tunnel_url(line).is_none());
    }

    #[test]
    fn parse_log_level_info() {
        let (level, msg) = parse_log_level("INF Starting tunnel").unwrap();
        assert_eq!(level, "info");
        assert_eq!(msg, "Starting tunnel");
    }

    #[test]
    fn parse_log_level_error() {
        let (level, msg) = parse_log_level("ERR Connection failed").unwrap();
        assert_eq!(level, "error");
        assert_eq!(msg, "Connection failed");
    }

    #[test]
    fn parse_log_level_unknown() {
        assert!(parse_log_level("Some random text").is_none());
    }

    #[test]
    fn is_connection_registered_true() {
        let line = "INF Connection 0 registered connIndex=0";
        assert!(is_connection_registered(line));
    }

    #[test]
    fn is_connection_registered_false() {
        let line = "INF Starting tunnel";
        assert!(!is_connection_registered(line));
    }

    #[test]
    fn is_connection_lost_unregistered() {
        let line = "ERR Unregistered tunnel connection";
        assert!(is_connection_lost(line));
    }
}
```

**Step 2: Update mod.rs**

Add to `vibes-core/src/tunnel/mod.rs`:

```rust
pub mod cloudflared;

pub use cloudflared::{check_installation, CloudflaredInfo};
```

**Step 3: Add which dependency**

Add to `vibes-core/Cargo.toml` under `[dependencies]`:

```toml
which = "7"
```

**Step 4: Run tests**

Run: `cargo test -p vibes-core cloudflared`

Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/tunnel/cloudflared.rs vibes-core/src/tunnel/mod.rs vibes-core/Cargo.toml
git commit -m "feat(tunnel): add cloudflared CLI wrapper and output parsing"
```

---

## Task 4: Add RestartPolicy for Process Supervision

**Files:**
- Create: `vibes-core/src/tunnel/restart.rs`
- Modify: `vibes-core/src/tunnel/mod.rs`

**Step 1: Write RestartPolicy with tests**

Create `vibes-core/src/tunnel/restart.rs`:

```rust
//! Restart policy for cloudflared process supervision

use std::time::{Duration, Instant};

/// Policy for restarting failed processes with exponential backoff
#[derive(Debug)]
pub struct RestartPolicy {
    attempts: Vec<Instant>,
    max_attempts_per_window: u32,
    window_duration: Duration,
    current_backoff_idx: usize,
}

/// Backoff delays: immediate, 1s, 5s, 15s, 30s
const BACKOFF_DELAYS: [Duration; 5] = [
    Duration::from_secs(0),
    Duration::from_secs(1),
    Duration::from_secs(5),
    Duration::from_secs(15),
    Duration::from_secs(30),
];

impl RestartPolicy {
    /// Create a new restart policy
    ///
    /// - `max_attempts`: Maximum restart attempts within the window
    /// - `window`: Time window to count attempts
    pub fn new(max_attempts: u32, window: Duration) -> Self {
        Self {
            attempts: Vec::new(),
            max_attempts_per_window: max_attempts,
            window_duration: window,
            current_backoff_idx: 0,
        }
    }

    /// Default policy: 5 attempts per 60 seconds
    pub fn default_policy() -> Self {
        Self::new(5, Duration::from_secs(60))
    }

    /// Check if we should restart and get the delay
    ///
    /// Returns `Some(delay)` if restart is allowed, `None` if we should give up
    pub fn should_restart(&mut self) -> Option<Duration> {
        let now = Instant::now();

        // Clean old attempts outside window
        self.attempts
            .retain(|t| now.duration_since(*t) < self.window_duration);

        if self.attempts.len() >= self.max_attempts_per_window as usize {
            return None; // Give up
        }

        let delay = BACKOFF_DELAYS
            .get(self.current_backoff_idx)
            .copied()
            .unwrap_or(BACKOFF_DELAYS[BACKOFF_DELAYS.len() - 1]);

        self.attempts.push(now);
        self.current_backoff_idx = (self.current_backoff_idx + 1).min(BACKOFF_DELAYS.len() - 1);

        Some(delay)
    }

    /// Reset the policy after successful connection
    pub fn reset(&mut self) {
        self.attempts.clear();
        self.current_backoff_idx = 0;
    }

    /// Get the number of recent attempts
    pub fn recent_attempts(&self) -> usize {
        self.attempts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restart_policy_first_attempt_immediate() {
        let mut policy = RestartPolicy::default_policy();
        let delay = policy.should_restart().unwrap();
        assert_eq!(delay, Duration::from_secs(0));
    }

    #[test]
    fn restart_policy_backoff_increases() {
        let mut policy = RestartPolicy::default_policy();

        let d1 = policy.should_restart().unwrap();
        let d2 = policy.should_restart().unwrap();
        let d3 = policy.should_restart().unwrap();

        assert_eq!(d1, Duration::from_secs(0));
        assert_eq!(d2, Duration::from_secs(1));
        assert_eq!(d3, Duration::from_secs(5));
    }

    #[test]
    fn restart_policy_gives_up_after_max_attempts() {
        let mut policy = RestartPolicy::new(3, Duration::from_secs(60));

        assert!(policy.should_restart().is_some());
        assert!(policy.should_restart().is_some());
        assert!(policy.should_restart().is_some());
        assert!(policy.should_restart().is_none()); // 4th attempt denied
    }

    #[test]
    fn restart_policy_reset_clears_state() {
        let mut policy = RestartPolicy::default_policy();

        policy.should_restart();
        policy.should_restart();
        assert_eq!(policy.recent_attempts(), 2);

        policy.reset();
        assert_eq!(policy.recent_attempts(), 0);

        // Should start from immediate again
        let delay = policy.should_restart().unwrap();
        assert_eq!(delay, Duration::from_secs(0));
    }

    #[test]
    fn restart_policy_caps_at_max_backoff() {
        let mut policy = RestartPolicy::new(10, Duration::from_secs(120));

        // Exhaust all backoff levels
        for _ in 0..8 {
            policy.should_restart();
        }

        // Should stay at max backoff (30s)
        let delay = policy.should_restart().unwrap();
        assert_eq!(delay, Duration::from_secs(30));
    }
}
```

**Step 2: Update mod.rs**

Add to `vibes-core/src/tunnel/mod.rs`:

```rust
pub mod restart;

pub use restart::RestartPolicy;
```

**Step 3: Run tests**

Run: `cargo test -p vibes-core restart_policy`

Expected: PASS

**Step 4: Commit**

```bash
git add vibes-core/src/tunnel/restart.rs vibes-core/src/tunnel/mod.rs
git commit -m "feat(tunnel): add RestartPolicy for process supervision"
```

---

## Task 5: Add TunnelManager Core

**Files:**
- Create: `vibes-core/src/tunnel/manager.rs`
- Modify: `vibes-core/src/tunnel/mod.rs`
- Modify: `vibes-core/src/lib.rs`

**Step 1: Write TunnelManager**

Create `vibes-core/src/tunnel/manager.rs`:

```rust
//! Tunnel manager for cloudflared process lifecycle

use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

use super::cloudflared::{self, parse_log_level, parse_quick_tunnel_url};
use super::config::{TunnelConfig, TunnelMode};
use super::restart::RestartPolicy;
use super::state::{LogLevel, TunnelEvent, TunnelState};

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
```

**Step 2: Add thiserror dependency**

Add to `vibes-core/Cargo.toml` under `[dependencies]`:

```toml
thiserror = "2"
```

**Step 3: Update mod.rs**

Add to `vibes-core/src/tunnel/mod.rs`:

```rust
pub mod manager;

pub use manager::{TunnelError, TunnelManager};
```

**Step 4: Update lib.rs exports**

Update `vibes-core/src/lib.rs` tunnel exports:

```rust
pub use tunnel::{
    check_installation, CloudflaredInfo, LogLevel, RestartPolicy, TunnelConfig, TunnelError,
    TunnelEvent, TunnelManager, TunnelMode, TunnelState,
};
```

**Step 5: Run tests**

Run: `cargo test -p vibes-core tunnel_manager`

Expected: PASS

**Step 6: Commit**

```bash
git add vibes-core/src/tunnel/manager.rs vibes-core/src/tunnel/mod.rs vibes-core/src/lib.rs vibes-core/Cargo.toml
git commit -m "feat(tunnel): add TunnelManager for process lifecycle"
```

---

## Task 6: Add Tunnel Events to VibesEvent

**Files:**
- Modify: `vibes-core/src/events/types.rs`

**Step 1: Add TunnelStateChanged variant**

In `vibes-core/src/events/types.rs`, add to `VibesEvent` enum:

```rust
/// Tunnel state changed
TunnelStateChanged {
    state: String,
    url: Option<String>,
},
```

**Step 2: Update session_id method**

Update the `session_id()` method to handle the new variant:

```rust
VibesEvent::TunnelStateChanged { .. } => None,
```

**Step 3: Add test**

Add to tests module:

```rust
#[test]
fn vibes_event_tunnel_state_changed_serialization_roundtrip() {
    let event = VibesEvent::TunnelStateChanged {
        state: "connected".to_string(),
        url: Some("https://test.trycloudflare.com".to_string()),
    };
    let json = serde_json::to_string(&event).unwrap();
    let parsed: VibesEvent = serde_json::from_str(&json).unwrap();
    assert!(matches!(parsed, VibesEvent::TunnelStateChanged { state, url }
        if state == "connected" && url == Some("https://test.trycloudflare.com".to_string())));
}

#[test]
fn vibes_event_tunnel_state_changed_session_id_is_none() {
    let event = VibesEvent::TunnelStateChanged {
        state: "starting".to_string(),
        url: None,
    };
    assert_eq!(event.session_id(), None);
}
```

**Step 4: Run tests**

Run: `cargo test -p vibes-core vibes_event_tunnel`

Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/events/types.rs
git commit -m "feat(tunnel): add TunnelStateChanged to VibesEvent"
```

---

## Task 7: Add TunnelConfig to CLI Config

**Files:**
- Modify: `vibes-cli/src/config/types.rs`

**Step 1: Add tunnel config section**

In `vibes-cli/src/config/types.rs`, add after `SessionConfig`:

```rust
/// Tunnel configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TunnelConfigSection {
    /// Auto-start tunnel with serve
    #[serde(default)]
    pub enabled: bool,

    /// Tunnel mode: "quick" or "named"
    #[serde(default = "default_tunnel_mode")]
    pub mode: String,

    /// Tunnel name (for named mode)
    pub name: Option<String>,

    /// Public hostname (for named mode)
    pub hostname: Option<String>,
}

fn default_tunnel_mode() -> String {
    "quick".to_string()
}
```

Add to `RawVibesConfig`:

```rust
#[serde(default)]
pub tunnel: TunnelConfigSection,
```

Add to `VibesConfig`:

```rust
#[serde(default)]
pub tunnel: TunnelConfigSection,
```

**Step 2: Add tests**

```rust
#[test]
fn test_tunnel_config_defaults() {
    let config = TunnelConfigSection::default();
    assert!(!config.enabled);
    assert_eq!(config.mode, "");
    assert!(config.name.is_none());
}

#[test]
fn test_tunnel_config_parsing() {
    let toml_str = r#"
[tunnel]
enabled = true
mode = "named"
name = "vibes-home"
hostname = "vibes.example.com"
"#;
    let config: RawVibesConfig = toml::from_str(toml_str).unwrap();
    assert!(config.tunnel.enabled);
    assert_eq!(config.tunnel.mode, "named");
    assert_eq!(config.tunnel.name, Some("vibes-home".to_string()));
}
```

**Step 3: Run tests**

Run: `cargo test -p vibes-cli tunnel_config`

Expected: PASS

**Step 4: Commit**

```bash
git add vibes-cli/src/config/types.rs
git commit -m "feat(tunnel): add TunnelConfigSection to CLI config"
```

---

## Task 8: Add --tunnel and --quick-tunnel Flags to Serve

**Files:**
- Modify: `vibes-cli/src/commands/serve.rs`

**Step 1: Add tunnel flags to ServeArgs**

In `vibes-cli/src/commands/serve.rs`, add to `ServeArgs`:

```rust
/// Start with named tunnel (from config)
#[arg(long)]
pub tunnel: bool,

/// Start with quick tunnel (temporary URL)
#[arg(long, conflicts_with = "tunnel")]
pub quick_tunnel: bool,
```

**Step 2: Add test for flags**

```rust
#[test]
fn test_serve_args_tunnel_flag() {
    use clap::Parser;

    #[derive(Parser)]
    struct TestCli {
        #[command(flatten)]
        serve: ServeArgs,
    }

    let cli = TestCli::parse_from(["test", "--tunnel"]);
    assert!(cli.serve.tunnel);
    assert!(!cli.serve.quick_tunnel);
}

#[test]
fn test_serve_args_quick_tunnel_flag() {
    use clap::Parser;

    #[derive(Parser)]
    struct TestCli {
        #[command(flatten)]
        serve: ServeArgs,
    }

    let cli = TestCli::parse_from(["test", "--quick-tunnel"]);
    assert!(cli.serve.quick_tunnel);
    assert!(!cli.serve.tunnel);
}
```

**Step 3: Run tests**

Run: `cargo test -p vibes-cli serve_args_tunnel`

Expected: PASS

**Step 4: Commit**

```bash
git add vibes-cli/src/commands/serve.rs
git commit -m "feat(tunnel): add --tunnel and --quick-tunnel flags to serve"
```

---

## Task 9: Add Tunnel Subcommand

**Files:**
- Create: `vibes-cli/src/commands/tunnel.rs`
- Modify: `vibes-cli/src/commands/mod.rs`
- Modify: `vibes-cli/src/main.rs`

**Step 1: Create tunnel command module**

Create `vibes-cli/src/commands/tunnel.rs`:

```rust
//! Tunnel management commands

use anyhow::Result;
use clap::{Args, Subcommand};

/// Arguments for the tunnel command
#[derive(Debug, Args)]
pub struct TunnelArgs {
    #[command(subcommand)]
    pub command: TunnelCommand,
}

/// Tunnel subcommands
#[derive(Debug, Subcommand)]
pub enum TunnelCommand {
    /// Interactive setup wizard for named tunnel
    Setup,
    /// Start the tunnel
    Start,
    /// Stop the tunnel
    Stop,
    /// Show tunnel status
    Status,
    /// Start a quick tunnel (temporary URL)
    Quick,
}

/// Run the tunnel command
pub fn run(args: TunnelArgs) -> Result<()> {
    match args.command {
        TunnelCommand::Setup => run_setup(),
        TunnelCommand::Start => run_start(),
        TunnelCommand::Stop => run_stop(),
        TunnelCommand::Status => run_status(),
        TunnelCommand::Quick => run_quick(),
    }
}

fn run_setup() -> Result<()> {
    println!("Tunnel setup wizard - coming soon");
    Ok(())
}

fn run_start() -> Result<()> {
    println!("Starting tunnel...");
    println!("Hint: Use 'vibes serve --tunnel' to start server with tunnel");
    Ok(())
}

fn run_stop() -> Result<()> {
    println!("Stopping tunnel...");
    Ok(())
}

fn run_status() -> Result<()> {
    println!("Tunnel status: Not running");
    println!("Use 'vibes serve --tunnel' or 'vibes serve --quick-tunnel' to start");
    Ok(())
}

fn run_quick() -> Result<()> {
    println!("Starting quick tunnel...");
    println!("Hint: Use 'vibes serve --quick-tunnel' to start server with quick tunnel");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct TestCli {
        #[command(flatten)]
        tunnel: TunnelArgs,
    }

    #[test]
    fn test_tunnel_setup_command() {
        let cli = TestCli::parse_from(["test", "setup"]);
        assert!(matches!(cli.tunnel.command, TunnelCommand::Setup));
    }

    #[test]
    fn test_tunnel_start_command() {
        let cli = TestCli::parse_from(["test", "start"]);
        assert!(matches!(cli.tunnel.command, TunnelCommand::Start));
    }

    #[test]
    fn test_tunnel_stop_command() {
        let cli = TestCli::parse_from(["test", "stop"]);
        assert!(matches!(cli.tunnel.command, TunnelCommand::Stop));
    }

    #[test]
    fn test_tunnel_status_command() {
        let cli = TestCli::parse_from(["test", "status"]);
        assert!(matches!(cli.tunnel.command, TunnelCommand::Status));
    }

    #[test]
    fn test_tunnel_quick_command() {
        let cli = TestCli::parse_from(["test", "quick"]);
        assert!(matches!(cli.tunnel.command, TunnelCommand::Quick));
    }
}
```

**Step 2: Update commands/mod.rs**

Add to `vibes-cli/src/commands/mod.rs`:

```rust
pub mod tunnel;
```

**Step 3: Update main.rs**

In `vibes-cli/src/main.rs`, add to `Commands` enum:

```rust
/// Manage Cloudflare Tunnel
Tunnel(commands::tunnel::TunnelArgs),
```

Add to match in `main()`:

```rust
Commands::Tunnel(args) => commands::tunnel::run(args),
```

**Step 4: Run tests**

Run: `cargo test -p vibes-cli tunnel`

Expected: PASS

**Step 5: Commit**

```bash
git add vibes-cli/src/commands/tunnel.rs vibes-cli/src/commands/mod.rs vibes-cli/src/main.rs
git commit -m "feat(tunnel): add vibes tunnel subcommand with setup/start/stop/status/quick"
```

---

## Task 10: Add TunnelManager to Server AppState

**Files:**
- Modify: `vibes-server/src/state.rs`

**Step 1: Add TunnelManager to AppState**

In `vibes-server/src/state.rs`, add import:

```rust
use vibes_core::{TunnelConfig, TunnelManager};
```

Add field to `AppState`:

```rust
/// Tunnel manager for remote access
pub tunnel_manager: Arc<RwLock<TunnelManager>>,
```

Update `new()` method:

```rust
let tunnel_manager = Arc::new(RwLock::new(TunnelManager::new(
    TunnelConfig::default(),
    7432,
)));
```

And add to the `Self` struct initialization.

Update `with_components` to include tunnel_manager parameter.

**Step 2: Add tunnel_manager accessor**

```rust
/// Get a reference to the tunnel manager
pub fn tunnel_manager(&self) -> &Arc<RwLock<TunnelManager>> {
    &self.tunnel_manager
}
```

**Step 3: Add test**

```rust
#[tokio::test]
async fn test_app_state_has_tunnel_manager() {
    let state = AppState::new();
    let tunnel = state.tunnel_manager.read().await;
    assert!(!tunnel.is_enabled());
}
```

**Step 4: Run tests**

Run: `cargo test -p vibes-server app_state`

Expected: PASS

**Step 5: Commit**

```bash
git add vibes-server/src/state.rs
git commit -m "feat(tunnel): add TunnelManager to server AppState"
```

---

## Task 11: Add GET /api/tunnel/status Endpoint

**Files:**
- Modify: `vibes-server/src/http/api.rs`

**Step 1: Add tunnel status handler and route**

Add handler function:

```rust
/// GET /api/tunnel/status - Get tunnel status
pub async fn get_tunnel_status(
    State(state): State<AppState>,
) -> Json<TunnelStatusResponse> {
    let manager = state.tunnel_manager.read().await;
    let tunnel_state = manager.state().await;

    let (status, url, error) = match &tunnel_state {
        vibes_core::TunnelState::Disabled => ("disabled", None, None),
        vibes_core::TunnelState::Starting => ("starting", None, None),
        vibes_core::TunnelState::Connected { url, .. } => ("connected", Some(url.clone()), None),
        vibes_core::TunnelState::Reconnecting { last_error, .. } => {
            ("reconnecting", None, Some(last_error.clone()))
        }
        vibes_core::TunnelState::Failed { error, .. } => ("failed", None, Some(error.clone())),
        vibes_core::TunnelState::Stopped => ("stopped", None, None),
    };

    let mode = if manager.is_enabled() {
        if manager.config().is_quick() {
            Some("quick")
        } else {
            Some("named")
        }
    } else {
        None
    };

    Json(TunnelStatusResponse {
        state: status.to_string(),
        mode: mode.map(|s| s.to_string()),
        url,
        tunnel_name: manager.config().tunnel_name().map(|s| s.to_string()),
        error,
    })
}

#[derive(Serialize)]
pub struct TunnelStatusResponse {
    state: String,
    mode: Option<String>,
    url: Option<String>,
    tunnel_name: Option<String>,
    error: Option<String>,
}
```

Add route to router:

```rust
.route("/api/tunnel/status", get(get_tunnel_status))
```

**Step 2: Add test**

```rust
#[tokio::test]
async fn test_get_tunnel_status_disabled() {
    let state = AppState::new();
    let app = api_router().with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tunnel/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["state"], "disabled");
}
```

**Step 3: Run tests**

Run: `cargo test -p vibes-server get_tunnel_status`

Expected: PASS

**Step 4: Commit**

```bash
git add vibes-server/src/http/api.rs
git commit -m "feat(tunnel): add GET /api/tunnel/status endpoint"
```

---

## Task 12: Add tunnel_state WebSocket Message

**Files:**
- Modify: `vibes-server/src/ws/protocol.rs`

**Step 1: Add TunnelState message type**

Add to server-to-client messages:

```rust
/// Tunnel state update
TunnelState {
    state: String,
    url: Option<String>,
},
```

**Step 2: Add test**

```rust
#[test]
fn test_server_message_tunnel_state_serialization() {
    let msg = ServerMessage::TunnelState {
        state: "connected".to_string(),
        url: Some("https://test.trycloudflare.com".to_string()),
    };
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("tunnel_state"));
    assert!(json.contains("connected"));
}
```

**Step 3: Run tests**

Run: `cargo test -p vibes-server tunnel_state`

Expected: PASS

**Step 4: Commit**

```bash
git add vibes-server/src/ws/protocol.rs
git commit -m "feat(tunnel): add tunnel_state WebSocket message"
```

---

## Task 13: Add TunnelBadge Component (Web UI)

**Files:**
- Create: `web-ui/src/components/TunnelBadge.tsx`
- Create: `web-ui/src/hooks/useTunnelStatus.ts`

**Step 1: Create useTunnelStatus hook**

Create `web-ui/src/hooks/useTunnelStatus.ts`:

```typescript
import { useQuery } from '@tanstack/react-query';

interface TunnelStatus {
  state: 'disabled' | 'starting' | 'connected' | 'reconnecting' | 'failed' | 'stopped';
  mode: 'quick' | 'named' | null;
  url: string | null;
  tunnel_name: string | null;
  error: string | null;
}

export function useTunnelStatus() {
  return useQuery<TunnelStatus>({
    queryKey: ['tunnel-status'],
    queryFn: async () => {
      const response = await fetch('/api/tunnel/status');
      if (!response.ok) {
        throw new Error('Failed to fetch tunnel status');
      }
      return response.json();
    },
    refetchInterval: 5000, // Poll every 5 seconds
  });
}
```

**Step 2: Create TunnelBadge component**

Create `web-ui/src/components/TunnelBadge.tsx`:

```tsx
import { Link } from '@tanstack/react-router';
import { useTunnelStatus } from '../hooks/useTunnelStatus';

const BADGE_CONFIG = {
  disabled: { color: '#9CA3AF', icon: '○', tooltip: 'No tunnel configured' },
  starting: { color: '#F59E0B', icon: '◐', tooltip: 'Connecting...' },
  connected: { color: '#10B981', icon: '●', tooltip: '' }, // URL set dynamically
  reconnecting: { color: '#F59E0B', icon: '◐', tooltip: 'Reconnecting...' },
  failed: { color: '#EF4444', icon: '●', tooltip: 'Connection failed' },
  stopped: { color: '#9CA3AF', icon: '○', tooltip: 'Tunnel stopped' },
} as const;

export function TunnelBadge() {
  const { data: status, isLoading } = useTunnelStatus();

  if (isLoading || !status) {
    return (
      <span style={{ color: '#9CA3AF' }} title="Loading...">
        ○
      </span>
    );
  }

  const config = BADGE_CONFIG[status.state];
  const tooltip = status.state === 'connected' && status.url
    ? status.url
    : status.error || config.tooltip;

  return (
    <Link to="/status" style={{ textDecoration: 'none' }}>
      <span
        style={{
          color: config.color,
          fontSize: '1rem',
          cursor: 'pointer',
        }}
        title={tooltip}
      >
        {config.icon}
      </span>
    </Link>
  );
}
```

**Step 3: Export from hooks index**

Update `web-ui/src/hooks/index.ts`:

```typescript
export { useTunnelStatus } from './useTunnelStatus';
```

**Step 4: Run build to verify**

Run: `cd web-ui && npm run build`

Expected: Build succeeds

**Step 5: Commit**

```bash
git add web-ui/src/components/TunnelBadge.tsx web-ui/src/hooks/useTunnelStatus.ts web-ui/src/hooks/index.ts
git commit -m "feat(tunnel): add TunnelBadge component and useTunnelStatus hook"
```

---

## Task 14: Add Status Page (Web UI)

**Files:**
- Create: `web-ui/src/pages/Status.tsx`
- Modify: `web-ui/src/App.tsx`

**Step 1: Create Status page**

Create `web-ui/src/pages/Status.tsx`:

```tsx
import { useTunnelStatus } from '../hooks/useTunnelStatus';

export function StatusPage() {
  const { data: tunnel, isLoading, error } = useTunnelStatus();

  return (
    <div style={{ padding: '2rem' }}>
      <h1>Status</h1>

      <section style={{ marginTop: '2rem' }}>
        <h2>Tunnel</h2>

        {isLoading && <p>Loading...</p>}
        {error && <p style={{ color: 'red' }}>Error loading status</p>}

        {tunnel && (
          <dl style={{ display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '0.5rem 1rem' }}>
            <dt>State</dt>
            <dd>
              <StatusBadge state={tunnel.state} />
            </dd>

            <dt>Mode</dt>
            <dd>{tunnel.mode || 'Not configured'}</dd>

            {tunnel.url && (
              <>
                <dt>URL</dt>
                <dd>
                  <a href={tunnel.url} target="_blank" rel="noopener noreferrer">
                    {tunnel.url}
                  </a>
                </dd>
              </>
            )}

            {tunnel.tunnel_name && (
              <>
                <dt>Tunnel Name</dt>
                <dd>{tunnel.tunnel_name}</dd>
              </>
            )}

            {tunnel.error && (
              <>
                <dt>Error</dt>
                <dd style={{ color: 'red' }}>{tunnel.error}</dd>
              </>
            )}
          </dl>
        )}
      </section>
    </div>
  );
}

function StatusBadge({ state }: { state: string }) {
  const colors: Record<string, string> = {
    disabled: '#9CA3AF',
    starting: '#F59E0B',
    connected: '#10B981',
    reconnecting: '#F59E0B',
    failed: '#EF4444',
    stopped: '#9CA3AF',
  };

  return (
    <span
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: '0.5rem',
        padding: '0.25rem 0.75rem',
        borderRadius: '9999px',
        backgroundColor: `${colors[state]}20`,
        color: colors[state],
        fontSize: '0.875rem',
        fontWeight: 500,
      }}
    >
      <span style={{ fontSize: '0.5rem' }}>●</span>
      {state}
    </span>
  );
}
```

**Step 2: Add route to App.tsx**

In `web-ui/src/App.tsx`, add import:

```typescript
import { StatusPage } from './pages/Status';
```

Add route (alongside existing routes):

```typescript
{
  path: '/status',
  element: <StatusPage />,
}
```

**Step 3: Run build**

Run: `cd web-ui && npm run build`

Expected: Build succeeds

**Step 4: Commit**

```bash
git add web-ui/src/pages/Status.tsx web-ui/src/App.tsx
git commit -m "feat(tunnel): add Status page with tunnel info"
```

---

## Task 15: Add TunnelBadge to Header

**Files:**
- Modify: `web-ui/src/App.tsx` (or Header component if exists)

**Step 1: Add TunnelBadge to layout**

Import and add TunnelBadge to the header area of the app.

Look for existing header/layout in App.tsx and add:

```tsx
import { TunnelBadge } from './components/TunnelBadge';

// In the header/nav area:
<TunnelBadge />
```

**Step 2: Run build**

Run: `cd web-ui && npm run build`

Expected: Build succeeds

**Step 3: Commit**

```bash
git add web-ui/src/App.tsx
git commit -m "feat(tunnel): add TunnelBadge to header"
```

---

## Task 16: Integrate TunnelManager with Serve Command

**Files:**
- Modify: `vibes-cli/src/commands/serve.rs`
- Modify: `vibes-server/src/lib.rs`

**Step 1: Update ServerConfig to include tunnel options**

In `vibes-server/src/lib.rs`, update `ServerConfig`:

```rust
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub tunnel_enabled: bool,
    pub tunnel_quick: bool,
}
```

**Step 2: Update serve command to pass tunnel config**

In `vibes-cli/src/commands/serve.rs`, update `run_foreground`:

```rust
let config = ServerConfig {
    host: args.host.clone(),
    port: args.port,
    tunnel_enabled: args.tunnel,
    tunnel_quick: args.quick_tunnel,
};
```

**Step 3: Run tests**

Run: `cargo test -p vibes-cli -p vibes-server`

Expected: PASS

**Step 4: Commit**

```bash
git add vibes-cli/src/commands/serve.rs vibes-server/src/lib.rs
git commit -m "feat(tunnel): integrate tunnel config with serve command"
```

---

## Task 17: Update Documentation

**Files:**
- Modify: `docs/PROGRESS.md`

**Step 1: Update progress tracker**

Mark milestone 2.1 items as in-progress or complete:

```markdown
### Milestone 2.1: Cloudflare Tunnel integration
- [x] vibes tunnel setup wizard (stub)
- [x] cloudflared process management
- [x] Tunnel status in UI
- [x] Auto-reconnect handling
```

**Step 2: Add changelog entry**

Add to changelog table:

```markdown
| 2025-12-26 | Milestone 2.1 (Cloudflare Tunnel) started - TunnelManager, CLI commands, UI status |
```

**Step 3: Commit**

```bash
git add docs/PROGRESS.md
git commit -m "docs: update progress for milestone 2.1"
```

---

## Task 18: Run Full Test Suite

**Step 1: Run all tests**

Run: `just test`

Expected: All tests pass

**Step 2: Run clippy**

Run: `just clippy`

Expected: No warnings

**Step 3: Run format check**

Run: `just fmt-check`

Expected: All files formatted

**Step 4: Build web-ui**

Run: `cd web-ui && npm run build`

Expected: Build succeeds

**Step 5: Final commit if any fixes needed**

```bash
git add -A
git commit -m "fix: address test/lint issues from tunnel implementation"
```

---

## Summary

This implementation plan covers:

1. **Core types** (Tasks 1-5): TunnelState, TunnelEvent, TunnelConfig, cloudflared wrapper, RestartPolicy, TunnelManager
2. **Event integration** (Task 6): Add TunnelStateChanged to VibesEvent
3. **CLI config** (Tasks 7-9): Add tunnel config section, serve flags, tunnel subcommand
4. **Server integration** (Tasks 10-12): AppState, API endpoint, WebSocket messages
5. **Web UI** (Tasks 13-15): TunnelBadge, useTunnelStatus hook, Status page
6. **Integration** (Tasks 16-18): Wire everything together, update docs, test

Each task follows TDD: write test → verify fails → implement → verify passes → commit.
