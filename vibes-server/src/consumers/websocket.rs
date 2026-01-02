//! WebSocket event consumer.
//!
//! This consumer reads from the EventLog and broadcasts events to all
//! connected WebSocket clients via the shared broadcast channel.

use std::time::Duration;

use tokio::sync::broadcast;
use vibes_core::StoredEvent;
use vibes_iggy::Offset;

use super::{ConsumerConfig, ConsumerManager, Result};

/// Start the WebSocket consumer that broadcasts events to WebSocket clients.
///
/// # Arguments
///
/// * `manager` - The consumer manager to spawn the consumer on
/// * `broadcaster` - The broadcast sender for WebSocket fan-out (sends offset, stored_event tuples)
///
/// # Returns
///
/// Returns Ok(()) on success, or an error if the consumer fails to start.
pub async fn start_websocket_consumer(
    manager: &mut ConsumerManager,
    broadcaster: broadcast::Sender<(Offset, StoredEvent)>,
) -> Result<()> {
    let config = ConsumerConfig::live("websocket")
        .with_poll_timeout(Duration::from_millis(50))
        .with_batch_size(100);

    manager.spawn_broadcast_consumer(config, broadcaster).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use vibes_core::VibesEvent;
    use vibes_iggy::{EventLog, InMemoryEventLog};

    fn make_stored_event(session_id: &str) -> StoredEvent {
        StoredEvent::new(VibesEvent::SessionCreated {
            session_id: session_id.to_string(),
            name: None,
        })
    }

    #[tokio::test]
    async fn test_start_websocket_consumer_receives_live_events() {
        let log = Arc::new(InMemoryEventLog::<StoredEvent>::new());
        let mut manager = ConsumerManager::new(log.clone());
        let (tx, mut rx) = broadcast::channel(100);

        // Start the consumer
        start_websocket_consumer(&mut manager, tx).await.unwrap();

        // Append an event (after consumer started - live mode)
        log.append(make_stored_event("live-event")).await.unwrap();

        // Should receive the StoredEvent with offset
        let (offset, received) = tokio::time::timeout(Duration::from_millis(100), rx.recv())
            .await
            .expect("should receive within timeout")
            .expect("should receive event");

        assert_eq!(offset, 0); // First event has offset 0
        assert!(matches!(
            &received.event,
            VibesEvent::SessionCreated { session_id, .. } if session_id == "live-event"
        ));

        manager.shutdown();
        manager.wait_for_shutdown().await;
    }

    #[tokio::test]
    async fn test_websocket_consumer_ignores_old_events() {
        let log = Arc::new(InMemoryEventLog::<StoredEvent>::new());

        // Append event BEFORE consumer starts
        log.append(make_stored_event("old-event")).await.unwrap();

        let mut manager = ConsumerManager::new(log.clone());
        let (tx, mut rx) = broadcast::channel(100);

        // Start the consumer (live mode - should ignore old events)
        start_websocket_consumer(&mut manager, tx).await.unwrap();

        // Give consumer time to start
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Append new event
        log.append(make_stored_event("new-event")).await.unwrap();

        // Should only receive the new event with offset 1 (second event)
        let (offset, received) = tokio::time::timeout(Duration::from_millis(100), rx.recv())
            .await
            .expect("should receive within timeout")
            .expect("should receive event");

        assert_eq!(offset, 1); // Second event has offset 1
        assert!(matches!(
            &received.event,
            VibesEvent::SessionCreated { session_id, .. } if session_id == "new-event"
        ));

        manager.shutdown();
        manager.wait_for_shutdown().await;
    }
}
