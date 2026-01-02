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

/// Event with offset - used inside batches (no type tag)
#[derive(Debug, Clone, Serialize)]
pub struct EventWithOffset {
    /// The event offset in the EventLog
    pub offset: Offset,
    /// The event data
    pub event: VibesEvent,
}

/// Server-to-client message: single live event with offset
#[derive(Debug, Serialize)]
pub struct FirehoseEventMessage {
    /// Message type discriminator
    #[serde(rename = "type")]
    pub msg_type: &'static str,
    /// The event offset in the EventLog
    pub offset: Offset,
    /// The event data
    pub event: VibesEvent,
}

impl FirehoseEventMessage {
    pub fn new(offset: Offset, event: VibesEvent) -> Self {
        Self {
            msg_type: "event",
            offset,
            event,
        }
    }
}

/// Server-to-client message: batch of events (initial load or pagination)
#[derive(Debug, Serialize)]
pub struct FirehoseEventsBatch {
    /// Message type discriminator
    #[serde(rename = "type")]
    pub msg_type: &'static str,
    /// The events in this batch with their offsets
    pub events: Vec<EventWithOffset>,
    /// The oldest offset in this batch (for pagination)
    pub oldest_offset: Option<Offset>,
    /// Whether there are more events before this batch
    pub has_more: bool,
}

impl FirehoseEventsBatch {
    pub fn new(
        events: Vec<EventWithOffset>,
        oldest_offset: Option<Offset>,
        has_more: bool,
    ) -> Self {
        Self {
            msg_type: "events_batch",
            events,
            oldest_offset,
            has_more,
        }
    }
}

/// Client-to-server messages for the firehose WebSocket
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FirehoseClientMessage {
    /// Request older events before a given offset
    FetchOlder {
        /// Load events before this offset (exclusive)
        before_offset: Offset,
        /// Maximum number of events to return (default: 100)
        #[serde(default)]
        limit: Option<u64>,
    },
    /// Update the active filters for this connection
    SetFilters {
        /// Filter by event types (e.g., ["Claude", "SessionCreated"])
        #[serde(default)]
        types: Option<Vec<String>>,
        /// Filter by session ID
        #[serde(default)]
        session: Option<String>,
    },
}

/// Default limit for fetch_older requests
const DEFAULT_FETCH_LIMIT: u64 = 100;

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

    // Parse filter types (mutable for dynamic filter updates)
    let mut filter_types: Option<Vec<String>> = query.types.as_ref().map(|t| {
        t.split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    });
    let mut filter_session = query.session.clone();

    // Subscribe to broadcast BEFORE loading historical events to avoid gaps
    let mut events_rx = state.subscribe_events();

    // Load historical events from EventLog (with offsets)
    let historical_events = load_historical_events(&state, HISTORICAL_EVENT_COUNT).await;

    // Determine if there's more history before the oldest loaded event
    let oldest_offset = historical_events.first().map(|(offset, _)| *offset);
    let has_more = oldest_offset.is_some_and(|o| o > 0);

    // Send initial batch of historical events
    let filtered_events: Vec<_> = historical_events
        .into_iter()
        .filter(|(_, event)| matches_filters(event, &filter_types, &filter_session))
        .map(|(offset, event)| EventWithOffset { offset, event })
        .collect();

    let batch = FirehoseEventsBatch::new(
        filtered_events.clone(),
        filtered_events.first().map(|e| e.offset),
        has_more,
    );

    if let Err(e) = send_json(&mut sender, &batch).await {
        warn!("Firehose send failed (initial batch): {}", e);
        return;
    }

    // Main event loop: handle both incoming messages and broadcast events
    loop {
        tokio::select! {
            // Handle incoming client messages
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(e) = handle_client_message(
                            &text,
                            &state,
                            &mut sender,
                            &mut filter_types,
                            &mut filter_session,
                        ).await {
                            warn!("Failed to handle client message: {}", e);
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {} // Ignore other message types
                }
            }

            // Handle broadcast events
            result = events_rx.recv() => {
                match result {
                    Ok((offset, event)) => {
                        if !matches_filters(&event, &filter_types, &filter_session) {
                            continue;
                        }

                        let msg = FirehoseEventMessage::new(offset, event);
                        if let Err(e) = send_json(&mut sender, &msg).await {
                            warn!("Firehose send failed: {}", e);
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("Firehose client lagged by {} events", n);
                    }
                }
            }
        }
    }
}

