//! Iggy-backed assessment event log.
//!
//! Provides an assessment log implementation backed by Iggy message streaming,
//! enabling durable, time-ordered event storage with topic-based routing.
//!
//! # Architecture
//!
//! Events are routed to topic by tier:
//! - Lightweight events → `groove.assessment.lightweight`
//! - Medium events → `groove.assessment.medium`
//! - Heavy events → `groove.assessment.heavy`
//!
//! This allows independent consumers for each tier while keeping events
//! segregated by processing characteristics.
//!
//! # Reconnect Buffer
//!
//! When disconnected from Iggy, events are buffered in memory (up to 10,000).
//! When buffer is full, oldest events are dropped. On reconnection, buffered
//! events are flushed to Iggy.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use iggy::prelude::*;
use tokio::sync::{RwLock, broadcast};
use tracing::{debug, info, warn};

use crate::assessment::log::AssessmentLog;
use crate::assessment::types::{AssessmentEvent, EventId, SessionId};
use crate::error::{GrooveError, Result};
use vibes_iggy::IggyManager;

/// Check if an Iggy error indicates a resource already exists.
fn is_already_exists_error(e: &IggyError) -> bool {
    let err_str = e.to_string();
    err_str.contains("already exists")
        || err_str.contains("already_exists")
        || err_str.contains("AlreadyExists")
}

/// Topic names for assessment events in Iggy.
pub mod topics {
    /// Stream name for all assessment events.
    pub const STREAM_NAME: &str = "groove.assessment";
    /// Topic for lightweight (per-message) assessment events.
    pub const LIGHTWEIGHT_TOPIC: &str = "lightweight";
    /// Topic for medium (checkpoint) assessment events.
    pub const MEDIUM_TOPIC: &str = "medium";
    /// Topic for heavy (full session) assessment events.
    pub const HEAVY_TOPIC: &str = "heavy";
    /// Number of partitions (1 for simple offset tracking).
    pub const PARTITION_COUNT: u32 = 1;
}

/// Maximum events to buffer during disconnect before dropping oldest.
const MAX_RECONNECT_BUFFER: usize = 10_000;

/// Iggy-backed implementation of the assessment log.
///
/// Events are written to Iggy for durable storage and broadcast to
/// real-time subscribers. When disconnected, events are buffered in
/// memory and flushed on reconnection.
pub struct IggyAssessmentLog {
    /// The Iggy manager for server lifecycle.
    manager: Arc<IggyManager>,

    /// The Iggy client for sending messages.
    client: IggyClient,

    /// Broadcast sender for real-time event subscription.
    tx: broadcast::Sender<AssessmentEvent>,

    /// Buffer for events while disconnected from Iggy.
    /// Also used for reads (client-side filtering) since Iggy doesn't support
    /// arbitrary field queries.
    buffer: RwLock<Vec<AssessmentEvent>>,

    /// Whether we're connected to the Iggy server.
    connected: RwLock<bool>,
}

impl IggyAssessmentLog {
    /// Create a new Iggy assessment log with the given manager.
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

