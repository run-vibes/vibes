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

use crate::{AppState, PtyEvent};
use base64::Engine;

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
        && value.eq_ignore_ascii_case("cli")
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
    /// PTY session IDs this connection is attached to
    attached_pty_sessions: HashSet<String>,
}

impl ConnectionState {
    fn new(client_type: InputSource) -> Self {
        Self {
            client_id: Uuid::new_v4().to_string(),
            client_type,
            subscribed_sessions: HashSet::new(),
            attached_pty_sessions: HashSet::new(),
        }
    }

    /// Get the client type
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

    /// Attach to a PTY session
    fn attach_pty(&mut self, session_id: &str) {
        self.attached_pty_sessions.insert(session_id.to_string());
    }

    /// Detach from a PTY session
    fn detach_pty(&mut self, session_id: &str) {
        self.attached_pty_sessions.remove(session_id);
    }

    /// Check if this connection is attached to a PTY session
    fn is_attached_to_pty(&self, session_id: &str) -> bool {
        self.attached_pty_sessions.contains(session_id)
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
    let mut pty_rx = state.subscribe_pty_events();
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

            // Handle PTY events from broadcast channel
            pty_event = pty_rx.recv() => {
                match pty_event {
                    Ok(event) => {
                        if let Err(e) = handle_pty_event(&event, &mut sender, &conn_state).await {
                            warn!("Failed to send PTY event to client: {}", e);
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        warn!("PTY broadcast channel closed");
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        warn!("Client lagged behind by {} PTY events", count);
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

    // Convert VibesEvent to ServerMessage (including UserInput which clients filter by source)
    if let Some(server_msg) = vibes_event_to_server_message(event) {
        let json = serde_json::to_string(&server_msg)?;
        sender.send(Message::Text(json)).await?;
    }

    Ok(())
}

/// Handle a PTY event from the broadcast channel
async fn handle_pty_event(
    event: &PtyEvent,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    conn_state: &ConnectionState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Extract session_id from event
    let session_id = match event {
        PtyEvent::Output { session_id, .. } => session_id,
        PtyEvent::Exit { session_id, .. } => session_id,
    };

    // Only send if client is attached to this PTY session
    if !conn_state.is_attached_to_pty(session_id) {
        return Ok(());
    }

    // Convert to ServerMessage and send
    let server_msg = match event {
        PtyEvent::Output { session_id, data } => ServerMessage::PtyOutput {
            session_id: session_id.clone(),
            data: data.clone(),
        },
        PtyEvent::Exit {
            session_id,
            exit_code,
        } => ServerMessage::PtyExit {
            session_id: session_id.clone(),
            exit_code: *exit_code,
        },
    };

    let json = serde_json::to_string(&server_msg)?;
    sender.send(Message::Text(json)).await?;

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

            // Get PTY sessions (the active terminal sessions)
            let pty_manager = state.pty_manager.read().await;
            let pty_sessions = pty_manager.list_sessions();

            let sessions: Vec<SessionInfo> = pty_sessions
                .into_iter()
                .map(|pty_info| {
                    use vibes_core::pty::PtyState;
                    let state_str = match pty_info.state {
                        PtyState::Running => "Running".to_string(),
                        PtyState::Exited(code) => format!("Exited({})", code),
                    };

                    SessionInfo {
                        id: pty_info.id,
                        name: pty_info.name,
                        state: state_str,
                        // PTY sessions don't have ownership tracking yet
                        owner_id: String::new(),
                        is_owner: true, // All clients can interact with PTY
                        subscriber_count: 0,
                        created_at: 0,
                        last_activity_at: 0,
                    }
                })
                .collect();

            let response = ServerMessage::SessionList {
                request_id,
                sessions,
            };
            let json = serde_json::to_string(&response)?;
            sender.send(Message::Text(json)).await?;
        }

        ClientMessage::KillSession { session_id } => {
            debug!("KillSession request: {}", session_id);

            // Kill PTY session
            let mut pty_manager = state.pty_manager.write().await;
            match pty_manager.kill_session(&session_id).await {
                Ok(()) => {
                    // Detach locally
                    conn_state.detach_pty(&session_id);

                    // Broadcast removal to all clients
                    let removal_msg = ServerMessage::SessionRemoved {
                        session_id: session_id.clone(),
                        reason: RemovalReason::Killed,
                    };
                    let json = serde_json::to_string(&removal_msg)?;
                    sender.send(Message::Text(json)).await?;

                    // Also broadcast PTY exit event
                    let exit_event = PtyEvent::Exit {
                        session_id,
                        exit_code: None,
                    };
                    state.broadcast_pty_event(exit_event);
                }
                Err(e) => {
                    warn!("Failed to kill PTY session {}: {}", session_id, e);
                    let error_msg = ServerMessage::Error {
                        session_id: Some(session_id),
                        message: e.to_string(),
                        code: "KILL_FAILED".to_string(),
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

        // PTY messages
        ClientMessage::Attach { session_id } => {
            debug!("PTY attach requested for session: {}", session_id);

            let mut pty_manager = state.pty_manager.write().await;

            // Check if session exists; if not, create it
            let (cols, rows) = if pty_manager.get_session(&session_id).is_some() {
                // Session exists, get dimensions from config (we don't track current size yet)
                (120, 40) // TODO: Track actual PTY dimensions
            } else {
                // Create new PTY session with the client's session ID
                match pty_manager.create_session_with_id(session_id.clone(), None) {
                    Ok(created_id) => {
                        debug!("Created new PTY session: {}", created_id);

                        // Get handle for output reading
                        if let Some(handle) = pty_manager.get_handle(&created_id) {
                            // Spawn background task to read PTY output
                            let state_clone = state.clone();
                            let session_id_clone = created_id.clone();
                            tokio::spawn(async move {
                                pty_output_reader(state_clone, session_id_clone, handle).await;
                            });
                        }

                        (120, 40) // Initial dimensions from config
                    }
                    Err(e) => {
                        let error = ServerMessage::Error {
                            session_id: Some(session_id),
                            message: format!("Failed to create PTY session: {}", e),
                            code: "PTY_CREATE_FAILED".to_string(),
                        };
                        let json = serde_json::to_string(&error)?;
                        sender.send(Message::Text(json)).await?;
                        return Ok(());
                    }
                }
            };

            // Mark this connection as attached
            conn_state.attach_pty(&session_id);

            // Send AttachAck
            let ack = ServerMessage::AttachAck {
                session_id,
                cols,
                rows,
            };
            let json = serde_json::to_string(&ack)?;
            sender.send(Message::Text(json)).await?;
        }

        ClientMessage::Detach { session_id } => {
            debug!("PTY detach requested for session: {}", session_id);
            conn_state.detach_pty(&session_id);
        }

        ClientMessage::PtyInput { session_id, data } => {
            debug!(
                "PTY input for session: {}, {} bytes",
                session_id,
                data.len()
            );

            // Decode base64 input
            let decoded = match base64::engine::general_purpose::STANDARD.decode(&data) {
                Ok(d) => d,
                Err(e) => {
                    warn!("Failed to decode PTY input: {}", e);
                    return Ok(());
                }
            };

            // Get handle and write
            let pty_manager = state.pty_manager.read().await;
            if let Some(handle) = pty_manager.get_handle(&session_id) {
                if let Err(e) = handle.write(&decoded).await {
                    warn!("Failed to write to PTY: {}", e);
                }
            } else {
                warn!("PTY session not found: {}", session_id);
            }
        }

        ClientMessage::PtyResize {
            session_id,
            cols,
            rows,
        } => {
            debug!("PTY resize for session: {}, {}x{}", session_id, cols, rows);

            let pty_manager = state.pty_manager.read().await;
            if let Some(handle) = pty_manager.get_handle(&session_id) {
                if let Err(e) = handle.resize(cols, rows).await {
                    warn!("Failed to resize PTY: {}", e);
                }
            } else {
                warn!("PTY session not found: {}", session_id);
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

/// Background task that reads from a PTY and broadcasts output
async fn pty_output_reader(
    state: Arc<AppState>,
    session_id: String,
    handle: vibes_core::pty::PtySessionHandle,
) {
    use std::time::Duration;
    use vibes_core::pty::PtyError;

    debug!("Starting PTY output reader for session: {}", session_id);

    loop {
        // Read from PTY
        match handle.read().await {
            Ok(data) if !data.is_empty() => {
                // Capture raw output in scrollback buffer before broadcasting
                handle.append_scrollback(&data);

                // Encode as base64 and broadcast
                let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
                let event = PtyEvent::Output {
                    session_id: session_id.clone(),
                    data: encoded,
                };
                state.broadcast_pty_event(event);
            }
            Ok(_) => {
                // No data available (non-blocking WouldBlock case), wait a bit
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(PtyError::Eof) => {
                // PTY process has exited (EOF on read)
                debug!("PTY EOF for session {}: process exited", session_id);

                // Broadcast exit event
                let event = PtyEvent::Exit {
                    session_id: session_id.clone(),
                    exit_code: None, // Could check child.try_wait() for actual code
                };
                state.broadcast_pty_event(event);
                break;
            }
            Err(e) => {
                // Other I/O error
                warn!("PTY read error for session {}: {}", session_id, e);

                // Broadcast exit event
                let event = PtyEvent::Exit {
                    session_id: session_id.clone(),
                    exit_code: None,
                };
                state.broadcast_pty_event(event);
                break;
            }
        }
    }

    debug!("PTY output reader finished for session: {}", session_id);
}
