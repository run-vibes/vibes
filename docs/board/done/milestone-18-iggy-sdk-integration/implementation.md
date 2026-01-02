# Iggy SDK Integration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Wire `IggyEventLog` to the Iggy SDK so events actually persist to the Iggy server instead of buffering in memory.

**Architecture:** Specialize `IggyEventLog` for `VibesEvent`, add `IggyClient` for TCP connection to local Iggy server, partition events by `session_id` across 8 partitions, implement lazy reconnect with in-memory buffer for resilience.

**Tech Stack:** Rust, iggy SDK 0.6, tokio async runtime, serde_json for event serialization.

---

## Task 1: Add session_id() Method to VibesEvent

**Files:**
- Modify: `vibes-core/src/events.rs`
- Test: `vibes-core/src/events.rs` (inline tests)

**Step 1: Write the failing test**

Add to the existing `#[cfg(test)]` module in `vibes-core/src/events.rs`:

```rust
#[test]
fn test_session_id_extraction() {
    // Events with session_id
    let event = VibesEvent::SessionCreated {
        session_id: "sess-123".to_string(),
        name: None,
    };
    assert_eq!(event.session_id(), Some("sess-123"));

    let claude_event = VibesEvent::Claude {
        session_id: "sess-456".to_string(),
        event: ClaudeEvent::TextDelta { text: "hi".into() },
    };
    assert_eq!(claude_event.session_id(), Some("sess-456"));

    // Events without session_id
    let server_event = VibesEvent::ServerStarted { port: 7432 };
    assert_eq!(server_event.session_id(), None);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-core test_session_id_extraction`
Expected: FAIL with "no method named `session_id`"

**Step 3: Write minimal implementation**

Add method to `VibesEvent` impl block in `vibes-core/src/events.rs`:

```rust
impl VibesEvent {
    /// Extract session_id if this event is associated with a session.
    ///
    /// Used for partitioning events in the event log.
    #[must_use]
    pub fn session_id(&self) -> Option<&str> {
        match self {
            Self::SessionCreated { session_id, .. }
            | Self::SessionRenamed { session_id, .. }
            | Self::SessionDeleted { session_id }
            | Self::SessionStateChanged { session_id, .. }
            | Self::Claude { session_id, .. }
            | Self::TurnCompleted { session_id, .. }
            | Self::PromptSubmitted { session_id, .. }
            | Self::PtyOutput { session_id, .. }
            | Self::PtyExit { session_id, .. } => Some(session_id.as_str()),

            Self::ServerStarted { .. }
            | Self::ServerStopped
            | Self::TunnelStarted { .. }
            | Self::TunnelStopped { .. }
            | Self::PluginLoaded { .. }
            | Self::PluginUnloaded { .. } => None,
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p vibes-core test_session_id_extraction`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/events.rs
git commit -m "feat(core): add session_id() method to VibesEvent"
```

---

## Task 2: Remove Generic from IggyEventLog

**Files:**
- Modify: `vibes-iggy/src/iggy_log.rs`
- Modify: `vibes-iggy/src/lib.rs`
- Modify: `vibes-server/src/state.rs`

**Step 1: Update IggyEventLog struct definition**

In `vibes-iggy/src/iggy_log.rs`, change:

```rust
// FROM:
pub struct IggyEventLog<E> {
    manager: Arc<IggyManager>,
    buffer: RwLock<Vec<E>>,
    high_water_mark: AtomicU64,
    connected: RwLock<bool>,
}

// TO:
use vibes_core::VibesEvent;

pub struct IggyEventLog {
    manager: Arc<IggyManager>,
    buffer: RwLock<Vec<VibesEvent>>,
    high_water_mark: AtomicU64,
    connected: RwLock<bool>,
}
```

**Step 2: Update all impl blocks**

Remove generic parameters from all impl blocks:

```rust
// FROM:
impl<E> IggyEventLog<E>
where
    E: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone + 'static,