/// Handle a client message from the WebSocket
async fn handle_client_message<S>(
    text: &str,
    state: &AppState,
    sender: &mut S,
    filter_types: &mut Option<Vec<String>>,
    filter_session: &mut Option<String>,
) -> Result<(), String>
where
    S: SinkExt<Message> + Unpin,
    S::Error: std::fmt::Display,
{
    let msg: FirehoseClientMessage =
        serde_json::from_str(text).map_err(|e| format!("Invalid message: {}", e))?;

    match msg {
        FirehoseClientMessage::FetchOlder {
            before_offset,
            limit,
        } => {
            let count = limit.unwrap_or(DEFAULT_FETCH_LIMIT);
            let events = load_events_before_offset(state, before_offset, count).await;

            let oldest_offset = events.first().map(|(offset, _)| *offset);
            let has_more = oldest_offset.is_some_and(|o| o > 0);

            let filtered_events: Vec<_> = events
                .into_iter()
                .filter(|(_, event)| matches_filters(event, filter_types, filter_session))
                .map(|(offset, event)| EventWithOffset { offset, event })
                .collect();

            let batch = FirehoseEventsBatch::new(
                filtered_events.clone(),
                filtered_events.first().map(|e| e.offset),
                has_more,
            );

            send_json(sender, &batch)
                .await
                .map_err(|e| format!("Failed to send batch: {}", e))?;
        }

        FirehoseClientMessage::SetFilters { types, session } => {
            // Update filters - normalize types to lowercase
            *filter_types = types.map(|t| t.into_iter().map(|s| s.to_lowercase()).collect());
            *filter_session = session;

            // Send fresh batch of latest events with new filters
            let events = load_historical_events(state, HISTORICAL_EVENT_COUNT).await;

            let oldest_offset = events.first().map(|(offset, _)| *offset);
            let has_more = oldest_offset.is_some_and(|o| o > 0);

            let filtered_events: Vec<_> = events
                .into_iter()
                .filter(|(_, event)| matches_filters(event, filter_types, filter_session))
                .map(|(offset, event)| EventWithOffset { offset, event })
                .collect();

            let batch = FirehoseEventsBatch::new(
                filtered_events.clone(),
                filtered_events.first().map(|e| e.offset),
                has_more,
            );

            send_json(sender, &batch)
                .await
                .map_err(|e| format!("Failed to send batch: {}", e))?;
        }
    }

    Ok(())
}

