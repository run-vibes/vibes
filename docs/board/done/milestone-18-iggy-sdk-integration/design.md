# Milestone 18: Iggy SDK Integration - Design Document

> Complete the IggyEventLog implementation by integrating with the Iggy Rust SDK for actual message streaming.

## Overview

Milestone 16 (Iggy Bundling) established the infrastructure: git submodule, subprocess management via `IggyManager`, and the `IggyEventLog` type. However, `IggyEventLog` is currently a **stub implementation** that buffers events in memory without ever communicating with Iggy.

This milestone completes the integration by wiring `IggyEventLog` to the Iggy SDK, enabling true persistent event storage.

### Problem Statement

The current `IggyEventLog` implementation:
- Marks itself as "connected" without connecting
- Buffers events to a `Vec<E>` instead of sending to Iggy
- Returns synthetic offsets that don't correspond to Iggy offsets
- Has no stream/topic/partition creation
- Provides no actual persistence

Result: Events are lost on restart despite Iggy server running.

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Transport** | TCP | Fast, reliable, standard for local IPC |
| **Authentication** | Default credentials (iggy/iggy) | Local subprocess only, not network-exposed |
| **Partitions** | 8 partitions | Balance of parallelism and overhead |
| **Partition key** | session_id | Enables per-session parallelism |
| **Consumer model** | Poll all partitions | Simple API, can add consumer groups later |
| **Offset commit** | Manual | Control over exactly when offsets advance |
| **Error handling** | Lazy reconnect with buffer | Non-blocking producers, graceful recovery |
| **Generic type** | Generic with Partitionable trait | Avoids cyclic dependency while enabling partition by session_id |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        vibes-server                              │
│  ┌──────────────┐    ┌─────────────────────────────────────┐    │
│  │ Event        │───▶│ IggyEventLog                        │    │
│  │ Producers    │    │  ├─ IggyClient (TCP)                │    │
│  │ (handlers,   │    │  ├─ reconnect_buffer (Vec<E>)       │    │
│  │  PTY, etc)   │    │  └─ high_water_mark                 │    │
│  └──────────────┘    └──────────────┬──────────────────────┘    │
│                                     │                            │
│  ┌──────────────┐                   │ TCP :8090                  │
│  │ Consumers    │◀──────────────────┤                            │
│  │ (WS, Notif,  │                   ▼                            │
│  │  Assessment) │    ┌─────────────────────────────────────┐    │
│  └──────────────┘    │ iggy-server (subprocess)            │    │
│                      │  Stream: "vibes"                     │    │
│                      │  Topic: "events" (8 partitions)      │    │
│                      │  Partitioning: by session_id         │    │
│                      └─────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

### Stream/Topic Structure

| Level | Name | Purpose |
|-------|------|---------|
| Stream | `vibes` | All vibes events |
| Topic | `events` | Main event log |
| Partitions | 8 | Parallel processing by session |

Partition assignment uses consistent hashing on `session_id`, ensuring all events for a session go to the same partition (maintaining per-session ordering).

---

## Implementation Details

### IggyEventLog Structure

```rust
use iggy::clients::client::IggyClient;

/// Stream and topic configuration.
pub mod topics {
    pub const STREAM_NAME: &str = "vibes";
    pub const STREAM_ID: u32 = 1;
    pub const EVENTS_TOPIC: &str = "events";
    pub const TOPIC_ID: u32 = 1;
    pub const PARTITION_COUNT: u32 = 8;
}

/// Maximum events to buffer during disconnect before dropping oldest.
const MAX_RECONNECT_BUFFER: usize = 10_000;

/// Iggy-backed EventLog implementation for VibesEvent.
pub struct IggyEventLog {
    /// Reference to the Iggy manager (for connection info).
    manager: Arc<IggyManager>,

    /// The Iggy client for sending/receiving messages.
    client: Arc<IggyClient>,

    /// Buffer for events during disconnect.
    /// Events are flushed when connection is restored.
    reconnect_buffer: RwLock<Vec<VibesEvent>>,

    /// Current high water mark (local offset counter).
    high_water_mark: AtomicU64,

    /// Whether we're connected to Iggy.
    connected: RwLock<bool>,
}
```