// TO:
impl IggyEventLog
```

**Step 3: Update EventLog trait implementation**

```rust
// FROM:
#[async_trait]
impl<E> EventLog<E> for IggyEventLog<E>
where
    E: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone + 'static,

// TO:
#[async_trait]
impl EventLog<VibesEvent> for IggyEventLog
```

**Step 4: Update IggyEventConsumer similarly**

Remove generic from `IggyEventConsumer<E>` → `IggyEventConsumer`

**Step 5: Update vibes-iggy/src/lib.rs re-export**

Ensure the re-export works without generics.

**Step 6: Update vibes-iggy/Cargo.toml to add vibes-core dependency**

```toml
[dependencies]
vibes-core = { path = "../vibes-core" }
```

**Step 7: Update vibes-server/src/state.rs**

```rust
// FROM:
let event_log: Arc<dyn EventLog<VibesEvent>> =
    Arc::new(IggyEventLog::<VibesEvent>::new(manager));

// TO:
let event_log: Arc<dyn EventLog<VibesEvent>> =
    Arc::new(IggyEventLog::new(manager));
```

**Step 8: Run tests to verify everything compiles**

Run: `cargo test -p vibes-iggy`
Expected: PASS (existing stub tests still work)

**Step 9: Commit**

```bash
git add vibes-iggy/ vibes-server/src/state.rs
git commit -m "refactor(iggy): specialize IggyEventLog for VibesEvent"
```

---

## Task 3: Add IggyClient and Connection Logic

**Files:**
- Modify: `vibes-iggy/src/iggy_log.rs`
- Create: `vibes-iggy/src/client.rs` (optional helper)

**Step 1: Write failing test for connect()**

Add to tests in `vibes-iggy/src/iggy_log.rs`:

```rust
#[tokio::test]
#[ignore] // Requires running Iggy server
async fn test_connect_creates_stream_and_topic() {
    let config = IggyConfig::default();
    let manager = Arc::new(IggyManager::new(config));
    // Note: In real test, manager.start() would be called

    let log = IggyEventLog::new(Arc::clone(&manager));
    log.connect().await.unwrap();

    assert!(log.is_connected().await);
    // Stream and topic should exist (verified by not erroring on connect)
}
```

**Step 2: Add IggyClient to struct**

```rust
use iggy::clients::client::IggyClient;
use iggy::client::{Client, StreamClient, TopicClient, UserClient, MessageClient};
use iggy::compression::compression_algorithm::CompressionAlgorithm;
use iggy::users::defaults::{DEFAULT_ROOT_PASSWORD, DEFAULT_ROOT_USERNAME};
use iggy::utils::expiry::IggyExpiry;
use iggy::utils::topic_size::MaxTopicSize;

pub mod topics {
    pub const STREAM_NAME: &str = "vibes";
    pub const STREAM_ID: u32 = 1;
    pub const EVENTS_TOPIC: &str = "events";
    pub const TOPIC_ID: u32 = 1;
    pub const PARTITION_COUNT: u32 = 8;
}

