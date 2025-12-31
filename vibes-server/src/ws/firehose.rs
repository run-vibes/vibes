//! WebSocket handler for firehose event streaming
//!
//! Provides a read-only WebSocket endpoint that streams all VibesEvents,
//! optionally filtered by event type and/or session ID.

use std::sync::Arc;

use axum::{
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::broadcast;
use tracing::warn;
use vibes_core::VibesEvent;

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

/// WebSocket upgrade handler for firehose
pub async fn firehose_ws(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(query): Query<FirehoseQuery>,
) -> Response {
    ws.on_upgrade(move |socket| handle_firehose(socket, state, query))
}

async fn handle_firehose(socket: WebSocket, state: Arc<AppState>, query: FirehoseQuery) {
    let (mut sender, mut receiver) = socket.split();
    let mut events_rx = state.subscribe_events();

    // Parse filter types
    let filter_types: Option<Vec<String>> = query.types.map(|t| {
        t.split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    });
    let filter_session = query.session;

    // Spawn task to forward events to WebSocket
    let send_task = tokio::spawn(async move {
        loop {
            match events_rx.recv().await {
                Ok(event) => {
                    // Apply type filter
                    if let Some(ref types) = filter_types {
                        let event_type = event_type_name(&event).to_lowercase();
                        if !types.iter().any(|t| event_type.contains(t)) {
                            continue;
                        }
                    }

                    // Apply session filter (uses VibesEvent's built-in method)
                    if let Some(ref session) = filter_session
                        && event.session_id() != Some(session.as_str())
                    {
                        continue;
                    }

                    // Serialize and send
                    match serde_json::to_string(&event) {
                        Ok(json) => {
                            if sender.send(Message::Text(json)).await.is_err() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use vibes_core::ClaudeEvent;

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
