//! PTY configuration

use std::path::PathBuf;

/// Configuration for PTY sessions
#[derive(Debug, Clone)]
pub struct PtyConfig {
    /// Path to claude binary (defaults to "claude", can be overridden via VIBES_PTY_COMMAND env var)
    pub claude_path: PathBuf,
    /// Initial terminal columns
    pub initial_cols: u16,
    /// Initial terminal rows
    pub initial_rows: u16,
}

impl Default for PtyConfig {
    fn default() -> Self {
        // Allow overriding the command via environment variable (useful for testing)
        let claude_path = std::env::var("VIBES_PTY_COMMAND")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("claude"));

        tracing::debug!(
            claude_path = %claude_path.display(),
            "PtyConfig initialized"
        );

        Self {
            claude_path,
            initial_cols: 120,
            initial_rows: 40,
        }
    }
}