### connect() Implementation

```rust
impl IggyEventLog {
    pub async fn connect(&self) -> Result<()> {
        // 1. Connect to Iggy server
        self.client.connect().await?;

        // 2. Login with default credentials
        self.client
            .login_user(DEFAULT_ROOT_USERNAME, DEFAULT_ROOT_PASSWORD)
            .await?;

        // 3. Create stream if not exists
        match self.client
            .create_stream(topics::STREAM_NAME, Some(topics::STREAM_ID))
            .await
        {
            Ok(_) => info!("Created stream '{}'", topics::STREAM_NAME),
            Err(e) if is_already_exists(&e) => {
                debug!("Stream '{}' already exists", topics::STREAM_NAME);
            }
            Err(e) => return Err(e.into()),
        }

        // 4. Create topic with 8 partitions if not exists
        match self.client
            .create_topic(
                &topics::STREAM_ID.try_into()?,
                topics::EVENTS_TOPIC,
                topics::PARTITION_COUNT,
                CompressionAlgorithm::None,
                None, // replication factor
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
            Err(e) if is_already_exists(&e) => {
                debug!("Topic '{}' already exists", topics::EVENTS_TOPIC);
            }
            Err(e) => return Err(e.into()),
        }

        // 5. Mark as connected
        *self.connected.write().await = true;
        info!("Connected to Iggy server");

        Ok(())
    }
}
```

### append() Implementation

```rust
impl EventLog<VibesEvent> for IggyEventLog {
    async fn append(&self, event: VibesEvent) -> Result<Offset> {
        match self.try_send(&event).await {
            Ok(offset) => Ok(offset),
            Err(e) if e.is_connection_error() => {
                // Buffer event for later
                self.buffer_event(event).await;
                // Trigger background reconnect
                self.trigger_reconnect();
                // Return synthetic offset
                Ok(self.high_water_mark.fetch_add(1, Ordering::SeqCst))
            }
            Err(e) => Err(e),
        }
    }
}

impl IggyEventLog {
    async fn try_send(&self, event: &VibesEvent) -> Result<Offset> {
        // 1. Extract session_id for partitioning
        let session_id = event.session_id().unwrap_or("unknown");

        // 2. Serialize to JSON
        let payload = serde_json::to_vec(event)?;

        // 3. Create Iggy message
        let message = IggyMessage::from_bytes(payload.into())?;

        // 4. Use session_id as partition key (consistent hashing)
        let partitioning = Partitioning::messages_key_str(session_id);

        // 5. Send to Iggy
        self.client
            .send_messages(
                &topics::STREAM_ID.try_into()?,
                &topics::TOPIC_ID.try_into()?,
                &partitioning,
                &mut vec![message],
            )
            .await?;

        // 6. Increment and return offset
        Ok(self.high_water_mark.fetch_add(1, Ordering::SeqCst))
    }
}
```

### Consumer Implementation

