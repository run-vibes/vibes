//! WebSocket handler for assessment event streaming
//!
//! Provides a read-only WebSocket endpoint that streams AssessmentEvents
//! from the assessment pipeline, optionally filtered by session ID.

use std::sync::Arc;

use axum::{
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use chrono::{Duration, Utc};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::warn;
use uuid::Uuid;
use vibes_groove::assessment::AssessmentEvent;

use crate::AppState;

/// Query parameters for assessment firehose filtering
#[derive(Debug, Deserialize)]
pub struct AssessmentQuery {
    /// Filter by session ID
    #[serde(default)]
    pub session: Option<String>,
}

/// Assessment event with its ID - used in batches
#[derive(Debug, Clone, Serialize)]
pub struct AssessmentEventWithId {
    /// The event ID (from the assessment event)
    pub event_id: Uuid,
    /// The assessment event data
    #[serde(flatten)]
    pub event: AssessmentEvent,
}

impl AssessmentEventWithId {
    pub fn new(event: AssessmentEvent) -> Self {
        Self {
            event_id: event.event_id().as_uuid(),
            event,
        }
    }
}

/// Server-to-client message: single live assessment event
#[derive(Debug, Serialize)]
pub struct AssessmentEventMessage {
    /// Message type discriminator
    #[serde(rename = "type")]
    pub msg_type: &'static str,
    /// The event ID
    pub event_id: Uuid,
    /// The assessment event data
    #[serde(flatten)]
    pub event: AssessmentEvent,
}

impl AssessmentEventMessage {
    pub fn new(event: AssessmentEvent) -> Self {
        Self {
            msg_type: "assessment_event",
            event_id: event.event_id().as_uuid(),
            event,
        }
    }
}

/// Server-to-client message: batch of assessment events
#[derive(Debug, Serialize)]
pub struct AssessmentEventsBatch {
    /// Message type discriminator
    #[serde(rename = "type")]
    pub msg_type: &'static str,
    /// The events in this batch
    pub events: Vec<AssessmentEventWithId>,
    /// The oldest event ID in this batch (for pagination)
    pub oldest_event_id: Option<Uuid>,
    /// Whether there are more events before this batch
    pub has_more: bool,
}

impl AssessmentEventsBatch {
    pub fn new(
        events: Vec<AssessmentEventWithId>,
        oldest_event_id: Option<Uuid>,
        has_more: bool,
    ) -> Self {
        Self {
            msg_type: "assessment_events_batch",
            events,
            oldest_event_id,
            has_more,
        }
    }
}

/// Client-to-server messages for the assessment WebSocket
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AssessmentClientMessage {
    /// Request older events before a given event ID
    FetchOlder {
        /// Load events before this event ID (exclusive)
        #[serde(default)]
        before_event_id: Option<Uuid>,
        /// Maximum number of events to return (default: 100)
        #[serde(default)]
        limit: Option<u64>,
    },
    /// Update the active filters for this connection
    SetFilters {
        /// Filter by session ID
        #[serde(default)]
        session: Option<String>,
    },
}

/// Default limit for fetch_older requests
const DEFAULT_FETCH_LIMIT: u64 = 100;

/// Duration for initial historical event load (last 24 hours)
const HISTORICAL_DURATION_HOURS: i64 = 24;

/// WebSocket upgrade handler for assessment firehose
pub async fn assessment_ws(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(query): Query<AssessmentQuery>,
) -> Response {
    ws.on_upgrade(move |socket| handle_assessment(socket, state, query))
}

async fn handle_assessment(socket: WebSocket, state: Arc<AppState>, query: AssessmentQuery) {
    let (mut sender, mut receiver) = socket.split();

    // Get the assessment log - if not available, close the connection
    let assessment_log = match state.assessment_log().await {
        Some(log) => log,
        None => {
            warn!("Assessment WebSocket: No assessment log available");
            let _ = sender.send(Message::Close(None)).await;
            return;
        }
    };

    // Parse filter from query params
    let mut filter_session = query.session.clone();

    tracing::debug!(
        ?filter_session,
        "Assessment WebSocket connection established"
    );

    // Subscribe to broadcast BEFORE client can request events to avoid gaps
    let mut events_rx = assessment_log.subscribe();

    // Track the last event ID we've sent to prevent duplicates
    let mut last_sent_event_id: Option<Uuid> = None;

    // Main event loop: handle both incoming messages and broadcast events
    loop {
        tokio::select! {
            // Handle incoming client messages
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(e) = handle_client_message(
                            &text,
                            &assessment_log,
                            &mut sender,
                            &mut filter_session,
                            &mut last_sent_event_id,
                        ).await {
                            warn!("Failed to handle assessment client message: {}", e);
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {} // Ignore other message types
                }
            }

            // Handle broadcast events
            result = events_rx.recv() => {
                match result {
                    Ok(event) => {
                        // Skip events we already sent
                        if let Some(last_id) = last_sent_event_id
                            && event.event_id().as_uuid() <= last_id
                        {
                            continue;
                        }

                        // Apply session filter
                        if !matches_session_filter(&event, &filter_session) {
                            continue;
                        }

                        let msg = AssessmentEventMessage::new(event.clone());
                        if let Err(e) = send_json(&mut sender, &msg).await {
                            warn!("Assessment send failed: {}", e);
                            break;
                        }
                        last_sent_event_id = Some(event.event_id().as_uuid());
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("Assessment client lagged by {} events", n);
                    }
                }
            }
        }
    }
}

