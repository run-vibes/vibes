//! Vibes serve command for running the daemon server
//!
//! The serve command runs the vibes server which provides:
//! - HTTP API for session management
//! - WebSocket for real-time event streaming
//! - Web UI for browser-based access

use anyhow::Result;
use clap::{Args, Subcommand};
use tracing::{info, warn};
use vibes_server::{ServerConfig, VibesServer};

use crate::config::ConfigLoader;
use crate::daemon::{
    DaemonState, clear_daemon_state, is_process_alive, read_daemon_state, terminate_process,
    write_daemon_state,
};
use crate::ollama::OllamaManager;

/// Arguments for the serve command
#[derive(Debug, Args)]
#[command(after_long_help = "\
Examples:
  vibes serve                      Run server in foreground
  vibes serve -d                   Run server as background daemon
  vibes serve --quick-tunnel       Run with temporary public URL
  vibes serve -p 8080              Run on custom port
  vibes serve status               Check if daemon is running
  vibes serve stop                 Stop the daemon
")]
pub struct ServeArgs {
    /// Subcommand (stop, status)
    #[command(subcommand)]
    pub command: Option<ServeCommand>,

    /// Port to listen on (defaults to config; config default is 7432)
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Host to bind to (defaults to config; config default is 127.0.0.1)
    #[arg(long)]
    pub host: Option<String>,

    /// Run as a background daemon
    #[arg(short, long)]
    pub daemon: bool,

    /// Start with named tunnel (from config)
    #[arg(long)]
    pub tunnel: bool,

    /// Start with quick tunnel (temporary URL)
    #[arg(long, conflicts_with = "tunnel")]
    pub quick_tunnel: bool,

    /// Enable push notifications
    #[arg(long)]
    pub notify: bool,
}

/// Subcommands for serve
#[derive(Debug, Subcommand)]
pub enum ServeCommand {
    /// Stop the running daemon
    Stop,
    /// Show daemon status
    Status,
}

/// Resolved server settings after merging config and CLI args
struct ResolvedSettings {
    host: String,
    port: u16,
    tunnel: bool,
    quick_tunnel: bool,
    notify: bool,
    /// Ollama base URL from config (e.g., "http://localhost:11434")
    ollama_base_url: Option<String>,
}

/// Run the serve command
pub async fn run(args: ServeArgs) -> Result<()> {
    match args.command {
        Some(ServeCommand::Stop) => stop_daemon().await,
        Some(ServeCommand::Status) => show_status().await,
        None => {
            // Load config and merge with CLI args
            let config = ConfigLoader::load()?;

            // Get Ollama base URL from config if enabled
            let ollama_base_url = if config.models.ollama.enabled {
                Some(config.models.ollama.base_url())
            } else {
                None
            };

            let settings = ResolvedSettings {
                host: args.host.unwrap_or(config.server.host),
                port: args.port.unwrap_or(config.server.port),
                tunnel: args.tunnel,
                quick_tunnel: args.quick_tunnel,
                notify: args.notify,
                ollama_base_url,
            };

            // Start Ollama if enabled
            let ollama_manager = OllamaManager::new(config.models.ollama);
            if let Err(e) = ollama_manager.start().await {
                warn!("Failed to start Ollama: {}", e);
            }

            if args.daemon {
                start_daemon(&settings).await
            } else {
                run_foreground(&settings, ollama_manager).await
            }
        }
    }
}

/// Run the server in the foreground
async fn run_foreground(settings: &ResolvedSettings, _ollama: OllamaManager) -> Result<()> {
    // Keep OllamaManager alive - it will be dropped (and stopped) when this function returns
    let config = ServerConfig {
        host: settings.host.clone(),
        port: settings.port,
        tunnel_enabled: settings.tunnel,
        tunnel_quick: settings.quick_tunnel,
        notify_enabled: settings.notify,
        ollama_base_url: settings.ollama_base_url.clone(),
    };

    info!("Starting vibes server on {}:{}", config.host, config.port);

    // Write daemon state file
    let state = DaemonState::new(settings.port);
    if let Err(e) = write_daemon_state(&state) {
        tracing::warn!("Failed to write daemon state file: {}", e);
    }

    // Create server with Iggy persistence
    let server = VibesServer::new_with_iggy(config).await?;
    let result = server.run().await;

    // Clear daemon state file on exit
    if let Err(e) = clear_daemon_state() {
        tracing::warn!("Failed to clear daemon state file: {}", e);
    }

    result.map_err(Into::into)
}

