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
