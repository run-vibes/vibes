//! Claude command - connects to vibes daemon PTY via WebSocket
//!
//! Provides a full terminal experience by connecting to a PTY session
//! on the vibes server and proxying terminal I/O over WebSocket.

use anyhow::{Result, anyhow};
use base64::Engine;
use clap::Args;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use std::time::Duration;
use tracing::debug;
use vibes_server::ws::ServerMessage;

use crate::client::VibesClient;
use crate::config::ConfigLoader;
use crate::daemon::ensure_daemon_running;
use crate::terminal::RawTerminal;

#[derive(Args)]
pub struct ClaudeArgs {
    // === Vibes-specific flags ===
    /// Human-friendly session name (shown in vibes UI)
    #[arg(long)]
    pub session_name: Option<String>,

    /// Disable background server for this session
    #[arg(long)]
    pub no_serve: bool,

    /// Start interactive mode (read prompts from stdin)
    #[arg(short = 'i', long)]
    pub interactive: bool,

    // === Common Claude flags (explicit for UX) ===
    /// Continue most recent session
    #[arg(short = 'c', long)]
    pub continue_session: bool,

    /// Resume specific session by ID
    #[arg(short = 'r', long)]
    pub resume: Option<String>,

    /// Model to use (e.g., claude-sonnet-4-20250514)
    #[arg(long)]
    pub model: Option<String>,

    /// Tools to allow without prompting (comma-separated)
    #[arg(long = "allowedTools")]
    pub allowed_tools: Option<String>,

    /// System prompt to use
    #[arg(long = "system-prompt")]
    pub system_prompt: Option<String>,

    // === Passthrough ===
    /// The prompt to send to Claude
    #[arg(value_name = "PROMPT")]
    pub prompt: Option<String>,

    /// Additional arguments passed directly to claude
    #[arg(last = true)]
    pub passthrough: Vec<String>,
}

pub async fn run(args: ClaudeArgs) -> Result<()> {
    let config = ConfigLoader::load()?;

    // Explicitly error if continue_session is requested, since not yet supported
    if args.continue_session {
        return Err(anyhow!(
            "The --continue / -c flag is not yet supported.\n\
             Please use --resume <SESSION_ID> to resume a specific session instead."
        ));
    }

    // Ensure daemon is running (unless --no-serve is set)
    if !args.no_serve {
        ensure_daemon_running(config.server.port).await?;
    }

    // Connect to daemon via WebSocket
    let url = format!("ws://127.0.0.1:{}/ws", config.server.port);
    let mut client = VibesClient::connect_url(&url).await?;

    // Attach to PTY session (creates it if needed)
    let session_id = if let Some(resume_id) = args.resume.clone() {
        resume_id
    } else {
        // Generate a session ID - the server will create the PTY on attach
        uuid::Uuid::new_v4().to_string()
    };

    // Send attach request
    client.attach(&session_id).await?;

    // Wait for attach acknowledgment
    let (initial_cols, initial_rows) = wait_for_attach_ack(&mut client, &session_id).await?;

    debug!(
        "Attached to session {} ({}x{})",
        session_id, initial_cols, initial_rows
    );

    // Run PTY loop
    let result = pty_loop(&mut client, &session_id).await;

    // Clean up - detach from session
    let _ = client.detach(&session_id).await;

    result
}

/// Wait for attach acknowledgment from server
async fn wait_for_attach_ack(
    client: &mut VibesClient,
    session_id: &str,
) -> Result<(u16, u16)> {
    let timeout = Duration::from_secs(10);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        match client.recv_timeout(Duration::from_secs(1)).await {
            Ok(Some(ServerMessage::AttachAck {
                session_id: sid,
                cols,
                rows,
            })) if sid == session_id => {
                return Ok((cols, rows));
            }
            Ok(Some(ServerMessage::Error { message, .. })) => {
                return Err(anyhow!("Failed to attach: {}", message));
            }
            Ok(Some(_)) => {
                // Not our response, continue waiting
                continue;
            }
            Ok(None) => {
                return Err(anyhow!("Connection closed while waiting for attach"));
            }
            Err(_) => {
                // Timeout on individual recv, continue loop
                continue;
            }
        }
    }

    Err(anyhow!("Timeout waiting for attach acknowledgment"))
}

/// Main PTY I/O loop
///
/// Proxies terminal I/O between the local terminal and the remote PTY session.
async fn pty_loop(client: &mut VibesClient, session_id: &str) -> Result<()> {
    // Enter raw terminal mode
    let terminal = RawTerminal::new()?;

    // Get and send initial terminal size
    if let Ok((cols, rows)) = terminal.size() {
        client.pty_resize(session_id, cols, rows).await?;
    }

    // Track last size for resize detection
    let mut last_size = terminal.size().unwrap_or((80, 24));

    loop {
        // Check for terminal resize
        if let Ok(current_size) = terminal.size() {
            if current_size != last_size {
                client
                    .pty_resize(session_id, current_size.0, current_size.1)
                    .await?;
                last_size = current_size;
            }
        }

        // Poll for terminal input
        if let Some(event) = terminal.read_event()? {
            match event {
                Event::Key(key_event) => {
                    // Convert key event to bytes and send
                    if let Some(data) = key_event_to_bytes(&key_event) {
                        let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
                        client.pty_input(session_id, &encoded).await?;
                    }
                }
                Event::Resize(cols, rows) => {
                    client.pty_resize(session_id, cols, rows).await?;
                    last_size = (cols, rows);
                }
                Event::Paste(text) => {
                    // Send pasted text directly
                    let encoded =
                        base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
                    client.pty_input(session_id, &encoded).await?;
                }
                _ => {
                    // Ignore other events (Mouse, FocusGained, FocusLost)
                }
            }
        }

        // Check for server messages (non-blocking)
        match tokio::time::timeout(Duration::from_millis(1), client.recv()).await {
            Ok(Some(msg)) => {
                match msg {
                    ServerMessage::PtyOutput { session_id: sid, data } if sid == session_id => {
                        // Decode and write to terminal
                        if let Ok(bytes) =
                            base64::engine::general_purpose::STANDARD.decode(&data)
                        {
                            terminal.write(&bytes)?;
                        }
                    }
                    ServerMessage::PtyExit {
                        session_id: sid,
                        exit_code,
                    } if sid == session_id => {
                        debug!("PTY exited with code: {:?}", exit_code);
                        break;
                    }
                    ServerMessage::Error {
                        session_id: Some(sid),
                        message,
                        ..
                    } if sid == session_id => {
                        eprintln!("\r\nError: {}\r\n", message);
                        break;
                    }
                    _ => {
                        // Ignore other messages
                    }
                }
            }
            Ok(None) => {
                // Connection closed
                break;
            }
            Err(_) => {
                // Timeout - no message available, continue loop
            }
        }
    }

    Ok(())
}