```rust
pub struct IggyEventConsumer {
    client: Arc<IggyClient>,
    group: String,

    /// Current offset per partition.
    offsets: [u64; 8],

    /// Last committed offset per partition.
    committed_offsets: [u64; 8],
}

impl EventConsumer<VibesEvent> for IggyEventConsumer {
    async fn poll(&mut self, max_count: usize, timeout: Duration) -> Result<EventBatch<VibesEvent>> {
        let mut all_events = Vec::new();
        let per_partition = max_count / 8 + 1;

        // Poll each partition
        for partition_id in 0..8u32 {
            let polled = self.client
                .poll_messages(
                    &topics::STREAM_ID.try_into()?,
                    &topics::TOPIC_ID.try_into()?,
                    Some(partition_id),
                    &Consumer::new(Identifier::named(&self.group)?),
                    &PollingStrategy::offset(self.offsets[partition_id as usize]),
                    per_partition as u32,
                    false, // auto_commit = false (manual commit)
                )
                .await?;

            for msg in polled.messages {
                let event: VibesEvent = serde_json::from_slice(&msg.payload)?;
                all_events.push((msg.offset, event));
            }

            // Update partition offset
            if let Some(last) = polled.messages.last() {
                self.offsets[partition_id as usize] = last.offset + 1;
            }
        }

        // Sort by offset for rough ordering (best effort across partitions)
        all_events.sort_by_key(|(offset, _)| *offset);

        Ok(EventBatch::new(all_events))
    }

    async fn commit(&mut self, _offset: Offset) -> Result<()> {
        // Store current offsets using Iggy consumer group API
        for partition_id in 0..8u32 {
            self.client
                .store_consumer_offset(
                    &Consumer::new(Identifier::named(&self.group)?),
                    &topics::STREAM_ID.try_into()?,
                    &topics::TOPIC_ID.try_into()?,
                    Some(partition_id),
                    self.offsets[partition_id as usize],
                )
                .await?;

            self.committed_offsets[partition_id as usize] =
                self.offsets[partition_id as usize];
        }

        debug!(group = %self.group, "Committed offsets");
        Ok(())
    }

    async fn seek(&mut self, position: SeekPosition) -> Result<()> {
        match position {
            SeekPosition::Beginning => {
                self.offsets = [0; 8];
            }
            SeekPosition::End => {
                // Query high water mark for each partition
                for partition_id in 0..8u32 {
                    // Use get_topic to get partition info
                    // Set offset to partition's current_offset
                }
            }
            SeekPosition::Offset(o) => {
                self.offsets = [o; 8];
            }
        }
        Ok(())
    }
}
```

---

## Error Handling & Reconnection

### Reconnect Buffer Behavior

When the Iggy connection is lost:

1. **On append failure**: Event is buffered in `reconnect_buffer`
2. **Buffer limit**: Maximum 10,000 events
3. **Buffer overflow**: Oldest events dropped with warning log
4. **Reconnect trigger**: Background task attempts reconnect with 1s delay
5. **On reconnect success**: Buffer is flushed in order
6. **Producer behavior**: `append()` returns immediately (non-blocking)

```rust
impl IggyEventLog {
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

    async fn flush_buffer(&self) -> Result<()> {
        let events = std::mem::take(&mut *self.reconnect_buffer.write().await);

        if !events.is_empty() {
            info!(count = events.len(), "Flushing reconnect buffer to Iggy");

            for event in events {
                self.try_send(&event).await?;
            }
        }

        Ok(())
    }

    fn trigger_reconnect(&self) {
        let client = Arc::clone(&self.client);
        let this = /* weak reference or channel */;

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;

                match client.connect().await {
                    Ok(_) => {
                        info!("Reconnected to Iggy server");
                        // Login and flush buffer
                        if let Err(e) = this.flush_buffer().await {
                            error!(error = %e, "Failed to flush buffer after reconnect");
                        }
                        break;
                    }
                    Err(e) => {
                        warn!(error = %e, "Reconnect attempt failed, retrying...");
                    }
                }
            }
        });
    }
}
```

**Important**: This behavior ensures producers are never blocked by Iggy issues, while providing best-effort persistence during transient failures.

---

## Testing Strategy

### Test Matrix

| Test Type | Backend | What it verifies |
|-----------|---------|------------------|
| **Unit tests** | Mock/None | Serialization, offset logic, buffer behavior |
| **Consumer unit tests** | `InMemoryEventLog` | Consumer logic (WebSocket, Notification, etc.) |
| **Integration tests** | Real Iggy | `IggyEventLog` ↔ Iggy server round-trip |
| **E2E tests** | Real Iggy | Full stack: HTTP API → EventLog → Iggy → Consumers |

### Unit Tests (No Iggy Required)

