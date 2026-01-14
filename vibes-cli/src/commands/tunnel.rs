//! Tunnel management commands

use anyhow::Result;
use clap::{Args, Subcommand};

use super::setup::tunnel_wizard;

/// Arguments for the tunnel command
#[derive(Debug, Args)]
pub struct TunnelArgs {
    #[command(subcommand)]
    pub command: TunnelCommand,
}

/// Tunnel subcommands
#[derive(Debug, Subcommand)]
pub enum TunnelCommand {
    /// Interactive setup wizard for tunnel configuration
    Setup,
    /// Start the tunnel
    Start,
    /// Stop the tunnel
    Stop,
    /// Show tunnel status
    Status,
    /// Start a quick tunnel (temporary URL)
    Quick,
}

/// Run the tunnel command
pub async fn run(args: TunnelArgs) -> Result<()> {
    match args.command {
        TunnelCommand::Setup => run_setup().await,
        TunnelCommand::Start => run_start(),
        TunnelCommand::Stop => run_stop(),
        TunnelCommand::Status => run_status(),
        TunnelCommand::Quick => run_quick(),
    }
}

async fn run_setup() -> Result<()> {
    tunnel_wizard::run().await?;
    Ok(())
}

fn run_start() -> Result<()> {
    println!("Starting tunnel...");
    println!("Hint: Use 'vibes serve --tunnel' to start server with tunnel");
    Ok(())
}

fn run_stop() -> Result<()> {
    println!("Stopping tunnel...");
    Ok(())
}

fn run_status() -> Result<()> {
    println!("Tunnel status: Not running");
    println!("Use 'vibes serve --tunnel' or 'vibes serve --quick-tunnel' to start");
    Ok(())
}

fn run_quick() -> Result<()> {
    println!("Starting quick tunnel...");
    println!("Hint: Use 'vibes serve --quick-tunnel' to start server with quick tunnel");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct TestCli {
        #[command(flatten)]
        tunnel: TunnelArgs,
    }

    #[test]
    fn test_tunnel_setup_command() {
        let cli = TestCli::parse_from(["test", "setup"]);
        assert!(matches!(cli.tunnel.command, TunnelCommand::Setup));
    }

    #[test]
    fn test_tunnel_start_command() {
        let cli = TestCli::parse_from(["test", "start"]);
        assert!(matches!(cli.tunnel.command, TunnelCommand::Start));
    }

    #[test]
    fn test_tunnel_stop_command() {
        let cli = TestCli::parse_from(["test", "stop"]);
        assert!(matches!(cli.tunnel.command, TunnelCommand::Stop));
    }

    #[test]
    fn test_tunnel_status_command() {
        let cli = TestCli::parse_from(["test", "status"]);
        assert!(matches!(cli.tunnel.command, TunnelCommand::Status));
    }

    #[test]
    fn test_tunnel_quick_command() {
        let cli = TestCli::parse_from(["test", "quick"]);
        assert!(matches!(cli.tunnel.command, TunnelCommand::Quick));
    }
}
