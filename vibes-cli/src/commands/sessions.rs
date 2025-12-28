//! Sessions management commands

use anyhow::Result;
use clap::{Args, Subcommand};
use tracing::info;
use vibes_core::ClaudeEvent;
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
                        let name = session.name.unwrap_or_else(|| "(unnamed)".to_string());
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

/// Attach to an existing session
async fn attach_session(session_id: &str) -> Result<()> {
    info!(session_id = %session_id, "Attaching to session");

    let mut client = VibesClient::connect().await?;

    // Subscribe to the session
    client.subscribe(vec![session_id.to_string()]).await?;

    println!("Attached to session: {}", session_id);
    println!("Listening for events... (Ctrl+C to detach)");
    println!();

    // Stream events from the session
    while let Some(msg) = client.recv().await {
        match msg {
            ServerMessage::Claude {
                session_id: sid,
                event,
            } if sid == session_id => match event {
                ClaudeEvent::TextDelta { text } => {
                    print!("{}", text);
                }
                ClaudeEvent::ThinkingDelta { text } => {
                    print!("\x1b[2m{}\x1b[0m", text); // Dim for thinking
                }
                ClaudeEvent::ToolUseStart { name, .. } => {
                    println!("\n\x1b[33m▶ Tool: {}\x1b[0m", name);
                }
                ClaudeEvent::ToolResult {
                    output, is_error, ..
                } => {
                    if is_error {
                        println!("\x1b[31m✗ Error: {}\x1b[0m", output);
                    } else {
                        println!("\x1b[32m✓ Result: {}\x1b[0m", output);
                    }
                }
                ClaudeEvent::TurnComplete { .. } => {
                    println!("\n\x1b[36m── Turn complete ──\x1b[0m\n");
                }
                ClaudeEvent::Error { message, .. } => {
                    println!("\x1b[31mError: {}\x1b[0m", message);
                }
                _ => {}
            },
            ServerMessage::SessionRemoved {
                session_id: sid, ..
            } if sid == session_id => {
                println!("\n\x1b[33mSession {} was removed\x1b[0m", session_id);
                break;
            }
            ServerMessage::OwnershipTransferred {
                session_id: sid,
                you_are_owner,
                ..
            } if sid == session_id => {
                if you_are_owner {
                    println!("\n\x1b[32mYou are now the owner of this session\x1b[0m");
                }
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