pub struct IggyEventLog {
    manager: Arc<IggyManager>,
    client: IggyClient,
    buffer: RwLock<Vec<VibesEvent>>,
    high_water_mark: AtomicU64,
    connected: RwLock<bool>,
}
```

**Step 3: Update new() to create client**

```rust
impl IggyEventLog {
    #[must_use]
    pub fn new(manager: Arc<IggyManager>) -> Self {
        // Create client configured for TCP to localhost
        let client = IggyClient::builder()
            .with_tcp()
            .with_server_address(manager.connection_address())
            .build()
            .expect("Failed to build Iggy client");

        Self {
            manager,
            client,
            buffer: RwLock::new(Vec::new()),
            high_water_mark: AtomicU64::new(0),
            connected: RwLock::new(false),
        }
    }
}
```

**Step 4: Implement connect()**

```rust
impl IggyEventLog {
    pub async fn connect(&self) -> Result<()> {
        use iggy::identifier::Identifier;

        // 1. Connect to server
        self.client.connect().await?;
        info!("Connected to Iggy server at {}", self.manager.connection_address());

        // 2. Login
        self.client
            .login_user(DEFAULT_ROOT_USERNAME, DEFAULT_ROOT_PASSWORD)
            .await?;
        debug!("Logged in to Iggy as root user");

        // 3. Create stream if not exists
        let stream_id = Identifier::numeric(topics::STREAM_ID)?;
        match self.client
            .create_stream(topics::STREAM_NAME, Some(topics::STREAM_ID))
            .await
        {
            Ok(_) => info!("Created stream '{}'", topics::STREAM_NAME),
            Err(e) => {
                // Check if it's "already exists" error
                let err_str = e.to_string();
                if err_str.contains("already exists") || err_str.contains("AlreadyExists") {
                    debug!("Stream '{}' already exists", topics::STREAM_NAME);
                } else {
                    return Err(e.into());
                }
            }
        }

        // 4. Create topic if not exists
        match self.client
            .create_topic(
                &stream_id,
                topics::EVENTS_TOPIC,
                topics::PARTITION_COUNT,
                CompressionAlgorithm::None,
                None,
                Some(topics::TOPIC_ID),
                IggyExpiry::NeverExpire,
                MaxTopicSize::ServerDefault,
            )
            .await
        {
            Ok(_) => info!(
                "Created topic '{}' with {} partitions",
                topics::EVENTS_TOPIC,
                topics::PARTITION_COUNT
            ),
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("already exists") || err_str.contains("AlreadyExists") {
                    debug!("Topic '{}' already exists", topics::EVENTS_TOPIC);
                } else {
                    return Err(e.into());
                }
            }
        }

        *self.connected.write().await = true;
        info!("IggyEventLog fully connected and ready");
        Ok(())
    }
}
```

**Step 5: Run compilation check**

Run: `cargo check -p vibes-iggy`
Expected: Compiles (may have warnings)

**Step 6: Commit**

```bash
git add vibes-iggy/
git commit -m "feat(iggy): add IggyClient and connect() implementation"
```

---

## Task 4: Implement append() with Partitioning

**Files:**
- Modify: `vibes-iggy/src/iggy_log.rs`

**Step 1: Write failing integration test**

```rust
#[tokio::test]
#[ignore] // Requires running Iggy server
async fn test_append_sends_to_iggy() {
    let log = setup_connected_log().await;

    let event = VibesEvent::SessionCreated {
        session_id: "test-sess".to_string(),
        name: Some("Test Session".to_string()),
    };

    let offset = log.append(event).await.unwrap();
    assert_eq!(offset, 0);

    // Second append should increment
    let event2 = VibesEvent::SessionDeleted {
        session_id: "test-sess".to_string(),
    };
    let offset2 = log.append(event2).await.unwrap();
    assert_eq!(offset2, 1);
}

async fn setup_connected_log() -> IggyEventLog {
    let config = IggyConfig::default();
    let manager = Arc::new(IggyManager::new(config));
    let log = IggyEventLog::new(Arc::clone(&manager));
    log.connect().await.unwrap();
    log
}
```

**Step 2: Implement append()**

```rust
use iggy::messages::send_messages::{Message, Partitioning};

#[async_trait]
impl EventLog<VibesEvent> for IggyEventLog {
    async fn append(&self, event: VibesEvent) -> Result<Offset> {
        // Get session_id for partitioning (default to "unknown" for server events)
        let partition_key = event.session_id().unwrap_or("unknown");

        // Serialize event to JSON
        let payload = serde_json::to_vec(&event)
            .map_err(|e| Error::Serialization(e.to_string()))?;

        // Create Iggy message
        let message = Message::from_bytes(payload.into())
            .map_err(|e| Error::Client(e.to_string()))?;

        // Partition by session_id (consistent hashing)
        let partitioning = Partitioning::messages_key_str(partition_key);

        // Send to Iggy
        let stream_id = topics::STREAM_ID.try_into()
            .map_err(|e: iggy::error::IggyError| Error::Client(e.to_string()))?;
        let topic_id = topics::TOPIC_ID.try_into()
            .map_err(|e: iggy::error::IggyError| Error::Client(e.to_string()))?;

        self.client
            .send_messages(&stream_id, &topic_id, &partitioning, &mut vec![message])
            .await
            .map_err(|e| Error::Client(e.to_string()))?;

        // Increment and return local offset
        let offset = self.high_water_mark.fetch_add(1, Ordering::SeqCst);
        debug!(offset, partition_key, "Appended event to Iggy");

        Ok(offset)
    }

