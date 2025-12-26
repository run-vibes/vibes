//! WebSocket connection handling

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use tracing::{debug, error, info, warn};

use crate::AppState;

use super::protocol::{ClientMessage, ServerMessage};

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle a WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    info!("WebSocket client connected");

    // Process incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) = handle_text_message(&text, &state, &mut sender).await {
                    error!("Error handling message: {}", e);
                    let error_msg = ServerMessage::Error {
                        session_id: None,
                        message: e.to_string(),
                        code: "INTERNAL_ERROR".to_string(),
                    };
                    if let Ok(json) = serde_json::to_string(&error_msg) {
                        let _ = sender.send(Message::Text(json.into())).await;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                debug!("WebSocket client sent close frame");
                break;
            }
            Ok(Message::Ping(data)) => {
                if sender.send(Message::Pong(data)).await.is_err() {
                    break;
                }
            }
            Ok(_) => {
                // Ignore binary and pong messages
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    info!("WebSocket client disconnected");
}

/// Handle a text message from the client
async fn handle_text_message(
    text: &str,
    state: &Arc<AppState>,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client_msg: ClientMessage = serde_json::from_str(text)?;

    match client_msg {
        ClientMessage::Subscribe { session_ids } => {
            debug!("Client subscribed to sessions: {:?}", session_ids);
            // TODO: Track subscriptions per connection
        }

        ClientMessage::Unsubscribe { session_ids } => {
            debug!("Client unsubscribed from sessions: {:?}", session_ids);
            // TODO: Remove subscriptions
        }

        ClientMessage::CreateSession { name, request_id } => {
            let session_id = state
                .session_manager
                .create_session(name.clone())
                .await;

            let response = ServerMessage::SessionCreated {
                request_id,
                session_id,
                name,
            };

            let json = serde_json::to_string(&response)?;
            sender.send(Message::Text(json.into())).await?;
        }

        ClientMessage::Input { session_id, content } => {
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
                sender.send(Message::Text(json.into())).await?;
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
                sender.send(Message::Text(json.into())).await?;
            }
        }
    }

    Ok(())
}
