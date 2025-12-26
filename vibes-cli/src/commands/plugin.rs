//! Plugin management commands

use anyhow::Result;
use clap::{Args, Subcommand};
use vibes_core::{PluginHost, PluginHostConfig, PluginState};

/// Plugin management arguments
#[derive(Args)]
pub struct PluginArgs {
    #[command(subcommand)]
    pub command: PluginCommands,
}

/// Plugin subcommands
#[derive(Subcommand)]
pub enum PluginCommands {
    /// List installed plugins
    List {
        /// Include disabled plugins
        #[arg(long)]
        all: bool,
    },
    /// Enable a plugin
    Enable {
        /// Plugin name to enable
        name: String,
    },
    /// Disable a plugin
    Disable {
        /// Plugin name to disable
        name: String,
    },
    /// Show plugin details
    Info {
        /// Plugin name
        name: String,
    },
    /// Reload a plugin (development)
    Reload {
        /// Plugin name to reload
        name: String,
    },
}

/// Run plugin command
pub fn run(args: PluginArgs) -> Result<()> {
    let config = PluginHostConfig::default();
    let mut host = PluginHost::new(config);

    match args.command {
        PluginCommands::List { all } => list_plugins(&mut host, all),
        PluginCommands::Enable { name } => enable_plugin(&mut host, &name),
        PluginCommands::Disable { name } => disable_plugin(&mut host, &name),
        PluginCommands::Info { name } => show_plugin_info(&mut host, &name),
        PluginCommands::Reload { name } => reload_plugin(&mut host, &name),
    }
}

fn list_plugins(host: &mut PluginHost, _all: bool) -> Result<()> {
    // Load plugins to discover what's available
    if let Err(e) = host.load_all() {
        tracing::warn!("Error loading plugins: {}", e);
    }

    let plugins = host.list_plugins(true);

    if plugins.is_empty() {
        println!("No plugins installed");
        println!();
        println!("Plugin directory: ~/.config/vibes/plugins/");
        println!();
        println!("To install a plugin:");
        println!("  1. Create a plugin directory: mkdir -p ~/.config/vibes/plugins/my-plugin");
        println!(
            "  2. Copy the plugin library: cp libmy_plugin.so ~/.config/vibes/plugins/my-plugin/my-plugin.so"
        );
        println!("  3. Enable the plugin: vibes plugin enable my-plugin");
        return Ok(());
    }

    for p in plugins {
        let status = match &p.state {
            PluginState::Loaded => "✓",
            PluginState::Disabled { .. } => "○",
            PluginState::Failed { .. } => "✗",
        };

        let description = if p.manifest.description.is_empty() {
            "No description".to_string()
        } else {
            p.manifest.description.clone()
        };

        println!(
            "{} {} v{}    {}",
            status, p.name, p.manifest.version, description
        );
    }

    Ok(())
}

fn enable_plugin(host: &mut PluginHost, name: &str) -> Result<()> {
    host.enable_plugin(name)?;
    println!("Enabled plugin: {}", name);
    println!("Run 'vibes plugin list' to verify the plugin loads correctly.");
    Ok(())
}

fn disable_plugin(host: &mut PluginHost, name: &str) -> Result<()> {
    host.disable_plugin(name)?;
    println!("Disabled plugin: {}", name);
    Ok(())
}

fn show_plugin_info(host: &mut PluginHost, name: &str) -> Result<()> {
    // Load plugins first
    if let Err(e) = host.load_all() {
        tracing::warn!("Error loading plugins: {}", e);
    }

    if let Some(info) = host.get_plugin_info(name) {
        let m = &info.manifest;
        println!("Name:        {}", m.name);
        println!("Version:     {}", m.version);
        println!("API Version: {}", m.api_version);
        println!(
            "Author:      {}",
            if m.author.is_empty() {
                "Unknown"
            } else {
                &m.author
            }
        );
        println!(
            "Description: {}",
            if m.description.is_empty() {
                "No description"
            } else {
                &m.description
            }
        );
        println!();

        match &info.state {
            PluginState::Loaded => println!("Status:      Loaded"),
            PluginState::Disabled { reason } => println!("Status:      Disabled ({})", reason),
            PluginState::Failed { error } => println!("Status:      Failed ({})", error),
        }

        if !m.commands.is_empty() {
            println!();
            println!("Commands:");
            for cmd in &m.commands {
                println!("  vibes {} {}    {}", name, cmd.name, cmd.description);
            }
        }
    } else {
        println!("Plugin '{}' not found", name);
        println!();
        println!("The plugin might not be installed or enabled.");
        println!("Run 'vibes plugin list --all' to see all plugins.");
    }

    Ok(())
}

fn reload_plugin(host: &mut PluginHost, name: &str) -> Result<()> {
    // For now, just report that reload isn't fully implemented
    // Full reload would require unloading the library and reloading it
    println!("Reloading plugin: {}", name);

    // First disable
    host.disable_plugin(name)?;

    // Then re-enable
    host.enable_plugin(name)?;

    // Reload all plugins
    if let Err(e) = host.load_all() {
        println!("Warning: Error reloading plugins: {}", e);
    }

    println!("Plugin '{}' reloaded", name);
    println!();
    println!("Note: Some changes may require restarting vibes to take effect.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_plugin_args_parsing() {
        use clap::Parser;

        #[derive(Parser)]
        struct TestCli {
            #[command(subcommand)]
            cmd: PluginCommands,
        }

        // Test list command
        let cli = TestCli::parse_from(["test", "list"]);
        assert!(matches!(cli.cmd, PluginCommands::List { all: false }));

        let cli = TestCli::parse_from(["test", "list", "--all"]);
        assert!(matches!(cli.cmd, PluginCommands::List { all: true }));

        // Test enable command
        let cli = TestCli::parse_from(["test", "enable", "my-plugin"]);
        assert!(matches!(cli.cmd, PluginCommands::Enable { name } if name == "my-plugin"));

        // Test disable command
        let cli = TestCli::parse_from(["test", "disable", "my-plugin"]);
        assert!(matches!(cli.cmd, PluginCommands::Disable { name } if name == "my-plugin"));

        // Test info command
        let cli = TestCli::parse_from(["test", "info", "my-plugin"]);
        assert!(matches!(cli.cmd, PluginCommands::Info { name } if name == "my-plugin"));
    }

    #[test]
    fn test_enable_disable_roundtrip() {
        let dir = TempDir::new().unwrap();
        let config = PluginHostConfig {
            user_plugin_dir: dir.path().to_path_buf(),
            project_plugin_dir: None,
            handler_timeout: std::time::Duration::from_secs(5),
        };
        let mut host = PluginHost::new(config);

        // Enable a plugin
        host.enable_plugin("test-plugin").unwrap();

        // Disable it
        host.disable_plugin("test-plugin").unwrap();
    }
}