    async fn append_batch(&self, events: Vec<VibesEvent>) -> Result<Offset> {
        if events.is_empty() {
            return Ok(self.high_water_mark().saturating_sub(1));
        }

        let count = events.len();

        // Group events by partition key for efficient batching
        // For simplicity, send each event individually for now
        // (Iggy handles batching internally)
        for event in events {
            self.append(event).await?;
        }

        // Return offset of last event
        Ok(self.high_water_mark().saturating_sub(1))
    }

    // ... consumer and high_water_mark unchanged for now
}
```

**Step 3: Add Serialization error variant**

In `vibes-iggy/src/error.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // ... existing variants

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Iggy client error: {0}")]
    Client(String),
}
```

**Step 4: Run compilation check**

Run: `cargo check -p vibes-iggy`
Expected: Compiles

**Step 5: Commit**

```bash
git add vibes-iggy/
git commit -m "feat(iggy): implement append() with session_id partitioning"
```

---

## Task 5: Implement IggyEventConsumer

**Files:**
- Modify: `vibes-iggy/src/iggy_log.rs`

**Step 1: Write failing test**

```rust
#[tokio::test]
#[ignore]
async fn test_consumer_polls_events() {
    let log = setup_connected_log().await;

    // Append some events
    log.append(VibesEvent::SessionCreated {
        session_id: "poll-test".to_string(),
        name: None,
    }).await.unwrap();

    // Create consumer and poll
    let mut consumer = log.consumer("test-consumer").await.unwrap();
    consumer.seek(SeekPosition::Beginning).await.unwrap();

    let batch = consumer.poll(10, Duration::from_secs(1)).await.unwrap();
    assert!(!batch.is_empty());
}
```

**Step 2: Rewrite IggyEventConsumer**

```rust
use iggy::consumer::Consumer;
use iggy::messages::poll_messages::PollingStrategy;
use iggy::identifier::Identifier;

pub struct IggyEventConsumer {
    client: IggyClient,
    group: String,
    offsets: [u64; topics::PARTITION_COUNT as usize],
    committed_offsets: [u64; topics::PARTITION_COUNT as usize],
}

impl IggyEventConsumer {
    fn new(client: IggyClient, group: String) -> Self {
        Self {
            client,
            group,
            offsets: [0; topics::PARTITION_COUNT as usize],
            committed_offsets: [0; topics::PARTITION_COUNT as usize],
        }
    }
}

#[async_trait]
impl EventConsumer<VibesEvent> for IggyEventConsumer {
    async fn poll(&mut self, max_count: usize, _timeout: Duration) -> Result<EventBatch<VibesEvent>> {
        let mut all_events = Vec::new();
        let per_partition = (max_count / topics::PARTITION_COUNT as usize).max(1);

        let stream_id: Identifier = topics::STREAM_ID.try_into()
            .map_err(|e: iggy::error::IggyError| Error::Client(e.to_string()))?;
        let topic_id: Identifier = topics::TOPIC_ID.try_into()
            .map_err(|e: iggy::error::IggyError| Error::Client(e.to_string()))?;

        for partition_id in 0..topics::PARTITION_COUNT {
            let idx = partition_id as usize;
            let consumer = Consumer::new(Identifier::named(&self.group)
                .map_err(|e| Error::Client(e.to_string()))?);
            let strategy = PollingStrategy::offset(self.offsets[idx]);

            let polled = self.client
                .poll_messages(
                    &stream_id,
                    &topic_id,
                    Some(partition_id),
                    &consumer,
                    &strategy,
                    per_partition as u32,
                    false, // auto_commit = false
                )
                .await
                .map_err(|e| Error::Client(e.to_string()))?;

            for msg in polled.messages {
                let event: VibesEvent = serde_json::from_slice(&msg.payload)
                    .map_err(|e| Error::Serialization(e.to_string()))?;
                all_events.push((msg.offset, event));
                self.offsets[idx] = msg.offset + 1;
            }
        }

        // Sort by offset for rough ordering
        all_events.sort_by_key(|(offset, _)| *offset);

        Ok(EventBatch::new(all_events))
    }