/// Start the daemon in the background
async fn start_daemon(settings: &ResolvedSettings) -> Result<()> {
    use crate::daemon::ensure_daemon_running;

    // Use the auto-start machinery to spawn a detached daemon
    ensure_daemon_running(&settings.host, settings.port).await?;

    println!(
        "Vibes daemon started on {}:{}",
        settings.host, settings.port
    );
    Ok(())
}

/// Stop the running daemon
async fn stop_daemon() -> Result<()> {
    match read_daemon_state() {
        Some(state) => {
            if is_process_alive(state.pid) {
                println!("Stopping vibes daemon (PID: {})", state.pid);
                if terminate_process(state.pid) {
                    // Wait briefly for process to terminate
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                    // Check if it actually stopped
                    if is_process_alive(state.pid) {
                        println!("Daemon still running after SIGTERM");
                    } else {
                        println!("Daemon stopped successfully");
                    }

                    // Clean up state file
                    let _ = clear_daemon_state();
                    Ok(())
                } else {
                    anyhow::bail!("Failed to send SIGTERM to daemon")
                }
            } else {
                println!("Daemon is not running (stale state file)");
                let _ = clear_daemon_state();
                Ok(())
            }
        }
        None => {
            println!("No daemon is running");
            Ok(())
        }
    }
}

/// Show the daemon status
async fn show_status() -> Result<()> {
    match read_daemon_state() {
        Some(state) => {
            if is_process_alive(state.pid) {
                let uptime = chrono::Utc::now() - state.started_at;
                println!("Vibes daemon is running");
                println!("  PID:     {}", state.pid);
                println!("  Port:    {}", state.port);
                println!("  Uptime:  {}s", uptime.num_seconds());
                Ok(())
            } else {
                println!("Vibes daemon is not running (stale state file)");
                // Clean up stale state file
                let _ = clear_daemon_state();
                Ok(())
            }
        }
        None => {
            println!("Vibes daemon is not running");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serve_args_defaults() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            serve: ServeArgs,
        }

        let cli = TestCli::parse_from(["test"]);
        // host and port default to None (loaded from config at runtime)
        assert!(cli.serve.port.is_none());
        assert!(cli.serve.host.is_none());
        assert!(!cli.serve.daemon);
        assert!(cli.serve.command.is_none());
    }

    #[test]
    fn test_serve_args_custom_port() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            serve: ServeArgs,
        }

        let cli = TestCli::parse_from(["test", "--port", "8080"]);
        assert_eq!(cli.serve.port, Some(8080));
    }

    #[test]
    fn test_serve_args_custom_host() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            serve: ServeArgs,
        }

        let cli = TestCli::parse_from(["test", "--host", "0.0.0.0"]);
        assert_eq!(cli.serve.host, Some("0.0.0.0".to_string()));
    }

    #[test]
    fn test_serve_args_daemon_flag() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            serve: ServeArgs,
        }

        let cli = TestCli::parse_from(["test", "--daemon"]);
        assert!(cli.serve.daemon);
    }

    #[test]
    fn test_serve_command_stop() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            serve: ServeArgs,
        }

        let cli = TestCli::parse_from(["test", "stop"]);
        assert!(matches!(cli.serve.command, Some(ServeCommand::Stop)));
    }

    #[test]
    fn test_serve_command_status() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            serve: ServeArgs,
        }

        let cli = TestCli::parse_from(["test", "status"]);
        assert!(matches!(cli.serve.command, Some(ServeCommand::Status)));
    }

    #[test]
    fn test_serve_args_tunnel_flag() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            serve: ServeArgs,
        }

        let cli = TestCli::parse_from(["test", "--tunnel"]);
        assert!(cli.serve.tunnel);
        assert!(!cli.serve.quick_tunnel);
    }

    #[test]
    fn test_serve_args_quick_tunnel_flag() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            serve: ServeArgs,
        }

        let cli = TestCli::parse_from(["test", "--quick-tunnel"]);
        assert!(cli.serve.quick_tunnel);
        assert!(!cli.serve.tunnel);
    }

    #[test]
    fn test_serve_args_tunnel_flags_conflict() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            serve: ServeArgs,
        }

        // Both flags together should fail
        let result = TestCli::try_parse_from(["test", "--tunnel", "--quick-tunnel"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_serve_args_notify_flag() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            serve: ServeArgs,
        }

        let cli = TestCli::parse_from(["test", "--notify"]);
        assert!(cli.serve.notify);
    }
}
