# Milestone 4.4.2a: EventLog Migration - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace pub/sub `MemoryEventBus` with producer/consumer `EventLog` backed by Iggy for persistent, crash-recoverable event storage.

**Architecture:** Create new `vibes-iggy` crate with `EventLog`/`EventConsumer` traits. Move `IggyManager` from vibes-groove. Implement `IggyEventLog` that writes to Iggy and `IggyEventConsumer` for offset-tracked consumption. Migrate vibes-server from broadcast subscription to consumer pattern. Move vibes-groove to `plugins/` directory.

**Tech Stack:** Iggy SDK (`iggy` crate), tokio async runtime, serde for serialization, existing vibes-core event types.

**Reference:** See design at `docs/plans/14-continual-learning/milestone-4.4.2a-design.md`

---

## Task 1: Create vibes-iggy Crate Skeleton

**Files:**
- Create: `vibes-iggy/Cargo.toml`
- Create: `vibes-iggy/src/lib.rs`
- Modify: `Cargo.toml` (workspace)

**Step 1: Create crate directory**

```bash
mkdir -p vibes-iggy/src
```

**Step 2: Create Cargo.toml**

Create `vibes-iggy/Cargo.toml`:

```toml
[package]
name = "vibes-iggy"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
description = "Iggy-backed event log for vibes"

[dependencies]
# Async runtime
tokio = { workspace = true }
async-trait = "0.1"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Iggy SDK
iggy = "0.6"

# Error handling
thiserror = "1"

# Logging
tracing = "0.1"

# Time
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
tokio = { workspace = true, features = ["test-util", "macros"] }
tempfile = "3"
```

**Step 3: Create lib.rs skeleton**

Create `vibes-iggy/src/lib.rs`:

```rust
//! Iggy-backed event log for vibes.
//!
//! This crate provides persistent event storage using Iggy as the backing store.
//! It implements a producer/consumer model with independent offset tracking
//! per consumer group.
//!
//! # Key Types
//!
//! - [`EventLog`] - Trait for appending events and creating consumers
//! - [`EventConsumer`] - Trait for polling events with offset tracking
//! - [`IggyEventLog`] - Iggy-backed implementation of EventLog
//! - [`IggyManager`] - Manages Iggy server subprocess lifecycle

pub mod config;
pub mod error;
pub mod manager;
pub mod traits;

// Re-exports
pub use config::IggyConfig;
pub use error::{Error, Result};
pub use manager::{IggyManager, IggyState};
pub use traits::{EventBatch, EventConsumer, EventLog, Offset, SeekPosition};
```

**Step 4: Add to workspace**

Modify `Cargo.toml` (root) - add to members list:

```toml
[workspace]
resolver = "2"
members = [
    "vibes-cli",
    "vibes-core",
    "vibes-iggy",
    "vibes-plugin-api",
    "vibes-server",
    "vibes-introspection",
    "vibes-groove",
]
```

**Step 5: Verify crate is recognized**

```bash
cargo check -p vibes-iggy
```