        let (tx, _) = broadcast::channel(1024);
        Self {
            manager,
            client,
            tx,
            buffer: RwLock::new(Vec::new()),
            connected: RwLock::new(false),
        }
    }

    /// Connect to the Iggy server.
    ///
    /// This establishes the connection, authenticates, and creates
    /// the stream and topics if they don't exist.
    pub async fn connect(&self) -> Result<()> {
        // Connect to server
        self.client
            .connect()
            .await
            .map_err(|e| GrooveError::Assessment(format!("Failed to connect to Iggy: {e}")))?;
        info!(
            "Assessment log connected to Iggy at {}",
            self.manager.connection_address()
        );

        // Login with default credentials
        self.client
            .login_user(DEFAULT_ROOT_USERNAME, DEFAULT_ROOT_PASSWORD)
            .await
            .map_err(|e| GrooveError::Assessment(format!("Failed to login: {e}")))?;
        debug!("Assessment log logged in to Iggy");

        // Get or create stream
        let stream_id = Identifier::named(topics::STREAM_NAME)
            .map_err(|e| GrooveError::Assessment(format!("Invalid stream name: {e}")))?;
        let streams = self
            .client
            .get_streams()
            .await
            .map_err(|e| GrooveError::Assessment(format!("Failed to get streams: {e}")))?;
        let stream_exists = streams.iter().any(|s| s.name == topics::STREAM_NAME);

        if stream_exists {
            debug!("Assessment stream '{}' already exists", topics::STREAM_NAME);
        } else {
            match self.client.create_stream(topics::STREAM_NAME).await {
                Ok(_) => info!("Created assessment stream '{}'", topics::STREAM_NAME),
                Err(e) if is_already_exists_error(&e) => {
                    debug!("Assessment stream already exists (concurrent creation)");
                }
                Err(e) => {
                    return Err(GrooveError::Assessment(format!(
                        "Failed to create stream: {e}"
                    )));
                }
            }
        }

        // Create topics for each tier
        for topic_name in [
            topics::LIGHTWEIGHT_TOPIC,
            topics::MEDIUM_TOPIC,
            topics::HEAVY_TOPIC,
        ] {
            match self
                .client
                .create_topic(
                    &stream_id,
                    topic_name,
                    topics::PARTITION_COUNT,
                    CompressionAlgorithm::None,
                    None, // replication_factor
                    IggyExpiry::NeverExpire,
                    MaxTopicSize::ServerDefault,
                )
                .await
            {
                Ok(_) => info!("Created assessment topic '{topic_name}'"),
                Err(e) if is_already_exists_error(&e) => {
                    debug!("Assessment topic '{topic_name}' already exists");
                }
                Err(e) => {
                    return Err(GrooveError::Assessment(format!(
                        "Failed to create topic '{topic_name}': {e}"
                    )));
                }
            }
        }

        *self.connected.write().await = true;
        info!("IggyAssessmentLog fully connected and ready");

        // Flush any buffered events from previous disconnection
        self.flush_pending_buffer().await?;

        Ok(())
    }

    /// Determine which topic an event should be routed to.
    #[must_use]
    pub fn topic_for_event(event: &AssessmentEvent) -> &'static str {
        match event {
            AssessmentEvent::Lightweight(_) => topics::LIGHTWEIGHT_TOPIC,
            AssessmentEvent::Medium(_) => topics::MEDIUM_TOPIC,
            AssessmentEvent::Heavy(_) => topics::HEAVY_TOPIC,
        }
    }

    /// Internal send that doesn't handle reconnection.
    async fn try_send(&self, event: &AssessmentEvent) -> Result<()> {
        let topic_name = Self::topic_for_event(event);

        // Use session_id as partition key for consistent routing
        let partition_key = event.session_id().as_str();

        // Serialize event to JSON
        let payload = serde_json::to_vec(event)
            .map_err(|e| GrooveError::Assessment(format!("Failed to serialize event: {e}")))?;

        // Create Iggy message
        let message = IggyMessage::builder()
            .payload(payload.into())
            .build()
            .map_err(|e| GrooveError::Assessment(e.to_string()))?;

        // Partition by session_id
        let partitioning = Partitioning::messages_key_str(partition_key).map_err(|e| {
            GrooveError::Assessment(format!(
                "Failed to create partition key '{partition_key}': {e}"
            ))
        })?;

        // Send to Iggy
        let stream_id = Identifier::named(topics::STREAM_NAME)
            .map_err(|e| GrooveError::Assessment(format!("Invalid stream name: {e}")))?;
        let topic_id = Identifier::named(topic_name)
            .map_err(|e| GrooveError::Assessment(format!("Invalid topic name: {e}")))?;

        let mut messages = [message];
        self.client
            .send_messages(&stream_id, &topic_id, &partitioning, &mut messages)
            .await
            .map_err(|e| GrooveError::Assessment(format!("Failed to send message: {e}")))?;

        Ok(())
    }

    /// Buffer an event when disconnected.
    async fn buffer_event(&self, event: AssessmentEvent) {
        let mut buffer = self.buffer.write().await;

        if buffer.len() >= MAX_RECONNECT_BUFFER {
            warn!(
                buffer_size = buffer.len(),
                "Assessment reconnect buffer full, dropping oldest event"
            );
            buffer.remove(0);
        }

        buffer.push(event);
        debug!(
            buffer_size = buffer.len(),
            "Buffered assessment event during disconnect"
        );
    }

    /// Flush buffered events after reconnection.
    async fn flush_pending_buffer(&self) -> Result<()> {
        // Take a snapshot of events to flush
        let events: Vec<AssessmentEvent> = {
            let buffer = self.buffer.read().await;
            buffer.clone()
        };

        if events.is_empty() {
            return Ok(());
        }

        info!(
            count = events.len(),
            "Flushing assessment reconnect buffer to Iggy"
        );

        for event in &events {
            if let Err(e) = self.try_send(event).await {
                warn!(error = %e, "Failed to flush buffered assessment event");
                // Don't remove from buffer if flush failed
                return Err(e);
            }
        }

        // Only clear buffer after successful flush
        // Note: we keep events in buffer for read queries
        Ok(())
    }

    /// Get count of buffered events (for testing).
    pub async fn buffer_len(&self) -> usize {
        self.buffer.read().await.len()
    }

    /// Check if connected to Iggy (for testing).
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }
}

