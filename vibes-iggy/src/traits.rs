//! Core traits for event log operations.

use async_trait::async_trait;

/// Offset into an event stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Offset(pub u64);

/// Position to seek to when starting a consumer.
#[derive(Debug, Clone, Copy)]
pub enum SeekPosition {
    /// Start from the beginning.
    Beginning,
    /// Start from the end (new events only).
    End,
    /// Start from a specific offset.
    Offset(Offset),
}

/// A batch of events returned from polling.
#[derive(Debug)]
pub struct EventBatch<T> {
    /// The events in this batch.
    pub events: Vec<T>,
    /// The offset of the last event in this batch.
    pub last_offset: Option<Offset>,
}

/// Trait for appending events and creating consumers.
#[async_trait]
pub trait EventLog<T>: Send + Sync {
    /// The consumer type returned by this event log.
    type Consumer: EventConsumer<T>;

    /// Append an event to the log.
    async fn append(&self, event: &T) -> crate::Result<Offset>;

    /// Create a consumer for this event log.
    async fn consumer(&self, group: &str, position: SeekPosition) -> crate::Result<Self::Consumer>;
}

/// Trait for polling events with offset tracking.
#[async_trait]
pub trait EventConsumer<T>: Send + Sync {
    /// Poll for new events.
    async fn poll(&mut self) -> crate::Result<EventBatch<T>>;

    /// Commit the current offset.
    async fn commit(&mut self) -> crate::Result<()>;
}
