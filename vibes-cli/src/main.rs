use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};

mod client;
mod commands;
mod config;
mod daemon;
mod iggy_client;
mod input;
mod ollama;
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

/// Check if user is requesting top-level help (not subcommand help)
fn is_top_level_help() -> bool {
    let args: Vec<String> = std::env::args().collect();
    is_top_level_help_args(&args)
}

/// Testable version that takes args as parameter
fn is_top_level_help_args(args: &[String]) -> bool {
    // No arguments - show help with plugins
    if args.len() == 1 {
        return true;
    }

    // Check for --help or -h as first argument after program name
    // e.g., "vibes --help" or "vibes -h"
    if args.len() == 2 && (args[1] == "--help" || args[1] == "-h") {
        return true;
    }

    // Also handle "vibes -v --help" or "vibes --verbose --help"
    if args.len() == 3
        && (args[1] == "-v" || args[1] == "--verbose")
        && (args[2] == "--help" || args[2] == "-h")
    {
        return true;
    }

    false
}

/// Print help with plugin commands included
fn print_help_with_plugins() {
    let mut cmd = Cli::command();
    let mut help_str = Vec::new();
    cmd.write_help(&mut help_str).ok();
    let base_help = String::from_utf8_lossy(&help_str);

    let plugins = commands::plugin_dispatch::get_plugin_summaries();
    let full_help = commands::plugin_dispatch::format_top_level_help(&base_help, &plugins);

    print!("{}", full_help);
}

#[derive(Subcommand)]
enum Commands {
    /// Manage agents
    Agent(commands::agent::AgentArgs),
    /// Manage Cloudflare Access authentication
    Auth(commands::auth::AuthArgs),
    /// Proxy Claude Code with vibes enhancements
    Claude(commands::claude::ClaudeArgs),
    /// Manage configuration
    Config(commands::config::ConfigArgs),
    /// Manage evaluation studies
    Eval(commands::eval::EvalArgs),
    /// Send events to the EventLog
    Event(commands::event::EventArgs),
    /// Manage AI models and credentials
    Models(commands::models::ModelsArgs),
    /// Manage plugins
    Plugin(commands::plugin::PluginArgs),
    /// Run the vibes server
    Serve(commands::serve::ServeArgs),
    /// Manage active sessions
    Sessions(commands::sessions::SessionsArgs),
    /// Manage Cloudflare Tunnel
    Tunnel(commands::tunnel::TunnelArgs),
    /// Plugin commands (e.g., `vibes groove trust levels`)
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[tokio::main]
async fn main() -> Result<()> {
    // Intercept top-level help to include plugin commands
    if is_top_level_help() {
        print_help_with_plugins();
        return Ok(());
    }

    let cli = Cli::parse();

    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt().with_env_filter(filter).init();

    match cli.command {
        Commands::Agent(args) => commands::agent::run(args).await,
        Commands::Auth(args) => commands::auth::run(args).await,
        Commands::Claude(args) => commands::claude::run(args).await,
        Commands::Config(args) => commands::config::run(args),
        Commands::Eval(args) => commands::eval::run(args).await,
        Commands::Event(args) => commands::event::run(args).await,
        Commands::Models(args) => commands::models::run(args).await,
        Commands::Plugin(args) => commands::plugin::run(args),
        Commands::Serve(args) => commands::serve::run(args).await,
        Commands::Sessions(args) => commands::sessions::run(args).await,
        Commands::Tunnel(args) => commands::tunnel::run(args).await,
        Commands::External(args) => commands::plugin_dispatch::run(args),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_args_shows_help() {
        // Just the program name, no arguments - should show help
        let args = vec!["vibes".to_string()];
        assert!(
            is_top_level_help_args(&args),
            "vibes with no args should show help"
        );
    }

    #[test]
    fn test_help_flag_shows_help() {
        let args = vec!["vibes".to_string(), "--help".to_string()];
        assert!(is_top_level_help_args(&args));

        let args = vec!["vibes".to_string(), "-h".to_string()];
        assert!(is_top_level_help_args(&args));
    }

    #[test]
    fn test_subcommand_does_not_show_top_help() {
        let args = vec!["vibes".to_string(), "plugin".to_string()];
        assert!(
            !is_top_level_help_args(&args),
            "vibes plugin should not show top-level help"
        );
    }
}
