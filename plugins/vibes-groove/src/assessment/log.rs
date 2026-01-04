//! Assessment event log abstraction.
//!
//! Provides a trait-based interface for assessment event storage,
//! with Iggy as the default implementation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::broadcast;

use super::{AssessmentEvent, EventId, SessionId};
use crate::error::Result;

/// Trait for assessment event log storage.
///
/// Implementations must support:
/// - Append-only event storage (immutable log)
/// - Session-scoped queries
/// - Time-range queries
/// - Real-time subscription
#[async_trait]
pub trait AssessmentLog: Send + Sync {
    /// Append an event to the immutable log.
    ///
    /// Returns the event ID assigned to the event.
    async fn append(&self, event: AssessmentEvent) -> Result<EventId>;

    /// Read all events for a specific session.
    async fn read_session(&self, session_id: &SessionId) -> Result<Vec<AssessmentEvent>>;

    /// Read events in a time range.
    async fn read_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AssessmentEvent>>;

    /// Subscribe to real-time events.
    ///
    /// Returns a broadcast receiver for new events.
    fn subscribe(&self) -> broadcast::Receiver<AssessmentEvent>;
}

/// In-memory implementation for testing.
#[derive(Debug)]
pub struct InMemoryAssessmentLog {
    events: std::sync::RwLock<Vec<AssessmentEvent>>,
    tx: broadcast::Sender<AssessmentEvent>,
}

impl InMemoryAssessmentLog {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            events: std::sync::RwLock::new(Vec::new()),
            tx,
        }
    }

    /// Get count of stored events (for testing)
    pub fn len(&self) -> usize {
        self.events.read().unwrap().len()
    }

    /// Check if log is empty (for testing)
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for InMemoryAssessmentLog {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AssessmentLog for InMemoryAssessmentLog {
    async fn append(&self, event: AssessmentEvent) -> Result<EventId> {
        let event_id = *event.event_id();
        self.events.write().unwrap().push(event.clone());
        // Ignore send errors (no subscribers)
        let _ = self.tx.send(event);
        Ok(event_id)
    }

    async fn read_session(&self, session_id: &SessionId) -> Result<Vec<AssessmentEvent>> {
        let events = self.events.read().unwrap();
        Ok(events
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
        let events = self.events.read().unwrap();
        Ok(events
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
    use crate::assessment::{AssessmentContext, LightweightEvent};

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

    #[tokio::test]
    async fn in_memory_log_append_and_read() {
        let log = InMemoryAssessmentLog::new();
        let event = make_lightweight_event("sess-1");

        let event_id = log.append(event).await.unwrap();
        assert!(!log.is_empty());

        let events = log.read_session(&"sess-1".into()).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(*events[0].event_id(), event_id);
    }

    #[tokio::test]
    async fn in_memory_log_filters_by_session() {
        let log = InMemoryAssessmentLog::new();
        log.append(make_lightweight_event("sess-1")).await.unwrap();
        log.append(make_lightweight_event("sess-2")).await.unwrap();
        log.append(make_lightweight_event("sess-1")).await.unwrap();

        let events = log.read_session(&"sess-1".into()).await.unwrap();
        assert_eq!(events.len(), 2);
    }

    #[tokio::test]
    async fn in_memory_log_subscribe() {
        let log = InMemoryAssessmentLog::new();
        let mut rx = log.subscribe();

        let event = make_lightweight_event("sess-1");
        log.append(event.clone()).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.session_id().as_str(), "sess-1");
    }
}
