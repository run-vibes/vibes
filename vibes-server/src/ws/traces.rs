//! WebSocket handler for trace event streaming
//!
//! Provides a WebSocket endpoint that streams trace events from the
//! tracing subscriber to connected CLI clients.

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
use tracing::{debug, warn};
use vibes_observe::TraceEvent;

use crate::AppState;
use crate::ws::protocol::{ClientMessage, ServerMessage};

/// Query parameters for trace filtering
#[derive(Debug, Deserialize)]
pub struct TraceQuery {
    /// Filter by session ID (prefix match)
    #[serde(default)]
    pub session: Option<String>,
    /// Filter by agent ID
    #[serde(default)]
    pub agent: Option<String>,
    /// Minimum log level (trace, debug, info, warn, error)
    #[serde(default)]
    pub level: Option<String>,
}

/// WebSocket upgrade handler for traces
pub async fn traces_ws(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(query): Query<TraceQuery>,
) -> Response {
    ws.on_upgrade(move |socket| handle_traces(socket, state, query))
}

/// Log level filter - each level includes all higher severity levels
fn level_matches(span_level: &str, filter_level: &str) -> bool {
    let level_order = ["trace", "debug", "info", "warn", "error"];

    let span_idx = level_order
        .iter()
        .position(|&l| l == span_level)
        .unwrap_or(2);
    let filter_idx = level_order
        .iter()
        .position(|&l| l == filter_level)
        .unwrap_or(2);

    span_idx >= filter_idx
}

/// Check if a trace event matches the filter criteria
fn matches_filters(event: &TraceEvent, query: &TraceQuery) -> bool {
    // Check level filter
    if let Some(ref level) = query.level
        && !level_matches(&event.level, level)
    {
        return false;
    }

    // Check session filter (prefix match)
    if let Some(ref session_filter) = query.session {
        match &event.session_id {
            Some(session_id) if session_id.starts_with(session_filter) => {}
            _ => return false,
        }
    }

    // Check agent filter
    if let Some(ref agent_filter) = query.agent {
        match &event.agent_id {
            Some(agent_id) if agent_id == agent_filter => {}
            _ => return false,
        }
    }

    true
}

async fn handle_traces(socket: WebSocket, state: Arc<AppState>, query: TraceQuery) {
    let (mut sender, mut receiver) = socket.split();

    debug!(
        session = ?query.session,
        agent = ?query.agent,
        level = ?query.level,
        "Trace WebSocket connection established"
    );

    // Subscribe to trace events
    let mut trace_rx = state.subscribe_traces();

    // Send confirmation
    let ack = ServerMessage::TraceSubscribed;
    if let Err(e) = send_json(&mut sender, &ack).await {
        warn!("Failed to send TraceSubscribed: {}", e);
        return;
    }

    // Main event loop
    loop {
        tokio::select! {
            // Handle incoming client messages
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // Handle unsubscribe or filter updates
                        if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text)
                            && matches!(client_msg, ClientMessage::UnsubscribeTraces)
                        {
                            let unsub = ServerMessage::TraceUnsubscribed;
                            let _ = send_json(&mut sender, &unsub).await;
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {} // Ignore other message types
                }
            }

            // Handle trace events from broadcaster
            result = trace_rx.recv() => {
                match result {
                    Ok(event) => {
                        if !matches_filters(&event, &query) {
                            continue;
                        }

                        let msg = ServerMessage::TraceEvent(event);
                        if let Err(e) = send_json(&mut sender, &msg).await {
                            warn!("Failed to send trace event: {}", e);
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("Trace client lagged by {} events", n);
                    }
                }
            }
        }
    }

    debug!("Trace WebSocket connection closed");
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;
    use vibes_observe::SpanStatus;

    fn make_trace_event(
        level: &str,
        session_id: Option<&str>,
        agent_id: Option<&str>,
    ) -> TraceEvent {
        TraceEvent {
            trace_id: "test".to_string(),
            span_id: "test".to_string(),
            parent_span_id: None,
            name: "test::span".to_string(),
            level: level.to_string(),
            timestamp: Utc::now(),
            duration_ms: Some(1.0),
            session_id: session_id.map(String::from),
            agent_id: agent_id.map(String::from),
            attributes: HashMap::new(),
            status: SpanStatus::Ok,
        }
    }

    #[test]
    fn level_matches_same_level() {
        assert!(level_matches("info", "info"));
        assert!(level_matches("error", "error"));
    }

    #[test]
    fn level_matches_higher_severity() {
        // info level filter should match warn and error
        assert!(level_matches("warn", "info"));
        assert!(level_matches("error", "info"));
    }

    #[test]
    fn level_matches_lower_severity_fails() {
        // error level filter should NOT match info
        assert!(!level_matches("info", "error"));
        assert!(!level_matches("debug", "info"));
    }

    #[test]
    fn matches_filters_no_filters() {
        let event = make_trace_event("info", Some("sess-1"), None);
        let query = TraceQuery {
            session: None,
            agent: None,
            level: None,
        };
        assert!(matches_filters(&event, &query));
    }

    #[test]
    fn matches_filters_session_prefix() {
        let event = make_trace_event("info", Some("sess-12345"), None);
        let query = TraceQuery {
            session: Some("sess-123".to_string()),
            agent: None,
            level: None,
        };
        assert!(matches_filters(&event, &query));
    }

    #[test]
    fn matches_filters_session_mismatch() {
        let event = make_trace_event("info", Some("sess-99999"), None);
        let query = TraceQuery {
            session: Some("sess-123".to_string()),
            agent: None,
            level: None,
        };
        assert!(!matches_filters(&event, &query));
    }

    #[test]
    fn matches_filters_agent_exact() {
        let event = make_trace_event("info", None, Some("agent-1"));
        let query = TraceQuery {
            session: None,
            agent: Some("agent-1".to_string()),
            level: None,
        };
        assert!(matches_filters(&event, &query));
    }

    #[test]
    fn matches_filters_level() {
        let event = make_trace_event("warn", None, None);
        let query = TraceQuery {
            session: None,
            agent: None,
            level: Some("info".to_string()),
        };
        assert!(matches_filters(&event, &query));
    }
}
