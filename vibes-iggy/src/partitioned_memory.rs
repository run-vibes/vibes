//! Partitioned in-memory EventLog for testing cross-partition behavior.
//!
//! This implementation simulates Iggy's partitioning to test:
//! - Events distributed across multiple partitions
//! - Partition-local offsets (not global)
//! - Cross-partition ordering behavior

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{Notify, RwLock};

use crate::error::Result;
use crate::traits::{EventBatch, EventConsumer, EventLog, Offset, Partitionable, SeekPosition};

/// Number of partitions (matches Iggy config)
const PARTITION_COUNT: usize = 8;

/// Shared state for a single partition.
struct PartitionState<E> {
    events: Vec<E>,
    next_offset: u64,
}

impl<E> Default for PartitionState<E> {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            next_offset: 0,
        }
    }
}

/// Shared state between EventLog and its consumers.
struct SharedState<E> {
    partitions: RwLock<[PartitionState<E>; PARTITION_COUNT]>,
    consumer_offsets: RwLock<HashMap<String, [Offset; PARTITION_COUNT]>>,
    /// Global event counter for high_water_mark (sum across all partitions)
    global_count: AtomicU64,
    notify: Notify,
}

/// Partitioned in-memory implementation of EventLog.
///
/// Simulates Iggy's partitioning behavior:
/// - Events are routed to partitions based on partition key hash
/// - Each partition has its own offset sequence
/// - Consumers read from all partitions
pub struct PartitionedInMemoryEventLog<E> {
    shared: Arc<SharedState<E>>,
}

impl<E> PartitionedInMemoryEventLog<E>
where
    E: Clone + Send + Sync + Partitionable + 'static,
{
    /// Create a new partitioned in-memory event log.
    #[must_use]
    pub fn new() -> Self {
        Self {
            shared: Arc::new(SharedState {
                partitions: RwLock::new(std::array::from_fn(|_| PartitionState::default())),
                consumer_offsets: RwLock::new(HashMap::new()),
                global_count: AtomicU64::new(0),
                notify: Notify::new(),
            }),
        }
    }

    /// Get the partition ID for a given key.
    fn partition_for_key(key: Option<&str>) -> usize {
        match key {
            Some(k) => {
                let mut hasher = DefaultHasher::new();
                k.hash(&mut hasher);
                (hasher.finish() as usize) % PARTITION_COUNT
            }
            None => 0, // Default to partition 0 if no key
        }
    }

    /// Get total event count across all partitions.
    pub async fn total_count(&self) -> usize {
        let partitions = self.shared.partitions.read().await;
        partitions.iter().map(|p| p.events.len()).sum()
    }
}

impl<E> Default for PartitionedInMemoryEventLog<E>
where
    E: Clone + Send + Sync + Partitionable + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<E> EventLog<E> for PartitionedInMemoryEventLog<E>
where
    E: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone + Partitionable + 'static,
{
    async fn append(&self, event: E) -> Result<Offset> {
        let partition_id = Self::partition_for_key(event.partition_key());
        let mut partitions = self.shared.partitions.write().await;
        let partition = &mut partitions[partition_id];

        let offset = partition.next_offset;
        partition.next_offset += 1;
        partition.events.push(event);

        // Track global count for high_water_mark
        self.shared.global_count.fetch_add(1, Ordering::Relaxed);
        self.shared.notify.notify_waiters();
        Ok(offset)
    }

    async fn append_batch(&self, events: Vec<E>) -> Result<Offset> {
        let mut last_offset = 0;
        for event in events {
            last_offset = self.append(event).await?;
        }
        Ok(last_offset)
    }

    async fn consumer(&self, group: &str) -> Result<Box<dyn EventConsumer<E>>> {
        // Get or create consumer offsets for all partitions
        let offsets = {
            let consumer_offsets = self.shared.consumer_offsets.read().await;
            consumer_offsets
                .get(group)
                .copied()
                .unwrap_or([0; PARTITION_COUNT])
        };

        Ok(Box::new(PartitionedInMemoryConsumer {
            group: group.to_string(),
            shared: Arc::clone(&self.shared),
            current_offsets: offsets,
            committed_offsets: offsets,
        }))
    }

    fn high_water_mark(&self) -> Offset {
        // Return total event count across all partitions
        // Note: This is a "virtual" global offset - individual partition offsets
        // are independent and can't be summed for seek purposes
        self.shared.global_count.load(Ordering::Relaxed)
    }
}

/// Partitioned in-memory consumer implementation.
struct PartitionedInMemoryConsumer<E> {
    group: String,
    shared: Arc<SharedState<E>>,
    current_offsets: [Offset; PARTITION_COUNT],
    committed_offsets: [Offset; PARTITION_COUNT],
}

