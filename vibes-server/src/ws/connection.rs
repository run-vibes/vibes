//! WebSocket connection handling

use std::collections::HashSet;
use std::sync::Arc;

use axum::Extension;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::Request;
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use vibes_core::history::{MessageQuery, MessageRole};
use vibes_core::{AuthContext, ClaudeEvent, EventBus, InputSource, VibesEvent};

use crate::AppState;

use super::protocol::{
    ClientMessage, HistoryEvent, RemovalReason, ServerMessage, SessionInfo,
    vibes_event_to_server_message,
};

/// Detect client type from request headers
///
/// CLI clients send `X-Vibes-Client-Type: cli` header.
/// Browser connections default to Web UI.
fn detect_client_type<B>(req: &Request<B>) -> InputSource {
    if let Some(header) = req.headers().get("X-Vibes-Client-Type")
        && let Ok(value) = header.to_str()
        && value == "cli"
    {
        return InputSource::Cli;
    }
    // Default to Web UI for browser connections
    InputSource::WebUi
}

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Extension(auth_context): Extension<AuthContext>,
    req: Request<axum::body::Body>,
) -> impl IntoResponse {
    let client_type = detect_client_type(&req);
    debug!(?client_type, "WebSocket connection detected");
    ws.on_upgrade(move |socket| handle_socket(socket, state, auth_context, client_type))
}

/// Per-connection state
struct ConnectionState {
    /// Unique identifier for this connection
    client_id: String,
    /// Type of client (CLI, Web UI)
    client_type: InputSource,
    /// Session IDs this connection is subscribed to
    subscribed_sessions: HashSet<String>,
}

impl ConnectionState {
    fn new(client_type: InputSource) -> Self {
        Self {
            client_id: Uuid::new_v4().to_string(),
            client_type,
            subscribed_sessions: HashSet::new(),
        }
    }

