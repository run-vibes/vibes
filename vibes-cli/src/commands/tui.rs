//! TUI command - launches the terminal user interface
//!
//! Provides a full-screen terminal interface for vibes,
//! connecting to the daemon via WebSocket.

use anyhow::Result;
use clap::Args;
use tracing::info;

use crate::config::ConfigLoader;
use crate::daemon::ensure_daemon_running;

#[derive(Args, Default)]
#[command(after_long_help = "\
Examples:
  vibes tui                         Launch the TUI
  vibes tui --session <id>          Open Session view
  vibes tui --agent <id>            Open Agent view
")]
pub struct TuiArgs {
    /// Use specific theme
    #[arg(long)]
    pub theme: Option<String>,

    /// Start in session view
    #[arg(long)]
    pub session: Option<String>,

    /// Start in agent view
    #[arg(long)]
    pub agent: Option<String>,
}

pub async fn run(args: TuiArgs) -> Result<()> {
    let config = ConfigLoader::load()?;

    // Auto-start daemon if needed
    ensure_daemon_running(&config.server.host, config.server.port).await?;

    info!("Starting TUI...");

    // Determine initial view based on args
    let initial_view = match (&args.session, &args.agent) {
        (Some(id), _) => vibes_tui::View::Session(id.clone()),
        (_, Some(id)) => vibes_tui::View::Agent(id.clone()),
        _ => vibes_tui::View::Dashboard,
    };

    // Connection URL for daemon
    let url = format!("ws://127.0.0.1:{}/ws", config.server.port);

    // Retry loop for connection
    loop {
        // Attempt to connect to daemon via WebSocket
        let mut app = match vibes_tui::TuiClient::connect(&url).await {
            Ok(client) => {
                let mut app = vibes_tui::App::with_client_url(client, url.clone());
                app.views.replace(initial_view.clone());
                app
            }
            Err(e) => {
                // Start TUI with error message so user can retry
                let mut app = vibes_tui::App::new();
                app.server_url = Some(url.clone());
                app.handle_connection_error(&e.to_string());
                app
            }
        };

        // Run the TUI
        app.run().await?;

        // Check if retry was requested
        if app.retry_requested {
            info!("Retrying connection...");
            continue;
        }

        // Normal exit
        break;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tui_args_default_has_no_values() {
        let args = TuiArgs::default();
        assert!(args.theme.is_none());
        assert!(args.session.is_none());
        assert!(args.agent.is_none());
    }

    #[test]
    fn tui_args_can_set_session() {
        let args = TuiArgs {
            session: Some("sess-123".to_string()),
            ..Default::default()
        };
        assert_eq!(args.session, Some("sess-123".to_string()));
    }

    #[test]
    fn tui_args_can_set_agent() {
        let args = TuiArgs {
            agent: Some("agent-456".to_string()),
            ..Default::default()
        };
        assert_eq!(args.agent, Some("agent-456".to_string()));
    }

    #[test]
    fn tui_args_can_set_theme() {
        let args = TuiArgs {
            theme: Some("dark".to_string()),
            ..Default::default()
        };
        assert_eq!(args.theme, Some("dark".to_string()));
    }
}
