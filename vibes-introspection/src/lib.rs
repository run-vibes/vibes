//! vibes-introspection - Harness capability discovery

pub mod capabilities;
pub mod error;
pub mod harness;
pub mod paths;
pub mod watcher;

#[cfg(feature = "claude-code")]
pub mod claude_code;

pub use capabilities::*;
pub use error::{IntrospectionError, Result};
pub use harness::{Harness, harness_for_command};
pub use paths::ConfigPaths;
pub use watcher::CapabilityWatcher;

#[cfg(feature = "claude-code")]
pub use claude_code::ClaudeCodeHarness;