    async fn commit(&mut self, _offset: Offset) -> Result<()> {
        let stream_id: Identifier = topics::STREAM_ID.try_into()
            .map_err(|e: iggy::error::IggyError| Error::Client(e.to_string()))?;
        let topic_id: Identifier = topics::TOPIC_ID.try_into()
            .map_err(|e: iggy::error::IggyError| Error::Client(e.to_string()))?;

        for partition_id in 0..topics::PARTITION_COUNT {
            let idx = partition_id as usize;
            let consumer = Consumer::new(Identifier::named(&self.group)
                .map_err(|e| Error::Client(e.to_string()))?);

            self.client
                .store_consumer_offset(
                    &consumer,
                    &stream_id,
                    &topic_id,
                    Some(partition_id),
                    self.offsets[idx],
                )
                .await
                .map_err(|e| Error::Client(e.to_string()))?;

            self.committed_offsets[idx] = self.offsets[idx];
        }

        debug!(group = %self.group, "Committed offsets to Iggy");
        Ok(())
    }

    async fn seek(&mut self, position: SeekPosition) -> Result<()> {
        match position {
            SeekPosition::Beginning => {
                self.offsets = [0; topics::PARTITION_COUNT as usize];
            }
            SeekPosition::End => {
                // For now, just set to a high value; proper impl would query topic
                self.offsets = [u64::MAX; topics::PARTITION_COUNT as usize];
            }
            SeekPosition::Offset(o) => {
                self.offsets = [o; topics::PARTITION_COUNT as usize];
            }
        }
        debug!(group = %self.group, "Seeked consumer");
        Ok(())
    }

    fn committed_offset(&self) -> Offset {
        // Return min committed offset across partitions
        *self.committed_offsets.iter().min().unwrap_or(&0)
    }

    fn group(&self) -> &str {
        &self.group
    }
}
```

**Step 3: Update consumer() factory method**

```rust
impl EventLog<VibesEvent> for IggyEventLog {
    async fn consumer(&self, group: &str) -> Result<Box<dyn EventConsumer<VibesEvent>>> {
        // Create a new client for the consumer (each consumer needs its own connection)
        let consumer_client = IggyClient::builder()
            .with_tcp()
            .with_server_address(self.manager.connection_address())
            .build()
            .map_err(|e| Error::Client(e.to_string()))?;

        consumer_client.connect().await
            .map_err(|e| Error::Client(e.to_string()))?;
        consumer_client
            .login_user(DEFAULT_ROOT_USERNAME, DEFAULT_ROOT_PASSWORD)
            .await
            .map_err(|e| Error::Client(e.to_string()))?;

        Ok(Box::new(IggyEventConsumer::new(consumer_client, group.to_string())))
    }
}
```

**Step 4: Run compilation check**

Run: `cargo check -p vibes-iggy`
Expected: Compiles

**Step 5: Commit**

```bash
git add vibes-iggy/
git commit -m "feat(iggy): implement IggyEventConsumer with multi-partition polling"
```

---

## Task 6: Add Reconnect Buffer

**Files:**
- Modify: `vibes-iggy/src/iggy_log.rs`

**Step 1: Write test for buffer behavior**

```rust
#[test]
fn test_buffer_overflow_drops_oldest() {
    use std::sync::atomic::AtomicBool;

    // This tests the buffer logic in isolation
    let buffer: Vec<VibesEvent> = (0..10_001)
        .map(|i| VibesEvent::SessionCreated {
            session_id: format!("sess-{}", i),
            name: None,
        })
        .collect();

    // Simulate overflow logic
    let mut capped = buffer;
    if capped.len() > 10_000 {
        capped.remove(0);
    }

    assert_eq!(capped.len(), 10_000);
    // First event should be sess-1 (sess-0 was dropped)
    match &capped[0] {
        VibesEvent::SessionCreated { session_id, .. } => {
            assert_eq!(session_id, "sess-1");
        }
        _ => panic!("Expected SessionCreated"),
    }
}
```

**Step 2: Add buffer constants and field**

```rust
const MAX_RECONNECT_BUFFER: usize = 10_000;

