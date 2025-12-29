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
    fn create_session(
        &self,
        id: String,
        name: Option<String>,
        cwd: Option<String>,
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
    ) -> Result<PtySession, PtyError> {
        tracing::info!(
            id = %id,
            name = ?name,
            cwd = ?cwd,
            command = %self.config.claude_path.display(),
            "Spawning real PTY session"
        );

        let pty_system = native_pty_system();

        let pair = pty_system
            .openpty(PtySize {
                rows: self.config.initial_rows,
                cols: self.config.initial_cols,
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
    ) -> Result<PtySession, PtyError> {
        tracing::info!(
            id = %id,
            name = ?name,
            cwd = ?cwd,
            "Creating mock PTY session (no real process)"
        );

        // Create a minimal PTY just to get a valid child process handle
        // We use 'true' which exits immediately
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
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
        let session = backend.create_session("test-id".to_string(), Some("test".to_string()), None);
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
        let session = backend.create_session("test-id".to_string(), Some("test".to_string()), cwd);
        assert!(session.is_ok());
        let session = session.unwrap();
        assert_eq!(session.id, "test-id");
    }
}