/// Handle a client message from the WebSocket
async fn handle_client_message<S>(
    text: &str,
    assessment_log: &Arc<dyn vibes_groove::assessment::AssessmentLog>,
    sender: &mut S,
    filter_session: &mut Option<String>,
    last_sent_event_id: &mut Option<Uuid>,
) -> Result<(), String>
where
    S: SinkExt<Message> + Unpin,
    S::Error: std::fmt::Display,
{
    let msg: AssessmentClientMessage =
        serde_json::from_str(text).map_err(|e| format!("Invalid message: {}", e))?;

    match msg {
        AssessmentClientMessage::FetchOlder {
            before_event_id,
            limit,
        } => {
            let count = limit.unwrap_or(DEFAULT_FETCH_LIMIT) as usize;

            // Load events - either before a specific event or recent history
            let events = if let Some(event_id) = before_event_id {
                load_events_before(assessment_log, event_id, count, filter_session).await
            } else {
                load_recent_events(assessment_log, count, filter_session).await
            };

            let oldest_event_id = events.first().map(|e| e.event_id);
            let newest_event_id = events.last().map(|e| e.event_id);
            let has_more = !events.is_empty(); // Simplified - assume more if we got any

            let batch = AssessmentEventsBatch::new(events, oldest_event_id, has_more);

            send_json(sender, &batch)
                .await
                .map_err(|e| format!("Failed to send batch: {}", e))?;

            // Update last_sent_event_id
            if let Some(id) = newest_event_id {
                *last_sent_event_id = Some(last_sent_event_id.unwrap_or(Uuid::nil()).max(id));
            }
        }

        AssessmentClientMessage::SetFilters { session } => {
            *filter_session = session;

            // Send fresh batch of recent events with new filters
            let events =
                load_recent_events(assessment_log, DEFAULT_FETCH_LIMIT as usize, filter_session)
                    .await;

            let oldest_event_id = events.first().map(|e| e.event_id);
            let newest_event_id = events.last().map(|e| e.event_id);
            let has_more = !events.is_empty();

            let batch = AssessmentEventsBatch::new(events, oldest_event_id, has_more);

            send_json(sender, &batch)
                .await
                .map_err(|e| format!("Failed to send batch: {}", e))?;

            // Update last_sent_event_id
            if let Some(id) = newest_event_id {
                *last_sent_event_id = Some(last_sent_event_id.unwrap_or(Uuid::nil()).max(id));
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

/// Check if an event matches the session filter
fn matches_session_filter(event: &AssessmentEvent, filter_session: &Option<String>) -> bool {
    if let Some(session) = filter_session {
        event.session_id().as_str() == session
    } else {
        true
    }
}

/// Load recent assessment events (last N hours)
async fn load_recent_events(
    log: &Arc<dyn vibes_groove::assessment::AssessmentLog>,
    limit: usize,
    filter_session: &Option<String>,
) -> Vec<AssessmentEventWithId> {
    let end = Utc::now();
    let start = end - Duration::hours(HISTORICAL_DURATION_HOURS);

    let events = match log.read_range(start, end).await {
        Ok(events) => events,
        Err(e) => {
            warn!("Failed to load recent assessment events: {}", e);
            return Vec::new();
        }
    };

    // Filter by session if specified, then take last N
    let filtered: Vec<_> = events
        .into_iter()
        .filter(|e| matches_session_filter(e, filter_session))
        .collect();

    // Take the last `limit` events (most recent)
    let start_idx = filtered.len().saturating_sub(limit);
    filtered[start_idx..]
        .iter()
        .map(|e| AssessmentEventWithId::new(e.clone()))
        .collect()
}

/// Load events before a specific event ID
async fn load_events_before(
    log: &Arc<dyn vibes_groove::assessment::AssessmentLog>,
    before_event_id: Uuid,
    limit: usize,
    filter_session: &Option<String>,
) -> Vec<AssessmentEventWithId> {
    // Extract timestamp from UUIDv7 to get the time range
    // UUIDv7 has timestamp in the first 48 bits
    let timestamp_ms = (before_event_id.as_u128() >> 80) as i64;
    let before_time =
        chrono::DateTime::from_timestamp_millis(timestamp_ms).unwrap_or_else(Utc::now);

    // Load events from 24 hours before the target event
    let start = before_time - Duration::hours(HISTORICAL_DURATION_HOURS);

    let events = match log.read_range(start, before_time).await {
        Ok(events) => events,
        Err(e) => {
            warn!(
                "Failed to load assessment events before {}: {}",
                before_event_id, e
            );
            return Vec::new();
        }
    };

    // Filter by session and exclude the target event
    let filtered: Vec<_> = events
        .into_iter()
        .filter(|e| {
            e.event_id().as_uuid() < before_event_id && matches_session_filter(e, filter_session)
        })
        .collect();

    // Take the last `limit` events (closest to the target)
    let start_idx = filtered.len().saturating_sub(limit);
    filtered[start_idx..]
        .iter()
        .map(|e| AssessmentEventWithId::new(e.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assessment_query_deserializes_with_defaults() {
        let query: AssessmentQuery = serde_json::from_str("{}").unwrap();
        assert!(query.session.is_none());
    }

    #[test]
    fn assessment_query_deserializes_with_session() {
        let query: AssessmentQuery = serde_json::from_str(r#"{"session":"sess-123"}"#).unwrap();
        assert_eq!(query.session, Some("sess-123".to_string()));
    }

    #[test]
    fn assessment_client_message_fetch_older_deserializes() {
        let msg: AssessmentClientMessage = serde_json::from_str(
            r#"{"type":"fetch_older","before_event_id":"01936f8a-1234-7000-8000-000000000001","limit":50}"#,
        )
        .unwrap();
        match msg {
            AssessmentClientMessage::FetchOlder {
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
    fn assessment_client_message_set_filters_deserializes() {
        let msg: AssessmentClientMessage =
            serde_json::from_str(r#"{"type":"set_filters","session":"sess-123"}"#).unwrap();
        match msg {
            AssessmentClientMessage::SetFilters { session } => {
                assert_eq!(session, Some("sess-123".to_string()));
            }
            _ => panic!("Expected SetFilters"),
        }
    }

    #[test]
    fn assessment_client_message_set_filters_with_null() {
        let msg: AssessmentClientMessage =
            serde_json::from_str(r#"{"type":"set_filters","session":null}"#).unwrap();
        match msg {
            AssessmentClientMessage::SetFilters { session } => {
                assert!(session.is_none());
            }
            _ => panic!("Expected SetFilters"),
        }
    }

    #[test]
    fn assessment_events_batch_serializes_correctly() {
        let batch = AssessmentEventsBatch::new(vec![], Some(Uuid::nil()), true);
        let json = serde_json::to_string(&batch).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["type"], "assessment_events_batch");
        assert!(parsed["events"].is_array());
        assert!(parsed["oldest_event_id"].is_string());
        assert_eq!(parsed["has_more"], true);
    }
}