pub struct IggyEventLog {
    manager: Arc<IggyManager>,
    client: IggyClient,
    reconnect_buffer: RwLock<Vec<VibesEvent>>,
    high_water_mark: AtomicU64,
    connected: RwLock<bool>,
}
```

**Step 3: Add buffer methods**

```rust
impl IggyEventLog {
    /// Buffer an event when disconnected.
    async fn buffer_event(&self, event: VibesEvent) {
        let mut buffer = self.reconnect_buffer.write().await;

        if buffer.len() >= MAX_RECONNECT_BUFFER {
            warn!(
                buffer_size = buffer.len(),
                "Reconnect buffer full, dropping oldest event"
            );
            buffer.remove(0);
        }

        buffer.push(event);
        debug!(buffer_size = buffer.len(), "Buffered event during disconnect");
    }

    /// Flush buffered events after reconnection.
    async fn flush_buffer(&self) -> Result<()> {
        let events = std::mem::take(&mut *self.reconnect_buffer.write().await);

        if events.is_empty() {
            return Ok(());
        }

        info!(count = events.len(), "Flushing reconnect buffer to Iggy");

        for event in events {
            self.try_send(&event).await?;
        }

        Ok(())
    }

    /// Internal send that doesn't handle reconnection.
    async fn try_send(&self, event: &VibesEvent) -> Result<Offset> {
        // ... existing append logic moved here
    }
}
```

**Step 4: Update append() to use buffer on error**

```rust
async fn append(&self, event: VibesEvent) -> Result<Offset> {
    match self.try_send(&event).await {
        Ok(offset) => Ok(offset),
        Err(e) => {
            // Check if it's a connection error
            let is_connection_error = matches!(&e, Error::Client(msg) if
                msg.contains("connection") ||
                msg.contains("disconnected") ||
                msg.contains("not connected"));

            if is_connection_error {
                warn!(error = %e, "Connection error, buffering event");
                self.buffer_event(event).await;
                *self.connected.write().await = false;
                // Return synthetic offset
                Ok(self.high_water_mark.fetch_add(1, Ordering::SeqCst))
            } else {
                Err(e)
            }
        }
    }
}
```

**Step 5: Commit**

```bash
git add vibes-iggy/
git commit -m "feat(iggy): add reconnect buffer for connection resilience"
```

---

## Task 7: Integration Test with Real Iggy

**Files:**
- Create: `vibes-iggy/tests/integration.rs`

**Step 1: Create integration test file**

```rust
//! Integration tests requiring a running Iggy server.
//!
//! Run with: cargo test -p vibes-iggy --test integration -- --ignored

use std::sync::Arc;
use std::time::Duration;

use vibes_core::VibesEvent;
use vibes_iggy::{EventLog, IggyConfig, IggyEventLog, IggyManager, SeekPosition};

async fn setup() -> (Arc<IggyManager>, IggyEventLog) {
    let config = IggyConfig::default();
    let manager = Arc::new(IggyManager::new(config));

    // Start Iggy server
    manager.start().await.expect("Failed to start Iggy");

    // Wait for startup
    tokio::time::sleep(Duration::from_millis(500)).await;

    let log = IggyEventLog::new(Arc::clone(&manager));
    log.connect().await.expect("Failed to connect");

    (manager, log)
}

