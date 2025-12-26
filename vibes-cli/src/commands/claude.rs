use crate::config::{ConfigLoader, VibesConfig};
use crate::server;
use anyhow::{Result, anyhow};
use clap::Args;
use std::io::Write;
use vibes_core::{
    BackendFactory, ClaudeEvent, PluginHost, PluginHostConfig, PrintModeBackendFactory,
    PrintModeConfig, VibesEvent,
};

#[derive(Args)]
pub struct ClaudeArgs {
    // === Vibes-specific flags ===
    /// Human-friendly session name (shown in vibes UI)
    #[arg(long)]
    pub session_name: Option<String>,

    /// Disable background server for this session
    #[arg(long)]
    pub no_serve: bool,

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
    let backend_config = build_backend_config(&args, &config);

    // Start server stub if enabled
    if config.server.auto_start && !args.no_serve {
        tokio::spawn(server::start_stub(config.server.port));
    }

    // Load plugins
    let mut plugin_host = PluginHost::new(PluginHostConfig::default());
    if let Err(e) = plugin_host.load_all() {
        tracing::warn!("Failed to load plugins: {}", e);
    }

    // Build prompt (required)
    let prompt = args
        .prompt
        .ok_or_else(|| anyhow!("No prompt provided. Usage: vibes claude \"your prompt\""))?;

    // Explicitly error if continue_session is requested, since the backend does not support it yet
    if args.continue_session {
        return Err(anyhow!(
            "The --continue / -c flag is not yet supported.\n\
             Please use --resume <SESSION_ID> to resume a specific session instead."
        ));
    }

    // Determine Claude session ID for resume
    let claude_session_id = args.resume.clone();

    // Create backend directly (simpler for CLI use case)
    let factory = PrintModeBackendFactory::new(backend_config);
    let mut backend = factory.create(claude_session_id);

    // Subscribe before sending
    let mut rx = backend.subscribe();

    // Generate a session ID for plugin events
    let session_id = args
        .session_name
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Notify plugins of session creation
    plugin_host.dispatch_event(&VibesEvent::SessionCreated {
        session_id: session_id.clone(),
        name: args.session_name.clone(),
    });

    // Send the prompt
    backend
        .send(&prompt)
        .await
        .map_err(|e| anyhow!("Failed to send prompt: {}", e))?;

    // Stream output to terminal, dispatching events to plugins
    stream_output(&mut rx, &mut plugin_host, &session_id).await
}

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
        claude_path: None, // Use default "claude"
        allowed_tools,
        working_dir,
        model,
        system_prompt: args.system_prompt.clone(),
    }
}

async fn stream_output(
    rx: &mut tokio::sync::broadcast::Receiver<ClaudeEvent>,
    plugin_host: &mut PluginHost,
    session_id: &str,
) -> Result<()> {
    loop {
        match rx.recv().await {
            Ok(event) => {
                // Dispatch event to plugins (wrapped in VibesEvent::Claude)
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
                        // Don't print extra newline - match claude's output exactly
                        break;
                    }
                    ClaudeEvent::Error { message, .. } => {
                        eprintln!("Error: {}", message);
                        break;
                    }
                    _ => {
                        // Ignore other events (ThinkingDelta, ToolUseStart, etc.)
                        // Output should be identical to raw claude command
                    }
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                break;
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                // Receiver lagged behind, continue receiving
                continue;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_backend_config_uses_cli_args() {
        let args = ClaudeArgs {
            session_name: None,
            no_serve: false,
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
