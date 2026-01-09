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
use vibes_core::{AuthContext, InputSource, VibesEvent};

use crate::{AppState, PtyEvent};
use base64::Engine;

use super::protocol::{ClientMessage, RemovalReason, ServerMessage, SessionInfo};

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
    #[allow(dead_code)] // Used for logging
    client_id: String,
    /// PTY session IDs this connection is attached to
    attached_pty_sessions: HashSet<String>,
    /// PTY session IDs that have received their scrollback replay.
    /// Replay is deferred until the first resize so the PTY dimensions
    /// match the client's terminal size (important for mobile clients).
    replay_sent_sessions: HashSet<String>,
    /// Scrollback buffer length at attach time for each session.
    /// Used to avoid sending content that arrived after attach as both
    /// pty_output (real-time) AND pty_replay (historical), which would
    /// cause duplicate content on the client.
    scrollback_snapshot_len: std::collections::HashMap<String, usize>,
}

impl ConnectionState {
    fn new(_client_type: InputSource) -> Self {
        Self {
            client_id: Uuid::new_v4().to_string(),
            attached_pty_sessions: HashSet::new(),
            replay_sent_sessions: HashSet::new(),
            scrollback_snapshot_len: std::collections::HashMap::new(),
        }
    }

    /// Attach to a PTY session, recording the current scrollback length
    fn attach_pty(&mut self, session_id: &str, scrollback_len: usize) {
        self.attached_pty_sessions.insert(session_id.to_string());
        self.scrollback_snapshot_len
            .insert(session_id.to_string(), scrollback_len);
    }

    /// Detach from a PTY session
    fn detach_pty(&mut self, session_id: &str) {
        self.attached_pty_sessions.remove(session_id);
        self.replay_sent_sessions.remove(session_id);
        self.scrollback_snapshot_len.remove(session_id);
    }

    /// Check if this connection is attached to a PTY session
    fn is_attached_to_pty(&self, session_id: &str) -> bool {
        self.attached_pty_sessions.contains(session_id)
    }

    /// Check if replay needs to be sent for this session
    fn needs_replay(&self, session_id: &str) -> bool {
        self.attached_pty_sessions.contains(session_id)
            && !self.replay_sent_sessions.contains(session_id)
    }

    /// Get the scrollback length that was recorded at attach time
    fn get_replay_limit(&self, session_id: &str) -> usize {
        self.scrollback_snapshot_len
            .get(session_id)
            .copied()
            .unwrap_or(0)
    }

