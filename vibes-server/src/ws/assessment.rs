//! WebSocket handler for assessment event streaming
//!
//! Provides a read-only WebSocket endpoint that streams assessment events
//! from plugins, optionally filtered by session ID.

use std::sync::Arc;

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
use vibes_plugin_api::{AssessmentQuery as PluginQuery, PluginAssessmentResult};

use crate::AppState;

/// Query parameters for assessment firehose filtering
#[derive(Debug, Deserialize)]
pub struct AssessmentQueryParams {
    /// Filter by session ID
    #[serde(default)]
    pub session: Option<String>,
}

/// Server-to-client message: single live assessment event
#[derive(Debug, Serialize)]
pub struct AssessmentEventMessage {
    /// Message type discriminator
    #[serde(rename = "type")]
    pub msg_type: &'static str,
    /// The assessment result from plugin
    #[serde(flatten)]
    pub result: PluginAssessmentResult,
}

impl AssessmentEventMessage {
    pub fn new(result: PluginAssessmentResult) -> Self {
        Self {
            msg_type: "assessment_event",
            result,
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
    pub events: Vec<PluginAssessmentResult>,
    /// The oldest event ID in this batch (for pagination)
    pub oldest_event_id: Option<String>,
    /// Whether there are more events before this batch
    pub has_more: bool,
}

impl AssessmentEventsBatch {
    pub fn new(
        events: Vec<PluginAssessmentResult>,
        oldest_event_id: Option<String>,
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
        before_event_id: Option<String>,
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
const DEFAULT_FETCH_LIMIT: usize = 100;

/// WebSocket upgrade handler for assessment firehose
pub async fn assessment_ws(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(query): Query<AssessmentQueryParams>,
) -> Response {
    ws.on_upgrade(move |socket| handle_assessment(socket, state, query))
}

async fn handle_assessment(socket: WebSocket, state: Arc<AppState>, query: AssessmentQueryParams) {
    let (mut sender, mut receiver) = socket.split();

    // Parse filter from query params
    let mut filter_session = query.session.clone();

    tracing::debug!(
        ?filter_session,
        "Assessment WebSocket connection established"
    );

    // Subscribe to broadcast BEFORE client can request events to avoid gaps
    let mut results_rx = state.subscribe_assessment_results();

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
                            &mut filter_session,
                        ).await {
                            warn!("Failed to handle assessment client message: {}", e);
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {} // Ignore other message types
                }
            }

            // Handle broadcast events
            result = results_rx.recv() => {
                match result {
                    Ok(event) => {
                        // Apply session filter
                        if !matches_session_filter(&event, &filter_session) {
                            continue;
                        }

                        let msg = AssessmentEventMessage::new(event);
                        if let Err(e) = send_json(&mut sender, &msg).await {
                            warn!("Assessment send failed: {}", e);
                            break;
                        }
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
    state: &Arc<AppState>,
    sender: &mut S,
    filter_session: &mut Option<String>,
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
            let count = limit.map(|l| l as usize).unwrap_or(DEFAULT_FETCH_LIMIT);

            // Query historical events from plugins
            let response = {
                let mut plugin_host = state.plugin_host().write().await;
                let query = PluginQuery {
                    session_id: filter_session.clone(),
                    result_types: vec![],
                    limit: count,
                    after_event_id: before_event_id,
                    newest_first: true,
                };
                plugin_host.dispatch_query_assessment(query)
            };

            let batch = AssessmentEventsBatch::new(
                response.results,
                response.oldest_event_id,
                response.has_more,
            );

            send_json(sender, &batch)
                .await
                .map_err(|e| format!("Failed to send batch: {}", e))?;
        }

        AssessmentClientMessage::SetFilters { session } => {
            *filter_session = session;

            // Query fresh batch of recent events with new filters
            let response = {
                let mut plugin_host = state.plugin_host().write().await;
                let query = PluginQuery {
                    session_id: filter_session.clone(),
                    result_types: vec![],
                    limit: DEFAULT_FETCH_LIMIT,
                    after_event_id: None,
                    newest_first: true,
                };
                plugin_host.dispatch_query_assessment(query)
            };

            let batch = AssessmentEventsBatch::new(
                response.results,
                response.oldest_event_id,
                response.has_more,
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

/// Check if an event matches the session filter
fn matches_session_filter(
    result: &PluginAssessmentResult,
    filter_session: &Option<String>,
) -> bool {
    if let Some(session) = filter_session {
        result.session_id == *session
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assessment_query_deserializes_with_defaults() {
        let query: AssessmentQueryParams = serde_json::from_str("{}").unwrap();
        assert!(query.session.is_none());
    }

    #[test]
    fn assessment_query_deserializes_with_session() {
        let query: AssessmentQueryParams =
            serde_json::from_str(r#"{"session":"sess-123"}"#).unwrap();
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
        let batch = AssessmentEventsBatch::new(vec![], Some("evt-123".to_string()), true);
        let json = serde_json::to_string(&batch).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["type"], "assessment_events_batch");
        assert!(parsed["events"].is_array());
        assert!(parsed["oldest_event_id"].is_string());
        assert_eq!(parsed["has_more"], true);
    }

    #[test]
    fn matches_session_filter_with_no_filter() {
        let result = PluginAssessmentResult::lightweight("session-1", "{}");
        assert!(matches_session_filter(&result, &None));
    }

    #[test]
    fn matches_session_filter_with_matching_session() {
        let result = PluginAssessmentResult::lightweight("session-1", "{}");
        assert!(matches_session_filter(
            &result,
            &Some("session-1".to_string())
        ));
    }

    #[test]
    fn matches_session_filter_with_non_matching_session() {
        let result = PluginAssessmentResult::lightweight("session-1", "{}");
        assert!(!matches_session_filter(
            &result,
            &Some("session-2".to_string())
        ));
    }
}
