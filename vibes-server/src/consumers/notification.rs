//! Notification event consumer.
//!
//! This consumer reads from the EventLog and dispatches events to the
//! NotificationService for push notification delivery.

use std::sync::Arc;
use std::time::Duration;

use vibes_core::NotificationService;

use super::{ConsumerConfig, ConsumerManager, EventHandler, Result};

/// Start the notification consumer that dispatches events to the notification service.
///
/// # Arguments
///
/// * `manager` - The consumer manager to spawn the consumer on
/// * `notification_service` - The notification service for push delivery
///
/// # Returns
///
/// Returns Ok(()) on success, or an error if the consumer fails to start.
pub async fn start_notification_consumer(
    manager: &mut ConsumerManager,
    notification_service: Arc<NotificationService>,
) -> Result<()> {
    let config = ConsumerConfig::live("notifications")
        .with_poll_timeout(Duration::from_millis(100))
        .with_batch_size(50);

    let handler: EventHandler = Arc::new(move |stored| {
        let service = notification_service.clone();
        Box::pin(async move {
            // Extract inner event - NotificationService works with VibesEvent
            service.process_event(&stored.event).await;
        })
    });

    manager.spawn_consumer(config, handler).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use vibes_core::StoredEvent;
    use vibes_iggy::InMemoryEventLog;

    #[tokio::test]
    async fn test_notification_consumer_config() {
        let log = Arc::new(InMemoryEventLog::<StoredEvent>::new());
        let manager = ConsumerManager::new(log);

        // We can't use a real NotificationService without VAPID keys,
        // so we test the consumer configuration instead.
        let config = ConsumerConfig::live("notifications")
            .with_poll_timeout(Duration::from_millis(100))
            .with_batch_size(50);

        assert_eq!(config.group, "notifications");
        assert_eq!(config.start_position, vibes_iggy::SeekPosition::End);

        manager.shutdown();
        manager.wait_for_shutdown().await;
    }

    #[tokio::test]
    async fn test_notification_consumer_uses_live_mode() {
        // Verify the consumer starts at End (live mode) - only new events
        let config = ConsumerConfig::live("notifications");
        assert_eq!(config.start_position, vibes_iggy::SeekPosition::End);
    }
}
