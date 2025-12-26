//! In-memory EventBus implementation
//!
//! MemoryEventBus stores events in a Vec for replay and uses a broadcast
//! channel for live subscribers.

use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use tokio::sync::{RwLock, broadcast};

use super::VibesEvent;
use super::bus::{EventBus, EventSeq};

/// In-memory implementation of EventBus
///
/// Uses a Vec for historical storage (enabling replay) and a broadcast
/// channel for live subscribers. Thread-safe via RwLock and atomics.
pub struct MemoryEventBus {
    /// Stored events with sequence numbers
    events: RwLock<Vec<(EventSeq, VibesEvent)>>,
    /// Next sequence number to assign
    next_seq: AtomicU64,
    /// Broadcast channel for live subscribers
    tx: broadcast::Sender<(EventSeq, VibesEvent)>,
}

impl MemoryEventBus {
    /// Create a new MemoryEventBus with the given broadcast channel capacity
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self {
            events: RwLock::new(Vec::new()),
            next_seq: AtomicU64::new(0),
            tx,
        }
    }
}

#[async_trait]
impl EventBus for MemoryEventBus {
    async fn publish(&self, event: VibesEvent) -> EventSeq {
        let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);

        // Store for replay
        self.events.write().await.push((seq, event.clone()));

        // Broadcast to live subscribers (ignore if no receivers)
        let _ = self.tx.send((seq, event));

