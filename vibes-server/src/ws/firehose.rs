//! WebSocket handler for firehose event streaming
//!
//! Provides a read-only WebSocket endpoint that streams all VibesEvents,
//! optionally filtered by event type and/or session ID.

use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::warn;
use vibes_core::VibesEvent;
use vibes_iggy::{Offset, SeekPosition};

use crate::AppState;

/// Query parameters for firehose filtering
#[derive(Debug, Deserialize)]
pub struct FirehoseQuery {
    /// Filter by event types (comma-separated, e.g., "Claude,SessionCreated")
    #[serde(default)]
    pub types: Option<String>,
    /// Filter by session ID
    #[serde(default)]
    pub session: Option<String>,
}

/// Server-to-client message: single live event with offset
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename = "event")]
pub struct FirehoseEventMessage {
    /// The event offset in the EventLog
    pub offset: Offset,
    /// The event data
    pub event: VibesEvent,
}

/// Server-to-client message: batch of events (initial load or pagination)
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename = "events_batch")]
pub struct FirehoseEventsBatch {
    /// The events in this batch with their offsets
    pub events: Vec<FirehoseEventMessage>,
    /// The oldest offset in this batch (for pagination)
    pub oldest_offset: Option<Offset>,
    /// Whether there are more events before this batch
    pub has_more: bool,
}

/// WebSocket upgrade handler for firehose
pub async fn firehose_ws(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(query): Query<FirehoseQuery>,
) -> Response {
    ws.on_upgrade(move |socket| handle_firehose(socket, state, query))
}

/// Number of historical events to send on firehose connection
const HISTORICAL_EVENT_COUNT: u64 = 100;

async fn handle_firehose(socket: WebSocket, state: Arc<AppState>, query: FirehoseQuery) {
    let (mut sender, mut receiver) = socket.split();

    // Parse filter types
    let filter_types: Option<Vec<String>> = query.types.as_ref().map(|t| {
        t.split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    });
    let filter_session = query.session.clone();

    // Subscribe to broadcast BEFORE loading historical events to avoid gaps
    let mut events_rx = state.subscribe_events();

    // Load historical events from EventLog (with offsets)
    let historical_events = load_historical_events(&state, HISTORICAL_EVENT_COUNT).await;

    // Clone filters for the send task
    let filter_types_clone = filter_types.clone();
    let filter_session_clone = filter_session.clone();

    // Determine if there's more history before the oldest loaded event
    let oldest_offset = historical_events.first().map(|(offset, _)| *offset);
    let has_more = oldest_offset.is_some_and(|o| o > 0);

    // Spawn task to forward events to WebSocket
    let send_task = tokio::spawn(async move {
        // First, send historical events as a batch
        let filtered_events: Vec<_> = historical_events
            .into_iter()
            .filter(|(_, event)| matches_filters(event, &filter_types_clone, &filter_session_clone))
            .map(|(offset, event)| FirehoseEventMessage { offset, event })
            .collect();

        let batch = FirehoseEventsBatch {
            oldest_offset: filtered_events.first().map(|e| e.offset),
            events: filtered_events,
            has_more,
        };

        match serde_json::to_string(&batch) {
            Ok(json) => {
                if let Err(e) = sender.send(Message::Text(json)).await {
                    warn!("Firehose send failed (initial batch): {}", e);
                    return;
                }
            }
            Err(e) => {
                warn!("Failed to serialize initial batch: {}", e);
            }
        }

        // Then stream live events from broadcast
        loop {
            match events_rx.recv().await {
                Ok((offset, event)) => {
                    if !matches_filters(&event, &filter_types_clone, &filter_session_clone) {
                        continue;
                    }

                    let msg = FirehoseEventMessage { offset, event };
                    match serde_json::to_string(&msg) {
                        Ok(json) => {
                            if let Err(e) = sender.send(Message::Text(json)).await {
                                warn!("Firehose send failed: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            warn!("Failed to serialize event: {}", e);
                        }
                    }
                }
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!("Firehose client lagged by {} events", n);
                }
            }
        }
    });

    // Handle incoming messages (just for close detection)
    while let Some(Ok(msg)) = receiver.next().await {
        if matches!(msg, Message::Close(_)) {
            break;
        }
    }

    send_task.abort();
}

/// Check if an event matches the filter criteria
fn matches_filters(
    event: &VibesEvent,
    filter_types: &Option<Vec<String>>,
    filter_session: &Option<String>,
) -> bool {
    // Apply type filter
    if let Some(types) = filter_types {
        let event_type = event_type_name(event).to_lowercase();
        if !types.iter().any(|t| event_type.contains(t)) {
            return false;
        }
    }

    // Apply session filter
    if let Some(session) = filter_session
        && event.session_id() != Some(session.as_str())
    {
        return false;
    }

    true
}

/// Get the event type name for filtering
fn event_type_name(event: &VibesEvent) -> &'static str {
    match event {
        VibesEvent::Claude { .. } => "Claude",
        VibesEvent::UserInput { .. } => "UserInput",
        VibesEvent::PermissionResponse { .. } => "PermissionResponse",
        VibesEvent::SessionCreated { .. } => "SessionCreated",
        VibesEvent::SessionStateChanged { .. } => "SessionStateChanged",
        VibesEvent::ClientConnected { .. } => "ClientConnected",
        VibesEvent::ClientDisconnected { .. } => "ClientDisconnected",
        VibesEvent::TunnelStateChanged { .. } => "TunnelStateChanged",
        VibesEvent::OwnershipTransferred { .. } => "OwnershipTransferred",
        VibesEvent::SessionRemoved { .. } => "SessionRemoved",
        VibesEvent::Hook { .. } => "Hook",
    }
}

