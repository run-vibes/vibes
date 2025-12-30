//! Dispatch commands to plugins
//!
//! This module will be used when plugin CLI integration is complete.

#![allow(dead_code)]

use anyhow::{Result, anyhow};
use vibes_core::PluginHost;
use vibes_plugin_api::CommandOutput;

/// Dispatch a command to a plugin
pub fn dispatch(
    plugin_host: &mut PluginHost,
    path: &[String],
    positional: Vec<String>,
    flags: std::collections::HashMap<String, String>,
) -> Result<()> {
    if path.is_empty() {
        return Err(anyhow!("No command specified"));
    }

    let plugin_name = &path[0];
    let cmd_path: Vec<&str> = path[1..].iter().map(|s| s.as_str()).collect();

    let args = vibes_plugin_api::CommandArgs {
        args: positional,
        flags,
    };

    let output = plugin_host.dispatch_command(plugin_name, &cmd_path, &args)?;

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