        seq
    }

    fn subscribe(&self) -> broadcast::Receiver<(EventSeq, VibesEvent)> {
        self.tx.subscribe()
    }

    async fn events_from(&self, seq: EventSeq) -> Vec<(EventSeq, VibesEvent)> {
        self.events
            .read()
            .await
            .iter()
            .filter(|(s, _)| *s >= seq)
            .cloned()
            .collect()
    }

    async fn get_session_events(&self, session_id: &str) -> Vec<(EventSeq, VibesEvent)> {
        self.events
            .read()
            .await
            .iter()
            .filter(|(_, event)| event.session_id() == Some(session_id))
            .cloned()
            .collect()
    }

    fn current_seq(&self) -> EventSeq {
        self.next_seq.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::super::bus::EventBus;
    use crate::events::{ClaudeEvent, VibesEvent};

    // ==================== Publish Tests ====================

    #[tokio::test]
    async fn publish_returns_sequence_number() {
        let bus = super::MemoryEventBus::new(100);
        let event = VibesEvent::SessionCreated {
            session_id: "s1".to_string(),
            name: None,
        };

        let seq = bus.publish(event).await;
        assert_eq!(seq, 0);
    }

    #[tokio::test]
    async fn publish_increments_sequence_number() {
        let bus = super::MemoryEventBus::new(100);

        let seq1 = bus
            .publish(VibesEvent::SessionCreated {
                session_id: "s1".to_string(),
                name: None,
            })
            .await;

        let seq2 = bus
            .publish(VibesEvent::SessionCreated {
                session_id: "s2".to_string(),
                name: None,
            })
            .await;

        let seq3 = bus
            .publish(VibesEvent::SessionCreated {
                session_id: "s3".to_string(),
                name: None,
            })
            .await;

        assert_eq!(seq1, 0);
        assert_eq!(seq2, 1);
        assert_eq!(seq3, 2);
    }

    #[tokio::test]
    async fn current_seq_reflects_published_count() {
        use super::super::bus::EventBus;

        let bus = super::MemoryEventBus::new(100);
        assert_eq!(bus.current_seq(), 0);

        bus.publish(VibesEvent::SessionCreated {
            session_id: "s1".to_string(),
            name: None,
        })
        .await;
        assert_eq!(bus.current_seq(), 1);

        bus.publish(VibesEvent::SessionCreated {
            session_id: "s2".to_string(),
            name: None,
        })
        .await;
        assert_eq!(bus.current_seq(), 2);
    }

    // ==================== Subscribe Tests ====================

    #[tokio::test]
    async fn subscribe_receives_new_events() {
        use super::super::bus::EventBus;

        let bus = super::MemoryEventBus::new(100);
        let mut rx = bus.subscribe();

        // Publish after subscribing
        bus.publish(VibesEvent::SessionCreated {
            session_id: "s1".to_string(),
            name: Some("Test".to_string()),
        })
        .await;

        let (seq, event) = rx.recv().await.unwrap();
        assert_eq!(seq, 0);
        assert!(matches!(
            event,
            VibesEvent::SessionCreated { session_id, .. } if session_id == "s1"
        ));
    }

    #[tokio::test]
    async fn subscribe_receives_multiple_events_in_order() {
        use super::super::bus::EventBus;

        let bus = super::MemoryEventBus::new(100);
        let mut rx = bus.subscribe();

        bus.publish(VibesEvent::SessionCreated {
            session_id: "s1".to_string(),
            name: None,
        })
        .await;
        bus.publish(VibesEvent::SessionCreated {
            session_id: "s2".to_string(),
            name: None,
        })
        .await;

        let (seq1, _) = rx.recv().await.unwrap();
        let (seq2, _) = rx.recv().await.unwrap();

        assert_eq!(seq1, 0);
        assert_eq!(seq2, 1);
    }

    // ==================== Events From Tests ====================

    #[tokio::test]
    async fn events_from_returns_events_starting_at_seq() {
        use super::super::bus::EventBus;

        let bus = super::MemoryEventBus::new(100);

        bus.publish(VibesEvent::SessionCreated {
            session_id: "s1".to_string(),
            name: None,
        })
        .await;
        bus.publish(VibesEvent::SessionCreated {
            session_id: "s2".to_string(),
            name: None,
        })
        .await;
        bus.publish(VibesEvent::SessionCreated {
            session_id: "s3".to_string(),
            name: None,
        })
        .await;

        let events = bus.events_from(1).await;
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].0, 1);
        assert_eq!(events[1].0, 2);
    }

    #[tokio::test]
    async fn events_from_zero_returns_all() {
        use super::super::bus::EventBus;

        let bus = super::MemoryEventBus::new(100);

        bus.publish(VibesEvent::SessionCreated {
            session_id: "s1".to_string(),
            name: None,
        })
        .await;
        bus.publish(VibesEvent::SessionCreated {
            session_id: "s2".to_string(),
            name: None,
        })
        .await;

        let events = bus.events_from(0).await;
        assert_eq!(events.len(), 2);
    }

    #[tokio::test]
    async fn events_from_beyond_current_returns_empty() {
        use super::super::bus::EventBus;

        let bus = super::MemoryEventBus::new(100);

        bus.publish(VibesEvent::SessionCreated {
            session_id: "s1".to_string(),
            name: None,
        })
        .await;

        let events = bus.events_from(100).await;
        assert!(events.is_empty());
    }

    // ==================== Get Session Events Tests ====================

    #[tokio::test]
    async fn get_session_events_filters_by_session_id() {
        use super::super::bus::EventBus;

        let bus = super::MemoryEventBus::new(100);

        // Events for session s1
        bus.publish(VibesEvent::Claude {
            session_id: "s1".to_string(),
            event: ClaudeEvent::TextDelta {
                text: "Hello".to_string(),
            },
        })
        .await;

        // Events for session s2
        bus.publish(VibesEvent::Claude {
            session_id: "s2".to_string(),
            event: ClaudeEvent::TextDelta {
                text: "World".to_string(),
            },
        })
        .await;

        // More events for session s1
        bus.publish(VibesEvent::Claude {
            session_id: "s1".to_string(),
            event: ClaudeEvent::TurnComplete {
                usage: crate::events::Usage::default(),
            },
        })
        .await;

        let s1_events = bus.get_session_events("s1").await;
        assert_eq!(s1_events.len(), 2);

        let s2_events = bus.get_session_events("s2").await;
        assert_eq!(s2_events.len(), 1);
    }

    #[tokio::test]
    async fn get_session_events_returns_empty_for_unknown_session() {
        use super::super::bus::EventBus;

        let bus = super::MemoryEventBus::new(100);

        bus.publish(VibesEvent::SessionCreated {
            session_id: "s1".to_string(),
            name: None,
        })
        .await;

        let events = bus.get_session_events("unknown").await;
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn get_session_events_excludes_client_events() {
        use super::super::bus::EventBus;

        let bus = super::MemoryEventBus::new(100);

        // Session event
        bus.publish(VibesEvent::SessionCreated {
            session_id: "s1".to_string(),
            name: None,
        })
        .await;

        // Client event (no session_id)
        bus.publish(VibesEvent::ClientConnected {
            client_id: "c1".to_string(),
        })
        .await;

        let events = bus.get_session_events("s1").await;
        assert_eq!(events.len(), 1);
    }

    // ==================== Concurrent Access Tests ====================

    #[tokio::test]
    async fn concurrent_publish_maintains_sequence_integrity() {
        use super::super::bus::EventBus;
        use std::sync::Arc;

        let bus = Arc::new(super::MemoryEventBus::new(1000));
        let mut handles = vec![];

        // Spawn 10 tasks each publishing 10 events
        for i in 0..10 {
            let bus = Arc::clone(&bus);
            handles.push(tokio::spawn(async move {
                for j in 0..10 {
                    bus.publish(VibesEvent::SessionCreated {
                        session_id: format!("s{}-{}", i, j),
                        name: None,
                    })
                    .await;
                }
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // All 100 events should be stored with unique sequence numbers
        assert_eq!(bus.current_seq(), 100);

        let all_events = bus.events_from(0).await;
        assert_eq!(all_events.len(), 100);

        // Verify all sequence numbers are unique and in range
        let seqs: Vec<_> = all_events.iter().map(|(seq, _)| *seq).collect();
        for i in 0..100u64 {
            assert!(seqs.contains(&i), "Missing sequence {}", i);
        }
    }

    #[tokio::test]
    async fn multiple_subscribers_receive_same_events() {
        use super::super::bus::EventBus;

        let bus = super::MemoryEventBus::new(100);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        bus.publish(VibesEvent::SessionCreated {
            session_id: "s1".to_string(),
            name: None,
        })
        .await;

        let (seq1, _) = rx1.recv().await.unwrap();
        let (seq2, _) = rx2.recv().await.unwrap();

        assert_eq!(seq1, 0);
        assert_eq!(seq2, 0);
    }
}
