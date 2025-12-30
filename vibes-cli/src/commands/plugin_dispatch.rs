//! Dispatch commands to plugins
//!
//! Handles external subcommands like `vibes groove trust levels`
//! by loading plugins and dispatching commands to them.

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use vibes_core::{PluginHost, PluginHostConfig};
use vibes_plugin_api::CommandOutput;

/// Run a plugin command from external subcommand args
///
/// Args format: `["groove", "trust", "levels"]` for `vibes groove trust levels`
pub fn run(args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("No plugin command specified"));
    }

    // Initialize plugin host
    let config = PluginHostConfig::default();
    let mut host = PluginHost::new(config);

    // Load all enabled plugins
    if let Err(e) = host.load_all() {
        return Err(anyhow!("Failed to load plugins: {}", e));
    }

    // Check if the first argument matches a loaded plugin
    let plugin_name = &args[0];
    if host.get_plugin_info(plugin_name).is_none() {
        // Check if it's a known built-in command that's missing
        let known_builtins = [
            "auth", "claude", "config", "plugin", "serve", "sessions", "tunnel",
        ];
        if known_builtins.contains(&plugin_name.as_str()) {
            return Err(anyhow!("Unknown command: {}", plugin_name));
        }
        return Err(anyhow!(
            "Unknown plugin '{}'. Run 'vibes plugin list' to see installed plugins.",
            plugin_name
        ));
    }

    // For now, all remaining args after the command path are positional
    // A more sophisticated parser would handle --flags
    let positional = Vec::new();
    let flags = HashMap::new();

    dispatch(&mut host, &args, positional, flags)
}

/// Dispatch a command to a plugin
///
/// Uses the command registry to find the longest matching command path,
/// then passes remaining path elements as positional arguments.
pub fn dispatch(
    plugin_host: &mut PluginHost,
    path: &[String],
    mut positional: Vec<String>,
    flags: HashMap<String, String>,
) -> Result<()> {
    if path.is_empty() {
        return Err(anyhow!("No command specified"));
    }

    // Use find_longest_match to properly separate command path from args
    let (cmd, match_len) = plugin_host
        .command_registry()
        .find_longest_match(path)
        .ok_or_else(|| {
            anyhow!(
                "Unknown command: {}. Run 'vibes plugin list' to see installed plugins.",
                path.join(" ")
            )
        })?;

    let plugin_name = cmd.plugin_name.clone();

    // Everything after the matched command path becomes positional args
    let extra_args: Vec<String> = path[match_len..].to_vec();
    positional.splice(0..0, extra_args);

    // Command path is everything after the plugin name, up to match_len
    let cmd_path: Vec<&str> = path[1..match_len].iter().map(|s| s.as_str()).collect();

    let args = vibes_plugin_api::CommandArgs {
        args: positional,
        flags,
    };

    let output = plugin_host.dispatch_command(&plugin_name, &cmd_path, &args)?;

    render_output(output);

    Ok(())
}

fn render_output(output: CommandOutput) {
    match output {
        CommandOutput::Text(text) => println!("{}", text),
        CommandOutput::Table { headers, rows } => {
            // Simple table rendering
            println!("{}", headers.join("\t"));
            for row in rows {
                println!("{}", row.join("\t"));
            }
        }
        CommandOutput::Success => {}
        CommandOutput::Exit(code) => std::process::exit(code),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_empty_args_returns_error() {
        let result = run(vec![]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No plugin command")
        );
    }

    #[test]
    fn test_run_unknown_plugin_returns_error() {
        // With a fresh plugin host (no plugins loaded), any plugin name should fail
        let result = run(vec!["nonexistent".to_string()]);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Unknown plugin") || err.contains("Unknown command"),
            "Expected unknown plugin error, got: {}",
            err
        );
    }

    #[test]
    fn test_dispatch_empty_path_returns_error() {
        let config = PluginHostConfig::default();
        let mut host = PluginHost::new(config);

        let result = dispatch(&mut host, &[], vec![], HashMap::new());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No command"));
    }
}
