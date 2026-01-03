//! In-memory EventLog implementation for testing.
//!
//! This implementation stores events in memory without persistence.
//! Useful for testing and development without running Iggy server.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{Notify, RwLock};

use crate::error::Result;
use crate::traits::{EventBatch, EventConsumer, EventLog, Offset, SeekPosition};

/// Shared state between EventLog and its consumers.
struct SharedState<E> {
    events: RwLock<Vec<E>>,
    consumer_offsets: RwLock<HashMap<String, Offset>>,
    notify: Notify,
}

/// In-memory implementation of EventLog for testing.
///
/// Unlike snapshot-based implementations, this shares the event
/// vector with consumers so they can see newly appended events.
pub struct InMemoryEventLog<E> {
    shared: Arc<SharedState<E>>,
    next_offset: AtomicU64,
}

impl<E> InMemoryEventLog<E>
where
    E: Clone + Send + Sync + 'static,
{
    /// Create a new in-memory event log.
    #[must_use]
    pub fn new() -> Self {
        Self {
            shared: Arc::new(SharedState {
                events: RwLock::new(Vec::new()),
                consumer_offsets: RwLock::new(HashMap::new()),
                notify: Notify::new(),
            }),
            next_offset: AtomicU64::new(0),
        }
    }

    /// Get the number of events in the log.
    pub async fn len(&self) -> usize {
        self.shared.events.read().await.len()
    }

    /// Check if the log is empty.
    pub async fn is_empty(&self) -> bool {
        self.shared.events.read().await.is_empty()
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
        self.shared.events.write().await.push(event);
        self.shared.notify.notify_waiters();
        Ok(offset)
    }

    async fn append_batch(&self, events: Vec<E>) -> Result<Offset> {
        let count = events.len() as u64;
        if count == 0 {
            return Ok(self.high_water_mark().saturating_sub(1));
        }

        let first_offset = self.next_offset.fetch_add(count, Ordering::SeqCst);
        self.shared.events.write().await.extend(events);
        self.shared.notify.notify_waiters();
        Ok(first_offset + count - 1)
    }

    async fn consumer(&self, group: &str) -> Result<Box<dyn EventConsumer<E>>> {
        // Get or create consumer offset
        let offset = {
            let offsets = self.shared.consumer_offsets.read().await;
            offsets.get(group).copied().unwrap_or(0)
        };

        Ok(Box::new(InMemoryConsumer {
            group: group.to_string(),
            shared: Arc::clone(&self.shared),
            current_offset: offset,
            committed_offset: offset,
        }))
    }

    fn high_water_mark(&self) -> Offset {
        self.next_offset.load(Ordering::SeqCst)
    }
}

/// In-memory consumer implementation.
///
/// Shares state with the parent EventLog so it can see newly appended events.
struct InMemoryConsumer<E> {
    group: String,
    shared: Arc<SharedState<E>>,
    current_offset: Offset,
    committed_offset: Offset,
}

#[async_trait]
impl<E> EventConsumer<E> for InMemoryConsumer<E>
where
    E: Send + Sync + Clone + 'static,
{
    async fn poll(&mut self, max_count: usize, timeout: Duration) -> Result<EventBatch<E>> {
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            let events = self.shared.events.read().await;
            let start = self.current_offset as usize;
            let end = std::cmp::min(start + max_count, events.len());

            if start < events.len() {
                let batch: Vec<(Offset, E)> = events[start..end]
                    .iter()
                    .enumerate()
                    .map(|(i, e)| ((start + i) as Offset, e.clone()))
                    .collect();

                if let Some((last_offset, _)) = batch.last() {
                    self.current_offset = last_offset + 1;
                }

                return Ok(EventBatch::new(batch));
            }

            drop(events); // Release lock before waiting

            // Wait for new events or timeout
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return Ok(EventBatch::empty());
            }

            tokio::select! {
                _ = self.shared.notify.notified() => {
                    // New events available, loop to check
                }
                _ = tokio::time::sleep(remaining) => {
                    return Ok(EventBatch::empty());
                }
            }
        }
    }

    async fn commit(&mut self, offset: Offset) -> Result<()> {
        self.committed_offset = offset;
        let mut offsets = self.shared.consumer_offsets.write().await;
        offsets.insert(self.group.clone(), offset);
        Ok(())
    }

    async fn seek(&mut self, position: SeekPosition) -> Result<()> {
        self.current_offset = match position {
            SeekPosition::Beginning => 0,
            SeekPosition::End => self.shared.events.read().await.len() as Offset,
            SeekPosition::Offset(o) => o,
            SeekPosition::FromEnd(n) => {
                let len = self.shared.events.read().await.len() as u64;
                len.saturating_sub(n)
            }
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

    #[tokio::test]
    async fn consumer_seek_from_end() {
        let log: InMemoryEventLog<String> = InMemoryEventLog::new();
        // Append 10 events (offsets 0-9)
        for i in 0..10 {
            log.append(format!("event-{i}")).await.unwrap();
        }

        let mut consumer = log.consumer("test-group").await.unwrap();

        // Seek to 3 events before the end (should start at offset 7)
        consumer.seek(SeekPosition::FromEnd(3)).await.unwrap();

        let batch = consumer.poll(10, Duration::from_secs(1)).await.unwrap();

        // Should get events at offsets 7, 8, 9 (last 3)
        assert_eq!(batch.len(), 3);
        assert_eq!(batch.first_offset(), Some(7));
        assert_eq!(batch.last_offset(), Some(9));
    }

    #[tokio::test]
    async fn consumer_seek_from_end_with_more_than_available() {
        let log: InMemoryEventLog<String> = InMemoryEventLog::new();
        // Append 5 events (offsets 0-4)
        for i in 0..5 {
            log.append(format!("event-{i}")).await.unwrap();
        }

        let mut consumer = log.consumer("test-group").await.unwrap();

        // Request 100 events before end, but only 5 exist
        // Should clamp to beginning (offset 0)
        consumer.seek(SeekPosition::FromEnd(100)).await.unwrap();

        let batch = consumer.poll(10, Duration::from_secs(1)).await.unwrap();

        // Should get all 5 events
        assert_eq!(batch.len(), 5);
        assert_eq!(batch.first_offset(), Some(0));
        assert_eq!(batch.last_offset(), Some(4));
    }
}
