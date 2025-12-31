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