#[async_trait]
impl<E> EventConsumer<E> for PartitionedInMemoryConsumer<E>
where
    E: Send + Sync + Clone + 'static,
{
    async fn poll(&mut self, max_count: usize, _timeout: Duration) -> Result<EventBatch<E>> {
        let partitions = self.shared.partitions.read().await;
        let per_partition = (max_count / PARTITION_COUNT).max(1);

        let mut all_events: Vec<(Offset, E)> = Vec::new();

        // Poll each partition
        for (partition_id, partition) in partitions.iter().enumerate() {
            let start = self.current_offsets[partition_id] as usize;
            let end = std::cmp::min(start + per_partition, partition.events.len());

            for i in start..end {
                all_events.push((i as Offset, partition.events[i].clone()));
            }

            if end > start {
                self.current_offsets[partition_id] = end as Offset;
            }
        }

        // INTENTIONALLY sort by offset (like broken Iggy behavior)
        // This is WRONG for cross-partition ordering - offsets are partition-local!
        all_events.sort_by_key(|(offset, _)| *offset);

        Ok(EventBatch::new(all_events))
    }

    async fn commit(&mut self, _offset: Offset) -> Result<()> {
        self.committed_offsets = self.current_offsets;
        let mut offsets = self.shared.consumer_offsets.write().await;
        offsets.insert(self.group.clone(), self.committed_offsets);
        Ok(())
    }

    async fn seek(&mut self, position: SeekPosition) -> Result<()> {
        match position {
            SeekPosition::Beginning => {
                self.current_offsets = [0; PARTITION_COUNT];
            }
            SeekPosition::End => {
                let partitions = self.shared.partitions.read().await;
                for (i, partition) in partitions.iter().enumerate() {
                    self.current_offsets[i] = partition.events.len() as Offset;
                }
            }
            SeekPosition::Offset(o) => {
                // Set ALL partitions to the same offset (like broken Iggy behavior)
                self.current_offsets = [o; PARTITION_COUNT];
            }
        }
        Ok(())
    }

    fn committed_offset(&self) -> Offset {
        // Return minimum across partitions (like Iggy)
        self.committed_offsets.iter().copied().min().unwrap_or(0)
    }

    fn group(&self) -> &str {
        &self.group
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent {
        id: String,
        data: String,
    }

    impl Partitionable for TestEvent {
        fn partition_key(&self) -> Option<&str> {
            Some(&self.id)
        }
    }

    #[tokio::test]
    async fn events_distributed_across_partitions() {
        let log = PartitionedInMemoryEventLog::new();

        // Append events with different keys (should go to different partitions)
        for i in 0..16 {
            log.append(TestEvent {
                id: format!("session-{}", i),
                data: format!("data-{}", i),
            })
            .await
            .unwrap();
        }

        assert_eq!(log.total_count().await, 16);
    }

    #[tokio::test]
    async fn consumer_reads_from_all_partitions() {
        let log = PartitionedInMemoryEventLog::new();

        // Append events to different partitions
        for i in 0..8 {
            log.append(TestEvent {
                id: format!("key-{}", i),
                data: format!("data-{}", i),
            })
            .await
            .unwrap();
        }

        let mut consumer = log.consumer("test").await.unwrap();
        let batch = consumer.poll(100, Duration::from_millis(10)).await.unwrap();

        // Should get all events from all partitions
        assert_eq!(batch.len(), 8);
    }

    #[tokio::test]
    async fn offset_sorting_is_wrong_for_cross_partition() {
        let log = PartitionedInMemoryEventLog::new();

        // Create events that will go to different partitions
        // Because of hash distribution, offset 0 in partition A is unrelated to
        // offset 0 in partition B, yet they'll be sorted together
        for i in 0..4 {
            log.append(TestEvent {
                id: format!("a-{}", i), // Goes to one partition
                data: format!("a-data-{}", i),
            })
            .await
            .unwrap();
            log.append(TestEvent {
                id: format!("b-{}", i), // Goes to different partition
                data: format!("b-data-{}", i),
            })
            .await
            .unwrap();
        }

        let mut consumer = log.consumer("test").await.unwrap();
        consumer.seek(SeekPosition::Beginning).await.unwrap();
        let batch = consumer.poll(100, Duration::from_millis(10)).await.unwrap();

        // Events are sorted by offset, which is partition-local
        // This means offset 0 from partition A and offset 0 from partition B
        // are interleaved, NOT in the order they were appended
        assert_eq!(batch.len(), 8);

        // The first events will have offset 0 (from different partitions)
        // This demonstrates the broken behavior
        let offsets: Vec<_> = batch.events.iter().map(|(o, _)| *o).collect();
        // With 8 events across partitions, we'll see repeated offsets like [0,0,1,1,2,2,3,3]
        // rather than [0,1,2,3,4,5,6,7] if it were a single partition
        assert!(
            offsets.windows(2).any(|w| w[0] == w[1]),
            "Expected repeated offsets from different partitions, got {:?}",
            offsets
        );
    }
}
