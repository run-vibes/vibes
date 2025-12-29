use anyhow::Result;
use clap::{Parser, Subcommand};

mod client;
mod commands;
mod config;
mod daemon;
mod input;
mod server;
mod terminal;

#[derive(Parser)]
#[command(name = "vibes", about = "Remote control for Claude Code")]
#[command(version, propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage Cloudflare Access authentication
    Auth(commands::auth::AuthArgs),
    /// Proxy Claude Code with vibes enhancements
    Claude(commands::claude::ClaudeArgs),
    /// Manage configuration
    Config(commands::config::ConfigArgs),
    /// Groove continual learning and security
    Groove(commands::groove::GrooveArgs),
    /// Manage plugins
    Plugin(commands::plugin::PluginArgs),
    /// Run the vibes server
    Serve(commands::serve::ServeArgs),
    /// Manage active sessions
    Sessions(commands::sessions::SessionsArgs),
    /// Manage Cloudflare Tunnel
    Tunnel(commands::tunnel::TunnelArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt().with_env_filter(filter).init();

    match cli.command {
        Commands::Auth(args) => commands::auth::run(args).await,
        Commands::Claude(args) => commands::claude::run(args).await,
        Commands::Config(args) => commands::config::run(args),
        Commands::Groove(args) => commands::groove::run(args),
        Commands::Plugin(args) => commands::plugin::run(args),
        Commands::Serve(args) => commands::serve::run(args).await,
        Commands::Sessions(args) => commands::sessions::run(args).await,
        Commands::Tunnel(args) => commands::tunnel::run(args),
    }
}
