//! Hello Plugin - A simple example plugin for vibes
//!
//! This plugin demonstrates:
//! - Basic plugin structure with the `export_plugin!` macro
//! - Implementing the `Plugin` trait
//! - Handling lifecycle events (`on_load`, `on_unload`)
//! - Tracking state across turns (`on_turn_complete`)
//!
//! ## Building
//!
//! ```bash
//! cargo build --release
//! ```
//!
//! ## Installing
//!
//! ```bash
//! mkdir -p ~/.config/vibes/plugins/hello
//! cp target/release/libhello_plugin.so ~/.config/vibes/plugins/hello/hello.so
//! vibes plugin enable hello
//! ```

use vibes_plugin_api::{export_plugin, Plugin, PluginContext, PluginError, PluginManifest, Usage};

/// A simple plugin that logs turn completions and tracks token usage.
#[derive(Default)]
pub struct HelloPlugin {
    /// Number of turns completed in this session
    turn_count: u32,
    /// Total input tokens across all turns
    total_input_tokens: u32,
    /// Total output tokens across all turns
    total_output_tokens: u32,
}

impl Plugin for HelloPlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            name: "hello".to_string(),
            version: "0.1.0".to_string(),
            description: "A simple example plugin that tracks token usage".to_string(),
            author: "vibes-team".to_string(),
            ..Default::default()
        }
    }

    fn on_load(&mut self, ctx: &mut PluginContext) -> Result<(), PluginError> {
        ctx.log_info("Hello plugin loaded!");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn on_session_created(
        &mut self,
        session_id: &str,
        name: Option<&str>,
        ctx: &mut PluginContext,
    ) {
        let session_name = name.unwrap_or("unnamed");
        ctx.log_info(&format!(
            "Session created: {} ({})",
            session_name, session_id
        ));
    }

    fn on_turn_start(&mut self, _session_id: &str, ctx: &mut PluginContext) {
        ctx.log_debug("Turn starting...");
    }

    fn on_turn_complete(&mut self, _session_id: &str, usage: &Usage, ctx: &mut PluginContext) {
        self.turn_count += 1;
        self.total_input_tokens += usage.input_tokens;
        self.total_output_tokens += usage.output_tokens;

        ctx.log_info(&format!(
            "Turn {} complete. This turn: {} in, {} out. Total: {} in, {} out",
            self.turn_count,
            usage.input_tokens,
            usage.output_tokens,
            self.total_input_tokens,
            self.total_output_tokens
        ));
    }

    fn on_tool_use_start(
        &mut self,
        _session_id: &str,
        _tool_id: &str,
        name: &str,
        ctx: &mut PluginContext,
    ) {
        ctx.log_debug(&format!("Tool starting: {}", name));
    }

    fn on_error(
        &mut self,
        _session_id: &str,
        message: &str,
        recoverable: bool,
        ctx: &mut PluginContext,
    ) {
        if recoverable {
            ctx.log_warn(&format!("Recoverable error: {}", message));
        } else {
            ctx.log_error(&format!("Fatal error: {}", message));
        }
    }
}

// This macro generates the C ABI entry points for dynamic loading
export_plugin!(HelloPlugin);
