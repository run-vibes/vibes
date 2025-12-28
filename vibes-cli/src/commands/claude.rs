//! Claude command - connects to vibes daemon via WebSocket

use anyhow::{Result, anyhow};
use clap::Args;
use std::io::{BufRead, Write};
use vibes_core::{ClaudeEvent, PluginHost, PluginHostConfig, VibesEvent};
use vibes_server::ws::ServerMessage;

use crate::client::VibesClient;
use crate::config::ConfigLoader;
use crate::daemon::ensure_daemon_running;

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

    // Determine if we're in interactive mode
    let interactive = args.interactive || (args.prompt.is_none() && args.resume.is_none());
    let prompt = args.prompt.clone();

    // Ensure daemon is running (unless --no-serve is set)
    if !args.no_serve {
        ensure_daemon_running(config.server.port).await?;
    }

    // Connect to daemon via WebSocket
    let url = format!("ws://127.0.0.1:{}/ws", config.server.port);
    let mut client = VibesClient::connect_url(&url).await?;

    // Load plugins
    let mut plugin_host = PluginHost::new(PluginHostConfig::default());
    if let Err(e) = plugin_host.load_all() {
        tracing::warn!("Failed to load plugins: {}", e);
    }

    // Create or resume session
    let session_id = if let Some(resume_id) = args.resume.clone() {
        // Resume existing session - just subscribe to it
        client.subscribe(vec![resume_id.clone()]).await?;
        resume_id
    } else {
        // Create new session
        client.create_session(args.session_name.clone()).await?
    };

    // Notify plugins of session
    plugin_host.dispatch_event(&VibesEvent::SessionCreated {
        session_id: session_id.clone(),
        name: args.session_name.clone(),
    });

    // Interactive mode: read prompts from stdin in a loop
    if interactive {
        interactive_loop(&mut client, &mut plugin_host, &session_id, prompt).await
    } else {
        // Non-interactive: send prompt and stream single response
        if let Some(prompt) = prompt {
            client.send_input(&session_id, &prompt).await?;
        }
        stream_output(&mut client, &mut plugin_host, &session_id).await
    }
}

/// Interactive mode: read prompts from stdin and send to session
async fn interactive_loop(
    client: &mut VibesClient,
    plugin_host: &mut PluginHost,
    session_id: &str,
    initial_prompt: Option<String>,
) -> Result<()> {
    let stdin = std::io::stdin();
    let mut reader = stdin.lock();

    // Send initial prompt if provided
    if let Some(prompt) = initial_prompt {
        client.send_input(session_id, &prompt).await?;
        stream_output(client, plugin_host, session_id).await?;
    }

    // Interactive prompt loop
    loop {
        // Print prompt indicator
        print!("\n> ");
        std::io::stdout().flush()?;

        // Read input line
        let mut input = String::new();
        match reader.read_line(&mut input) {
            Ok(0) => {
                // EOF (Ctrl+D)
                println!("\nGoodbye!");
                break;
            }
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }

                // Exit commands
                if matches!(input, "exit" | "quit" | "/exit" | "/quit") {
                    println!("Goodbye!");
                    break;
                }

                // Send input to session
                client.send_input(session_id, input).await?;
                stream_output(client, plugin_host, session_id).await?;
            }
            Err(e) => {
                return Err(anyhow!("Failed to read input: {}", e));
            }
        }
    }

    Ok(())
}

