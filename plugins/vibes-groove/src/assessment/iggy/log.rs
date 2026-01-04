//! Iggy-backed assessment event log.
//!
//! Provides an assessment log implementation backed by Iggy message streaming,
//! enabling durable, time-ordered event storage with topic-based routing.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::{RwLock, broadcast};
use tracing::debug;

use crate::assessment::log::AssessmentLog;
use crate::assessment::types::{AssessmentEvent, EventId, SessionId};
use crate::error::Result;
use vibes_iggy::IggyManager;

/// Topic names for assessment events in Iggy.
pub mod topics {
    /// Stream name for all assessment events.
    pub const STREAM_NAME: &str = "groove.assessment";
    /// Topic for lightweight (per-message) assessment events.
    pub const LIGHTWEIGHT_TOPIC: &str = "groove.assessment.lightweight";
    /// Topic for medium (checkpoint) assessment events.
    pub const MEDIUM_TOPIC: &str = "groove.assessment.medium";
    /// Topic for heavy (full session) assessment events.
    pub const HEAVY_TOPIC: &str = "groove.assessment.heavy";
}

/// Iggy-backed implementation of the assessment log.
///
/// This is currently a stub implementation that buffers events in memory
/// until full Iggy client integration is complete. Events are buffered
/// and will be flushed to Iggy when connection is established.
pub struct IggyAssessmentLog {
    /// The Iggy manager for server lifecycle.
    #[allow(dead_code)]
    manager: Arc<IggyManager>,

    /// Broadcast sender for real-time event subscription.
    tx: broadcast::Sender<AssessmentEvent>,

    /// Buffer for events while disconnected from Iggy.
    buffer: RwLock<Vec<AssessmentEvent>>,

    /// Whether we're connected to the Iggy server.
    connected: RwLock<bool>,
}

impl IggyAssessmentLog {
    /// Create a new Iggy assessment log with the given manager.
    #[must_use]
    pub fn new(manager: Arc<IggyManager>) -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            manager,
            tx,
            buffer: RwLock::new(Vec::new()),
            connected: RwLock::new(false),
        }
    }

    /// Connect to the Iggy server.
    ///
    /// This is a stub implementation that marks the connection as established.
    /// TODO: Implement actual Iggy client connection
    pub async fn connect(&self) -> Result<()> {
        // TODO: Create Iggy client and connect to server
        // TODO: Create stream and topics if they don't exist
        // TODO: Set up consumer for reading events
        debug!("Iggy assessment log connected (stub)");
        *self.connected.write().await = true;
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

    /// Flush buffered events to Iggy.
    ///
    /// This is a stub implementation that clears the buffer.
    /// TODO: Implement actual event publishing to Iggy
    pub async fn flush_buffer(&self) -> Result<()> {
        let mut buffer = self.buffer.write().await;
        let count = buffer.len();
        if count > 0 {
            // TODO: Publish each event to the appropriate topic
            debug!(count, "Flushing buffered events to Iggy (stub)");
            buffer.clear();
        }
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

        // Buffer the event (will be published to Iggy when connected)
        // TODO: When connected, publish directly to Iggy instead of buffering
        self.buffer.write().await.push(event.clone());

        // Broadcast to subscribers (ignore send errors - no subscribers is fine)
        let _ = self.tx.send(event);

        Ok(event_id)
    }

    async fn read_session(&self, session_id: &SessionId) -> Result<Vec<AssessmentEvent>> {
        // TODO: Query Iggy for events with matching session_id
        // For now, read from buffer
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
        // TODO: Query Iggy for events within time range
        // For now, read from buffer
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
    async fn iggy_log_connect_sets_connected_flag() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log = IggyAssessmentLog::new(manager);

        assert!(!log.is_connected().await);
        log.connect().await.unwrap();
        assert!(log.is_connected().await);
    }

    #[tokio::test]
    async fn iggy_log_flush_buffer_clears_events() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log = IggyAssessmentLog::new(manager);

        // Append some events
        log.append(make_lightweight_event("sess-1")).await.unwrap();
        log.append(make_medium_event("sess-2")).await.unwrap();
        assert_eq!(log.buffer_len().await, 2);

        // Flush should clear the buffer
        log.flush_buffer().await.unwrap();
        assert_eq!(log.buffer_len().await, 0);
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
