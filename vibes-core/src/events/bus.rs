//! EventBus trait definition
//!
//! The EventBus abstraction enables event-driven communication with
//! late-joiner replay support (ADR-007).

use async_trait::async_trait;
use tokio::sync::broadcast;

use super::VibesEvent;

/// Sequence number for events (monotonically increasing)
pub type EventSeq = u64;

/// Event bus for publishing and subscribing to VibesEvents
///
/// Implementations must support:
/// - Publishing events with sequence numbers
/// - Live subscriptions via broadcast channel
/// - Historical replay for late joiners
/// - Session-scoped event retrieval
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish an event, returns its sequence number
    async fn publish(&self, event: VibesEvent) -> EventSeq;

    /// Subscribe to all events from now (live stream)
    fn subscribe(&self) -> broadcast::Receiver<(EventSeq, VibesEvent)>;

    /// Get all events starting from a sequence number (for replay)
    async fn events_from(&self, seq: EventSeq) -> Vec<(EventSeq, VibesEvent)>;

    /// Get all events for a specific session (for late joiners)
    async fn get_session_events(&self, session_id: &str) -> Vec<(EventSeq, VibesEvent)>;

    /// Current sequence number (high water mark)
    fn current_seq(&self) -> EventSeq;
}
