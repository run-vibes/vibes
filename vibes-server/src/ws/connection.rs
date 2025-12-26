//! WebSocket connection handling

use std::collections::HashSet;
use std::sync::Arc;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use vibes_core::VibesEvent;

use crate::AppState;

use super::protocol::{ClientMessage, ServerMessage, vibes_event_to_server_message};

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Per-connection state
struct ConnectionState {
    /// Session IDs this connection is subscribed to
    subscribed_sessions: HashSet<String>,
}

impl ConnectionState {
    fn new() -> Self {
        Self {
            subscribed_sessions: HashSet::new(),
        }
    }

    /// Check if this connection should receive events for a given session
    fn is_subscribed_to(&self, session_id: &str) -> bool {
        self.subscribed_sessions.contains(session_id)
    }

    /// Subscribe to session events
    fn subscribe(&mut self, session_ids: &[String]) {
        for id in session_ids {
            self.subscribed_sessions.insert(id.clone());
        }
    }

    /// Unsubscribe from session events
    fn unsubscribe(&mut self, session_ids: &[String]) {
        for id in session_ids {
            self.subscribed_sessions.remove(id);
        }
    }
}

/// Handle a WebSocket connection with bidirectional event streaming
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut event_rx = state.subscribe_events();
    let mut conn_state = ConnectionState::new();

    info!("WebSocket client connected");

    loop {
        tokio::select! {
            // Handle incoming client messages
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(e) = handle_text_message(&text, &state, &mut sender, &mut conn_state).await {
                            error!("Error handling message: {}", e);
                            let error_msg = ServerMessage::Error {
                                session_id: None,
                                message: e.to_string(),
                                code: "INTERNAL_ERROR".to_string(),
                            };
                            if let Ok(json) = serde_json::to_string(&error_msg) {
                                let _ = sender.send(Message::Text(json)).await;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        debug!("WebSocket client sent close frame");
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(_)) => {
                        // Ignore binary and pong messages
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        // Client disconnected
                        break;
                    }
                }
            }

            // Handle outgoing events from broadcast channel
            event = event_rx.recv() => {
                match event {
                    Ok(vibes_event) => {
                        if let Err(e) = handle_broadcast_event(&vibes_event, &mut sender, &conn_state).await {
                            warn!("Failed to send event to client: {}", e);
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        warn!("Event broadcast channel closed");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        warn!("Client lagged behind by {} events", count);
                        // Continue receiving - the channel will skip missed events
                    }
                }
            }
        }
    }

    info!("WebSocket client disconnected");
}

/// Handle a broadcast event, sending it to the client if subscribed
async fn handle_broadcast_event(
    event: &VibesEvent,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    conn_state: &ConnectionState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Only send if client is subscribed to this session
    if let Some(session_id) = event.session_id()
        && !conn_state.is_subscribed_to(session_id)
    {
        return Ok(());
    }

    // Convert VibesEvent to ServerMessage
    if let Some(server_msg) = vibes_event_to_server_message(event) {
        let json = serde_json::to_string(&server_msg)?;
        sender.send(Message::Text(json)).await?;
    }

    Ok(())
}

/// Handle a text message from the client
async fn handle_text_message(
    text: &str,
    state: &Arc<AppState>,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    conn_state: &mut ConnectionState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client_msg: ClientMessage = serde_json::from_str(text)?;

    match client_msg {
        ClientMessage::Subscribe { session_ids } => {
            debug!("Client subscribed to sessions: {:?}", session_ids);
            conn_state.subscribe(&session_ids);
        }

        ClientMessage::Unsubscribe { session_ids } => {
            debug!("Client unsubscribed from sessions: {:?}", session_ids);
            conn_state.unsubscribe(&session_ids);
        }

        ClientMessage::CreateSession { name, request_id } => {
            let session_id = state.session_manager.create_session(name.clone()).await;

            let response = ServerMessage::SessionCreated {
                request_id,
                session_id,
                name,
            };

            let json = serde_json::to_string(&response)?;
            sender.send(Message::Text(json)).await?;
        }

        ClientMessage::Input {
            session_id,
            content,
        } => {
            if let Err(e) = state
                .session_manager
                .send_to_session(&session_id, &content)
                .await
            {
                warn!("Failed to send input to session {}: {}", session_id, e);
                let code = if e.to_string().contains("not found") {
                    "NOT_FOUND"
                } else {
                    "SEND_FAILED"
                };
                let error_msg = ServerMessage::Error {
                    session_id: Some(session_id),
                    message: e.to_string(),
                    code: code.to_string(),
                };
                let json = serde_json::to_string(&error_msg)?;
                sender.send(Message::Text(json)).await?;
            }
        }

        ClientMessage::PermissionResponse {
            session_id,
            request_id,
            approved,
        } => {
            if let Err(e) = state
                .session_manager
                .respond_permission(&session_id, &request_id, approved)
                .await
            {
                warn!(
                    "Failed to respond to permission {} in session {}: {}",
                    request_id, session_id, e
                );
                let code = if e.to_string().contains("not found") {
                    "NOT_FOUND"
                } else {
                    "PERMISSION_FAILED"
                };
                let error_msg = ServerMessage::Error {
                    session_id: Some(session_id),
                    message: e.to_string(),
                    code: code.to_string(),
                };
                let json = serde_json::to_string(&error_msg)?;
                sender.send(Message::Text(json)).await?;
            }
        }
    }

    Ok(())
}
