//! Consumer infrastructure for EventLog-based event processing.
//!
//! This module provides the `ConsumerManager` which spawns and manages consumer tasks
//! that process events from the unified EventLog. Each consumer runs in its own task
//! and can be configured with different starting positions and handlers.
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────┐
//! │                     Iggy EventLog                                 │
//! │  [VibesEvent0][VibesEvent1][VibesEvent2]...[VibesEventN]         │
//! └───────────────────────────┬──────────────────────────────────────┘
//!                             │
//!      ┌──────────────────────┼───────────────────────────┐
//!      │                      │                           │
//!      ▼                      ▼                           ▼
//! ┌─────────────────┐   ┌─────────────────┐      ┌────────────────────┐
//! │ websocket       │   │ chat-history    │      │ assessment         │
//! │ (End, live)     │   │ (Beginning)     │      │ (Beginning)        │
//! └─────────────────┘   └─────────────────┘      └────────────────────┘
//! ```
//!
//! Each consumer is "just another consumer" of the EventLog, tracking its own offset.

pub mod assessment;
pub mod notification;
pub mod websocket;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace, warn};
use vibes_core::StoredEvent;
use vibes_iggy::{EventLog, Offset, SeekPosition};

/// Result type for consumer operations.
pub type Result<T> = std::result::Result<T, ConsumerError>;

/// Errors that can occur in consumer operations.
#[derive(Debug, thiserror::Error)]
pub enum ConsumerError {
    #[error("Failed to create consumer: {0}")]
    Creation(String),

    #[error("Consumer poll failed: {0}")]
    Poll(String),

    #[error("Consumer seek failed: {0}")]
    Seek(String),

    #[error("Consumer commit failed: {0}")]
    Commit(String),
}

/// Type alias for async event handlers (without offset).
pub type EventHandler =
    Arc<dyn Fn(StoredEvent) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

/// Type alias for async event handlers with offset.
pub type OffsetEventHandler =
    Arc<dyn Fn(Offset, StoredEvent) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

/// Configuration for a consumer.
pub struct ConsumerConfig {
    /// Consumer group name (determines offset tracking).
    pub group: String,
    /// Where to start reading from.
    pub start_position: SeekPosition,
    /// Maximum events per poll.
    pub batch_size: usize,
    /// Poll timeout.
    pub poll_timeout: Duration,
}

impl ConsumerConfig {
    /// Create a live consumer that only receives new events.
    pub fn live(group: impl Into<String>) -> Self {
        Self {
            group: group.into(),
            start_position: SeekPosition::End,
            batch_size: 100,
            poll_timeout: Duration::from_millis(50),
        }
    }

    /// Create a replay consumer that processes all events from the beginning.
    pub fn replay(group: impl Into<String>) -> Self {
        Self {
            group: group.into(),
            start_position: SeekPosition::Beginning,
            batch_size: 100,
            poll_timeout: Duration::from_secs(1),
        }
    }

    /// Set the batch size.
    #[must_use]
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Set the poll timeout.
    #[must_use]
    pub fn with_poll_timeout(mut self, timeout: Duration) -> Self {
        self.poll_timeout = timeout;
        self
    }
}

/// Manages consumer tasks that process events from the EventLog.
pub struct ConsumerManager {
    event_log: Arc<dyn EventLog<StoredEvent>>,
    handles: Vec<JoinHandle<()>>,
    shutdown: CancellationToken,
}

impl ConsumerManager {
    /// Create a new consumer manager.
    pub fn new(event_log: Arc<dyn EventLog<StoredEvent>>) -> Self {
        Self {
            event_log,
            handles: Vec::new(),
            shutdown: CancellationToken::new(),
        }
    }