/// Stream events from WebSocket to terminal
async fn stream_output(
    client: &mut VibesClient,
    plugin_host: &mut PluginHost,
    session_id: &str,
) -> Result<()> {
    loop {
        match client.recv().await {
            Some(ServerMessage::Claude {
                session_id: sid,
                event,
            }) if sid == session_id => {
                // Dispatch event to plugins
                plugin_host.dispatch_event(&VibesEvent::Claude {
                    session_id: session_id.to_string(),
                    event: event.clone(),
                });

                // Handle event for terminal output
                match event {
                    ClaudeEvent::TextDelta { text } => {
                        print!("{}", text);
                        std::io::stdout().flush()?;
                    }
                    ClaudeEvent::TurnComplete { .. } => {
                        break;
                    }
                    ClaudeEvent::Error { message, .. } => {
                        eprintln!("Error: {}", message);
                        break;
                    }
                    _ => {
                        // Ignore other events
                    }
                }
            }
            Some(ServerMessage::SessionState {
                session_id: sid,
                state,
            }) if sid == session_id => {
                tracing::debug!("Session state changed: {}", state);
                plugin_host.dispatch_event(&VibesEvent::SessionStateChanged {
                    session_id: session_id.to_string(),
                    state,
                });
            }
            Some(ServerMessage::Error {
                session_id: Some(sid),
                message,
                ..
            }) if sid == session_id => {
                eprintln!("Error: {}", message);
                break;
            }
            Some(_) => {
                // Ignore other messages
            }
            None => {
                // Connection closed
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::VibesConfig;
    use vibes_core::PrintModeConfig;

    fn build_backend_config(args: &ClaudeArgs, config: &VibesConfig) -> PrintModeConfig {
        // Merge CLI args with config defaults
        let allowed_tools = args
            .allowed_tools
            .as_ref()
            .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
            .or_else(|| config.session.default_allowed_tools.clone())
            .unwrap_or_default();

        let working_dir = config
            .session
            .working_dir
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());

        // CLI args override config defaults
        let model = args
            .model
            .clone()
            .or_else(|| config.session.default_model.clone());

        PrintModeConfig {
            claude_path: None,
            allowed_tools,
            working_dir,
            model,
            system_prompt: args.system_prompt.clone(),
        }
    }

    #[test]
    fn test_build_backend_config_uses_cli_args() {
        let args = ClaudeArgs {
            session_name: None,
            no_serve: false,
            interactive: false,
            continue_session: false,
            resume: None,
            model: Some("claude-opus-4-5".to_string()),
            allowed_tools: Some("Read,Write,Bash".to_string()),
            system_prompt: None,
            prompt: Some("test".to_string()),
            passthrough: vec![],
        };
        let config = VibesConfig::default();

        let backend_config = build_backend_config(&args, &config);

        assert_eq!(
            backend_config.allowed_tools,
            vec!["Read".to_string(), "Write".to_string(), "Bash".to_string()]
        );
    }

    #[test]
    fn test_build_backend_config_uses_config_defaults() {
        let args = ClaudeArgs {
            session_name: None,
            no_serve: false,
            interactive: false,
            continue_session: false,
            resume: None,
            model: None,
            allowed_tools: None,
            system_prompt: None,
            prompt: Some("test".to_string()),
            passthrough: vec![],
        };
        let mut config = VibesConfig::default();
        config.session.default_allowed_tools = Some(vec!["Glob".to_string(), "Grep".to_string()]);

        let backend_config = build_backend_config(&args, &config);

        assert_eq!(
            backend_config.allowed_tools,
            vec!["Glob".to_string(), "Grep".to_string()]
        );
    }

    #[test]
    fn test_build_backend_config_cli_overrides_config() {
        let args = ClaudeArgs {
            session_name: None,
            no_serve: false,
            interactive: false,
            continue_session: false,
            resume: None,
            model: None,
            allowed_tools: Some("Read".to_string()),
            system_prompt: None,
            prompt: Some("test".to_string()),
            passthrough: vec![],
        };
        let mut config = VibesConfig::default();
        config.session.default_allowed_tools = Some(vec!["Glob".to_string(), "Grep".to_string()]);

        let backend_config = build_backend_config(&args, &config);

        // CLI takes precedence
        assert_eq!(backend_config.allowed_tools, vec!["Read".to_string()]);
    }

    #[test]
    fn test_build_backend_config_model_from_cli() {
        let args = ClaudeArgs {
            session_name: None,
            no_serve: false,
            interactive: false,
            continue_session: false,
            resume: None,
            model: Some("claude-opus-4-5".to_string()),
            allowed_tools: None,
            system_prompt: None,
            prompt: Some("test".to_string()),
            passthrough: vec![],
        };
        let config = VibesConfig::default();

        let backend_config = build_backend_config(&args, &config);

        assert_eq!(backend_config.model, Some("claude-opus-4-5".to_string()));
    }

    #[test]
    fn test_build_backend_config_model_from_config() {
        let args = ClaudeArgs {
            session_name: None,
            no_serve: false,
            interactive: false,
            continue_session: false,
            resume: None,
            model: None,
            allowed_tools: None,
            system_prompt: None,
            prompt: Some("test".to_string()),
            passthrough: vec![],
        };
        let mut config = VibesConfig::default();
        config.session.default_model = Some("claude-sonnet-4".to_string());

        let backend_config = build_backend_config(&args, &config);

        assert_eq!(backend_config.model, Some("claude-sonnet-4".to_string()));
    }

    #[test]
    fn test_build_backend_config_model_cli_overrides_config() {
        let args = ClaudeArgs {
            session_name: None,
            no_serve: false,
            interactive: false,
            continue_session: false,
            resume: None,
            model: Some("claude-opus-4-5".to_string()),
            allowed_tools: None,
            system_prompt: None,
            prompt: Some("test".to_string()),
            passthrough: vec![],
        };
        let mut config = VibesConfig::default();
        config.session.default_model = Some("claude-sonnet-4".to_string());

        let backend_config = build_backend_config(&args, &config);

        // CLI takes precedence
        assert_eq!(backend_config.model, Some("claude-opus-4-5".to_string()));
    }

    #[test]
    fn test_build_backend_config_system_prompt() {
        let args = ClaudeArgs {
            session_name: None,
            no_serve: false,
            interactive: false,
            continue_session: false,
            resume: None,
            model: None,
            allowed_tools: None,
            system_prompt: Some("You are a helpful assistant.".to_string()),
            prompt: Some("test".to_string()),
            passthrough: vec![],
        };
        let config = VibesConfig::default();

        let backend_config = build_backend_config(&args, &config);

        assert_eq!(
            backend_config.system_prompt,
            Some("You are a helpful assistant.".to_string())
        );
    }
}
