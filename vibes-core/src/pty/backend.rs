//! PTY backend trait and implementations
//!
//! This module provides a trait-based abstraction for PTY backends,
//! allowing for different implementations (real PTY vs mock for testing).

use chrono::Utc;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::scrollback::ScrollbackBuffer;
use super::session::{PtySession, PtySessionHandle, PtySessionInner, PtyState};
use super::{PtyConfig, PtyError};

/// Trait for PTY backend implementations
pub trait PtyBackend: Send + Sync {
    /// Create a new PTY session
    ///
    /// If cols/rows are provided, they override the config defaults.
    fn create_session(
        &self,
        id: String,
        name: Option<String>,
        cwd: Option<String>,
        cols: Option<u16>,
        rows: Option<u16>,
    ) -> Result<PtySession, PtyError>;
}

/// Real PTY backend using portable_pty
pub struct RealPtyBackend {
    config: PtyConfig,
}

impl RealPtyBackend {
    /// Create a new real PTY backend
    pub fn new(config: PtyConfig) -> Self {
        Self { config }
    }
}

impl PtyBackend for RealPtyBackend {
    fn create_session(
        &self,
        id: String,
        name: Option<String>,
        cwd: Option<String>,
        cols: Option<u16>,
        rows: Option<u16>,
    ) -> Result<PtySession, PtyError> {
        // Use provided dimensions or fall back to config defaults
        let actual_cols = cols.unwrap_or(self.config.initial_cols);
        let actual_rows = rows.unwrap_or(self.config.initial_rows);

        tracing::info!(
            id = %id,
            name = ?name,
            cwd = ?cwd,
            cols = actual_cols,
            rows = actual_rows,
            command = %self.config.claude_path.display(),
            "Spawning real PTY session"
        );

        let pty_system = native_pty_system();

        let pair = pty_system
            .openpty(PtySize {
                rows: actual_rows,
                cols: actual_cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::CreateFailed(e.to_string()))?;

        let mut cmd = CommandBuilder::new(&self.config.claude_path);
        for arg in &self.config.claude_args {
            cmd.arg(arg);
        }

        // Set working directory if provided
        if let Some(dir) = cwd {
            cmd.cwd(dir);
        }

        // Set VIBES_BIN so hooks can find the vibes binary during development
        // This is especially important when vibes isn't on PATH
        if let Ok(current_exe) = std::env::current_exe() {
            cmd.env("VIBES_BIN", current_exe);
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| PtyError::IoError(std::io::Error::other(e)))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| PtyError::IoError(std::io::Error::other(e)))?;

        let inner = PtySessionInner {
            master: pair.master,
            child,
        };

        let handle = PtySessionHandle {
            inner: Arc::new(Mutex::new(inner)),
            reader: Arc::new(std::sync::Mutex::new(reader)),
            writer: Arc::new(std::sync::Mutex::new(writer)),
            scrollback: Arc::new(std::sync::Mutex::new(ScrollbackBuffer::default())),
        };

        Ok(PtySession {
            id,
            name,
            state: PtyState::Running,
            handle,
            created_at: Utc::now(),
        })
    }
}

/// Mock PTY backend for testing - uses no real PTY
pub struct MockPtyBackend;

impl MockPtyBackend {
    /// Create a new mock PTY backend
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockPtyBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// A mock reader that returns WouldBlock (non-blocking empty reads)
struct MockReader;

impl Read for MockReader {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        // Return WouldBlock to simulate non-blocking empty read
        Err(std::io::Error::new(
            std::io::ErrorKind::WouldBlock,
            "mock reader has no data",
        ))
    }
}

/// A mock writer that discards all data
struct MockWriter;