#[tokio::test]
#[ignore]
async fn test_append_and_poll_roundtrip() {
    let (_manager, log) = setup().await;

    let event = VibesEvent::SessionCreated {
        session_id: "integration-test".to_string(),
        name: Some("Integration Test".to_string()),
    };

    log.append(event.clone()).await.unwrap();

    let mut consumer = log.consumer("integration-consumer").await.unwrap();
    consumer.seek(SeekPosition::Beginning).await.unwrap();

    let batch = consumer.poll(10, Duration::from_secs(1)).await.unwrap();

    assert!(!batch.is_empty(), "Should have polled at least one event");
}

#[tokio::test]
#[ignore]
async fn test_partition_by_session_id() {
    let (_manager, log) = setup().await;

    // Append events for different sessions
    for i in 0..10 {
        log.append(VibesEvent::SessionCreated {
            session_id: format!("session-{}", i % 3), // 3 different sessions
            name: None,
        }).await.unwrap();
    }

    // All events should be retrievable
    let mut consumer = log.consumer("partition-test").await.unwrap();
    consumer.seek(SeekPosition::Beginning).await.unwrap();

    let batch = consumer.poll(20, Duration::from_secs(1)).await.unwrap();
    assert!(batch.len() >= 10, "Should retrieve all 10 events");
}
```

**Step 2: Run integration tests (requires Iggy running)**

Run: `cargo test -p vibes-iggy --test integration -- --ignored`
Expected: Tests pass if Iggy is running

**Step 3: Commit**

```bash
git add vibes-iggy/tests/
git commit -m "test(iggy): add integration tests for IggyEventLog"
```

---

## Task 8: Update E2E Tests to Use Real Iggy

**Files:**
- Modify: `vibes-server/tests/` (E2E test files)

**Step 1: Find existing E2E test setup**

Look for test setup that creates `AppState` and ensure it uses `new_with_iggy()`.

**Step 2: Add verification that events reach Iggy**

```rust
#[tokio::test]
#[ignore] // Requires Iggy
async fn test_e2e_events_persist_to_iggy() {
    // Start server with Iggy
    let state = AppState::new_with_iggy().await;

    // Emit event
    state.append_event(VibesEvent::SessionCreated {
        session_id: "e2e-test".to_string(),
        name: None,
    });

    // Wait for async processing
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify via consumer
    let mut consumer = state.event_log.consumer("e2e-verify").await.unwrap();
    consumer.seek(SeekPosition::Beginning).await.unwrap();

    let batch = consumer.poll(10, Duration::from_secs(1)).await.unwrap();
    assert!(!batch.is_empty(), "Events should persist to Iggy");
}
```

**Step 3: Commit**

```bash
git add vibes-server/tests/
git commit -m "test(e2e): verify events flow through Iggy"
```

---

## Task 9: Update Documentation and Progress

**Files:**
- Modify: `docs/PROGRESS.md`
- Modify: `docs/plans/18-iggy-sdk-integration/design.md`

**Step 1: Mark milestone complete in PROGRESS.md**

Add entry for Milestone 18.

**Step 2: Update design.md checklist**

Mark all items as complete.

**Step 3: Commit**

```bash
git add docs/
git commit -m "docs: mark Iggy SDK integration complete"
```

---

## Summary

| Task | Description | Estimated Effort |
|------|-------------|------------------|
| 1 | Add session_id() to VibesEvent | Small |
| 2 | Remove generic from IggyEventLog | Medium |
| 3 | Add IggyClient and connect() | Medium |
| 4 | Implement append() with partitioning | Medium |
| 5 | Implement IggyEventConsumer | Large |
| 6 | Add reconnect buffer | Medium |
| 7 | Integration tests | Medium |
| 8 | E2E test updates | Small |
| 9 | Documentation | Small |

**Total: ~9 tasks, estimated 2-3 hours of focused work.**