    /// Get the client type
    #[allow(dead_code)]
    fn client_type(&self) -> InputSource {
        self.client_type
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
async fn handle_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    auth_context: AuthContext,
    client_type: InputSource,
) {
    let (mut sender, mut receiver) = socket.split();
    let mut event_rx = state.subscribe_events();
    let mut conn_state = ConnectionState::new(client_type);

    info!(
        client_id = %conn_state.client_id,
        ?client_type,
        "WebSocket client connected"
    );

    // Send auth context immediately on connection
    let auth_msg = ServerMessage::AuthContext(auth_context);
    if let Ok(json) = serde_json::to_string(&auth_msg)
        && sender.send(Message::Text(json)).await.is_err()
    {
        warn!("Failed to send auth context to client");
        return;
    }

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

    // Handle lifecycle cleanup on disconnect
    let result = state
        .lifecycle
        .handle_client_disconnect(&conn_state.client_id)
        .await;

    // Log any ownership transfers
    for (session_id, new_owner) in &result.transfers {
        info!(
            session_id = %session_id,
            new_owner = %new_owner,
            "Ownership transferred due to client disconnect"
        );

        // Broadcast ownership transfer event
        let event = VibesEvent::OwnershipTransferred {
            session_id: session_id.clone(),
            new_owner_id: new_owner.clone(),
        };
        state.broadcast_event(event);
    }

    // Log any cleanups
    for session_id in &result.cleanups {
        info!(
            session_id = %session_id,
            "Session cleaned up due to client disconnect"
        );

        // Broadcast session removal event
        let event = VibesEvent::SessionRemoved {
            session_id: session_id.clone(),
            reason: "owner_disconnected".to_string(),
        };
        state.broadcast_event(event);
    }

    info!(
        client_id = %conn_state.client_id,
        transfers = result.transfers.len(),
        cleanups = result.cleanups.len(),
        "WebSocket client disconnected"
    );
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

    // Handle OwnershipTransferred specially (needs client-specific you_are_owner)
    if let VibesEvent::OwnershipTransferred {
        session_id,
        new_owner_id,
    } = event
    {
        let server_msg = ServerMessage::OwnershipTransferred {
            session_id: session_id.clone(),
            new_owner_id: new_owner_id.clone(),
            you_are_owner: *new_owner_id == conn_state.client_id,
        };
        let json = serde_json::to_string(&server_msg)?;
        sender.send(Message::Text(json)).await?;
        return Ok(());
    }

    // Handle UserInput specially - broadcast to other subscribers
    // Clients filter by source to avoid echoing back their own input
    if let VibesEvent::UserInput {
        session_id,
        content,
        source,
    } = event
    {
        let server_msg = ServerMessage::UserInput {
            session_id: session_id.clone(),
            content: content.clone(),
            source: *source,
        };
        let json = serde_json::to_string(&server_msg)?;
        sender.send(Message::Text(json)).await?;
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
        ClientMessage::Subscribe {
            session_ids,
            catch_up,
        } => {
            debug!(
                "Client subscribed to sessions: {:?}, catch_up: {}",
                session_ids, catch_up
            );
            conn_state.subscribe(&session_ids);

            // Send SubscribeAck with history if catch_up is requested
            if catch_up {
                for session_id in &session_ids {
                    let (history, current_seq, has_more) =
                        get_session_history(state.as_ref(), session_id, 50);

                    let ack = ServerMessage::SubscribeAck {
                        session_id: session_id.clone(),
                        current_seq,
                        history,
                        has_more,
                    };
                    let json = serde_json::to_string(&ack)?;
                    sender.send(Message::Text(json)).await?;
                }
            }
        }

        ClientMessage::Unsubscribe { session_ids } => {
            debug!("Client unsubscribed from sessions: {:?}", session_ids);
            conn_state.unsubscribe(&session_ids);
        }

        ClientMessage::CreateSession { name, request_id } => {
            let session_id = state.session_manager.create_session(name.clone()).await;

            // Auto-subscribe to the newly created session
            conn_state.subscribe(std::slice::from_ref(&session_id));
            debug!("Auto-subscribed to new session: {}", session_id);

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
            // Publish input event with source attribution for other subscribers
            let input_event = VibesEvent::UserInput {
                session_id: session_id.clone(),
                content: content.clone(),
                source: conn_state.client_type(),
            };
            state.event_bus.publish(input_event).await;

            // Forward to session
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

        ClientMessage::ListSessions { request_id } => {
            debug!("ListSessions request: {}", request_id);

            // Collect session info for all sessions
            let session_ids = state.session_manager.list_sessions().await;
            let mut sessions = Vec::with_capacity(session_ids.len());

            for id in session_ids {
                if let Ok(info) = state
                    .session_manager
                    .with_session(&id, |session| {
                        let ownership = session.ownership();
                        SessionInfo {
                            id: id.clone(),
                            name: session.name().map(|s| s.to_string()),
                            state: format!("{:?}", session.state()),
                            owner_id: ownership.owner_id.clone(),
                            is_owner: ownership.is_owner(&conn_state.client_id),
                            subscriber_count: ownership.subscriber_count() as u32,
                            created_at: session
                                .created_at()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_secs() as i64)
                                .unwrap_or(0),
                            last_activity_at: session
                                .last_activity_at()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_secs() as i64)
                                .unwrap_or(0),
                        }
                    })
                    .await
                {
                    sessions.push(info);
                }
            }

            let response = ServerMessage::SessionList {
                request_id,
                sessions,
            };
            let json = serde_json::to_string(&response)?;
            sender.send(Message::Text(json)).await?;
        }

        ClientMessage::KillSession { session_id } => {
            debug!("KillSession request: {}", session_id);

            match state.session_manager.remove_session(&session_id).await {
                Ok(()) => {
                    // Unsubscribe locally
                    conn_state.unsubscribe(std::slice::from_ref(&session_id));

                    // Broadcast removal to all clients
                    let removal_msg = ServerMessage::SessionRemoved {
                        session_id,
                        reason: RemovalReason::Killed,
                    };
                    let json = serde_json::to_string(&removal_msg)?;
                    sender.send(Message::Text(json)).await?;
                }
                Err(e) => {
                    warn!("Failed to kill session {}: {}", session_id, e);
                    let code = if e.to_string().contains("not found") {
                        "NOT_FOUND"
                    } else {
                        "KILL_FAILED"
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

        ClientMessage::RequestHistory {
            session_id,
            before_seq,
            limit,
        } => {
            if !conn_state.is_subscribed_to(&session_id) {
                let error = ServerMessage::Error {
                    session_id: Some(session_id),
                    message: "Not subscribed to session".to_string(),
                    code: "NOT_SUBSCRIBED".to_string(),
                };
                let json = serde_json::to_string(&error)?;
                sender.send(Message::Text(json)).await?;
            } else {
                let (events, oldest_seq, has_more) =
                    get_history_page(state.as_ref(), &session_id, before_seq, limit);

                let page = ServerMessage::HistoryPage {
                    session_id,
                    events,
                    has_more,
                    oldest_seq,
                };
                let json = serde_json::to_string(&page)?;
                sender.send(Message::Text(json)).await?;
            }
        }
    }

    Ok(())
}

/// Convert a HistoricalMessage to a VibesEvent for catch-up
fn message_to_vibes_event(msg: &vibes_core::history::HistoricalMessage) -> VibesEvent {
    match msg.role {
        MessageRole::User => VibesEvent::UserInput {
            session_id: msg.session_id.clone(),
            content: msg.content.clone(),
            source: msg.source,
        },
        MessageRole::Assistant => VibesEvent::Claude {
            session_id: msg.session_id.clone(),
            event: ClaudeEvent::TextDelta {
                text: msg.content.clone(),
            },
        },
        MessageRole::ToolUse => VibesEvent::Claude {
            session_id: msg.session_id.clone(),
            event: ClaudeEvent::ToolUseStart {
                id: msg.tool_id.clone().unwrap_or_default(),
                name: msg.tool_name.clone().unwrap_or_default(),
            },
        },
        MessageRole::ToolResult => VibesEvent::Claude {
            session_id: msg.session_id.clone(),
            event: ClaudeEvent::ToolResult {
                id: msg.tool_id.clone().unwrap_or_default(),
                output: msg.content.clone(),
                is_error: false,
            },
        },
    }
}

/// Get session history for catch-up
///
/// Returns (history events, current sequence, has_more_pages)
fn get_session_history(
    state: &AppState,
    session_id: &str,
    limit: u32,
) -> (Vec<HistoryEvent>, u64, bool) {
    let Some(history_service) = &state.history else {
        return (vec![], 0, false);
    };

    let query = MessageQuery {
        limit: limit + 1, // Request one extra to detect has_more
        offset: 0,
        role: None,
        before_id: None,
    };

    let result = match history_service.get_messages(session_id, &query) {
        Ok(r) => r,
        Err(_) => return (vec![], 0, false),
    };

    let has_more = result.messages.len() > limit as usize;
    let messages: Vec<_> = result.messages.into_iter().take(limit as usize).collect();

    let current_seq = messages.last().map(|m| m.id as u64).unwrap_or(0);

    let history: Vec<HistoryEvent> = messages
        .into_iter()
        .map(|m| HistoryEvent {
            seq: m.id as u64,
            event: message_to_vibes_event(&m),
            timestamp: m.created_at * 1000, // Convert to milliseconds
        })
        .collect();

    (history, current_seq, has_more)
}

/// Get paginated history for RequestHistory
///
/// Returns (history events, oldest sequence in page, has_more_pages)
fn get_history_page(
    state: &AppState,
    session_id: &str,
    before_seq: u64,
    limit: u32,
) -> (Vec<HistoryEvent>, u64, bool) {
    let Some(history_service) = &state.history else {
        return (vec![], 0, false);
    };

    let query = MessageQuery {
        limit: limit + 1, // Request one extra to detect has_more
        offset: 0,
        role: None,
        before_id: Some(before_seq as i64),
    };

    let result = match history_service.get_messages(session_id, &query) {
        Ok(r) => r,
        Err(_) => return (vec![], 0, false),
    };

    let has_more = result.messages.len() > limit as usize;
    let messages: Vec<_> = result.messages.into_iter().take(limit as usize).collect();

    let oldest_seq = messages.first().map(|m| m.id as u64).unwrap_or(0);

    let events: Vec<HistoryEvent> = messages
        .into_iter()
        .map(|m| HistoryEvent {
            seq: m.id as u64,
            event: message_to_vibes_event(&m),
            timestamp: m.created_at * 1000,
        })
        .collect();

    (events, oldest_seq, has_more)
}
