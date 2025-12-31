//! Iggy-backed EventLog implementation.
//!
//! This module provides persistent event storage using Iggy as the backend.
//! Events are written to an Iggy stream/topic and consumers track their
//! offsets independently.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::error::Result;
use crate::manager::IggyManager;
use crate::traits::{EventBatch, EventConsumer, EventLog, Offset, SeekPosition};

/// Stream and topic names for the event log.
pub mod topics {
    /// The stream name for vibes events.
    pub const STREAM_NAME: &str = "vibes";
    /// The topic name for the main event log.
    pub const EVENTS_TOPIC: &str = "events";
}

/// Iggy-backed implementation of EventLog.
///
/// Provides persistent event storage with consumer group offset tracking.
/// Currently uses in-memory buffering until Iggy SDK integration is complete.
pub struct IggyEventLog<E> {
    /// Reference to the Iggy manager (for connection info)
    #[allow(dead_code)]
    manager: Arc<IggyManager>,

    /// In-memory buffer for events (until Iggy client connected)
    buffer: RwLock<Vec<E>>,

    /// Current high water mark
    high_water_mark: AtomicU64,

    /// Whether we're connected to Iggy
    connected: RwLock<bool>,
}

impl<E> IggyEventLog<E>
where
    E: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone + 'static,
{
    /// Create a new IggyEventLog.
    ///
    /// The manager should be started before calling this.
    #[must_use]
    pub fn new(manager: Arc<IggyManager>) -> Self {
        Self {
            manager,
            buffer: RwLock::new(Vec::new()),
            high_water_mark: AtomicU64::new(0),
            connected: RwLock::new(false),
        }
    }

    /// Connect to the Iggy server.
    ///
    /// This establishes the connection and creates streams/topics if needed.
    pub async fn connect(&self) -> Result<()> {
        // TODO: Implement actual Iggy client connection
        // For now, mark as connected and use buffer
        info!("IggyEventLog connecting (stub implementation)");
        *self.connected.write().await = true;
        Ok(())
    }

    /// Check if connected to Iggy.
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }
}

#[async_trait]
impl<E> EventLog<E> for IggyEventLog<E>
where
    E: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone + 'static,
{
    async fn append(&self, event: E) -> Result<Offset> {
        let offset = self.high_water_mark.fetch_add(1, Ordering::SeqCst);

        // TODO: Write to Iggy when connected
        // For now, buffer in memory
        self.buffer.write().await.push(event);

        debug!(offset, "Appended event to log");
        Ok(offset)
    }

    async fn append_batch(&self, events: Vec<E>) -> Result<Offset> {
        let count = events.len() as u64;
        if count == 0 {
            return Ok(self.high_water_mark().saturating_sub(1));
        }

        let first_offset = self.high_water_mark.fetch_add(count, Ordering::SeqCst);

        // TODO: Write batch to Iggy
        self.buffer.write().await.extend(events);

        debug!(first_offset, count, "Appended batch to log");
        Ok(first_offset + count - 1)
    }

    async fn consumer(&self, group: &str) -> Result<Box<dyn EventConsumer<E>>> {
        // TODO: Create actual Iggy consumer
        // For now, create an in-memory consumer over the buffer
        let events = self.buffer.read().await.clone();

        Ok(Box::new(IggyEventConsumer {
            group: group.to_string(),
            events: Arc::new(events),
            current_offset: 0,
            committed_offset: 0,
        }))
    }

    fn high_water_mark(&self) -> Offset {
        self.high_water_mark.load(Ordering::SeqCst)
    }
}

/// Iggy-backed consumer implementation.
///
/// Currently uses in-memory snapshot until Iggy SDK integration.
struct IggyEventConsumer<E> {
    group: String,
    events: Arc<Vec<E>>,
    current_offset: Offset,
    committed_offset: Offset,
}

#[async_trait]
impl<E> EventConsumer<E> for IggyEventConsumer<E>
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
        // TODO: Commit to Iggy
        self.committed_offset = offset;
        debug!(group = %self.group, offset, "Committed offset");
        Ok(())
    }

    async fn seek(&mut self, position: SeekPosition) -> Result<()> {
        self.current_offset = match position {
            SeekPosition::Beginning => 0,
            SeekPosition::End => self.events.len() as Offset,
            SeekPosition::Offset(o) => o,
        };
        debug!(group = %self.group, offset = self.current_offset, "Seeked to position");
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

    #[tokio::test]
    async fn iggy_log_append_and_read() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log: IggyEventLog<String> = IggyEventLog::new(manager);

        let o1 = log.append("first".to_string()).await.unwrap();
        let o2 = log.append("second".to_string()).await.unwrap();

        assert_eq!(o1, 0);
        assert_eq!(o2, 1);
        assert_eq!(log.high_water_mark(), 2);
    }

    #[tokio::test]
    async fn iggy_log_consumer_polls() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log: IggyEventLog<String> = IggyEventLog::new(manager);

        log.append("event-1".to_string()).await.unwrap();
        log.append("event-2".to_string()).await.unwrap();

        let mut consumer = log.consumer("test").await.unwrap();
        let batch = consumer.poll(10, Duration::from_secs(1)).await.unwrap();

        assert_eq!(batch.len(), 2);
    }

    #[tokio::test]
    async fn iggy_log_connect() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log: IggyEventLog<String> = IggyEventLog::new(manager);

        assert!(!log.is_connected().await);
        log.connect().await.unwrap();
        assert!(log.is_connected().await);
    }

    #[tokio::test]
    async fn iggy_log_append_batch() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log: IggyEventLog<String> = IggyEventLog::new(manager);

        let offset = log
            .append_batch(vec!["a".to_string(), "b".to_string(), "c".to_string()])
            .await
            .unwrap();

        assert_eq!(offset, 2); // Last offset
        assert_eq!(log.high_water_mark(), 3);
    }

    #[tokio::test]
    async fn iggy_consumer_seek() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log: IggyEventLog<String> = IggyEventLog::new(manager);

        for i in 0..5 {
            log.append(format!("event-{i}")).await.unwrap();
        }

        let mut consumer = log.consumer("test-group").await.unwrap();

        // Poll some events
        let batch1 = consumer.poll(3, Duration::from_secs(1)).await.unwrap();
        assert_eq!(batch1.len(), 3);

        // Seek back to beginning
        consumer.seek(SeekPosition::Beginning).await.unwrap();

        let batch2 = consumer.poll(2, Duration::from_secs(1)).await.unwrap();
        assert_eq!(batch2.first_offset(), Some(0));
    }

    #[tokio::test]
    async fn iggy_consumer_commit() {
        let config = IggyConfig::default();
        let manager = Arc::new(IggyManager::new(config));
        let log: IggyEventLog<String> = IggyEventLog::new(manager);

        log.append("event".to_string()).await.unwrap();

        let mut consumer = log.consumer("test-group").await.unwrap();
        assert_eq!(consumer.committed_offset(), 0);

        consumer.commit(42).await.unwrap();
        assert_eq!(consumer.committed_offset(), 42);
    }
}
