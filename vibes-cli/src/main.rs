use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod config;
mod daemon;
mod server;

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
    /// Proxy Claude Code with vibes enhancements
    Claude(commands::claude::ClaudeArgs),
    /// Manage configuration
    Config(commands::config::ConfigArgs),
    /// Manage plugins
    Plugin(commands::plugin::PluginArgs),
    /// Run the vibes server
    Serve(commands::serve::ServeArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt().with_env_filter(filter).init();

    match cli.command {
        Commands::Claude(args) => commands::claude::run(args).await,
        Commands::Config(args) => commands::config::run(args),
        Commands::Plugin(args) => commands::plugin::run(args),
        Commands::Serve(args) => commands::serve::run(args).await,
    }
}
