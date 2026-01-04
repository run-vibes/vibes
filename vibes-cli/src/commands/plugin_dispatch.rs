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

    // Show help if --help flag or plugin name only (no subcommand)
    if wants_help(&args) || args.len() == 1 {
        // Collect commands for this plugin from the registry
        let commands: Vec<_> = host
            .command_registry()
            .all_commands()
            .filter(|(_, cmd)| cmd.plugin_name == *plugin_name)
            .map(|(_, cmd)| cmd.spec.clone())
            .collect();
        println!("{}", format_plugin_help(plugin_name, &commands));
        return Ok(());
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

/// Check if args contain a help flag
fn wants_help(args: &[String]) -> bool {
    args.iter().any(|a| a == "--help" || a == "-h")
}

/// Format help text for a plugin's commands
fn format_plugin_help(plugin_name: &str, commands: &[vibes_plugin_api::CommandSpec]) -> String {
    let mut help = format!("Usage: vibes {} <COMMAND>\n\nCommands:\n", plugin_name);

    // Sort commands by path for consistent output
    let mut sorted_commands: Vec<_> = commands.iter().collect();
    sorted_commands.sort_by(|a, b| a.path.cmp(&b.path));

    // Calculate max width for alignment
    let max_width = sorted_commands
        .iter()
        .map(|cmd| {
            let path = cmd.path.join(" ");
            let args_str: String = cmd
                .args
                .iter()
                .map(|a| {
                    if a.required {
                        format!(" <{}>", a.name)
                    } else {
                        format!(" [{}]", a.name)
                    }
                })
                .collect();
            path.len() + args_str.len()
        })
        .max()
        .unwrap_or(20);

    for cmd in sorted_commands {
        let path = cmd.path.join(" ");
        let args_str: String = cmd
            .args
            .iter()
            .map(|a| {
                if a.required {
                    format!(" <{}>", a.name)
                } else {
                    format!(" [{}]", a.name)
                }
            })
            .collect();

        let cmd_with_args = format!("{}{}", path, args_str);
        help.push_str(&format!(
            "  {:width$}  {}\n",
            cmd_with_args,
            cmd.description,
            width = max_width
        ));
    }

    help.push_str(&format!(
        "\nRun 'vibes {} <command> --help' for more info on a command.\n",
        plugin_name
    ));

    help
}

/// Get summaries of all loaded plugins for top-level help
///
/// Returns a list of (name, description) tuples
pub fn get_plugin_summaries() -> Vec<(String, String)> {
    let config = PluginHostConfig::default();
    let mut host = PluginHost::new(config);

    if host.load_all().is_err() {
        return vec![];
    }

    host.list_plugins(false)
        .into_iter()
        .map(|info| (info.name, info.manifest.description))
        .collect()
}

/// Format top-level help with plugin commands appended
pub fn format_top_level_help(base_help: &str, plugins: &[(String, String)]) -> String {
    if plugins.is_empty() {
        return base_help.to_string();
    }

    let mut help = base_help.to_string();

    // Find where to insert plugin commands (before Options or at end)
    let insert_point = help
        .find("\nOptions:")
        .or_else(|| help.find("\n\n"))
        .unwrap_or(help.len());

    // Build plugin section
    let mut plugin_section = String::from("\nPlugins:\n");
    for (name, desc) in plugins {
        plugin_section.push_str(&format!("  {:12} {}\n", name, desc));
    }

    help.insert_str(insert_point, &plugin_section);
    help
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
    #[ignore = "Loads external plugins; dynamically loaded plugins cause SIGABRT in tests"]
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

    #[test]
    fn test_wants_help_detects_help_flag() {
        // --help anywhere in args should be detected
        assert!(wants_help(&["groove".into(), "--help".into()]));
        assert!(wants_help(&[
            "groove".into(),
            "trust".into(),
            "--help".into()
        ]));

        // -h should also work
        assert!(wants_help(&["groove".into(), "-h".into()]));

        // No help flag should return false
        assert!(!wants_help(&[
            "groove".into(),
            "trust".into(),
            "levels".into()
        ]));
        assert!(!wants_help(&["groove".into()]));
    }

    #[test]
    #[ignore = "Loads external plugins; dynamically loaded plugins cause SIGABRT in tests"]
    fn test_run_plugin_name_only_shows_help() {
        // When only the plugin name is provided (no subcommand), show help
        let result = run(vec!["groove".into()]);

        // Should succeed (show help) or fail with "Unknown plugin" if not installed
        // Should NOT fail with "Unknown command" trying to dispatch
        match result {
            Ok(()) => {} // Help was shown
            Err(e) => {
                let msg = e.to_string();
                assert!(
                    msg.contains("Unknown plugin"),
                    "Should show help or unknown plugin error, got: {}",
                    msg
                );
            }
        }
    }

    #[test]
    #[ignore = "Loads external plugins; dynamically loaded plugins cause SIGABRT in tests"]
    fn test_run_with_help_flag_returns_help_result() {
        // When --help is passed, run() should return a HelpRequested result
        // rather than attempting to dispatch a command
        let result = run(vec!["groove".into(), "--help".into()]);

        // Should not error - help is a valid request
        // (Though it may error if groove isn't installed, we check the error message)
        match result {
            Ok(()) => {} // Help was shown
            Err(e) => {
                let msg = e.to_string();
                // If groove is installed, should not get "Unknown command" error
                // If groove is NOT installed, we expect "Unknown plugin" error
                assert!(
                    !msg.contains("Unknown command: groove --help"),
                    "Should not treat --help as part of command path, got: {}",
                    msg
                );
            }
        }
    }

    #[test]
    fn test_format_plugin_help_shows_commands() {
        use vibes_plugin_api::CommandSpec;

        let commands = vec![
            CommandSpec {
                path: vec!["trust".into(), "levels".into()],
                description: "Show trust level hierarchy".into(),
                args: vec![],
            },
            CommandSpec {
                path: vec!["trust".into(), "role".into()],
                description: "Show role permissions".into(),
                args: vec![vibes_plugin_api::ArgSpec {
                    name: "role".into(),
                    description: "Role name".into(),
                    required: true,
                }],
            },
        ];

        let help = format_plugin_help("groove", &commands);

        // Should contain plugin name
        assert!(help.contains("groove"), "Should contain plugin name");
        // Should contain command paths
        assert!(
            help.contains("trust levels"),
            "Should contain 'trust levels'"
        );
        assert!(help.contains("trust role"), "Should contain 'trust role'");
        // Should contain descriptions
        assert!(help.contains("Show trust level hierarchy"));
        // Should show required args
        assert!(help.contains("<role>") || help.contains("role"));
    }

    #[test]
    #[ignore = "Loads external plugins; dynamically loaded plugins cause SIGABRT in tests"]
    fn test_get_plugin_summaries_returns_name_and_description() {
        // This function should return plugin name and description for top-level help
        let summaries = get_plugin_summaries();

        // Each summary should be (name, description)
        for (name, desc) in &summaries {
            assert!(!name.is_empty(), "Plugin name should not be empty");
            assert!(!desc.is_empty(), "Plugin description should not be empty");
        }

        // If groove is installed, it should appear
        // (This test may pass with empty vec if no plugins installed)
    }

    #[test]
    fn test_format_top_level_help_includes_plugins() {
        let base_help = "Usage: vibes <COMMAND>\n\nCommands:\n  auth    Auth stuff\n";
        let plugins = vec![
            (
                "groove".to_string(),
                "Continual learning system".to_string(),
            ),
            ("other".to_string(), "Another plugin".to_string()),
        ];

        let help = format_top_level_help(base_help, &plugins);

        // Should contain original commands
        assert!(help.contains("auth"), "Should contain original commands");
        // Should contain plugin section
        assert!(
            help.contains("groove") && help.contains("Continual learning"),
            "Should contain groove plugin"
        );
        assert!(help.contains("other"), "Should contain other plugin");
    }
}
