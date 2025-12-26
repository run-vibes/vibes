use crate::config::ConfigLoader;
use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration (merged)
    Show,
    /// Show configuration file paths
    Path,
}

pub fn run(args: ConfigArgs) -> Result<()> {
    match args.command {
        ConfigCommands::Show => show_config(),
        ConfigCommands::Path => show_paths(),
    }
}

fn show_config() -> Result<()> {
    let config = ConfigLoader::load()?;
    let toml_str = toml::to_string_pretty(&config)?;
    println!("{}", toml_str);
    Ok(())
}

fn show_paths() -> Result<()> {
    println!("User config:    {:?}", ConfigLoader::user_config_path());
    println!("Project config: {:?}", ConfigLoader::project_config_path());
    Ok(())
}