/// Convert a crossterm key event to bytes to send to PTY
fn key_event_to_bytes(event: &crossterm::event::KeyEvent) -> Option<Vec<u8>> {
    use KeyCode::*;

    let bytes = match event.code {
        Char(c) => {
            if event.modifiers.contains(KeyModifiers::CONTROL) {
                // Ctrl+letter produces control character
                let ctrl_char = (c.to_ascii_lowercase() as u8).wrapping_sub(b'a' - 1);
                vec![ctrl_char]
            } else if event.modifiers.contains(KeyModifiers::ALT) {
                // Alt+letter sends ESC followed by the letter
                vec![0x1b, c as u8]
            } else {
                c.to_string().into_bytes()
            }
        }
        Enter => vec![b'\r'],
        Backspace => vec![0x7f], // DEL character
        Tab => vec![b'\t'],
        BackTab => vec![0x1b, b'[', b'Z'], // Shift+Tab
        Esc => vec![0x1b],
        Up => vec![0x1b, b'[', b'A'],
        Down => vec![0x1b, b'[', b'B'],
        Right => vec![0x1b, b'[', b'C'],
        Left => vec![0x1b, b'[', b'D'],
        Home => vec![0x1b, b'[', b'H'],
        End => vec![0x1b, b'[', b'F'],
        PageUp => vec![0x1b, b'[', b'5', b'~'],
        PageDown => vec![0x1b, b'[', b'6', b'~'],
        Insert => vec![0x1b, b'[', b'2', b'~'],
        Delete => vec![0x1b, b'[', b'3', b'~'],
        F(n) => {
            // F1-F12 escape sequences
            match n {
                1 => vec![0x1b, b'O', b'P'],
                2 => vec![0x1b, b'O', b'Q'],
                3 => vec![0x1b, b'O', b'R'],
                4 => vec![0x1b, b'O', b'S'],
                5 => vec![0x1b, b'[', b'1', b'5', b'~'],
                6 => vec![0x1b, b'[', b'1', b'7', b'~'],
                7 => vec![0x1b, b'[', b'1', b'8', b'~'],
                8 => vec![0x1b, b'[', b'1', b'9', b'~'],
                9 => vec![0x1b, b'[', b'2', b'0', b'~'],
                10 => vec![0x1b, b'[', b'2', b'1', b'~'],
                11 => vec![0x1b, b'[', b'2', b'3', b'~'],
                12 => vec![0x1b, b'[', b'2', b'4', b'~'],
                _ => return None,
            }
        }
        Null => vec![0],
        CapsLock | ScrollLock | NumLock | PrintScreen | Pause | Menu | KeypadBegin => {
            return None;
        }
        _ => return None,
    };

    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState};

    fn make_key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    #[test]
    fn test_key_to_bytes_simple_char() {
        let event = make_key_event(KeyCode::Char('a'), KeyModifiers::empty());
        let bytes = key_event_to_bytes(&event);
        assert_eq!(bytes, Some(vec![b'a']));
    }

    #[test]
    fn test_key_to_bytes_ctrl_c() {
        let event = make_key_event(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let bytes = key_event_to_bytes(&event);
        assert_eq!(bytes, Some(vec![3])); // ETX (Ctrl+C)
    }

    #[test]
    fn test_key_to_bytes_enter() {
        let event = make_key_event(KeyCode::Enter, KeyModifiers::empty());
        let bytes = key_event_to_bytes(&event);
        assert_eq!(bytes, Some(vec![b'\r']));
    }

    #[test]
    fn test_key_to_bytes_arrow_up() {
        let event = make_key_event(KeyCode::Up, KeyModifiers::empty());
        let bytes = key_event_to_bytes(&event);
        assert_eq!(bytes, Some(vec![0x1b, b'[', b'A']));
    }

    #[test]
    fn test_key_to_bytes_escape() {
        let event = make_key_event(KeyCode::Esc, KeyModifiers::empty());
        let bytes = key_event_to_bytes(&event);
        assert_eq!(bytes, Some(vec![0x1b]));
    }

    #[test]
    fn test_key_to_bytes_f1() {
        let event = make_key_event(KeyCode::F(1), KeyModifiers::empty());
        let bytes = key_event_to_bytes(&event);
        assert_eq!(bytes, Some(vec![0x1b, b'O', b'P']));
    }
}
