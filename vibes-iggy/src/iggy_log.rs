//! Iggy-backed EventLog implementation.
//!
//! This module provides persistent event storage using Iggy as the backend.
//! Events are written to an Iggy stream/topic and consumers track their
//! offsets independently.
//!
//! # Reconnect Buffer
//!
//! When the connection to Iggy is lost, events are buffered in memory up to
//! `MAX_RECONNECT_BUFFER` (10,000 events). When the buffer is full, the oldest
//! events are dropped. When connection is restored, buffered events are flushed.
//!
//! This ensures producers are never blocked by Iggy issues while providing
//! best-effort persistence during transient failures.

use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use iggy::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, trace, warn};

use crate::error::{Error, Result};
use crate::manager::IggyManager;
use crate::traits::{EventBatch, EventConsumer, EventLog, Offset, Partitionable, SeekPosition};

/// Check if an Iggy error indicates a resource already exists.
fn is_already_exists_error(e: &IggyError) -> bool {
    let err_str = e.to_string();
    err_str.contains("already exists")
        || err_str.contains("already_exists")
        || err_str.contains("AlreadyExists")
}

/// Stream and topic configuration for the event log.
pub mod topics {
    /// The stream name for vibes events.
    pub const STREAM_NAME: &str = "vibes";
    /// The topic name for the main event log.
    pub const EVENTS_TOPIC: &str = "events";
    /// Number of partitions. Using 1 partition makes offset globally unique
    /// and simplifies seeking/pagination.
    pub const PARTITION_COUNT: u32 = 1;
}

/// Maximum events to buffer during disconnect before dropping oldest.
const MAX_RECONNECT_BUFFER: usize = 10_000;

/// Iggy-backed implementation of EventLog.
///
/// Provides persistent event storage with consumer group offset tracking.
/// Events are partitioned using the `Partitionable` trait for parallel processing.
pub struct IggyEventLog<E> {
    /// Reference to the Iggy manager (for connection info).
    manager: Arc<IggyManager>,

    /// The Iggy client for sending messages.
    client: IggyClient,

    /// Buffer for events during disconnect.
    reconnect_buffer: RwLock<Vec<E>>,

    /// Current high water mark (local offset counter).
    high_water_mark: AtomicU64,

    /// Whether we're connected to Iggy.
    connected: RwLock<bool>,

    /// Phantom data for generic type.
    _phantom: PhantomData<E>,
}

