//! Plugin system for vibes
//!
//! This module provides the infrastructure for loading and managing plugins:
//!
//! - [`PluginHost`]: The main plugin manager that loads, unloads, and dispatches events
//! - [`PluginRegistry`]: Tracks which plugins are enabled/disabled
//! - [`PluginHostError`]: Error types for plugin operations
//!
//! # Plugin Discovery
//!
//! Plugins are discovered from two directories:
//! 1. Project plugins: `.vibes/plugins/` (takes precedence)
//! 2. User plugins: `~/.config/vibes/plugins/`
//!
//! # Plugin Structure
//!
//! Each plugin directory should contain:
//! - `<name>.<version>.so` (or `.dylib`/`.dll`) - the versioned binary
//! - `<name>.so` - symlink to the versioned binary
//! - `config.toml` (optional) - plugin configuration
//!
//! # Example
//!
//! ```ignore
//! use vibes_core::plugins::{PluginHost, PluginHostConfig};
//!
//! let config = PluginHostConfig::default();
//! let mut host = PluginHost::new(config);
//!
//! // Load all enabled plugins
//! host.load_all()?;
//!
//! // Dispatch events to plugins
//! host.dispatch_event(&event);
//!
//! // Manage plugins
//! host.enable_plugin("analytics")?;
//! host.disable_plugin("history")?;
//! ```

mod error;
mod host;
mod registry;

pub use error::PluginHostError;
pub use host::{PluginHost, PluginHostConfig, PluginInfo, PluginState};
pub use registry::PluginRegistry;
