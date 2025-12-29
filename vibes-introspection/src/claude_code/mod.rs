//! Claude Code harness implementation
//!
//! This module provides the [`ClaudeCodeHarness`] implementation of the [`Harness`](crate::Harness)
//! trait for introspecting Claude Code capabilities.
//!
//! # Features
//!
//! The harness can detect:
//! - Installed hooks (pre_tool_use, post_tool_use, stop, notification)
//! - Configuration files (settings.json, .clauderc)
//! - Injection targets (CLAUDE.md files at system, user, and project scopes)
//!
//! # Example
//!
//! ```no_run
//! use vibes_introspection::{ClaudeCodeHarness, Harness};
//! use std::path::Path;
//!
//! #[tokio::main]
//! async fn main() {
//!     let harness = ClaudeCodeHarness;
//!     let project_root = Path::new("/path/to/project");
//!     let capabilities = harness.introspect(Some(project_root)).await.unwrap();
//!
//!     println!("Harness type: {}", capabilities.harness_type);
//!     if let Some(version) = &capabilities.version {
//!         println!("Version: {}", version);
//!     }
//! }
//! ```

mod detection;
mod harness;

pub use harness::ClaudeCodeHarness;
