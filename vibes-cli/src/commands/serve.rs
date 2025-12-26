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
    };

    info!("Starting vibes server on {}:{}", config.host, config.port);

    let server = VibesServer::new(config);
    server.run().await?;

    Ok(())
}

/// Start the daemon in the background (stub)
async fn start_daemon(args: &ServeArgs) -> Result<()> {
    // TODO: Implement in Task 3.4
    info!(
        "Starting daemon on {}:{} (not yet implemented)",
        args.host, args.port
    );
    anyhow::bail!("Daemon mode not yet implemented. Run without --daemon for foreground mode.")
}

/// Stop the running daemon (stub)
async fn stop_daemon() -> Result<()> {
    // TODO: Implement in Task 3.3
    info!("Stopping daemon (not yet implemented)");
    anyhow::bail!("Daemon stop not yet implemented")
}

/// Show the daemon status (stub)
async fn show_status() -> Result<()> {
    // TODO: Implement in Task 3.3
    info!("Checking daemon status (not yet implemented)");
    anyhow::bail!("Daemon status not yet implemented")
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
}