Expected: Compilation errors about missing modules (we'll add them next)

**Step 6: Commit skeleton**

```bash
git add vibes-iggy/ Cargo.toml
git commit -m "feat(iggy): create vibes-iggy crate skeleton"
```

---

## Task 2: Define Error Types

**Files:**
- Create: `vibes-iggy/src/error.rs`

**Step 1: Write error types**

Create `vibes-iggy/src/error.rs`:

```rust
//! Error types for vibes-iggy.

use std::io;
use thiserror::Error;

/// Result type for vibes-iggy operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in vibes-iggy.
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error (file system, process, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Iggy client error
    #[error("Iggy error: {0}")]
    Iggy(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Consumer group not found
    #[error("Consumer group not found: {0}")]
    ConsumerNotFound(String),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Server not running
    #[error("Iggy server not running")]
    ServerNotRunning,

    /// Invalid offset
    #[error("Invalid offset: {0}")]
    InvalidOffset(u64),
}

impl From<iggy::error::IggyError> for Error {
    fn from(err: iggy::error::IggyError) -> Self {
        Error::Iggy(err.to_string())
    }
}
```

**Step 2: Verify compilation**

```bash
cargo check -p vibes-iggy
```

**Step 3: Commit**

```bash
git add vibes-iggy/src/error.rs
git commit -m "feat(iggy): add error types"
```

---

## Task 3: Define EventLog and EventConsumer Traits

**Files:**
- Create: `vibes-iggy/src/traits.rs`

**Step 1: Write trait definitions with tests**

Create `vibes-iggy/src/traits.rs`:

```rust
//! Core traits for event log and consumer abstractions.
//!
//! These traits define the producer/consumer model for event storage.
//! Unlike pub/sub, each consumer group tracks its own offset independently.

use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Monotonically increasing position in the event log.
///
/// Offsets are assigned sequentially as events are appended.
/// Consumers use offsets to track their position in the log.
pub type Offset = u64;

/// Represents a batch of events returned from polling.
#[derive(Debug, Clone)]
pub struct EventBatch<E> {
    /// The events in this batch with their offsets.
    pub events: Vec<(Offset, E)>,
}

impl<E> EventBatch<E> {
    /// Create a new empty batch.
    #[must_use]
    pub fn empty() -> Self {
        Self { events: Vec::new() }
    }

    /// Create a new batch from events.
    #[must_use]
    pub fn new(events: Vec<(Offset, E)>) -> Self {
        Self { events }
    }

    /// Check if the batch is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get the number of events in the batch.
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Get the offset of the last event, if any.
    #[must_use]
    pub fn last_offset(&self) -> Option<Offset> {
        self.events.last().map(|(o, _)| *o)
    }

    /// Get the offset of the first event, if any.
    #[must_use]
    pub fn first_offset(&self) -> Option<Offset> {
        self.events.first().map(|(o, _)| *o)
    }
}

impl<E> Default for EventBatch<E> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<E> IntoIterator for EventBatch<E> {
    type Item = (Offset, E);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.events.into_iter()
    }
}

/// Where to seek in the event log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekPosition {
    /// Start of the log (offset 0).
    Beginning,
    /// End of the log (only receive new events).
    End,
    /// Specific offset in the log.
    Offset(Offset),
}

/// The event log - append-only, durable storage for events.
///
/// Events are serialized and stored in order. Each event is assigned
/// a monotonically increasing offset that consumers use to track position.
///
/// # Type Parameters
///
/// - `E`: The event type, must be serializable.
#[async_trait]
pub trait EventLog<E>: Send + Sync
where
    E: Serialize + for<'de> Deserialize<'de> + Send + Clone + 'static,
{
    /// Append an event to the log.
    ///
    /// Returns the offset assigned to this event.
    async fn append(&self, event: E) -> Result<Offset>;

    /// Append multiple events atomically.
    ///
    /// Returns the offset of the last event appended.
    async fn append_batch(&self, events: Vec<E>) -> Result<Offset>;

    /// Create a consumer for a specific consumer group.
    ///
    /// Each consumer group tracks its own offset independently.
    /// If the group doesn't exist, it starts from the beginning.
    async fn consumer(&self, group: &str) -> Result<Box<dyn EventConsumer<E>>>;

    /// Get the current high-water mark (latest offset + 1).
    ///
    /// This is the offset that will be assigned to the next appended event.
    fn high_water_mark(&self) -> Offset;
}

/// A consumer that reads events and tracks its position.
///
/// Each consumer belongs to a consumer group. Within a group, all consumers
/// share the same offset (for load balancing). Different groups have
/// independent offsets.
#[async_trait]
pub trait EventConsumer<E>: Send
where
    E: Send + Clone + 'static,
{
    /// Poll for the next batch of events.
    ///
    /// Blocks until events are available or timeout expires.
    /// Returns an empty batch on timeout.
    async fn poll(&mut self, max_count: usize, timeout: Duration) -> Result<EventBatch<E>>;

    /// Commit the consumer's offset.
    ///
    /// This acknowledges that all events up to and including this offset
    /// have been processed. On restart, the consumer will resume from
    /// the committed offset.
    async fn commit(&mut self, offset: Offset) -> Result<()>;

    /// Seek to a specific position in the log.
    ///
    /// Changes where the next poll will read from.
    async fn seek(&mut self, position: SeekPosition) -> Result<()>;

    /// Get the last committed offset for this consumer.
    fn committed_offset(&self) -> Offset;

    /// Get the consumer group name.
    fn group(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_batch_empty() {
        let batch: EventBatch<String> = EventBatch::empty();
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
        assert_eq!(batch.last_offset(), None);
        assert_eq!(batch.first_offset(), None);
    }

    #[test]
    fn event_batch_with_events() {
        let batch = EventBatch::new(vec![
            (0, "first".to_string()),
            (1, "second".to_string()),
            (2, "third".to_string()),
        ]);

        assert!(!batch.is_empty());
        assert_eq!(batch.len(), 3);
        assert_eq!(batch.first_offset(), Some(0));
        assert_eq!(batch.last_offset(), Some(2));
    }

    #[test]
    fn event_batch_into_iter() {
        let batch = EventBatch::new(vec![(0, "a".to_string()), (1, "b".to_string())]);

        let collected: Vec<_> = batch.into_iter().collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0], (0, "a".to_string()));
        assert_eq!(collected[1], (1, "b".to_string()));
    }

    #[test]
    fn seek_position_equality() {
        assert_eq!(SeekPosition::Beginning, SeekPosition::Beginning);
        assert_eq!(SeekPosition::End, SeekPosition::End);
        assert_eq!(SeekPosition::Offset(42), SeekPosition::Offset(42));
        assert_ne!(SeekPosition::Offset(1), SeekPosition::Offset(2));
        assert_ne!(SeekPosition::Beginning, SeekPosition::End);
    }
}
```

**Step 2: Run tests**

```bash
cargo test -p vibes-iggy traits
```

Expected: All tests pass

**Step 3: Commit**

```bash
git add vibes-iggy/src/traits.rs
git commit -m "feat(iggy): add EventLog and EventConsumer traits"
```

---

## Task 4: Move IggyConfig to vibes-iggy

**Files:**
- Create: `vibes-iggy/src/config.rs`

**Step 1: Write config module**

Create `vibes-iggy/src/config.rs`:

```rust
//! Configuration for Iggy server and client.

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Configuration for the Iggy server subprocess.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IggyConfig {
    /// Path to the iggy-server binary.
    #[serde(default = "default_binary_path")]
    pub binary_path: PathBuf,

    /// Directory where Iggy stores its data.
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,

    /// TCP port for Iggy server.
    #[serde(default = "default_port")]
    pub port: u16,

    /// Interval between health checks.
    #[serde(default = "default_health_check_interval", with = "humantime_serde")]
    pub health_check_interval: Duration,

    /// Maximum number of restart attempts before giving up.
    #[serde(default = "default_max_restart_attempts")]
    pub max_restart_attempts: u32,
}

fn default_binary_path() -> PathBuf {
    PathBuf::from("iggy-server")
}

fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .map(|d| d.join("vibes").join("iggy"))
        .unwrap_or_else(|| PathBuf::from("/tmp/vibes/iggy"))
}

fn default_port() -> u16 {
    8090
}

fn default_health_check_interval() -> Duration {
    Duration::from_secs(5)
}

fn default_max_restart_attempts() -> u32 {
    3
}

impl Default for IggyConfig {
    fn default() -> Self {
        Self {
            binary_path: default_binary_path(),
            data_dir: default_data_dir(),
            port: default_port(),
            health_check_interval: default_health_check_interval(),
            max_restart_attempts: default_max_restart_attempts(),
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

    /// Get the TCP connection address for clients.
    #[must_use]
    pub fn connection_address(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_values() {
        let config = IggyConfig::default();

        assert_eq!(config.binary_path, PathBuf::from("iggy-server"));
        assert_eq!(config.port, 8090);
        assert_eq!(config.max_restart_attempts, 3);
        assert_eq!(config.health_check_interval, Duration::from_secs(5));
    }

    #[test]
    fn config_builder_pattern() {
        let config = IggyConfig::default()
            .with_binary_path("/usr/bin/iggy")
            .with_data_dir("/var/lib/iggy")
            .with_port(9000);

        assert_eq!(config.binary_path, PathBuf::from("/usr/bin/iggy"));
        assert_eq!(config.data_dir, PathBuf::from("/var/lib/iggy"));
        assert_eq!(config.port, 9000);
    }

    #[test]
    fn config_connection_address() {
        let config = IggyConfig::default().with_port(8091);
        assert_eq!(config.connection_address(), "127.0.0.1:8091");
    }
}
```

**Step 2: Add humantime-serde dependency**

Update `vibes-iggy/Cargo.toml` dependencies:

```toml
[dependencies]
# ... existing deps ...
humantime-serde = "1"
dirs = "5"
```

**Step 3: Run tests**

```bash
cargo test -p vibes-iggy config
```

**Step 4: Commit**

```bash
git add vibes-iggy/src/config.rs vibes-iggy/Cargo.toml
git commit -m "feat(iggy): add IggyConfig"
```

---

## Task 5: Move IggyManager to vibes-iggy

**Files:**
- Create: `vibes-iggy/src/manager.rs`

**Step 1: Write IggyManager (adapted from vibes-groove)**

Create `vibes-iggy/src/manager.rs`:

```rust
//! Iggy server subprocess lifecycle management.
//!
//! Manages starting, stopping, and supervising the Iggy server process
//! with automatic health checks and restart capabilities.

use std::process::Child;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::config::IggyConfig;
use crate::error::{Error, Result};

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
            .map_err(|e| Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to spawn iggy-server: {}", e),
            )))?;

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
```

**Step 2: Run tests**

```bash
cargo test -p vibes-iggy manager
```

**Step 3: Verify full crate compiles**

```bash
cargo check -p vibes-iggy
```

**Step 4: Commit**

```bash
git add vibes-iggy/src/manager.rs
git commit -m "feat(iggy): add IggyManager for subprocess lifecycle"
```

---

## Task 6: Create InMemoryEventLog for Testing

**Files:**
- Create: `vibes-iggy/src/memory.rs`
- Modify: `vibes-iggy/src/lib.rs`

**Step 1: Write tests first**

Create `vibes-iggy/src/memory.rs`:

```rust
//! In-memory EventLog implementation for testing.
//!
//! This implementation stores events in memory without persistence.
//! Useful for testing and development without running Iggy server.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};

use crate::error::Result;
use crate::traits::{EventBatch, EventConsumer, EventLog, Offset, SeekPosition};

/// In-memory implementation of EventLog for testing.
pub struct InMemoryEventLog<E> {
    /// Stored events
    events: RwLock<Vec<E>>,
    /// Next offset to assign
    next_offset: AtomicU64,
    /// Consumer group offsets
    consumer_offsets: RwLock<HashMap<String, Offset>>,
}

impl<E> InMemoryEventLog<E>
where
    E: Clone + Send + Sync + 'static,
{
    /// Create a new in-memory event log.
    #[must_use]
    pub fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            next_offset: AtomicU64::new(0),
            consumer_offsets: RwLock::new(HashMap::new()),
        }
    }

    /// Get the number of events in the log.
    pub async fn len(&self) -> usize {
        self.events.read().await.len()
    }

    /// Check if the log is empty.
    pub async fn is_empty(&self) -> bool {
        self.events.read().await.is_empty()
    }
}

impl<E> Default for InMemoryEventLog<E>
where
    E: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<E> EventLog<E> for InMemoryEventLog<E>
where
    E: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone + 'static,
{
    async fn append(&self, event: E) -> Result<Offset> {
        let offset = self.next_offset.fetch_add(1, Ordering::SeqCst);
        self.events.write().await.push(event);
        Ok(offset)
    }

    async fn append_batch(&self, events: Vec<E>) -> Result<Offset> {
        let count = events.len() as u64;
        if count == 0 {
            return Ok(self.high_water_mark().saturating_sub(1));
        }

        let first_offset = self.next_offset.fetch_add(count, Ordering::SeqCst);
        self.events.write().await.extend(events);
        Ok(first_offset + count - 1)
    }

    async fn consumer(&self, group: &str) -> Result<Box<dyn EventConsumer<E>>> {
        // Get or create consumer offset
        let offset = {
            let offsets = self.consumer_offsets.read().await;
            offsets.get(group).copied().unwrap_or(0)
        };

        Ok(Box::new(InMemoryConsumer {
            group: group.to_string(),
            events: Arc::new(self.events.read().await.clone()),
            current_offset: offset,
            committed_offset: offset,
            log_offsets: Arc::new(Mutex::new(self.consumer_offsets.write().await.clone())),
        }))
    }

    fn high_water_mark(&self) -> Offset {
        self.next_offset.load(Ordering::SeqCst)
    }
}

/// In-memory consumer implementation.
struct InMemoryConsumer<E> {
    group: String,
    events: Arc<Vec<E>>,
    current_offset: Offset,
    committed_offset: Offset,
    log_offsets: Arc<Mutex<HashMap<String, Offset>>>,
}

#[async_trait]
impl<E> EventConsumer<E> for InMemoryConsumer<E>
where
    E: Send + Sync + Clone + 'static,
{
    async fn poll(&mut self, max_count: usize, _timeout: Duration) -> Result<EventBatch<E>> {
        let start = self.current_offset as usize;
        let end = std::cmp::min(start + max_count, self.events.len());

        if start >= self.events.len() {
            return Ok(EventBatch::empty());
        }

        let events: Vec<(Offset, E)> = self.events[start..end]
            .iter()
            .enumerate()
            .map(|(i, e)| ((start + i) as Offset, e.clone()))
            .collect();

        if let Some((last_offset, _)) = events.last() {
            self.current_offset = last_offset + 1;
        }

        Ok(EventBatch::new(events))
    }

    async fn commit(&mut self, offset: Offset) -> Result<()> {
        self.committed_offset = offset;
        let mut offsets = self.log_offsets.lock().await;
        offsets.insert(self.group.clone(), offset);
        Ok(())
    }

    async fn seek(&mut self, position: SeekPosition) -> Result<()> {
        self.current_offset = match position {
            SeekPosition::Beginning => 0,
            SeekPosition::End => self.events.len() as Offset,
            SeekPosition::Offset(o) => o,
        };
        Ok(())
    }

    fn committed_offset(&self) -> Offset {
        self.committed_offset
    }

    fn group(&self) -> &str {
        &self.group
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn append_returns_incrementing_offsets() {
        let log: InMemoryEventLog<String> = InMemoryEventLog::new();

        let o1 = log.append("first".to_string()).await.unwrap();
        let o2 = log.append("second".to_string()).await.unwrap();
        let o3 = log.append("third".to_string()).await.unwrap();

        assert_eq!(o1, 0);
        assert_eq!(o2, 1);
        assert_eq!(o3, 2);
        assert_eq!(log.high_water_mark(), 3);
    }

    #[tokio::test]
    async fn append_batch_returns_last_offset() {
        let log: InMemoryEventLog<String> = InMemoryEventLog::new();

        let offset = log
            .append_batch(vec!["a".to_string(), "b".to_string(), "c".to_string()])
            .await
            .unwrap();

        assert_eq!(offset, 2); // Last offset
        assert_eq!(log.high_water_mark(), 3);
    }

    #[tokio::test]
    async fn consumer_polls_events() {
        let log: InMemoryEventLog<String> = InMemoryEventLog::new();
        log.append("first".to_string()).await.unwrap();
        log.append("second".to_string()).await.unwrap();

        let mut consumer = log.consumer("test-group").await.unwrap();
        let batch = consumer.poll(10, Duration::from_secs(1)).await.unwrap();

        assert_eq!(batch.len(), 2);
        assert_eq!(batch.first_offset(), Some(0));
        assert_eq!(batch.last_offset(), Some(1));
    }

    #[tokio::test]
    async fn consumer_respects_max_count() {
        let log: InMemoryEventLog<String> = InMemoryEventLog::new();
        for i in 0..10 {
            log.append(format!("event-{i}")).await.unwrap();
        }

        let mut consumer = log.consumer("test-group").await.unwrap();
        let batch = consumer.poll(3, Duration::from_secs(1)).await.unwrap();

        assert_eq!(batch.len(), 3);
        assert_eq!(batch.last_offset(), Some(2));
    }

    #[tokio::test]
    async fn consumer_continues_from_last_position() {
        let log: InMemoryEventLog<String> = InMemoryEventLog::new();
        for i in 0..10 {
            log.append(format!("event-{i}")).await.unwrap();
        }

        let mut consumer = log.consumer("test-group").await.unwrap();

        // First poll
        let batch1 = consumer.poll(3, Duration::from_secs(1)).await.unwrap();
        assert_eq!(batch1.first_offset(), Some(0));
        assert_eq!(batch1.last_offset(), Some(2));

        // Second poll continues where we left off
        let batch2 = consumer.poll(3, Duration::from_secs(1)).await.unwrap();
        assert_eq!(batch2.first_offset(), Some(3));
        assert_eq!(batch2.last_offset(), Some(5));
    }

    #[tokio::test]
    async fn consumer_seek_to_beginning() {
        let log: InMemoryEventLog<String> = InMemoryEventLog::new();
        for i in 0..5 {
            log.append(format!("event-{i}")).await.unwrap();
        }

        let mut consumer = log.consumer("test-group").await.unwrap();

        // Poll some events
        consumer.poll(3, Duration::from_secs(1)).await.unwrap();

        // Seek back to beginning
        consumer.seek(SeekPosition::Beginning).await.unwrap();

        let batch = consumer.poll(2, Duration::from_secs(1)).await.unwrap();
        assert_eq!(batch.first_offset(), Some(0));
    }

    #[tokio::test]
    async fn consumer_seek_to_end() {
        let log: InMemoryEventLog<String> = InMemoryEventLog::new();
        for i in 0..5 {
            log.append(format!("event-{i}")).await.unwrap();
        }

        let mut consumer = log.consumer("test-group").await.unwrap();
        consumer.seek(SeekPosition::End).await.unwrap();

        // Should get empty batch since we're at the end
        let batch = consumer.poll(10, Duration::from_secs(1)).await.unwrap();
        assert!(batch.is_empty());
    }

    #[tokio::test]
    async fn consumer_commit_tracks_offset() {
        let log: InMemoryEventLog<String> = InMemoryEventLog::new();
        log.append("event".to_string()).await.unwrap();

        let mut consumer = log.consumer("test-group").await.unwrap();
        assert_eq!(consumer.committed_offset(), 0);

        consumer.commit(42).await.unwrap();
        assert_eq!(consumer.committed_offset(), 42);
    }

    #[tokio::test]
    async fn consumer_group_name() {
        let log: InMemoryEventLog<String> = InMemoryEventLog::new();
        let consumer = log.consumer("my-group").await.unwrap();
        assert_eq!(consumer.group(), "my-group");
    }

    #[tokio::test]
    async fn independent_consumer_groups() {
        let log: InMemoryEventLog<String> = InMemoryEventLog::new();
        for i in 0..5 {
            log.append(format!("event-{i}")).await.unwrap();
        }

        let mut consumer_a = log.consumer("group-a").await.unwrap();
        let mut consumer_b = log.consumer("group-b").await.unwrap();

        // Consumer A reads 3
        let batch_a = consumer_a.poll(3, Duration::from_secs(1)).await.unwrap();
        assert_eq!(batch_a.len(), 3);

        // Consumer B should still start from beginning
        let batch_b = consumer_b.poll(2, Duration::from_secs(1)).await.unwrap();
        assert_eq!(batch_b.first_offset(), Some(0));
    }
}
```

**Step 2: Update lib.rs**

Add to `vibes-iggy/src/lib.rs`:

```rust
pub mod memory;

// Add to re-exports
pub use memory::InMemoryEventLog;
```

**Step 3: Run tests**

```bash
cargo test -p vibes-iggy memory
```

Expected: All tests pass

**Step 4: Commit**

```bash
git add vibes-iggy/src/memory.rs vibes-iggy/src/lib.rs
git commit -m "feat(iggy): add InMemoryEventLog for testing"
```

---

## Task 7: Move vibes-groove to plugins/ Directory

**Files:**
- Move: `vibes-groove/` → `plugins/vibes-groove/`
- Modify: `Cargo.toml` (workspace)
- Modify: `plugins/vibes-groove/Cargo.toml`

**Step 1: Create plugins directory and move**

```bash
mkdir -p plugins
git mv vibes-groove plugins/vibes-groove
```

**Step 2: Update workspace Cargo.toml**

Modify `Cargo.toml` (root):

```toml
[workspace]
resolver = "2"
members = [
    "vibes-cli",
    "vibes-core",
    "vibes-iggy",
    "vibes-plugin-api",
    "vibes-server",
    "vibes-introspection",
    "plugins/vibes-groove",
]
exclude = ["examples/plugins/hello-plugin"]
```

**Step 3: Verify compilation**

```bash
cargo check
```

**Step 4: Run all tests**

```bash
cargo test
```

**Step 5: Commit**

```bash
git add -A
git commit -m "refactor: move vibes-groove to plugins/ directory

Establishes plugins/ directory convention for first-party plugins.
This makes it clear that groove is a plugin, not a core component."
```

---

## Task 8: Update vibes-groove to Use vibes-iggy

**Files:**
- Modify: `plugins/vibes-groove/Cargo.toml`
- Modify: `plugins/vibes-groove/src/assessment/iggy/mod.rs`
- Delete: `plugins/vibes-groove/src/assessment/iggy/manager.rs`
- Modify: `plugins/vibes-groove/src/assessment/mod.rs`

**Step 1: Add vibes-iggy dependency**

Modify `plugins/vibes-groove/Cargo.toml`:

```toml
[dependencies]
vibes-iggy = { path = "../../vibes-iggy" }
# ... keep other deps
```

**Step 2: Update iggy module to re-export from vibes-iggy**

Modify `plugins/vibes-groove/src/assessment/iggy/mod.rs`:

```rust
//! Iggy integration for assessment event log.
//!
//! Re-exports IggyManager and types from vibes-iggy crate.

pub mod log;

// Re-export from vibes-iggy
pub use vibes_iggy::{IggyConfig, IggyManager, IggyState};
pub use log::IggyAssessmentLog;
```

**Step 3: Delete the old manager.rs**

```bash
rm plugins/vibes-groove/src/assessment/iggy/manager.rs
```

**Step 4: Update assessment mod.rs exports**

Modify `plugins/vibes-groove/src/assessment/mod.rs` to update imports:

```rust
//! Assessment framework for measuring session outcomes.

pub mod config;
pub mod iggy;
pub mod log;
pub mod processor;
pub mod types;

pub use config::{
    AssessmentConfig, CircuitBreakerConfig, IggyServerConfig, LlmConfig, PatternConfig,
    RetentionConfig, SamplingConfig, SessionEndConfig,
};
// Use vibes-iggy types via iggy module
pub use iggy::{IggyAssessmentLog, IggyConfig, IggyManager, IggyState};
pub use log::{AssessmentLog, InMemoryAssessmentLog};
pub use processor::AssessmentProcessor;
pub use types::*;
```

**Step 5: Verify compilation**

```bash
cargo check -p vibes-groove
```

**Step 6: Run tests**

```bash
cargo test -p vibes-groove
```

**Step 7: Commit**

```bash
git add -A
git commit -m "refactor(groove): use vibes-iggy for IggyManager

Remove duplicated IggyManager code and import from vibes-iggy crate.
This centralizes Iggy management for use by both vibes-server and groove."
```

---

## Task 9: Add vibes-core Dependency on vibes-iggy

**Files:**
- Modify: `vibes-core/Cargo.toml`
- Modify: `vibes-core/src/lib.rs`
- Modify: `vibes-core/src/events/mod.rs`

**Step 1: Add dependency**

Modify `vibes-core/Cargo.toml`:

```toml
[dependencies]
vibes-iggy = { path = "../vibes-iggy" }
# ... keep other deps
```

**Step 2: Re-export EventLog traits**

Modify `vibes-core/src/events/mod.rs`:

```rust
//! Event system for vibes

pub mod bus;
pub mod memory;
pub mod types;

// Re-export key types for convenience
pub use bus::{EventBus, EventSeq};
pub use memory::MemoryEventBus;
pub use types::{ClaudeEvent, InputSource, Usage, VibesEvent};

// Re-export EventLog types from vibes-iggy
pub use vibes_iggy::{EventBatch, EventConsumer, EventLog, Offset, SeekPosition};
```

**Step 3: Update lib.rs if needed**

Ensure `vibes-core/src/lib.rs` exports the new types:

```rust
// Add to exports
pub use events::{EventBatch, EventConsumer, EventLog, Offset, SeekPosition};
```

**Step 4: Verify**

```bash
cargo check -p vibes-core
```

**Step 5: Commit**

```bash
git add vibes-core/
git commit -m "feat(core): re-export EventLog types from vibes-iggy"
```

---

## Task 10: Create IggyEventLog Implementation (Stub)

**Files:**
- Create: `vibes-iggy/src/iggy_log.rs`
- Modify: `vibes-iggy/src/lib.rs`

**Step 1: Write stub implementation**

Create `vibes-iggy/src/iggy_log.rs`:

```rust
//! Iggy-backed EventLog implementation.
//!
//! This module provides persistent event storage using Iggy as the backend.
//! Events are written to an Iggy stream/topic and consumers track their
//! offsets independently.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::config::IggyConfig;
use crate::error::{Error, Result};
use crate::manager::IggyManager;
use crate::traits::{EventBatch, EventConsumer, EventLog, Offset, SeekPosition};

/// Stream and topic names for the event log.
pub mod topics {
    /// The stream name for vibes events.
    pub const STREAM_NAME: &str = "vibes";
    /// The topic name for the main event log.
    pub const EVENTS_TOPIC: &str = "events";
}

/// Iggy-backed implementation of EventLog.
///
/// Provides persistent event storage with consumer group offset tracking.
/// Currently uses in-memory buffering until Iggy SDK integration is complete.
pub struct IggyEventLog<E> {
    /// Reference to the Iggy manager (for connection info)
    #[allow(dead_code)]
    manager: Arc<IggyManager>,

    /// In-memory buffer for events (until Iggy client connected)
    buffer: RwLock<Vec<E>>,

    /// Current high water mark
    high_water_mark: AtomicU64,

    /// Whether we're connected to Iggy
    connected: RwLock<bool>,
}

impl<E> IggyEventLog<E>
where
    E: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone + 'static,
{
    /// Create a new IggyEventLog.
    ///
    /// The manager should be started before calling this.
    pub fn new(manager: Arc<IggyManager>) -> Self {
        Self {
            manager,
            buffer: RwLock::new(Vec::new()),
            high_water_mark: AtomicU64::new(0),
            connected: RwLock::new(false),
        }
    }

    /// Connect to the Iggy server.
    ///
    /// This establishes the connection and creates streams/topics if needed.
    pub async fn connect(&self) -> Result<()> {
        // TODO: Implement actual Iggy client connection
        // For now, mark as connected and use buffer
        info!("IggyEventLog connecting (stub implementation)");
        *self.connected.write().await = true;
        Ok(())
    }

    /// Check if connected to Iggy.
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }
}

#[async_trait]
impl<E> EventLog<E> for IggyEventLog<E>
where
    E: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone + 'static,
{
    async fn append(&self, event: E) -> Result<Offset> {
        let offset = self.high_water_mark.fetch_add(1, Ordering::SeqCst);

        // TODO: Write to Iggy when connected
        // For now, buffer in memory
        self.buffer.write().await.push(event);

        debug!(offset, "Appended event to log");
        Ok(offset)
    }

    async fn append_batch(&self, events: Vec<E>) -> Result<Offset> {
        let count = events.len() as u64;
        if count == 0 {
            return Ok(self.high_water_mark().saturating_sub(1));
        }

        let first_offset = self.high_water_mark.fetch_add(count, Ordering::SeqCst);

        // TODO: Write batch to Iggy
        self.buffer.write().await.extend(events);

        debug!(first_offset, count, "Appended batch to log");
        Ok(first_offset + count - 1)
    }

    async fn consumer(&self, group: &str) -> Result<Box<dyn EventConsumer<E>>> {
        // TODO: Create actual Iggy consumer
        // For now, create an in-memory consumer over the buffer
        let events = self.buffer.read().await.clone();

        Ok(Box::new(IggyEventConsumer {
            group: group.to_string(),
            events: Arc::new(events),
            current_offset: 0,
            committed_offset: 0,
        }))
    }

    fn high_water_mark(&self) -> Offset {
        self.high_water_mark.load(Ordering::SeqCst)
    }
}

/// Iggy-backed consumer implementation.
///
/// Currently uses in-memory snapshot until Iggy SDK integration.
struct IggyEventConsumer<E> {
    group: String,
    events: Arc<Vec<E>>,
    current_offset: Offset,
    committed_offset: Offset,
}

#[async_trait]
impl<E> EventConsumer<E> for IggyEventConsumer<E>
where
    E: Send + Sync + Clone + 'static,
{
    async fn poll(&mut self, max_count: usize, _timeout: Duration) -> Result<EventBatch<E>> {
        let start = self.current_offset as usize;
        let end = std::cmp::min(start + max_count, self.events.len());

        if start >= self.events.len() {
            return Ok(EventBatch::empty());
        }

        let events: Vec<(Offset, E)> = self.events[start..end]
            .iter()
            .enumerate()
            .map(|(i, e)| ((start + i) as Offset, e.clone()))
            .collect();

        if let Some((last_offset, _)) = events.last() {
            self.current_offset = last_offset + 1;
        }

        Ok(EventBatch::new(events))
    }

    async fn commit(&mut self, offset: Offset) -> Result<()> {
        // TODO: Commit to Iggy
        self.committed_offset = offset;
        debug!(group = %self.group, offset, "Committed offset");
        Ok(())
    }

    async fn seek(&mut self, position: SeekPosition) -> Result<()> {
        self.current_offset = match position {
            SeekPosition::Beginning => 0,
            SeekPosition::End => self.events.len() as Offset,
            SeekPosition::Offset(o) => o,
        };
        debug!(group = %self.group, offset = self.current_offset, "Seeked to position");
        Ok(())
    }

    fn committed_offset(&self) -> Offset {
        self.committed_offset
    }

    fn group(&self) -> &str {
        &self.group
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn iggy_log_append_and_read() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log: IggyEventLog<String> = IggyEventLog::new(manager);

        let o1 = log.append("first".to_string()).await.unwrap();
        let o2 = log.append("second".to_string()).await.unwrap();

        assert_eq!(o1, 0);
        assert_eq!(o2, 1);
        assert_eq!(log.high_water_mark(), 2);
    }

    #[tokio::test]
    async fn iggy_log_consumer_polls() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log: IggyEventLog<String> = IggyEventLog::new(manager);

        log.append("event-1".to_string()).await.unwrap();
        log.append("event-2".to_string()).await.unwrap();

        let mut consumer = log.consumer("test").await.unwrap();
        let batch = consumer.poll(10, Duration::from_secs(1)).await.unwrap();

        assert_eq!(batch.len(), 2);
    }
}
```

**Step 2: Update lib.rs**

Add to `vibes-iggy/src/lib.rs`:

```rust
pub mod iggy_log;

pub use iggy_log::IggyEventLog;
```

**Step 3: Run tests**

```bash
cargo test -p vibes-iggy iggy_log
```

**Step 4: Commit**

```bash
git add vibes-iggy/src/iggy_log.rs vibes-iggy/src/lib.rs
git commit -m "feat(iggy): add IggyEventLog stub implementation

Provides EventLog trait implementation backed by in-memory buffer.
Full Iggy SDK integration will be completed in follow-up work."
```

---

## Task 11: Update PROGRESS.md

**Files:**
- Modify: `docs/PROGRESS.md`

**Step 1: Mark completed items**

Update the 4.4.2a section in `docs/PROGRESS.md`:

```markdown
**4.4.2a: EventLog Migration (In Progress)**
> **Design:** [milestone-4.4.2a-design.md](plans/14-continual-learning/milestone-4.4.2a-design.md)

- [x] Create `vibes-iggy` crate with EventLog/EventConsumer traits
- [x] Move `IggyManager` from vibes-groove to vibes-iggy
- [x] Implement `InMemoryEventLog` for testing
- [x] Move `vibes-groove/` → `plugins/vibes-groove/`
- [~] Implement `IggyEventLog` (stub, full SDK integration pending)
- [ ] Migrate vibes-server subscribers to consumer pattern
- [ ] Remove `MemoryEventBus` and old `EventBus` trait
```

**Step 2: Add changelog entry**

Add to the Changelog table at the bottom of PROGRESS.md:

```markdown
| 2025-12-30 | Created vibes-iggy crate, moved vibes-groove to plugins/ |
```

**Step 3: Commit**

```bash
git add docs/PROGRESS.md
git commit -m "docs: update PROGRESS.md with 4.4.2a progress"
```

---

## Task 12: Create PR

**Step 1: Ensure all tests pass**

```bash
just pre-commit
```

**Step 2: Push branch**

```bash
git push -u origin assessment-logic
```

**Step 3: Create PR**

```bash
gh pr create --title "feat(iggy): EventLog migration foundation (4.4.2a partial)" --body "$(cat <<'EOF'
## Summary
- Create `vibes-iggy` crate with EventLog/EventConsumer traits
- Move `IggyManager` from vibes-groove to vibes-iggy
- Add `InMemoryEventLog` for testing
- Move `vibes-groove/` to `plugins/vibes-groove/`
- Add `IggyEventLog` stub implementation

## Architecture Change
Replaces pub/sub EventBus model with producer/consumer EventLog model:
- Each consumer group tracks its own offset
- Events persist across restarts (when Iggy SDK integration complete)
- Crash recovery via committed offsets

## Remaining Work (follow-up PRs)
- Complete Iggy SDK integration in IggyEventLog
- Migrate vibes-server subscribers to consumer pattern
- Remove MemoryEventBus

## Test Plan
- [x] All unit tests passing (`just test`)
- [x] Pre-commit checks pass (`just pre-commit`)
- [x] vibes-groove still compiles with new location

Part of milestone 4.4.2a: EventLog Migration
See design: docs/plans/14-continual-learning/milestone-4.4.2a-design.md
EOF
)"
```

---

## Summary

This implementation plan covers the foundational work for 4.4.2a:

| Task | Component | Status |
|------|-----------|--------|
| 1 | vibes-iggy crate skeleton | Ready |
| 2 | Error types | Ready |
| 3 | EventLog/EventConsumer traits | Ready |
| 4 | IggyConfig | Ready |
| 5 | IggyManager | Ready |
| 6 | InMemoryEventLog | Ready |
| 7 | Move vibes-groove to plugins/ | Ready |
| 8 | Update vibes-groove imports | Ready |
| 9 | vibes-core re-exports | Ready |
| 10 | IggyEventLog stub | Ready |
| 11 | Update PROGRESS.md | Ready |
| 12 | Create PR | Ready |

**Remaining for future PRs:**
- Complete Iggy SDK integration (actual wire protocol)
- Migrate vibes-server subscribers to consumer pattern
- Remove MemoryEventBus and EventBus trait
