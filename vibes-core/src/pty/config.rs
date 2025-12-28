//! PTY configuration

use std::path::PathBuf;

/// Configuration for PTY sessions
#[derive(Debug, Clone)]
pub struct PtyConfig {
    /// Path to claude binary (defaults to "claude")
    pub claude_path: PathBuf,
    /// Initial terminal columns
    pub initial_cols: u16,
    /// Initial terminal rows
    pub initial_rows: u16,
}

impl Default for PtyConfig {
    fn default() -> Self {
        Self {
            claude_path: PathBuf::from("claude"),
            initial_cols: 120,
            initial_rows: 40,
        }
    }
}