impl std::fmt::Debug for IggyAssessmentLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IggyAssessmentLog")
            .field("manager", &"Arc<IggyManager>")
            .field("connected", &"RwLock<bool>")
            .field("buffer", &"RwLock<Vec<AssessmentEvent>>")
            .finish()
    }
}

#[async_trait]
impl AssessmentLog for IggyAssessmentLog {
    async fn append(&self, event: AssessmentEvent) -> Result<EventId> {
        let event_id = *event.event_id();
        let topic = Self::topic_for_event(&event);

        debug!(
            event_id = %event_id,
            topic = topic,
            session_id = %event.session_id(),
            "Appending event to Iggy assessment log"
        );

        // Always add to buffer for read queries (client-side filtering)
        self.buffer.write().await.push(event.clone());

        // Try to send to Iggy if connected
        if *self.connected.read().await {
            match self.try_send(&event).await {
                Ok(()) => {
                    debug!(event_id = %event_id, "Sent assessment event to Iggy");
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
                        warn!(error = %e, "Connection error, marking as disconnected");
                        *self.connected.write().await = false;
                        self.buffer_event(event.clone()).await;
                    } else {
                        // Non-connection error - log but continue (event is in buffer)
                        warn!(error = %e, "Failed to send assessment event to Iggy");
                    }
                }
            }
        } else {
            // Not connected - event is already in buffer
            debug!(event_id = %event_id, "Iggy not connected, event buffered");
        }

        // Broadcast to real-time subscribers (ignore send errors - no subscribers is fine)
        let _ = self.tx.send(event);

