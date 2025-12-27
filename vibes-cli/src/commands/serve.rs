//! Vibes serve command for running the daemon server
//!
//! The serve command runs the vibes server which provides:
//! - HTTP API for session management
//! - WebSocket for real-time event streaming
//! - Web UI for browser-based access

use anyhow::Result;
use clap::{Args, Subcommand};
use tracing::info;
use vibes_server::{ServerConfig, VibesServer};

use crate::daemon::{
    DaemonState, clear_daemon_state, is_process_alive, read_daemon_state, terminate_process,
    write_daemon_state,
};

/// Default port for the vibes server
pub const DEFAULT_PORT: u16 = 7743;
/// Default host for the vibes server
pub const DEFAULT_HOST: &str = "127.0.0.1";

/// Arguments for the serve command
#[derive(Debug, Args)]
pub struct ServeArgs {
    /// Subcommand (stop, status)
    #[command(subcommand)]
    pub command: Option<ServeCommand>,

    /// Port to listen on
    #[arg(short, long, default_value_t = DEFAULT_PORT)]
    pub port: u16,

    /// Host to bind to
    #[arg(long, default_value = DEFAULT_HOST)]
    pub host: String,

    /// Run as a background daemon
    #[arg(short, long)]
    pub daemon: bool,

    /// Start with named tunnel (from config)
    #[arg(long)]
    pub tunnel: bool,

    /// Start with quick tunnel (temporary URL)
    #[arg(long, conflicts_with = "tunnel")]
    pub quick_tunnel: bool,
}

/// Subcommands for serve
#[derive(Debug, Subcommand)]
pub enum ServeCommand {
    /// Stop the running daemon
    Stop,
    /// Show daemon status
    Status,
}

/// Run the serve command
pub async fn run(args: ServeArgs) -> Result<()> {
    match args.command {
        Some(ServeCommand::Stop) => stop_daemon().await,
        Some(ServeCommand::Status) => show_status().await,
        None if args.daemon => start_daemon(&args).await,
        None => run_foreground(&args).await,
    }
}

/// Run the server in the foreground
async fn run_foreground(args: &ServeArgs) -> Result<()> {
    let config = ServerConfig {
        host: args.host.clone(),
        port: args.port,
        tunnel_enabled: args.tunnel,
        tunnel_quick: args.quick_tunnel,
    };

    info!("Starting vibes server on {}:{}", config.host, config.port);

    // Write daemon state file
    let state = DaemonState::new(args.port);
    if let Err(e) = write_daemon_state(&state) {
        tracing::warn!("Failed to write daemon state file: {}", e);
    }

    // Run the server
    let server = VibesServer::new(config);
    let result = server.run().await;

    // Clear daemon state file on exit
    if let Err(e) = clear_daemon_state() {
        tracing::warn!("Failed to clear daemon state file: {}", e);
    }

    result.map_err(Into::into)
}

/// Start the daemon in the background
async fn start_daemon(args: &ServeArgs) -> Result<()> {
    use crate::daemon::ensure_daemon_running;

    // Use the auto-start machinery to spawn a detached daemon
    ensure_daemon_running(args.port).await?;

    println!("Vibes daemon started on {}:{}", args.host, args.port);
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
    fn test_default_port() {
        assert_eq!(DEFAULT_PORT, 7743);
    }

    #[test]
    fn test_default_host() {
        assert_eq!(DEFAULT_HOST, "127.0.0.1");
    }

    #[test]
    fn test_serve_args_defaults() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(flatten)]
            serve: ServeArgs,
        }

        let cli = TestCli::parse_from(["test"]);
        assert_eq!(cli.serve.port, DEFAULT_PORT);
        assert_eq!(cli.serve.host, DEFAULT_HOST);
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
        assert_eq!(cli.serve.port, 8080);
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
}