impl<E> IggyEventLog<E>
where
    E: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone + Partitionable + 'static,
{
    /// Create a new IggyEventLog.
    ///
    /// The manager should be started before calling this.
    /// Call `connect()` to establish the connection.
    #[must_use]
    pub fn new(manager: Arc<IggyManager>) -> Self {
        let client = IggyClient::builder()
            .with_tcp()
            .with_server_address(manager.connection_address())
            .build()
            .expect("Failed to build Iggy client");

        Self {
            manager,
            client,
            reconnect_buffer: RwLock::new(Vec::new()),
            high_water_mark: AtomicU64::new(0),
            connected: RwLock::new(false),
            _phantom: PhantomData,
        }
    }

    /// Connect to the Iggy server.
    ///
    /// This establishes the connection, authenticates, and creates
    /// the stream/topic if they don't exist.
    pub async fn connect(&self) -> Result<()> {
        // Connect to server
        self.client.connect().await?;
        info!(
            "Connected to Iggy server at {}",
            self.manager.connection_address()
        );

        // Login with default credentials
        self.client
            .login_user(DEFAULT_ROOT_USERNAME, DEFAULT_ROOT_PASSWORD)
            .await?;
        debug!("Logged in to Iggy as root user");

        // Get or create stream
        let streams = self.client.get_streams().await?;
        let stream_id = Identifier::named(topics::STREAM_NAME)
            .map_err(|e| Error::Iggy(format!("Invalid stream name: {}", e)))?;
        let stream_exists = streams.iter().any(|s| s.name == topics::STREAM_NAME);

        if stream_exists {
            debug!("Stream '{}' already exists", topics::STREAM_NAME);
        } else {
            match self.client.create_stream(topics::STREAM_NAME).await {
                Ok(_) => info!("Created stream '{}'", topics::STREAM_NAME),
                Err(e) if is_already_exists_error(&e) => {
                    debug!("Stream already exists (concurrent creation)");
                }
                Err(e) => return Err(e.into()),
            }
        }

        // Get or create topic
        match self
            .client
            .create_topic(
                &stream_id,
                topics::EVENTS_TOPIC,
                topics::PARTITION_COUNT,
                CompressionAlgorithm::None,
                None, // replication_factor
                IggyExpiry::NeverExpire,
                MaxTopicSize::ServerDefault,
            )
            .await
        {
            Ok(_) => info!("Created topic '{}'", topics::EVENTS_TOPIC),
            Err(e) if is_already_exists_error(&e) => {
                debug!("Topic already exists");
            }
            Err(e) => return Err(e.into()),
        }

        // Query the topic to get the actual message count
        // This initializes high_water_mark correctly on server restart
        let topic_id = Identifier::named(topics::EVENTS_TOPIC)
            .map_err(|e| Error::Iggy(format!("Invalid topic name: {}", e)))?;
        if let Some(topic_details) = self.client.get_topic(&stream_id, &topic_id).await? {
            let message_count = topic_details.messages_count;
            self.high_water_mark.store(message_count, Ordering::SeqCst);
            info!(
                "Initialized high_water_mark to {} from existing topic",
                message_count
            );
        }

        *self.connected.write().await = true;
        info!("IggyEventLog fully connected and ready");

        // Flush any buffered events from previous disconnection
        self.flush_buffer().await?;

        Ok(())
    }

    /// Check if connected to Iggy.
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    /// Internal send that doesn't handle reconnection.
    async fn try_send(&self, event: &E) -> Result<()> {
        // Get partition key from the Partitionable trait
        let partition_key = event.partition_key().unwrap_or("unknown");

        // Serialize event to JSON
        let payload = serde_json::to_vec(event)?;

        // Create Iggy message using builder pattern
        let message = IggyMessage::builder()
            .payload(payload.into())
            .build()
            .map_err(|e| Error::Iggy(e.to_string()))?;

        // Partition by key (consistent hashing)
        let partitioning = Partitioning::messages_key_str(partition_key).map_err(|e| {
            Error::Iggy(format!(
                "Failed to create partition key '{}': {}",
                partition_key, e
            ))
        })?;

        // Send to Iggy
        let stream_id = Identifier::named(topics::STREAM_NAME)
            .map_err(|e| Error::Iggy(format!("Invalid stream name: {}", e)))?;
        let topic_id = Identifier::named(topics::EVENTS_TOPIC)
            .map_err(|e| Error::Iggy(format!("Invalid topic name: {}", e)))?;

        let mut messages = [message];
        self.client
            .send_messages(&stream_id, &topic_id, &partitioning, &mut messages)
            .await?;

        Ok(())
    }

    /// Buffer an event when disconnected.
    async fn buffer_event(&self, event: E) {
        let mut buffer = self.reconnect_buffer.write().await;

        if buffer.len() >= MAX_RECONNECT_BUFFER {
            warn!(
                buffer_size = buffer.len(),
                "Reconnect buffer full, dropping oldest event"
            );
            buffer.remove(0);
        }

        buffer.push(event);
        debug!(
            buffer_size = buffer.len(),
            "Buffered event during disconnect"
        );
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

    /// Flush the Iggy server's in-memory buffer to disk and sync.
    ///
    /// This is critical for read consistency when the Iggy server uses io_uring
    /// (via compio) with separate file handles for reading and writing.
    /// Without an explicit flush, writes may not be visible to readers.
    ///
    /// Call this before reading historical events to ensure all data is visible.
    pub async fn flush_to_disk(&self) -> Result<()> {
        let stream_id = Identifier::named(topics::STREAM_NAME)
            .map_err(|e| Error::Iggy(format!("Invalid stream name: {}", e)))?;
        let topic_id = Identifier::named(topics::EVENTS_TOPIC)
            .map_err(|e| Error::Iggy(format!("Invalid topic name: {}", e)))?;

        debug!("Flushing Iggy server buffer to disk with fsync");

        // Partition 0 is the only partition we use
        self.client
            .flush_unsaved_buffer(&stream_id, &topic_id, 0, true)
            .await
            .map_err(|e| Error::Iggy(format!("Failed to flush buffer: {}", e)))?;

        debug!("Iggy buffer flushed successfully");
        Ok(())
    }
}

#[async_trait]
impl<E> EventLog<E> for IggyEventLog<E>
where
    E: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone + Partitionable + 'static,
{
    async fn append(&self, event: E) -> Result<Offset> {
        match self.try_send(&event).await {
            Ok(()) => {
                let offset = self.high_water_mark.fetch_add(1, Ordering::SeqCst);
                debug!(offset, "Appended event to Iggy");
                Ok(offset)
            }
            Err(e) => {
                // Check if it's a connection error
                let err_str = e.to_string().to_lowercase();
                let is_connection_error = err_str.contains("connection")
                    || err_str.contains("disconnected")
                    || err_str.contains("not connected")
                    || err_str.contains("broken pipe")
                    || err_str.contains("reset");

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

    async fn append_batch(&self, events: Vec<E>) -> Result<Offset> {
        if events.is_empty() {
            return Ok(self.high_water_mark().saturating_sub(1));
        }

        // Send each event (Iggy handles internal batching)
        for event in events {
            self.append(event).await?;
        }

        // Return offset of last event
        Ok(self.high_water_mark().saturating_sub(1))
    }

    async fn consumer(&self, group: &str) -> Result<Box<dyn EventConsumer<E>>> {
        // Create a new client for the consumer (each consumer needs its own connection)
        let consumer_client = IggyClient::builder()
            .with_tcp()
            .with_server_address(self.manager.connection_address())
            .build()
            .map_err(|e| Error::Connection(e.to_string()))?;

        consumer_client.connect().await?;
        consumer_client
            .login_user(DEFAULT_ROOT_USERNAME, DEFAULT_ROOT_PASSWORD)
            .await?;

        Ok(Box::new(IggyEventConsumer::new(
            consumer_client,
            group.to_string(),
        )))
    }

    fn high_water_mark(&self) -> Offset {
        self.high_water_mark.load(Ordering::SeqCst)
    }

    async fn flush_to_disk(&self) -> Result<()> {
        // Call the inherent method that does the actual flush
        IggyEventLog::flush_to_disk(self).await
    }
}

/// Iggy-backed consumer implementation.
///
/// Polls events from partition 0 and tracks a single offset.
pub struct IggyEventConsumer<E> {
    client: IggyClient,
    group: String,
    /// Unique consumer ID for this instance (avoids Iggy's cached offset issue).
    consumer_id: u32,
    /// Current read position in partition 0.
    offset: u64,
    /// Last committed offset.
    committed_offset: u64,
    /// Whether this is the first poll (use PollingStrategy::first() to bypass caching).
    first_poll: bool,
    _phantom: PhantomData<E>,
}

/// Generate a unique consumer ID for each instance.
/// Uses high-resolution timestamp + counter to ensure uniqueness.
/// Iggy persists consumer offsets by ID, so we must never reuse IDs across restarts.
fn generate_unique_consumer_id() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    // Get high-resolution timestamp (microseconds since epoch, truncated to u32)
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u32)
        .unwrap_or(0);

    // Increment counter for this process
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);

    // Mix timestamp and counter to generate unique ID
    // XOR with counter shifted to avoid simple collisions
    timestamp ^ (count.wrapping_mul(2654435761)) // Golden ratio hash
}

impl<E> IggyEventConsumer<E>
where
    E: for<'de> Deserialize<'de> + Send + Clone + 'static,
{
    fn new(client: IggyClient, group: String) -> Self {
        // Generate a unique consumer ID per instance to avoid Iggy's cached offset issue.
        // Even explicit PollingStrategy::offset(N) doesn't override cached offsets for
        // consumer IDs that have stored offsets in Iggy.
        let consumer_id = generate_unique_consumer_id();
        Self {
            client,
            group,
            consumer_id,
            offset: 0,
            committed_offset: 0,
            first_poll: true,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<E> EventConsumer<E> for IggyEventConsumer<E>
where
    E: for<'de> Deserialize<'de> + Send + Clone + 'static,
{
    async fn poll(&mut self, max_count: usize, _timeout: Duration) -> Result<EventBatch<E>> {
        let poll_offset = self.offset; // Capture the offset we're about to use
        trace!(
            group = %self.group,
            consumer_id = self.consumer_id,
            poll_offset,
            max_count,
            "Polling: about to request from offset"
        );
        let stream_id = Identifier::named(topics::STREAM_NAME)
            .map_err(|e| Error::Iggy(format!("Invalid stream name: {}", e)))?;
        let topic_id = Identifier::named(topics::EVENTS_TOPIC)
            .map_err(|e| Error::Iggy(format!("Invalid topic name: {}", e)))?;
        // Use a unique numeric consumer ID per instance to avoid Iggy's cached offset issue.
        // Even PollingStrategy::first() doesn't override cached offsets for shared consumer IDs.
        let consumer =
            Consumer::new(Identifier::numeric(self.consumer_id).expect("valid consumer ID"));
        // Use explicit offset tracking - we maintain self.offset locally and advance it
        // after each batch. Use PollingStrategy::offset() with our explicit offset.
        // Always use explicit offset to bypass Iggy's consumer offset caching.
        let strategy = PollingStrategy::offset(poll_offset);

        // Clear first_poll flag (we track offset ourselves, don't need special first poll handling)
        self.first_poll = false;

        let polled = match self
            .client
            .poll_messages(
                &stream_id,
                &topic_id,
                Some(0), // partition 0
                &consumer,
                &strategy,
                max_count as u32,
                false, // auto_commit = false (manual commit)
            )
            .await
        {
            Ok(messages) => messages,
            Err(e) => {
                // Handle invalid offset errors (e.g., messages were purged)
                let err_str = e.to_string().to_lowercase();
                if err_str.contains("offset")
                    || err_str.contains("not found")
                    || err_str.contains("invalid")
                {
                    warn!(
                        offset = self.offset,
                        error = %e,
                        "Invalid offset, resetting to beginning"
                    );
                    self.offset = 0;
                    return Ok(EventBatch::new(Vec::new()));
                }
                return Err(e.into());
            }
        };

        let offsets: Vec<u64> = polled.messages.iter().map(|m| m.header.offset).collect();
        let first_offset = offsets.first().copied();
        let last_offset = offsets.last().copied();
        trace!(
            messages_received = polled.messages.len(),
            ?first_offset,
            ?last_offset,
            requested_offset = poll_offset,
            "Iggy poll_messages returned"
        );
        let mut events = Vec::with_capacity(polled.messages.len());
        for msg in polled.messages {
            let event: E = serde_json::from_slice(&msg.payload)?;
            events.push((msg.header.offset, event));
            self.offset = msg.header.offset + 1;
        }

        Ok(EventBatch::new(events))
    }

    async fn commit(&mut self, _offset: Offset) -> Result<()> {
        let stream_id = Identifier::named(topics::STREAM_NAME)
            .map_err(|e| Error::Iggy(format!("Invalid stream name: {}", e)))?;
        let topic_id = Identifier::named(topics::EVENTS_TOPIC)
            .map_err(|e| Error::Iggy(format!("Invalid topic name: {}", e)))?;
        let consumer =
            Consumer::new(Identifier::named(&self.group).map_err(|e| Error::Iggy(e.to_string()))?);

        self.client
            .store_consumer_offset(&consumer, &stream_id, &topic_id, Some(0), self.offset)
            .await?;

        self.committed_offset = self.offset;
        debug!(group = %self.group, offset = self.offset, "Committed offset to Iggy");
        Ok(())
    }

    async fn seek(&mut self, position: SeekPosition) -> Result<()> {
        let stream_id = Identifier::named(topics::STREAM_NAME)
            .map_err(|e| Error::Iggy(format!("Invalid stream name: {}", e)))?;
        let topic_id = Identifier::named(topics::EVENTS_TOPIC)
            .map_err(|e| Error::Iggy(format!("Invalid topic name: {}", e)))?;

        match position {
            SeekPosition::Beginning => {
                self.offset = 0;
            }
            SeekPosition::End => {
                // Query topic to get current offset, then set to end
                if let Some(topic_details) = self.client.get_topic(&stream_id, &topic_id).await? {
                    if let Some(partition) = topic_details.partitions.first() {
                        if topic_details.messages_count == 0 {
                            // Topic is empty - start from 0 to catch the first message
                            // Without this check, we'd set offset=1 and miss offset=0
                            self.offset = 0;
                        } else {
                            // Topic has messages - start from one past the last
                            self.offset = partition.current_offset.saturating_add(1);
                        }
                    } else {
                        self.offset = 0;
                    }
                } else {
                    self.offset = 0;
                }
            }
            SeekPosition::Offset(o) => {
                self.offset = o;
            }
            SeekPosition::FromEnd(n) => {
                // Query topic to get current offset
                if let Some(topic_details) = self.client.get_topic(&stream_id, &topic_id).await? {
                    if let Some(partition) = topic_details.partitions.first() {
                        debug!(
                            current_offset = partition.current_offset,
                            messages_count = topic_details.messages_count,
                            n,
                            "FromEnd seek: partition state"
                        );
                        // current_offset is the last written offset, so add 1 to get count,
                        // then subtract n to get start position
                        self.offset = (partition.current_offset + 1).saturating_sub(n);
                        debug!(
                            calculated_offset = self.offset,
                            "FromEnd seek: calculated start offset"
                        );
                    } else {
                        self.offset = 0;
                    }
                } else {
                    self.offset = 0;
                }
            }
        }

        // We track offsets locally with self.offset and use explicit
        // PollingStrategy::offset(self.offset) in poll(), so we don't need to
        // store consumer offset on the server.
        debug!(group = %self.group, offset = self.offset, "Seeked consumer to local offset");
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
    use crate::config::IggyConfig;
    use serde::{Deserialize, Serialize};

    /// Test event type that implements Partitionable.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestEvent {
        id: String,
        data: String,
    }

    impl Partitionable for TestEvent {
        fn partition_key(&self) -> Option<&str> {
            Some(&self.id)
        }
    }

    // Test helpers - only available in tests
    impl<E> IggyEventLog<E>
    where
        E: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone + Partitionable + 'static,
    {
        /// Get the current reconnect buffer length (test only).
        async fn buffer_len(&self) -> usize {
            self.reconnect_buffer.read().await.len()
        }

        /// Get the first event in the buffer (test only).
        async fn buffer_first(&self) -> Option<E> {
            self.reconnect_buffer.read().await.first().cloned()
        }
    }

    #[tokio::test]
    async fn test_buffer_overflow_drops_oldest() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log: IggyEventLog<TestEvent> = IggyEventLog::new(manager);

        // Buffer MAX_RECONNECT_BUFFER + 1 events
        for i in 0..=MAX_RECONNECT_BUFFER {
            let event = TestEvent {
                id: format!("id-{}", i),
                data: format!("data-{}", i),
            };
            log.buffer_event(event).await;
        }

        // Buffer should be at max capacity
        assert_eq!(log.buffer_len().await, MAX_RECONNECT_BUFFER);

        // First event (id-0) should have been dropped, id-1 should be first
        let first = log
            .buffer_first()
            .await
            .expect("buffer should not be empty");
        assert_eq!(first.id, "id-1", "oldest event should have been dropped");
    }

    #[test]
    fn test_partition_count_is_one() {
        // Single partition for globally unique offsets
        assert_eq!(topics::PARTITION_COUNT, 1);
    }

    #[tokio::test]
    async fn test_iggy_log_new_creates_instance() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log: IggyEventLog<TestEvent> = IggyEventLog::new(manager);

        assert!(!log.is_connected().await);
        assert_eq!(log.high_water_mark(), 0);
    }

    // Note: Integration tests for IggyEventLog (connect, append, poll) are in
    // vibes-iggy/tests/integration.rs which properly starts the Iggy server.
}
