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
use uuid::Uuid;
use vibes_core::{StoredEvent, VibesEvent};
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
    /// Globally unique, time-ordered event identifier (UUIDv7)
    pub event_id: Uuid,
    /// The event offset in the EventLog (partition-scoped, use event_id for uniqueness)
    pub offset: Offset,
    /// The event data (nested to match FirehoseEventMessage structure)
    pub event: VibesEvent,
}

impl EventWithOffset {
    pub fn new(stored: StoredEvent, offset: Offset) -> Self {
        Self {
            event_id: stored.event_id,
            offset,
            event: stored.event,
        }
    }
}

/// Server-to-client message: single live event with offset
#[derive(Debug, Serialize)]
pub struct FirehoseEventMessage {
    /// Message type discriminator
    #[serde(rename = "type")]
    pub msg_type: &'static str,
    /// Globally unique, time-ordered event identifier (UUIDv7)
    pub event_id: Uuid,
    /// The event offset in the EventLog (partition-scoped, use event_id for uniqueness)
    pub offset: Offset,
    /// The event data (nested to avoid type field conflict)
    pub event: VibesEvent,
}

impl FirehoseEventMessage {
    pub fn new(offset: Offset, stored: StoredEvent) -> Self {
        Self {
            msg_type: "event",
            event_id: stored.event_id,
            offset,
            event: stored.event,
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
    /// The oldest event ID in this batch (for pagination)
    pub oldest_event_id: Option<Uuid>,
    /// Whether there are more events before this batch
    pub has_more: bool,
}

impl FirehoseEventsBatch {
    pub fn new(
        events: Vec<EventWithOffset>,
        oldest_event_id: Option<Uuid>,
        has_more: bool,
    ) -> Self {
        Self {
            msg_type: "events_batch",
            events,
            oldest_event_id,
            has_more,
        }
    }
}

/// Client-to-server messages for the firehose WebSocket
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FirehoseClientMessage {
    /// Request older events before a given event ID
    FetchOlder {
        /// Load events before this event ID (exclusive) - UUIDv7 for time-ordering
        #[serde(default)]
        before_event_id: Option<Uuid>,
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

    // Parse filter types from query params (mutable for dynamic filter updates)
    let mut filter_types: Option<Vec<String>> = query.types.as_ref().map(|t| {
        t.split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    });
    let mut filter_session = query.session.clone();

    tracing::debug!(
        ?filter_types,
        ?filter_session,
        "Firehose connection established (waiting for client to request events)"
    );

    // Subscribe to broadcast BEFORE client can request events to avoid gaps.
    // Broadcast events received before the client requests initial events will
    // be deduplicated using last_sent_offset.
    let mut events_rx = state.subscribe_events();

    // Track the highest offset we've sent to prevent duplicates.
    // Updated when we send historical batches; broadcast events at or below
    // this offset are skipped (they were included in the historical batch).
    let mut last_sent_offset: Option<Offset> = None;

    // NOTE: We do NOT automatically load/send historical events on connection.
    // The client must send a set_filters message (even with empty filters) to
    // initiate the first load. This ensures a single code path and prevents
    // race conditions between automatic load and client filter updates.

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
                            &mut last_sent_offset,
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
                    Ok((offset, stored)) => {
                        // Skip events we already sent in a historical batch.
                        // Events written during historical load appear in both the batch
                        // AND the broadcast channel - don't send duplicates.
                        if let Some(last_offset) = last_sent_offset
                            && offset <= last_offset
                        {
                            continue;
                        }

                        if !matches_filters(&stored.event, &filter_types, &filter_session) {
                            continue;
                        }

                        let msg = FirehoseEventMessage::new(offset, stored.clone());
                        if let Err(e) = send_json(&mut sender, &msg).await {
                            warn!("Firehose send failed: {}", e);
                            break;
                        }
                        // Update last_sent_offset so we don't re-send this event
                        last_sent_offset = Some(offset);
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

/// Handle a client message from the WebSocket.
///
/// Updates `last_sent_offset` when sending batches to prevent duplicate
/// events from being sent via the broadcast channel.
async fn handle_client_message<S>(
    text: &str,
    state: &AppState,
    sender: &mut S,
    filter_types: &mut Option<Vec<String>>,
    filter_session: &mut Option<String>,
    last_sent_offset: &mut Option<Offset>,
) -> Result<(), String>
where
    S: SinkExt<Message> + Unpin,
    S::Error: std::fmt::Display,
{
    let msg: FirehoseClientMessage =
        serde_json::from_str(text).map_err(|e| format!("Invalid message: {}", e))?;

    match msg {
        FirehoseClientMessage::FetchOlder {
            before_event_id,
            limit,
        } => {
            let count = limit.unwrap_or(DEFAULT_FETCH_LIMIT);

            // Load events before the specified event_id
            let events = if let Some(event_id) = before_event_id {
                load_events_before_event_id(state, event_id, count).await
            } else {
                // No event_id specified, load from end
                load_historical_events(state, count).await
            };

            let oldest_offset = events.first().map(|(offset, _)| *offset);
            let newest_offset = events.last().map(|(offset, _)| *offset);
            let has_more = oldest_offset.is_some_and(|o| o > 0);

            let filtered_events: Vec<_> = events
                .into_iter()
                .filter(|(_, stored)| matches_filters(&stored.event, filter_types, filter_session))
                .map(|(offset, stored)| EventWithOffset::new(stored, offset))
                .collect();

            let batch = FirehoseEventsBatch::new(
                filtered_events.clone(),
                filtered_events.first().map(|e| e.event_id),
                has_more,
            );

            send_json(sender, &batch)
                .await
                .map_err(|e| format!("Failed to send batch: {}", e))?;

            // Update last_sent_offset to prevent duplicates from broadcast
            if let Some(offset) = newest_offset {
                *last_sent_offset = Some(last_sent_offset.unwrap_or(0).max(offset));
            }
        }

        FirehoseClientMessage::SetFilters { types, session } => {
            // Update filters - normalize types to lowercase
            *filter_types = types.map(|t| t.into_iter().map(|s| s.to_lowercase()).collect());
            *filter_session = session;

            // Send fresh batch of latest events with new filters
            let events = load_historical_events(state, HISTORICAL_EVENT_COUNT).await;

            let oldest_offset = events.first().map(|(offset, _)| *offset);
            let newest_offset = events.last().map(|(offset, _)| *offset);
            let has_more = oldest_offset.is_some_and(|o| o > 0);

            let filtered_events: Vec<_> = events
                .into_iter()
                .filter(|(_, stored)| matches_filters(&stored.event, filter_types, filter_session))
                .map(|(offset, stored)| EventWithOffset::new(stored, offset))
                .collect();

            let batch = FirehoseEventsBatch::new(
                filtered_events.clone(),
                filtered_events.first().map(|e| e.event_id),
                has_more,
            );

            send_json(sender, &batch)
                .await
                .map_err(|e| format!("Failed to send batch: {}", e))?;

            // Update last_sent_offset to prevent duplicates from broadcast
            if let Some(offset) = newest_offset {
                *last_sent_offset = Some(last_sent_offset.unwrap_or(0).max(offset));
            }
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
    // Apply type filter - match against display categories (session, claude, hook, etc.)
    if let Some(types) = filter_types {
        let category = event_display_category(event);
        if !types.iter().any(|t| t == category) {
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

/// Get the display category for an event.
///
/// These categories match the frontend's event type chips (SESSION, CLAUDE, TOOL, HOOK, etc.)
/// rather than the Rust enum variant names.
fn event_display_category(event: &VibesEvent) -> &'static str {
    match event {
        // Session-related events
        VibesEvent::SessionCreated { .. }
        | VibesEvent::SessionStateChanged { .. }
        | VibesEvent::ClientConnected { .. }
        | VibesEvent::ClientDisconnected { .. }
        | VibesEvent::TunnelStateChanged { .. }
        | VibesEvent::OwnershipTransferred { .. }
        | VibesEvent::SessionRemoved { .. } => "session",

        // Claude/AI interaction events
        VibesEvent::Claude { .. }
        | VibesEvent::UserInput { .. }
        | VibesEvent::PermissionResponse { .. } => "claude",

        // Hook events
        VibesEvent::Hook { .. } => "hook",
    }
}

/// Load the last N events from the EventLog.
///
/// Returns stored events in chronological order (oldest first) with their offsets.
///
/// Note: Iggy's poll_messages may return partial batches due to segment boundaries
/// or internal batching limits, so we poll in a loop until we have enough events
/// or reach the end of available messages.
async fn load_historical_events(state: &AppState, count: u64) -> Vec<(Offset, StoredEvent)> {
    let hwm = state.event_log.high_water_mark();
    tracing::debug!(hwm, count, "load_historical_events: starting");
    if hwm == 0 {
        tracing::debug!("load_historical_events: hwm is 0, returning empty");
        return Vec::new();
    }

    // Flush Iggy's in-memory buffer to disk before reading.
    // This is critical for Iggy's io_uring backend (via compio) which uses separate
    // file handles for reading and writing. Without flushing, recent writes may not
    // be visible to the reader.
    if let Err(e) = state.event_log.flush_to_disk().await {
        warn!(
            "Failed to flush event log to disk: {} (continuing anyway)",
            e
        );
    }

    // Create a temporary consumer to read historical events
    let consumer_group = format!("firehose-historical-{}", uuid::Uuid::new_v4());
    let mut consumer = match state.event_log.consumer(&consumer_group).await {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to create historical event consumer: {}", e);
            return Vec::new();
        }
    };

    // Seek backwards from end
    if let Err(e) = consumer.seek(SeekPosition::FromEnd(count)).await {
        warn!("Failed to seek historical consumer: {}", e);
        return Vec::new();
    }
    tracing::debug!("load_historical_events: seeked to FromEnd({})", count);

    // Poll for events in a loop - Iggy may return partial batches
    let mut events: Vec<(Offset, StoredEvent)> = Vec::new();
    let target_count = count as usize;
    let max_iterations = 100; // Safety limit to prevent infinite loops

    for iteration in 0..max_iterations {
        let remaining = target_count.saturating_sub(events.len());
        if remaining == 0 {
            break;
        }

        match consumer.poll(remaining, Duration::from_millis(100)).await {
            Ok(batch) => {
                let batch_len = batch.len();
                tracing::trace!(
                    iteration,
                    batch_len,
                    total = events.len() + batch_len,
                    "load_historical_events: polled batch"
                );

                if batch_len == 0 {
                    // No more events available
                    break;
                }

                events.extend(batch.into_iter());
            }
            Err(e) => {
                warn!("Failed to poll historical events: {}", e);
                break;
            }
        }
    }

    // Sort by event_id (UUIDv7 timestamp) for correct ordering
    events.sort_by_key(|(_, stored)| stored.event_id);

    tracing::debug!(
        final_count = events.len(),
        "load_historical_events: complete"
    );

    events
}

/// Load events before a specific event ID from the EventLog.
///
/// Returns stored events in chronological order (oldest first) with their offsets.
/// Used for pagination - fetching older events when scrolling up.
///
/// This implementation loads events from the beginning and finds the target by event_id.
/// A future optimization could use Iggy's timestamp-based seeking via SeekPosition::BeforeEventId.
async fn load_events_before_event_id(
    state: &AppState,
    before_event_id: Uuid,
    count: u64,
) -> Vec<(Offset, StoredEvent)> {
    // Flush Iggy's in-memory buffer to disk before reading.
    // This is critical for Iggy's io_uring backend (via compio) which uses separate
    // file handles for reading and writing. Without flushing, recent writes may not
    // be visible to the reader.
    if let Err(e) = state.event_log.flush_to_disk().await {
        tracing::warn!(
            "Failed to flush event log to disk: {} (continuing anyway)",
            e
        );
    }

    // Load all events from beginning to find the target event
    let consumer_group = format!("firehose-pagination-{}", uuid::Uuid::new_v4());
    let mut consumer = match state.event_log.consumer(&consumer_group).await {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to create pagination consumer: {}", e);
            return Vec::new();
        }
    };

    // Seek to beginning
    if let Err(e) = consumer.seek(SeekPosition::Beginning).await {
        warn!("Failed to seek pagination consumer: {}", e);
        return Vec::new();
    }

    // Poll all events up to high water mark
    let hwm = state.event_log.high_water_mark();
    if hwm == 0 {
        return Vec::new();
    }

    // Poll in a loop - Iggy may return partial batches
    let mut all_events: Vec<(Offset, StoredEvent)> = Vec::new();
    let max_iterations = 100;

    for _ in 0..max_iterations {
        let remaining = (hwm as usize).saturating_sub(all_events.len());
        if remaining == 0 {
            break;
        }

        match consumer.poll(remaining, Duration::from_millis(100)).await {
            Ok(batch) => {
                if batch.is_empty() {
                    break;
                }
                all_events.extend(batch.into_iter());
            }
            Err(e) => {
                warn!("Failed to poll events for pagination: {}", e);
                break;
            }
        }
    }

    // Find the index of the target event_id
    let target_index = all_events
        .iter()
        .position(|(_, stored)| stored.event_id == before_event_id);

    match target_index {
        Some(0) => Vec::new(), // Target is first event, nothing before it
        Some(idx) => {
            // Return up to `count` events before the target
            let start = idx.saturating_sub(count as usize);
            all_events[start..idx].to_vec()
        }
        None => {
            // Target not found, return empty
            warn!("Event ID {} not found in log", before_event_id);
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vibes_core::ClaudeEvent;

    fn make_stored_event(event: VibesEvent) -> StoredEvent {
        StoredEvent::new(event)
    }

    // Serialization format tests - prevent regression of message format bugs

    #[test]
    fn events_batch_serializes_with_event_id_and_nested_event() {
        // Test that events in batches include event_id and nested event object
        let stored1 = make_stored_event(VibesEvent::ClientConnected {
            client_id: "c1".to_string(),
        });
        let stored2 = make_stored_event(VibesEvent::SessionCreated {
            session_id: "s1".to_string(),
            name: Some("Test".to_string()),
        });

        let oldest_event_id = stored1.event_id;
        let events = vec![
            EventWithOffset::new(stored1.clone(), 10),
            EventWithOffset::new(stored2.clone(), 11),
        ];

        let batch = FirehoseEventsBatch::new(events, Some(oldest_event_id), true);
        let json = serde_json::to_string(&batch).unwrap();

        // Parse and verify structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Top-level type should be "events_batch"
        assert_eq!(parsed["type"], "events_batch");

        // Events array should exist
        let events_arr = parsed["events"].as_array().unwrap();
        assert_eq!(events_arr.len(), 2);

        // Each event should have event_id, offset, and nested event object
        let first = &events_arr[0];
        assert_eq!(first["offset"], 10);
        assert!(first["event_id"].is_string()); // UUIDv7 as string
        // Event should be nested (matching FirehoseEventMessage structure)
        assert_eq!(first["event"]["type"], "client_connected");
        assert_eq!(first["event"]["client_id"], "c1");
    }

    #[test]
    fn single_event_message_serializes_with_event_id_and_nested_event() {
        let stored = make_stored_event(VibesEvent::ClientConnected {
            client_id: "c1".to_string(),
        });
        let event_id = stored.event_id;

        let msg = FirehoseEventMessage::new(42, stored);
        let json = serde_json::to_string(&msg).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Top-level type should be "event" (the message type)
        assert_eq!(parsed["type"], "event");
        assert_eq!(parsed["offset"], 42);
        // event_id should be present at top level
        assert_eq!(parsed["event_id"], event_id.to_string());
        // Event should be nested (not flattened) to avoid type field conflict
        assert_eq!(parsed["event"]["type"], "client_connected");
        assert_eq!(parsed["event"]["client_id"], "c1");
    }

    #[tokio::test]
    async fn load_historical_events_returns_last_n_events_with_offsets() {
        use crate::AppState;

        // Create state with in-memory event log
        let state = AppState::new();

        // Append 5 events to the EventLog (state.append_event wraps in StoredEvent)
        for i in 0..5 {
            let stored = make_stored_event(VibesEvent::SessionCreated {
                session_id: format!("session-{}", i),
                name: Some(format!("Session {}", i)),
            });
            state.event_log.append(stored).await.unwrap();
        }

        // Load last 3 events
        let events = load_historical_events(&state, 3).await;

        // Should get the last 3 events (sessions 2, 3, 4) with offsets
        assert_eq!(events.len(), 3);

        // Verify offsets (should be 2, 3, 4)
        assert_eq!(events[0].0, 2);
        assert_eq!(events[1].0, 3);
        assert_eq!(events[2].0, 4);

        // Verify they're the correct events (last 3) - now accessing .event
        if let VibesEvent::SessionCreated { session_id, .. } = &events[0].1.event {
            assert_eq!(session_id, "session-2");
        } else {
            panic!("Expected SessionCreated event");
        }

        if let VibesEvent::SessionCreated { session_id, .. } = &events[2].1.event {
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
            let stored = make_stored_event(VibesEvent::SessionCreated {
                session_id: format!("session-{}", i),
                name: None,
            });
            state.event_log.append(stored).await.unwrap();
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
    fn event_display_category_returns_correct_categories() {
        // Claude events map to "claude" category
        let event = VibesEvent::Claude {
            session_id: "test".to_string(),
            event: ClaudeEvent::TurnStart,
        };
        assert_eq!(event_display_category(&event), "claude");

        let event = VibesEvent::UserInput {
            session_id: "test".to_string(),
            content: "hello".to_string(),
            source: Default::default(),
        };
        assert_eq!(event_display_category(&event), "claude");

        // Session-related events map to "session" category
        let event = VibesEvent::SessionCreated {
            session_id: "test".to_string(),
            name: None,
        };
        assert_eq!(event_display_category(&event), "session");

        let event = VibesEvent::ClientConnected {
            client_id: "c1".to_string(),
        };
        assert_eq!(event_display_category(&event), "session");

        let event = VibesEvent::TunnelStateChanged {
            state: "connected".to_string(),
            url: None,
        };
        assert_eq!(event_display_category(&event), "session");
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
        // Test the new protocol with before_event_id
        let msg: FirehoseClientMessage = serde_json::from_str(
            r#"{"type":"fetch_older","before_event_id":"01936f8a-1234-7000-8000-000000000001","limit":50}"#,
        )
        .unwrap();
        match msg {
            FirehoseClientMessage::FetchOlder {
                before_event_id,
                limit,
            } => {
                assert!(before_event_id.is_some());
                assert_eq!(limit, Some(50));
            }
            _ => panic!("Expected FetchOlder"),
        }
    }

    #[test]
    fn firehose_client_message_fetch_older_with_default_limit() {
        let msg: FirehoseClientMessage = serde_json::from_str(
            r#"{"type":"fetch_older","before_event_id":"01936f8a-1234-7000-8000-000000000001"}"#,
        )
        .unwrap();
        match msg {
            FirehoseClientMessage::FetchOlder {
                before_event_id,
                limit,
            } => {
                assert!(before_event_id.is_some());
                assert_eq!(limit, None);
            }
            _ => panic!("Expected FetchOlder"),
        }
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

    #[tokio::test]
    async fn load_historical_events_returns_events_sorted_by_event_id() {
        use crate::AppState;

        // Create state with in-memory event log
        let state = AppState::new();

        // Append several events rapidly (they'll get sequential UUIDv7 event_ids)
        for i in 0..10 {
            let stored = make_stored_event(VibesEvent::SessionCreated {
                session_id: format!("session-{}", i),
                name: Some(format!("Session {}", i)),
            });
            state.event_log.append(stored).await.unwrap();
        }

        // Load all events
        let events = load_historical_events(&state, 100).await;
        assert_eq!(events.len(), 10);

        // Verify events are sorted by event_id (UUIDv7 is time-ordered)
        // Each subsequent event_id should be >= previous (UUIDv7 is monotonic)
        for i in 1..events.len() {
            let prev_id = events[i - 1].1.event_id;
            let curr_id = events[i].1.event_id;
            assert!(
                curr_id >= prev_id,
                "Events should be sorted by event_id (UUIDv7 timestamp). \
                 Event {} has id {} which is less than event {} with id {}",
                i,
                curr_id,
                i - 1,
                prev_id
            );
        }
    }

    // ============================================================
    // TDD: Event ID-based pagination tests (new protocol)
    // ============================================================

    #[test]
    fn firehose_client_message_fetch_older_with_event_id_deserializes() {
        // New protocol: fetch_older uses before_event_id instead of before_offset
        let msg: FirehoseClientMessage = serde_json::from_str(
            r#"{"type":"fetch_older","before_event_id":"01936f8a-1234-7000-8000-000000000001","limit":50}"#,
        )
        .unwrap();
        match msg {
            FirehoseClientMessage::FetchOlder {
                before_event_id,
                limit,
            } => {
                assert!(before_event_id.is_some());
                assert_eq!(
                    before_event_id.unwrap().to_string(),
                    "01936f8a-1234-7000-8000-000000000001"
                );
                assert_eq!(limit, Some(50));
            }
            _ => panic!("Expected FetchOlder"),
        }
    }

    #[test]
    fn events_batch_includes_oldest_event_id() {
        // Batch should include oldest_event_id for pagination
        let stored = make_stored_event(VibesEvent::ClientConnected {
            client_id: "c1".to_string(),
        });
        let event_id = stored.event_id;
        let events = vec![EventWithOffset::new(stored, 10)];

        let batch = FirehoseEventsBatch::new(events, Some(event_id), true);
        let json = serde_json::to_string(&batch).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Should have oldest_event_id field
        assert!(parsed["oldest_event_id"].is_string());
        assert_eq!(parsed["oldest_event_id"], event_id.to_string());
    }

    #[tokio::test]
    async fn load_events_before_event_id_returns_events_before_given_event() {
        use crate::AppState;

        let state = AppState::new();

        // Append 10 events and track their event_ids
        let mut event_ids = Vec::new();
        for i in 0..10 {
            let stored = make_stored_event(VibesEvent::SessionCreated {
                session_id: format!("session-{}", i),
                name: Some(format!("Session {}", i)),
            });
            event_ids.push(stored.event_id);
            state.event_log.append(stored).await.unwrap();
        }

        // Load 3 events before event_id[7] (should get events 4, 5, 6)
        let target_event_id = event_ids[7];
        let events = load_events_before_event_id(&state, target_event_id, 3).await;

        assert_eq!(events.len(), 3);

        // Verify we got events 4, 5, 6 (before event 7)
        let loaded_ids: Vec<_> = events.iter().map(|(_, s)| s.event_id).collect();
        assert_eq!(loaded_ids, vec![event_ids[4], event_ids[5], event_ids[6]]);
    }

    #[tokio::test]
    async fn load_events_before_event_id_handles_beginning_of_log() {
        use crate::AppState;

        let state = AppState::new();

        // Append 5 events
        let mut event_ids = Vec::new();
        for i in 0..5 {
            let stored = make_stored_event(VibesEvent::SessionCreated {
                session_id: format!("session-{}", i),
                name: None,
            });
            event_ids.push(stored.event_id);
            state.event_log.append(stored).await.unwrap();
        }

        // Load 100 events before event_id[3] (should only get events 0, 1, 2)
        let events = load_events_before_event_id(&state, event_ids[3], 100).await;

        assert_eq!(events.len(), 3);
        let loaded_ids: Vec<_> = events.iter().map(|(_, s)| s.event_id).collect();
        assert_eq!(loaded_ids, vec![event_ids[0], event_ids[1], event_ids[2]]);
    }

    #[tokio::test]
    async fn load_events_before_event_id_returns_empty_for_first_event() {
        use crate::AppState;

        let state = AppState::new();

        // Append some events
        let mut event_ids = Vec::new();
        for i in 0..5 {
            let stored = make_stored_event(VibesEvent::SessionCreated {
                session_id: format!("session-{}", i),
                name: None,
            });
            event_ids.push(stored.event_id);
            state.event_log.append(stored).await.unwrap();
        }

        // Load events before the first event (should be empty)
        let events = load_events_before_event_id(&state, event_ids[0], 100).await;
        assert!(events.is_empty());
    }

    // ============================================================
    // End TDD tests
    // ============================================================
}