/// Helper to serialize and send a JSON message
async fn send_json<S, T>(sender: &mut S, msg: &T) -> Result<(), String>
where
    S: SinkExt<Message> + Unpin,
    S::Error: std::fmt::Display,
    T: Serialize,
{
    let json = serde_json::to_string(msg).map_err(|e| format!("Serialize error: {}", e))?;
    sender
        .send(Message::Text(json))
        .await
        .map_err(|e| format!("Send error: {}", e))
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

/// Load events before a specific offset from the EventLog.
///
/// Returns events in chronological order (oldest first) with their offsets.
/// Used for pagination - fetching older events when scrolling up.
async fn load_events_before_offset(
    state: &AppState,
    before_offset: Offset,
    count: u64,
) -> Vec<(Offset, VibesEvent)> {
    if before_offset == 0 {
        return Vec::new();
    }

    // Calculate the range to fetch
    let end_offset = before_offset; // exclusive
    let start_offset = end_offset.saturating_sub(count);

    // Create a temporary consumer
    let consumer_group = format!("firehose-pagination-{}", uuid::Uuid::new_v4());
    let mut consumer = match state.event_log.consumer(&consumer_group).await {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to create pagination consumer: {}", e);
            return Vec::new();
        }
    };

    // Seek to the start position
    if let Err(e) = consumer.seek(SeekPosition::Offset(start_offset)).await {
        warn!("Failed to seek pagination consumer: {}", e);
        return Vec::new();
    }

    // Calculate how many events to fetch (may be fewer if start_offset was clamped)
    let events_to_fetch = (end_offset - start_offset) as usize;

    // Poll for events
    match consumer
        .poll(events_to_fetch, Duration::from_millis(100))
        .await
    {
        Ok(batch) => batch
            .into_iter()
            .filter(|(offset, _)| *offset < before_offset)
            .collect(),
        Err(e) => {
            warn!("Failed to poll pagination events: {}", e);
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vibes_core::ClaudeEvent;

    // Serialization format tests - prevent regression of message format bugs

    #[test]
    fn events_batch_serializes_without_nested_type_tags() {
        // CRITICAL: Events inside batches must NOT have "type": "event" nested inside them
        // This was a bug where serde's tag attribute incorrectly tagged nested events
        let events = vec![
            EventWithOffset {
                offset: 10,
                event: VibesEvent::ClientConnected {
                    client_id: "c1".to_string(),
                },
            },
            EventWithOffset {
                offset: 11,
                event: VibesEvent::SessionCreated {
                    session_id: "s1".to_string(),
                    name: Some("Test".to_string()),
                },
            },
        ];

        let batch = FirehoseEventsBatch::new(events, Some(10), true);
        let json = serde_json::to_string(&batch).unwrap();

        // Parse and verify structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Top-level type should be "events_batch"
        assert_eq!(parsed["type"], "events_batch");

        // Events array should exist
        let events_arr = parsed["events"].as_array().unwrap();
        assert_eq!(events_arr.len(), 2);

        // Each event should have offset and event fields, but NO top-level "type" in the wrapper
        let first = &events_arr[0];
        assert_eq!(first["offset"], 10);
        assert!(first["event"].is_object());
        // The wrapper itself should not have a "type" field (only the inner event does)
        assert!(
            first.get("type").is_none(),
            "EventWithOffset should not have a 'type' field - only the inner event should"
        );

        // The inner event should have its own type tag (from VibesEvent's serde)
        assert_eq!(first["event"]["type"], "client_connected");
    }

    #[test]
    fn single_event_message_serializes_with_type_at_top_level() {
        let msg = FirehoseEventMessage::new(
            42,
            VibesEvent::ClientConnected {
                client_id: "c1".to_string(),
            },
        );
        let json = serde_json::to_string(&msg).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Top-level type should be "event" (the message type)
        assert_eq!(parsed["type"], "event");
        assert_eq!(parsed["offset"], 42);

        // The event field should contain the actual event with its own type
        assert_eq!(parsed["event"]["type"], "client_connected");
        assert_eq!(parsed["event"]["client_id"], "c1");
    }

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

    #[test]
    fn firehose_client_message_fetch_older_deserializes() {
        let msg: FirehoseClientMessage =
            serde_json::from_str(r#"{"type":"fetch_older","before_offset":100,"limit":50}"#)
                .unwrap();
        match msg {
            FirehoseClientMessage::FetchOlder {
                before_offset,
                limit,
            } => {
                assert_eq!(before_offset, 100);
                assert_eq!(limit, Some(50));
            }
            _ => panic!("Expected FetchOlder"),
        }
    }

    #[test]
    fn firehose_client_message_fetch_older_with_default_limit() {
        let msg: FirehoseClientMessage =
            serde_json::from_str(r#"{"type":"fetch_older","before_offset":100}"#).unwrap();
        match msg {
            FirehoseClientMessage::FetchOlder {
                before_offset,
                limit,
            } => {
                assert_eq!(before_offset, 100);
                assert_eq!(limit, None);
            }
            _ => panic!("Expected FetchOlder"),
        }
    }

    #[tokio::test]
    async fn load_events_before_offset_returns_events_before_given_offset() {
        use crate::AppState;

        let state = AppState::new();

        // Append 10 events (offsets 0-9)
        for i in 0..10 {
            let event = VibesEvent::SessionCreated {
                session_id: format!("session-{}", i),
                name: Some(format!("Session {}", i)),
            };
            state.event_log.append(event).await.unwrap();
        }

        // Load 3 events before offset 7 (should get offsets 4, 5, 6)
        let events = load_events_before_offset(&state, 7, 3).await;

        assert_eq!(events.len(), 3);

        // Verify offsets are 4, 5, 6 (in chronological order)
        assert_eq!(events[0].0, 4);
        assert_eq!(events[1].0, 5);
        assert_eq!(events[2].0, 6);

        // Verify they're the correct events
        if let VibesEvent::SessionCreated { session_id, .. } = &events[0].1 {
            assert_eq!(session_id, "session-4");
        } else {
            panic!("Expected SessionCreated event");
        }
    }

    #[tokio::test]
    async fn load_events_before_offset_handles_beginning_of_log() {
        use crate::AppState;

        let state = AppState::new();

        // Append 5 events (offsets 0-4)
        for i in 0..5 {
            let event = VibesEvent::SessionCreated {
                session_id: format!("session-{}", i),
                name: None,
            };
            state.event_log.append(event).await.unwrap();
        }

        // Load 100 events before offset 3 (should only get offsets 0, 1, 2)
        let events = load_events_before_offset(&state, 3, 100).await;

        assert_eq!(events.len(), 3);
        assert_eq!(events[0].0, 0);
        assert_eq!(events[1].0, 1);
        assert_eq!(events[2].0, 2);
    }

    #[tokio::test]
    async fn load_events_before_offset_returns_empty_when_at_beginning() {
        use crate::AppState;

        let state = AppState::new();

        // Append some events
        for i in 0..5 {
            let event = VibesEvent::SessionCreated {
                session_id: format!("session-{}", i),
                name: None,
            };
            state.event_log.append(event).await.unwrap();
        }

        // Load events before offset 0 (should be empty)
        let events = load_events_before_offset(&state, 0, 100).await;

        assert!(events.is_empty());
    }

    #[test]
    fn firehose_client_message_set_filters_deserializes() {
        let msg: FirehoseClientMessage = serde_json::from_str(
            r#"{"type":"set_filters","types":["Claude","SessionCreated"],"session":"sess-123"}"#,
        )
        .unwrap();
        match msg {
            FirehoseClientMessage::SetFilters { types, session } => {
                assert_eq!(
                    types,
                    Some(vec!["Claude".to_string(), "SessionCreated".to_string()])
                );
                assert_eq!(session, Some("sess-123".to_string()));
            }
            _ => panic!("Expected SetFilters"),
        }
    }

    #[test]
    fn firehose_client_message_set_filters_with_empty_filters() {
        let msg: FirehoseClientMessage = serde_json::from_str(r#"{"type":"set_filters"}"#).unwrap();
        match msg {
            FirehoseClientMessage::SetFilters { types, session } => {
                assert!(types.is_none());
                assert!(session.is_none());
            }
            _ => panic!("Expected SetFilters"),
        }
    }

    #[test]
    fn firehose_client_message_set_filters_clears_with_null() {
        let msg: FirehoseClientMessage =
            serde_json::from_str(r#"{"type":"set_filters","types":null,"session":null}"#).unwrap();
        match msg {
            FirehoseClientMessage::SetFilters { types, session } => {
                assert!(types.is_none());
                assert!(session.is_none());
            }
            _ => panic!("Expected SetFilters"),
        }
    }
}
