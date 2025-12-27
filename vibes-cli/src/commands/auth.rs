//! Auth subcommands for vibes CLI

use anyhow::Result;
use clap::{Args, Subcommand};
use vibes_core::JwtValidator;

use crate::config::ConfigLoader;

#[derive(Debug, Args)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommand,
}

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// Show current auth configuration and status
    Status,
    /// Test auth configuration by fetching JWKS
    Test,
}

pub async fn run(args: AuthArgs) -> Result<()> {
    match args.command {
        AuthCommand::Status => status().await,
        AuthCommand::Test => test().await,
    }
}

async fn status() -> Result<()> {
    let config = ConfigLoader::load()?;

    println!("Auth Configuration:");
    println!("  Enabled: {}", config.auth.enabled);

    if config.auth.enabled {
        println!("  Team: {}", config.auth.team);
        println!(
            "  AUD: {}",
            if config.auth.aud.is_empty() {
                "(not set)"
            } else {
                &config.auth.aud
            }
        );
        println!("  Bypass localhost: {}", config.auth.bypass_localhost);
        println!("  Clock skew: {}s", config.auth.clock_skew_seconds);
        println!();

        if config.auth.is_valid() {
            println!("Status: Ready");
        } else {
            println!("Status: Invalid configuration (missing team or aud)");
        }
    } else {
        println!("Status: Disabled");
    }

    Ok(())
}

async fn test() -> Result<()> {
    let config = ConfigLoader::load()?;

    if !config.auth.enabled {
        println!("Auth is disabled. Enable it in config to test.");
        return Ok(());
    }

    if !config.auth.is_valid() {
        anyhow::bail!("Auth configuration is invalid (missing team or aud)");
    }

    println!("Testing auth configuration...");
    println!("Fetching JWKS from: {}", config.auth.jwks_url());

    let validator = JwtValidator::new(config.auth);
    validator.refresh_jwks().await?;

    println!("Success! JWKS fetched and cached.");

    Ok(())
}