    /// Spawn a consumer task with the given configuration and handler.
    ///
    /// The consumer will run until shutdown is called or the handler returns an error.
    pub async fn spawn_consumer(
        &mut self,
        config: ConsumerConfig,
        handler: EventHandler,
    ) -> Result<()> {
        let mut consumer = self
            .event_log
            .consumer(&config.group)
            .await
            .map_err(|e| ConsumerError::Creation(e.to_string()))?;

        // Seek to the configured position
        consumer
            .seek(config.start_position)
            .await
            .map_err(|e| ConsumerError::Seek(e.to_string()))?;

        let shutdown = self.shutdown.clone();
        let group = config.group.clone();

        let handle = tokio::spawn(async move {
            info!(group = %group, "Consumer started");

            loop {
                tokio::select! {
                    _ = shutdown.cancelled() => {
                        info!(group = %group, "Consumer received shutdown signal");
                        break;
                    }
                    result = consumer.poll(config.batch_size, config.poll_timeout) => {
                        match result {
                            Ok(batch) => {
                                if batch.is_empty() {
                                    trace!(group = %group, "Empty batch, continuing");
                                    continue;
                                }

                                debug!(group = %group, count = batch.len(), "Processing batch");

                                let mut last_offset = None;
                                for (offset, event) in batch {
                                    handler(event).await;
                                    last_offset = Some(offset);
                                }

                                // Commit after processing batch
                                if let Some(offset) = last_offset
                                    && let Err(e) = consumer.commit(offset).await
                                {
                                    error!(group = %group, error = %e, "Failed to commit offset");
                                }
                            }
                            Err(e) => {
                                error!(group = %group, error = %e, "Poll failed");
                                // Back off on error
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        }
                    }
                }
            }

            info!(group = %group, "Consumer stopped");
        });

        self.handles.push(handle);
        Ok(())
    }

    /// Spawn a consumer task with the given configuration and offset-aware handler.
    ///
    /// Like `spawn_consumer` but the handler receives the offset along with the event.
    pub async fn spawn_consumer_with_offset(
        &mut self,
        config: ConsumerConfig,
        handler: OffsetEventHandler,
    ) -> Result<()> {
        let mut consumer = self
            .event_log
            .consumer(&config.group)
            .await
            .map_err(|e| ConsumerError::Creation(e.to_string()))?;

        // Seek to the configured position
        consumer
            .seek(config.start_position)
            .await
            .map_err(|e| ConsumerError::Seek(e.to_string()))?;

        let shutdown = self.shutdown.clone();
        let group = config.group.clone();

        let handle = tokio::spawn(async move {
            info!(group = %group, "Consumer started");

            loop {
                tokio::select! {
                    _ = shutdown.cancelled() => {
                        info!(group = %group, "Consumer received shutdown signal");
                        break;
                    }
                    result = consumer.poll(config.batch_size, config.poll_timeout) => {
                        match result {
                            Ok(batch) => {
                                if batch.is_empty() {
                                    trace!(group = %group, "Empty batch, continuing");
                                    continue;
                                }

                                debug!(group = %group, count = batch.len(), "Processing batch");

                                let mut last_offset = None;
                                for (offset, event) in batch {
                                    handler(offset, event).await;
                                    last_offset = Some(offset);
                                }

                                // Commit after processing batch
                                if let Some(offset) = last_offset
                                    && let Err(e) = consumer.commit(offset).await
                                {
                                    error!(group = %group, error = %e, "Failed to commit offset");
                                }
                            }
                            Err(e) => {
                                error!(group = %group, error = %e, "Poll failed");
                                // Back off on error
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        }
                    }
                }
            }

            info!(group = %group, "Consumer stopped");
        });

        self.handles.push(handle);
        Ok(())
    }

    /// Spawn a consumer that broadcasts events to a tokio broadcast channel.
    ///
    /// This is useful for fan-out to multiple WebSocket connections.
    /// Events are sent as (offset, stored_event) tuples.
    pub async fn spawn_broadcast_consumer(
        &mut self,
        config: ConsumerConfig,
        sender: broadcast::Sender<(Offset, StoredEvent)>,
    ) -> Result<()> {
        let handler: OffsetEventHandler = Arc::new(move |offset, stored| {
            let sender = sender.clone();
            Box::pin(async move {
                // Ignore send errors (no receivers)
                let _ = sender.send((offset, stored));
            })
        });

        self.spawn_consumer_with_offset(config, handler).await
    }

    /// Signal all consumers to shut down gracefully.
    pub fn shutdown(&self) {
        info!("Signaling consumer shutdown");
        self.shutdown.cancel();
    }

    /// Wait for all consumer tasks to complete.
    pub async fn wait_for_shutdown(self) {
        info!(count = self.handles.len(), "Waiting for consumers to stop");
        for handle in self.handles {
            if let Err(e) = handle.await {
                warn!(error = %e, "Consumer task panicked");
            }
        }
        info!("All consumers stopped");
    }

    /// Get the number of active consumers.
    pub fn consumer_count(&self) -> usize {
        self.handles.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use vibes_core::VibesEvent;
    use vibes_iggy::InMemoryEventLog;

    fn make_stored_event(session_id: &str) -> StoredEvent {
        StoredEvent::new(VibesEvent::SessionCreated {
            session_id: session_id.to_string(),
            name: None,
        })
    }

    #[tokio::test]
    async fn test_consumer_manager_spawns_task() {
        let log = Arc::new(InMemoryEventLog::<StoredEvent>::new());
        let mut manager = ConsumerManager::new(log.clone());

        // Append an event
        log.append(make_stored_event("session-1")).await.unwrap();

        // Track processed events
        let processed = Arc::new(AtomicUsize::new(0));
        let processed_clone = processed.clone();

        let handler: EventHandler = Arc::new(move |_event| {
            let processed = processed_clone.clone();
            Box::pin(async move {
                processed.fetch_add(1, Ordering::SeqCst);
            })
        });

        manager
            .spawn_consumer(ConsumerConfig::replay("test-group"), handler)
            .await
            .unwrap();

        // Give the consumer time to process
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert!(processed.load(Ordering::SeqCst) >= 1);
        assert_eq!(manager.consumer_count(), 1);

        manager.shutdown();
        manager.wait_for_shutdown().await;
    }

    #[tokio::test]
    async fn test_consumer_manager_shutdown_graceful() {
        let log = Arc::new(InMemoryEventLog::<StoredEvent>::new());
        let mut manager = ConsumerManager::new(log.clone());

        let handler: EventHandler = Arc::new(|_event| Box::pin(async {}));

        manager
            .spawn_consumer(ConsumerConfig::live("test-group"), handler)
            .await
            .unwrap();

        // Shutdown should be graceful
        manager.shutdown();
        manager.wait_for_shutdown().await;

        // No panic = success
    }

    #[tokio::test]
    async fn test_consumer_manager_broadcast_consumer() {
        let log = Arc::new(InMemoryEventLog::<StoredEvent>::new());
        let mut manager = ConsumerManager::new(log.clone());

        let (tx, mut rx) = broadcast::channel(100);

        manager
            .spawn_broadcast_consumer(ConsumerConfig::replay("broadcast-group"), tx)
            .await
            .unwrap();

        // Append an event
        log.append(make_stored_event("session-1")).await.unwrap();

        // Give the consumer time to process
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Check if we received the stored event with offset
        let (offset, _stored) = rx.try_recv().expect("should receive event");
        assert_eq!(offset, 0); // First event has offset 0

        manager.shutdown();
        manager.wait_for_shutdown().await;
    }

    #[tokio::test]
    async fn test_consumer_config_live() {
        let config = ConsumerConfig::live("my-group");
        assert_eq!(config.group, "my-group");
        assert_eq!(config.start_position, SeekPosition::End);
    }

    #[tokio::test]
    async fn test_consumer_config_replay() {
        let config = ConsumerConfig::replay("my-group");
        assert_eq!(config.group, "my-group");
        assert_eq!(config.start_position, SeekPosition::Beginning);
    }

    #[tokio::test]
    async fn test_websocket_consumer_broadcasts_events() {
        // Setup: EventLog with WebSocket consumer
        let log = Arc::new(InMemoryEventLog::<StoredEvent>::new());
        let mut manager = ConsumerManager::new(log.clone());

        let (tx, mut rx) = broadcast::channel(100);

        // Consumer uses "live" mode (SeekPosition::End) - only new events
        manager
            .spawn_broadcast_consumer(ConsumerConfig::live("websocket"), tx)
            .await
            .unwrap();

        // Append event AFTER consumer started (live mode)
        log.append(make_stored_event("ws-session-1")).await.unwrap();

        // Should receive the stored event quickly with offset
        let (offset, received) = tokio::time::timeout(Duration::from_millis(100), rx.recv())
            .await
            .expect("should receive within timeout")
            .expect("should receive event");

        assert_eq!(offset, 0); // First event has offset 0
        assert!(matches!(
            &received.event,
            VibesEvent::SessionCreated { session_id, .. } if session_id == "ws-session-1"
        ));

        manager.shutdown();
        manager.wait_for_shutdown().await;
    }

    #[tokio::test]
    async fn test_websocket_consumer_low_latency() {
        // Verify events are delivered quickly (< 100ms)
        let log = Arc::new(InMemoryEventLog::<StoredEvent>::new());
        let mut manager = ConsumerManager::new(log.clone());

        let (tx, mut rx) = broadcast::channel(100);

        manager
            .spawn_broadcast_consumer(
                ConsumerConfig::live("websocket").with_poll_timeout(Duration::from_millis(10)),
                tx,
            )
            .await
            .unwrap();

        // Measure latency
        let start = std::time::Instant::now();
        log.append(make_stored_event("latency-test")).await.unwrap();

        let (_offset, _event) = tokio::time::timeout(Duration::from_millis(100), rx.recv())
            .await
            .expect("should receive within 100ms")
            .expect("should receive event");

        let latency = start.elapsed();
        assert!(
            latency < Duration::from_millis(100),
            "latency was {:?}, should be < 100ms",
            latency
        );

        manager.shutdown();
        manager.wait_for_shutdown().await;
    }

    #[tokio::test]
    async fn test_consumer_processes_multiple_events() {
        let log = Arc::new(InMemoryEventLog::<StoredEvent>::new());
        let mut manager = ConsumerManager::new(log.clone());

        // Append multiple events
        for i in 0..5 {
            log.append(make_stored_event(&format!("session-{i}")))
                .await
                .unwrap();
        }

        let processed = Arc::new(AtomicUsize::new(0));
        let processed_clone = processed.clone();

        let handler: EventHandler = Arc::new(move |_event| {
            let processed = processed_clone.clone();
            Box::pin(async move {
                processed.fetch_add(1, Ordering::SeqCst);
            })
        });

        manager
            .spawn_consumer(ConsumerConfig::replay("multi-group"), handler)
            .await
            .unwrap();

        // Give time to process all events
        tokio::time::sleep(Duration::from_millis(200)).await;

        assert_eq!(processed.load(Ordering::SeqCst), 5);

        manager.shutdown();
        manager.wait_for_shutdown().await;
    }
}