```rust
#[test]
fn test_session_id_extraction() {
    let event = VibesEvent::SessionCreated {
        session_id: "sess-123".into(),
        name: None,
    };
    assert_eq!(event.session_id(), Some("sess-123"));
}

#[test]
fn test_buffer_overflow_drops_oldest() {
    // Create buffer at max capacity
    // Add one more event
    // Verify oldest was dropped
}

#[test]
fn test_event_serialization_roundtrip() {
    let event = VibesEvent::Claude { ... };
    let json = serde_json::to_vec(&event).unwrap();
    let recovered: VibesEvent = serde_json::from_slice(&json).unwrap();
    assert_eq!(event, recovered);
}
```

### Integration Tests (Real Iggy)

```rust
#[tokio::test]
#[ignore] // Requires running Iggy server
async fn test_append_and_poll_roundtrip() {
    let log = setup_iggy_event_log().await;

    let event = VibesEvent::SessionCreated {
        session_id: "test-session".into(),
        name: Some("Test".into()),
    };

    log.append(event.clone()).await.unwrap();

    let mut consumer = log.consumer("test-consumer").await.unwrap();
    let batch = consumer.poll(10, Duration::from_secs(1)).await.unwrap();

    assert_eq!(batch.len(), 1);
    assert_eq!(batch.events[0].1, event);
}

#[tokio::test]
#[ignore]
async fn test_consumer_offset_persistence() {
    let log = setup_iggy_event_log().await;

    // Append events
    for i in 0..5 {
        log.append(make_event(i)).await.unwrap();
    }

    // Poll and commit
    let mut consumer = log.consumer("persistent-group").await.unwrap();
    let batch = consumer.poll(3, Duration::from_secs(1)).await.unwrap();
    consumer.commit(batch.last_offset().unwrap()).await.unwrap();

    // Create new consumer with same group
    let mut consumer2 = log.consumer("persistent-group").await.unwrap();
    let batch2 = consumer2.poll(10, Duration::from_secs(1)).await.unwrap();

    // Should resume from offset 3, getting events 3 and 4
    assert_eq!(batch2.len(), 2);
}
```

### E2E Tests (Full Stack with Iggy)

E2E tests MUST use `AppState::new_with_iggy()` to verify the complete pipeline:

```rust
#[tokio::test]
#[ignore] // Requires Iggy
async fn test_e2e_event_persistence() {
    let state = AppState::new_with_iggy().await;

    // Emit event through normal API
    state.append_event(VibesEvent::SessionCreated {
        session_id: "e2e-test".into(),
        name: None,
    });

    // Wait for async processing
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify event reached Iggy by checking stats
    let stats = reqwest::get("http://localhost:3001/stats")
        .await.unwrap()
        .json::<IggyStats>().await.unwrap();

    assert!(stats.messages_count > 0, "Events should flow to Iggy");
}
```

---

## Deliverables

### Checklist

**Core Implementation:**
- [ ] Specialize `IggyEventLog` for `VibesEvent` (remove generic)
- [ ] Add `IggyClient` field to `IggyEventLog`
- [ ] Implement `connect()` with stream/topic creation
- [ ] Implement `append()` with session_id partitioning
- [ ] Implement `IggyEventConsumer` with multi-partition polling
- [ ] Implement manual offset commit
- [ ] Add reconnect buffer logic
- [ ] Add background reconnect task

**VibesEvent Changes:**
- [ ] Add `session_id()` method to `VibesEvent`

**Testing:**
- [ ] Unit tests for serialization and buffer logic
- [ ] Integration tests with real Iggy
- [ ] Update E2E tests to use real Iggy backend
- [ ] Add test for reconnection scenario

**Documentation:**
- [ ] Update PROGRESS.md
- [ ] Document reconnect buffer behavior in code comments

---

## Dependencies

This milestone requires:
- Milestone 16 (Iggy Bundling) - Complete
- `iggy = "0.6"` crate (already in Cargo.toml)

No new crate dependencies required.
