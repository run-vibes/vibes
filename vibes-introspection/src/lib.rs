//! vibes-introspection - Harness capability discovery

pub mod capabilities;
pub mod error;
pub mod harness;
pub mod paths;

#[cfg(feature = "claude-code")]
pub mod claude_code;

pub use capabilities::*;
pub use error::{IntrospectionError, Result};
pub use harness::{harness_for_command, Harness};
pub use paths::ConfigPaths;

#[cfg(feature = "claude-code")]
pub use claude_code::ClaudeCodeHarness;