impl Write for MockWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Pretend we wrote everything
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl PtyBackend for MockPtyBackend {
    fn create_session(
        &self,
        id: String,
        name: Option<String>,
        cwd: Option<String>,
        cols: Option<u16>,
        rows: Option<u16>,
    ) -> Result<PtySession, PtyError> {
        // Use provided dimensions or defaults for mock
        let actual_cols = cols.unwrap_or(80);
        let actual_rows = rows.unwrap_or(24);

        tracing::info!(
            id = %id,
            name = ?name,
            cwd = ?cwd,
            cols = actual_cols,
            rows = actual_rows,
            "Creating mock PTY session (no real process)"
        );

        // Create a minimal PTY just to get a valid child process handle
        // We use 'true' which exits immediately
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: actual_rows,
                cols: actual_cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::CreateFailed(e.to_string()))?;

        let cmd = CommandBuilder::new("true");
        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        let inner = PtySessionInner {
            master: pair.master,
            child,
        };

        // Use mock reader/writer instead of real PTY I/O
        let handle = PtySessionHandle {
            inner: Arc::new(Mutex::new(inner)),
            reader: Arc::new(std::sync::Mutex::new(
                Box::new(MockReader) as Box<dyn Read + Send>
            )),
            writer: Arc::new(std::sync::Mutex::new(
                Box::new(MockWriter) as Box<dyn Write + Send>
            )),
            scrollback: Arc::new(std::sync::Mutex::new(ScrollbackBuffer::default())),
        };

        Ok(PtySession {
            id,
            name,
            state: PtyState::Running,
            handle,
            created_at: Utc::now(),
        })
    }
}

/// Create the appropriate backend based on configuration
pub fn create_backend(config: PtyConfig) -> Box<dyn PtyBackend> {
    if config.mock_mode {
        tracing::info!("Using mock PTY backend");
        Box::new(MockPtyBackend::new())
    } else {
        tracing::info!(
            command = %config.claude_path.display(),
            "Using real PTY backend"
        );
        Box::new(RealPtyBackend::new(config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_backend_creates_session() {
        let backend = MockPtyBackend::new();
        let session = backend.create_session(
            "test-id".to_string(),
            Some("test".to_string()),
            None,
            None,
            None,
        );
        assert!(session.is_ok());
        let session = session.unwrap();
        assert_eq!(session.id, "test-id");
        assert_eq!(session.name, Some("test".to_string()));
    }

    #[test]
    fn create_backend_respects_mock_mode() {
        let config = PtyConfig {
            mock_mode: true,
            ..Default::default()
        };
        let _backend = create_backend(config);
        // Just verify it doesn't panic
    }

    #[test]
    fn mock_backend_creates_session_with_cwd() {
        let backend = MockPtyBackend::new();
        let cwd = Some("/tmp/test-dir".to_string());
        let session = backend.create_session(
            "test-id".to_string(),
            Some("test".to_string()),
            cwd,
            None,
            None,
        );
        assert!(session.is_ok());
        let session = session.unwrap();
        assert_eq!(session.id, "test-id");
    }

    /// Test that VIBES_BIN is set in the child process environment.
    ///
    /// This is critical for hooks to find the vibes binary during development
    /// when vibes isn't on PATH.
    #[tokio::test]
    async fn real_backend_sets_vibes_bin_env() {
        // Use printenv VIBES_BIN as the command - it will print the value if set
        let config = PtyConfig {
            claude_path: "printenv".into(),
            claude_args: vec!["VIBES_BIN".to_string()],
            ..Default::default()
        };
        let backend = RealPtyBackend::new(config);
        let session = backend
            .create_session("test-env".to_string(), None, None, None, None)
            .expect("Failed to create session");

        // Give the process time to run and produce output
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // Read the output - should contain the path to current executable
        let output = session.handle.read().await.unwrap_or_default();
        let output_str = String::from_utf8_lossy(&output);

        // The output should contain a path (the current executable path)
        // It may have trailing newline
        let trimmed = output_str.trim();
        assert!(
            !trimmed.is_empty(),
            "VIBES_BIN should be set in child environment"
        );
        assert!(
            trimmed.contains('/') || trimmed.contains('\\'),
            "VIBES_BIN should be a path, got: {}",
            trimmed
        );
    }
}