    /// Mark replay as sent for this session
    fn mark_replay_sent(&mut self, session_id: &str) {
        self.replay_sent_sessions.insert(session_id.to_string());
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
    let mut pty_rx = state.subscribe_pty_events();
    let mut conn_state = ConnectionState::new(client_type);

    info!(
        client_id = %conn_state.client_id,
        ?client_type,
        "WebSocket client connected"
    );

    // Append client connected event to EventLog for consumer processing
    state.append_event(VibesEvent::ClientConnected {
        client_id: conn_state.client_id.clone(),
    });

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

    // Append client disconnected event to EventLog for consumer processing
    state.append_event(VibesEvent::ClientDisconnected {
        client_id: conn_state.client_id.clone(),
    });

    info!(
        client_id = %conn_state.client_id,
        "WebSocket client disconnected"
    );
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

                    let created_ts = pty_info.created_at.timestamp();
                    SessionInfo {
                        id: pty_info.id,
                        name: pty_info.name,
                        state: state_str,
                        // PTY sessions don't have ownership tracking yet
                        owner_id: String::new(),
                        is_owner: true, // All clients can interact with PTY
                        subscriber_count: 0,
                        created_at: created_ts,
                        // FUTURE: Track last_activity_at on I/O events for idle detection
                        last_activity_at: created_ts,
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

                    // Append session removed event to EventLog for consumer processing
                    state.append_event(VibesEvent::SessionRemoved {
                        session_id: session_id.clone(),
                        reason: "killed".to_string(),
                    });

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

        // PTY messages
        ClientMessage::Attach {
            session_id,
            name,
            cwd,
        } => {
            debug!("PTY attach requested for session: {}", session_id);

            // Clone name before it's moved for potential SessionCreated event
            let session_name = name.clone();

            let mut pty_manager = state.pty_manager.write().await;

            // Check if session exists and capture scrollback length for replay limiting
            let (cols, rows, _scrollback_len) =
                if let Some(handle) = pty_manager.get_handle(&session_id) {
                    // Session exists - capture scrollback length BEFORE marking as attached.
                    // This prevents duplicate content: any output that arrives after this point
                    // will be sent as pty_output AND would also be in scrollback for replay.
                    // By recording the length now, replay will only include content up to here.
                    let scrollback_len = handle.get_scrollback().len();

                    // Mark as attached with the scrollback snapshot length
                    conn_state.attach_pty(&session_id, scrollback_len);

                    // NOTE: Default PTY dimensions (120x40) are used initially.
                    // The client overrides these via PtyResize message after attach.
                    (120, 40, scrollback_len)
                } else {
                    // Create new PTY session with the client's session ID
                    match pty_manager.create_session_with_id(session_id.clone(), name, cwd) {
                        Ok(created_id) => {
                            debug!("Created new PTY session: {}", created_id);

                            // Append session created event to EventLog for consumer processing
                            state.append_event(VibesEvent::SessionCreated {
                                session_id: created_id.clone(),
                                name: session_name,
                            });

                            // New session has no scrollback yet
                            conn_state.attach_pty(&session_id, 0);

                            // Get handle for output reading
                            if let Some(handle) = pty_manager.get_handle(&created_id) {
                                // Spawn background task to read PTY output
                                let state_clone = state.clone();
                                let session_id_clone = created_id.clone();
                                tokio::spawn(async move {
                                    pty_output_reader(state_clone, session_id_clone, handle).await;
                                });
                            }

                            // Initial dimensions from config - client will resize after attach
                            // scrollback_len is 0 for new sessions
                            (120, 40, 0)
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

            // Note: Scrollback replay is deferred until first PtyResize.
            // This ensures the PTY dimensions match the client's terminal size,
            // which is critical for mobile clients with narrower screens.

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
                // Resize the PTY first
                if let Err(e) = handle.resize(cols, rows).await {
                    warn!("Failed to resize PTY: {}", e);
                }

                // Send scrollback replay on first resize after attach.
                // This ensures the client has set the correct terminal dimensions
                // before receiving historical output (important for mobile clients).
                if conn_state.needs_replay(&session_id) {
                    // Only send scrollback up to the length recorded at attach time.
                    // Content added after attach was already sent as pty_output,
                    // so including it in replay would cause duplicates.
                    let replay_limit = conn_state.get_replay_limit(&session_id);
                    let scrollback = handle.get_scrollback();
                    let replay_data = if scrollback.len() > replay_limit {
                        &scrollback[..replay_limit]
                    } else {
                        &scrollback[..]
                    };

                    if !replay_data.is_empty() {
                        let data = base64::engine::general_purpose::STANDARD.encode(replay_data);
                        let replay_msg = ServerMessage::PtyReplay {
                            session_id: session_id.clone(),
                            data,
                        };
                        let json = serde_json::to_string(&replay_msg)?;
                        sender.send(Message::Text(json)).await?;
                        debug!(
                            "Sent {} bytes of scrollback replay (limit: {}) for session {} after resize to {}x{}",
                            replay_data.len(),
                            replay_limit,
                            session_id,
                            cols,
                            rows
                        );
                    }
                    conn_state.mark_replay_sent(&session_id);
                }
            } else {
                warn!("PTY session not found: {}", session_id);
            }
        }
    }

    Ok(())
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
                // No data available (non-blocking WouldBlock case), wait a bit.
                // We use polling here because portable-pty doesn't provide async I/O.
                // The 10ms sleep balances responsiveness with CPU usage. Alternative
                // approaches like tokio::io::unix::AsyncFd aren't portable across
                // all platforms that portable-pty supports.
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