/// Load the last N events from the EventLog.
///
/// Returns events in chronological order (oldest first) with their offsets.
async fn load_historical_events(state: &AppState, count: u64) -> Vec<(Offset, VibesEvent)> {
    let hwm = state.event_log.high_water_mark();
    if hwm == 0 {
        return Vec::new();
    }

    let start_offset = hwm.saturating_sub(count);

    // Create a temporary consumer to read historical events
    let consumer_group = format!("firehose-historical-{}", uuid::Uuid::new_v4());
    let mut consumer = match state.event_log.consumer(&consumer_group).await {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to create historical event consumer: {}", e);
            return Vec::new();
        }
    };

    // Seek to the start position
    if let Err(e) = consumer.seek(SeekPosition::Offset(start_offset)).await {
        warn!("Failed to seek historical consumer: {}", e);
        return Vec::new();
    }

    // Poll for events (short timeout since they should already exist)
    match consumer
        .poll(count as usize, Duration::from_millis(100))
        .await
    {
        Ok(batch) => batch.into_iter().collect(),
        Err(e) => {
            warn!("Failed to poll historical events: {}", e);
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vibes_core::ClaudeEvent;

    #[tokio::test]
    async fn load_historical_events_returns_last_n_events_with_offsets() {
        use crate::AppState;

        // Create state with in-memory event log
        let state = AppState::new();

        // Append 5 events to the EventLog
        for i in 0..5 {
            let event = VibesEvent::SessionCreated {
                session_id: format!("session-{}", i),
                name: Some(format!("Session {}", i)),
            };
            state.event_log.append(event).await.unwrap();
        }

        // Load last 3 events
        let events = load_historical_events(&state, 3).await;

        // Should get the last 3 events (sessions 2, 3, 4) with offsets
        assert_eq!(events.len(), 3);

        // Verify offsets (should be 2, 3, 4)
        assert_eq!(events[0].0, 2);
        assert_eq!(events[1].0, 3);
        assert_eq!(events[2].0, 4);

        // Verify they're the correct events (last 3)
        if let VibesEvent::SessionCreated { session_id, .. } = &events[0].1 {
            assert_eq!(session_id, "session-2");
        } else {
            panic!("Expected SessionCreated event");
        }

        if let VibesEvent::SessionCreated { session_id, .. } = &events[2].1 {
            assert_eq!(session_id, "session-4");
        } else {
            panic!("Expected SessionCreated event");
        }
    }

    #[tokio::test]
    async fn load_historical_events_returns_all_when_fewer_than_requested() {
        use crate::AppState;

        let state = AppState::new();

        // Append only 2 events
        for i in 0..2 {
            let event = VibesEvent::SessionCreated {
                session_id: format!("session-{}", i),
                name: None,
            };
            state.event_log.append(event).await.unwrap();
        }

        // Request 100 events but only 2 exist
        let events = load_historical_events(&state, 100).await;

        assert_eq!(events.len(), 2);
        // Offsets should be 0 and 1
        assert_eq!(events[0].0, 0);
        assert_eq!(events[1].0, 1);
    }

    #[tokio::test]
    async fn load_historical_events_returns_empty_when_no_events() {
        use crate::AppState;

        let state = AppState::new();

        let events = load_historical_events(&state, 100).await;

        assert!(events.is_empty());
    }

    #[test]
    fn event_type_name_returns_correct_names() {
        let event = VibesEvent::Claude {
            session_id: "test".to_string(),
            event: ClaudeEvent::TurnStart,
        };
        assert_eq!(event_type_name(&event), "Claude");

        let event = VibesEvent::SessionCreated {
            session_id: "test".to_string(),
            name: None,
        };
        assert_eq!(event_type_name(&event), "SessionCreated");

        let event = VibesEvent::UserInput {
            session_id: "test".to_string(),
            content: "hello".to_string(),
            source: Default::default(),
        };
        assert_eq!(event_type_name(&event), "UserInput");
    }

    #[test]
    fn firehose_query_deserializes_with_defaults() {
        let query: FirehoseQuery = serde_json::from_str("{}").unwrap();
        assert!(query.types.is_none());
        assert!(query.session.is_none());
    }

    #[test]
    fn firehose_query_deserializes_with_filters() {
        let query: FirehoseQuery =
            serde_json::from_str(r#"{"types":"Claude,UserInput","session":"sess-123"}"#).unwrap();
        assert_eq!(query.types, Some("Claude,UserInput".to_string()));
        assert_eq!(query.session, Some("sess-123".to_string()));
    }
}
