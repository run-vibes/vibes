//! vibes-plugin-api - Plugin API for the vibes Claude Code proxy
//!
//! This crate provides the traits and types needed to write plugins for vibes.
//! Plugins are native Rust dynamic libraries that can react to events, register
//! CLI commands, and integrate with the vibes server.
//!
//! # Example
//!
//! ```ignore
//! use vibes_plugin_api::{Plugin, PluginContext, PluginError, PluginManifest, export_plugin};
//!
//! #[derive(Default)]
//! pub struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn manifest(&self) -> PluginManifest {
//!         PluginManifest {
//!             name: "my-plugin".to_string(),
//!             version: "0.1.0".to_string(),
//!             description: "My custom plugin".to_string(),
//!             ..Default::default()
//!         }
//!     }
//!
//!     fn on_load(&mut self, ctx: &mut PluginContext) -> Result<(), PluginError> {
//!         ctx.log_info("Plugin loaded!");
//!         Ok(())
//!     }
//!
//!     fn on_unload(&mut self) -> Result<(), PluginError> {
//!         Ok(())
//!     }
//! }
//!
//! export_plugin!(MyPlugin);
//! ```

pub mod context;
pub mod error;
pub mod types;

pub use context::{Capability, Harness, PluginConfig, PluginContext};
pub use error::PluginError;
pub use types::*;

/// Current plugin API version. Plugins must match this exactly.
/// This will be checked when loading plugins to ensure compatibility.
pub const API_VERSION: u32 = 1;

/// The core plugin trait - implement this to create a vibes plugin.
///
/// All event handlers have default no-op implementations, so plugins only
/// need to override the handlers they care about.
pub trait Plugin: Send + Sync {
    /// Return plugin metadata
    fn manifest(&self) -> PluginManifest;

    /// Called when plugin is loaded. Use this to initialize state and register commands.
    fn on_load(&mut self, ctx: &mut PluginContext) -> Result<(), PluginError>;

    /// Called when plugin is unloaded. Use this to clean up resources.
    fn on_unload(&mut self) -> Result<(), PluginError>;

    // ─── Event Handlers (default no-ops) ─────────────────────────────

    /// Called when a new session is created
    fn on_session_created(
        &mut self,
        _session_id: &str,
        _name: Option<&str>,
        _ctx: &mut PluginContext,
    ) {
    }

    /// Called when session state changes
    fn on_session_state_changed(
        &mut self,
        _session_id: &str,
        _state: &SessionState,
        _ctx: &mut PluginContext,
    ) {
    }

    /// Called when a turn starts (user message sent)
    fn on_turn_start(&mut self, _session_id: &str, _ctx: &mut PluginContext) {}

    /// Called when a turn completes (full response received)
    fn on_turn_complete(&mut self, _session_id: &str, _usage: &Usage, _ctx: &mut PluginContext) {}

    /// Called when text is streamed from Claude
    fn on_text_delta(&mut self, _session_id: &str, _text: &str, _ctx: &mut PluginContext) {}

    /// Called when thinking text is streamed (extended thinking)
    fn on_thinking_delta(&mut self, _session_id: &str, _text: &str, _ctx: &mut PluginContext) {}

    /// Called when a tool use starts
    fn on_tool_use_start(
        &mut self,
        _session_id: &str,
        _tool_id: &str,
        _name: &str,
        _ctx: &mut PluginContext,
    ) {
    }

    /// Called when a tool returns a result
    fn on_tool_result(
        &mut self,
        _session_id: &str,
        _tool_id: &str,
        _output: &str,
        _is_error: bool,
        _ctx: &mut PluginContext,
    ) {
    }

    /// Called when an error occurs
    fn on_error(
        &mut self,
        _session_id: &str,
        _message: &str,
        _recoverable: bool,
        _ctx: &mut PluginContext,
    ) {
    }
}

/// Export a plugin type for dynamic loading.
///
/// This macro generates the C ABI entry points that vibes uses to load
/// and unload plugins dynamically.
///
/// # Usage
///
/// ```ignore
/// vibes_plugin_api::export_plugin!(MyPlugin);
/// ```
///
/// # Generated Functions
///
/// - `_vibes_plugin_create()`: Creates a new plugin instance
/// - `_vibes_plugin_api_version()`: Returns the API version
/// - `_vibes_plugin_destroy()`: Destroys a plugin instance
#[macro_export]
macro_rules! export_plugin {
    ($plugin_type:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn _vibes_plugin_create() -> *mut dyn $crate::Plugin {
            let plugin: Box<dyn $crate::Plugin> = Box::new(<$plugin_type>::default());
            Box::into_raw(plugin)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn _vibes_plugin_api_version() -> u32 {
            $crate::API_VERSION
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn _vibes_plugin_destroy(ptr: *mut dyn $crate::Plugin) {
            if !ptr.is_null() {
                unsafe {
                    drop(Box::from_raw(ptr));
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version_is_set() {
        assert_eq!(API_VERSION, 1);
    }

    #[test]
    fn test_plugin_trait_is_object_safe() {
        // This compiles only if Plugin is object-safe
        fn _takes_boxed_plugin(_: Box<dyn Plugin>) {}
    }

    #[test]
    fn test_manifest_default_has_correct_api_version() {
        let manifest = PluginManifest::default();
        assert_eq!(manifest.api_version, API_VERSION);
    }
}
