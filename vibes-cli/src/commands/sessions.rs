//! Sessions management commands

use anyhow::Result;
use base64::Engine;
use clap::{Args, Subcommand};
use std::io::{self, Write};
use tracing::info;
use vibes_server::ws::ServerMessage;

use crate::client::VibesClient;

/// Sessions management arguments
#[derive(Args, Debug)]
pub struct SessionsArgs {
    #[command(subcommand)]
    pub command: SessionsCommands,
}

/// Sessions subcommands
#[derive(Subcommand, Debug)]
pub enum SessionsCommands {
    /// List all active sessions
    List,
    /// Attach to an existing session
    Attach {
        /// Session ID to attach to
        session_id: String,
    },
    /// Kill a session
    Kill {
        /// Session ID to kill
        session_id: String,
    },
}

/// Run sessions command
pub async fn run(args: SessionsArgs) -> Result<()> {
    match args.command {
        SessionsCommands::List => list_sessions().await,
        SessionsCommands::Attach { session_id } => attach_session(&session_id).await,
        SessionsCommands::Kill { session_id } => kill_session(&session_id).await,
    }
}

/// List all active sessions
async fn list_sessions() -> Result<()> {
    let mut client = VibesClient::connect().await?;

    // Send ListSessions request
    let request_id = uuid::Uuid::new_v4().to_string();
    client.send_list_sessions(&request_id).await?;

    // Wait for response
    while let Some(msg) = client.recv().await {
        match msg {
            ServerMessage::SessionList {
                request_id: _,
                sessions,
            } => {
                if sessions.is_empty() {
                    println!("No active sessions");
                } else {
                    println!("Active sessions:");
                    println!();
                    for session in sessions {
                        let name = session.name.as_deref().unwrap_or("(unnamed)");
                        let owner_marker = if session.is_owner { " (owner)" } else { "" };
                        println!("  {} - {}{}", session.id, name, owner_marker);
                        println!(
                            "    State: {}, Subscribers: {}",
                            session.state, session.subscriber_count
                        );
                    }
                }
                break;
            }
            ServerMessage::Error { message, .. } => {
                anyhow::bail!("Error listing sessions: {}", message);
            }
            _ => {
                // Ignore other messages while waiting for response
            }
        }
    }

    Ok(())
}

/// Attach to an existing session (read-only view of PTY output)
async fn attach_session(session_id: &str) -> Result<()> {
    info!(session_id = %session_id, "Attaching to session");

    let mut client = VibesClient::connect().await?;

    // Attach to the session to receive output (no name, cwd, or dimensions since session already exists)
    client.attach(session_id, None, None, None, None).await?;

    eprintln!("Attached to session: {}", session_id);
    eprintln!("Streaming PTY output... (Ctrl+C to detach)");
    eprintln!();

    let mut stdout = io::stdout();

    // Stream PTY output from the session
    while let Some(msg) = client.recv().await {
        match msg {
            ServerMessage::AttachAck {
                session_id: sid, ..
            } if sid == session_id => {
                // Successfully attached
            }
            ServerMessage::PtyReplay {
                session_id: sid,
                data,
            } if sid == session_id => {
                // Replay scrollback buffer
                if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(&data) {
                    let _ = stdout.write_all(&decoded);
                    let _ = stdout.flush();
                }
            }
            ServerMessage::PtyOutput {
                session_id: sid,
                data,
            } if sid == session_id => {
                // Real-time PTY output
                if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(&data) {
                    let _ = stdout.write_all(&decoded);
                    let _ = stdout.flush();
                }
            }
            ServerMessage::PtyExit {
                session_id: sid,
                exit_code,
            } if sid == session_id => {
                eprintln!(
                    "\n\x1b[33mSession {} exited with code {:?}\x1b[0m",
                    session_id, exit_code
                );
                break;
            }
            ServerMessage::SessionRemoved {
                session_id: sid, ..
            } if sid == session_id => {
                eprintln!("\n\x1b[33mSession {} was removed\x1b[0m", session_id);
                break;
            }
            ServerMessage::Error {
                session_id: Some(sid),
                message,
                ..
            } if sid == session_id => {
                anyhow::bail!("Error: {}", message);
            }
            _ => {}
        }
    }

    Ok(())
}

/// Kill a session
async fn kill_session(session_id: &str) -> Result<()> {
    info!(session_id = %session_id, "Killing session");

    let mut client = VibesClient::connect().await?;

    // Send KillSession request
    client.send_kill_session(session_id).await?;

    // Wait for response
    while let Some(msg) = client.recv().await {
        match msg {
            ServerMessage::SessionRemoved {
                session_id: sid, ..
            } if sid == session_id => {
                println!("Session {} killed", session_id);
                break;
            }
            ServerMessage::Error {
                session_id: Some(sid),
                message,
                ..
            } if sid == session_id => {
                anyhow::bail!("Error killing session: {}", message);
            }
            _ => {
                // Ignore other messages while waiting for response
            }
        }
    }

    Ok(())
}