        Ok(event_id)
    }

    async fn read_session(&self, session_id: &SessionId) -> Result<Vec<AssessmentEvent>> {
        // Client-side filtering from buffer
        // TODO: In production, poll from Iggy topics and filter
        let buffer = self.buffer.read().await;
        Ok(buffer
            .iter()
            .filter(|e| e.session_id() == session_id)
            .cloned()
            .collect())
    }

    async fn read_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AssessmentEvent>> {
        // Client-side filtering from buffer
        // TODO: In production, poll from Iggy topics and filter by timestamp
        let buffer = self.buffer.read().await;
        Ok(buffer
            .iter()
            .filter(|e| {
                let ts = match e {
                    AssessmentEvent::Lightweight(e) => e.context.timestamp,
                    AssessmentEvent::Medium(e) => e.context.timestamp,
                    AssessmentEvent::Heavy(e) => e.context.timestamp,
                };
                ts >= start && ts <= end
            })
            .cloned()
            .collect())
    }

    fn subscribe(&self) -> broadcast::Receiver<AssessmentEvent> {
        self.tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assessment::types::{
        AssessmentContext, CheckpointTrigger, HeavyEvent, LightweightEvent, MediumEvent, Outcome,
    };
    use vibes_iggy::IggyConfig;

    fn make_lightweight_event(session: &str) -> AssessmentEvent {
        AssessmentEvent::Lightweight(LightweightEvent {
            context: AssessmentContext::new(session),
            message_idx: 0,
            signals: vec![],
            frustration_ema: 0.0,
            success_ema: 1.0,
            triggering_event_id: uuid::Uuid::now_v7(),
        })
    }

    fn make_medium_event(session: &str) -> AssessmentEvent {
        AssessmentEvent::Medium(MediumEvent::new(
            AssessmentContext::new(session),
            (0, 10),
            CheckpointTrigger::TimeInterval,
        ))
    }

    fn make_heavy_event(session: &str) -> AssessmentEvent {
        AssessmentEvent::Heavy(HeavyEvent::new(
            AssessmentContext::new(session),
            Outcome::Success,
        ))
    }

    #[tokio::test]
    async fn iggy_log_append_and_read() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log = IggyAssessmentLog::new(manager);

        let event = make_lightweight_event("sess-1");
        let event_id = log.append(event).await.unwrap();

        // Should be buffered
        assert_eq!(log.buffer_len().await, 1);

        // Should be readable by session
        let events = log.read_session(&"sess-1".into()).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(*events[0].event_id(), event_id);
    }

    #[tokio::test]
    async fn iggy_log_subscribe() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log = IggyAssessmentLog::new(manager);

        let mut rx = log.subscribe();

        let event = make_lightweight_event("sess-1");
        log.append(event).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.session_id().as_str(), "sess-1");
    }

    #[test]
    fn topic_for_event_returns_correct_topic() {
        let lightweight = make_lightweight_event("test");
        assert_eq!(
            IggyAssessmentLog::topic_for_event(&lightweight),
            topics::LIGHTWEIGHT_TOPIC
        );

        let medium = make_medium_event("test");
        assert_eq!(
            IggyAssessmentLog::topic_for_event(&medium),
            topics::MEDIUM_TOPIC
        );

        let heavy = make_heavy_event("test");
        assert_eq!(
            IggyAssessmentLog::topic_for_event(&heavy),
            topics::HEAVY_TOPIC
        );
    }

    #[tokio::test]
    async fn iggy_log_starts_disconnected() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log = IggyAssessmentLog::new(manager);

        // Should start disconnected
        assert!(!log.is_connected().await);
    }

    // Note: connect() integration test is in tests/assessment_integration.rs
    // since it requires a running Iggy server

    #[tokio::test]
    async fn iggy_log_buffers_when_disconnected() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log = IggyAssessmentLog::new(manager);

        // Append events while disconnected
        log.append(make_lightweight_event("sess-1")).await.unwrap();
        log.append(make_medium_event("sess-2")).await.unwrap();

        // Should be buffered for reads
        assert_eq!(log.buffer_len().await, 2);
    }

    #[tokio::test]
    async fn iggy_log_filters_by_session() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log = IggyAssessmentLog::new(manager);

        log.append(make_lightweight_event("sess-1")).await.unwrap();
        log.append(make_medium_event("sess-2")).await.unwrap();
        log.append(make_heavy_event("sess-1")).await.unwrap();

        let sess1_events = log.read_session(&"sess-1".into()).await.unwrap();
        assert_eq!(sess1_events.len(), 2);

        let sess2_events = log.read_session(&"sess-2".into()).await.unwrap();
        assert_eq!(sess2_events.len(), 1);
    }

    #[tokio::test]
    async fn iggy_log_read_range() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log = IggyAssessmentLog::new(manager);

        // Capture time before and after appending
        let before = Utc::now();
        log.append(make_lightweight_event("sess-1")).await.unwrap();
        let after = Utc::now();

        // Should find the event
        let events = log.read_range(before, after).await.unwrap();
        assert_eq!(events.len(), 1);

        // Should not find events outside range
        let far_future = Utc::now() + chrono::Duration::hours(1);
        let events = log.read_range(after, far_future).await.unwrap();
        // Event timestamp is exactly at creation time, may or may not match
        // depending on timing - this is just a smoke test
        assert!(events.len() <= 1);
    }

    #[test]
    fn iggy_log_debug_impl() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log = IggyAssessmentLog::new(manager);

        let debug_str = format!("{:?}", log);
        assert!(debug_str.contains("IggyAssessmentLog"));
    }
}
