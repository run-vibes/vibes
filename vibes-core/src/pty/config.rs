//! PTY configuration

use std::path::PathBuf;

/// Configuration for PTY sessions
#[derive(Debug, Clone)]
pub struct PtyConfig {
    /// Path to claude binary (defaults to "claude", can be overridden via VIBES_PTY_COMMAND env var)
    pub claude_path: PathBuf,
    /// Arguments to pass to the command (from VIBES_PTY_COMMAND if it contains spaces)
    pub claude_args: Vec<String>,
    /// Initial terminal columns
    pub initial_cols: u16,
    /// Initial terminal rows
    pub initial_rows: u16,
    /// Mock mode - don't spawn actual process (for testing)
    /// Enabled via VIBES_MOCK_PTY=1 env var
    pub mock_mode: bool,
}

impl Default for PtyConfig {
    fn default() -> Self {
        // Check for mock mode (useful for CI testing without real PTY)
        let mock_mode = std::env::var("VIBES_MOCK_PTY")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        // Allow overriding the command via environment variable (useful for testing)
        // Supports "command arg1 arg2" format
        let command_str =
            std::env::var("VIBES_PTY_COMMAND").unwrap_or_else(|_| "claude".to_string());
        let parts: Vec<&str> = command_str.split_whitespace().collect();
        let (claude_path, claude_args) = if parts.is_empty() {
            (PathBuf::from("claude"), Vec::new())
        } else {
            (
                PathBuf::from(parts[0]),
                parts[1..].iter().map(|s| s.to_string()).collect(),
            )
        };

        tracing::debug!(
            claude_path = %claude_path.display(),
            claude_args = ?claude_args,
            mock_mode = mock_mode,
            "PtyConfig initialized"
        );

        Self {
            claude_path,
            claude_args,
            initial_cols: 120,
            initial_rows: 40,
            mock_mode,
        }
    }
}
