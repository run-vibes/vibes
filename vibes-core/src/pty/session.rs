//! PTY session management

use chrono::{DateTime, Utc};
use portable_pty::PtySize;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::PtyError;
use super::scrollback::ScrollbackBuffer;

/// State of a PTY session
#[derive(Debug, Clone, PartialEq)]
pub enum PtyState {
    Running,
    Exited(i32),
}

/// Handle to interact with a PTY session
///
/// # Threading Model
///
/// This handle uses `std::sync::Mutex` (not `tokio::sync::Mutex`) for the reader,
/// writer, and scrollback buffer. This is safe because:
///
/// 1. Each mutex guards a separate, independent resource (reader vs writer vs scrollback)
/// 2. We never hold multiple locks simultaneously
/// 3. Operations use `spawn_blocking` to move blocking I/O off the async runtime
/// 4. The `tokio::sync::Mutex` on `inner` is only used for resize operations which
///    don't overlap with read/write
///
/// Using `std::sync::Mutex` with `spawn_blocking` is the recommended pattern for
/// wrapping synchronous I/O in async contexts, as per tokio documentation.
#[derive(Clone)]
pub struct PtySessionHandle {
    pub(crate) inner: Arc<Mutex<PtySessionInner>>,
    /// Separate mutex for the reader to avoid blocking writes while reading
    pub(crate) reader: Arc<std::sync::Mutex<Box<dyn std::io::Read + Send>>>,
    /// Separate mutex for the writer to avoid blocking reads while writing
    pub(crate) writer: Arc<std::sync::Mutex<Box<dyn std::io::Write + Send>>>,
    /// Scrollback buffer for replay on reconnect
    pub(crate) scrollback: Arc<std::sync::Mutex<ScrollbackBuffer>>,
}

impl PtySessionHandle {
    /// Write data to the PTY
    pub async fn write(&self, data: &[u8]) -> Result<(), PtyError> {
        let data = data.to_vec();
        let writer = Arc::clone(&self.writer);

        tokio::task::spawn_blocking(move || {
            let mut guard = writer
                .lock()
                .map_err(|_| PtyError::IoError(std::io::Error::other("writer mutex poisoned")))?;
            use std::io::Write;
            guard.write_all(&data)?;
            guard.flush()?;
            Ok(())
        })
        .await
        .map_err(|e| PtyError::IoError(std::io::Error::other(e)))?
    }

    /// Read available data from the PTY
    ///
    /// This uses spawn_blocking internally since the underlying reader
    /// may block. Use this in async contexts where blocking would be problematic.
    ///
    /// Returns `PtyError::Eof` when the PTY process has exited and there's no more data.
    pub async fn read(&self) -> Result<Vec<u8>, PtyError> {
        let reader = Arc::clone(&self.reader);

        tokio::task::spawn_blocking(move || {
            let mut guard = reader
                .lock()
                .map_err(|_| PtyError::IoError(std::io::Error::other("reader mutex poisoned")))?;
            let mut buf = vec![0u8; 4096];

            use std::io::Read;
            match guard.read(&mut buf) {
                Ok(n) if n > 0 => {
                    buf.truncate(n);
                    Ok(buf)
                }
                Ok(_) => {
                    // 0 bytes read in blocking mode = EOF (process exited)
                    Err(PtyError::Eof)
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // Non-blocking mode: no data available yet
                    Ok(vec![])
                }
                Err(e) => Err(PtyError::IoError(e)),
            }
        })
        .await
        .map_err(|e| PtyError::IoError(std::io::Error::other(e)))?
    }

    /// Resize the PTY
    pub async fn resize(&self, cols: u16, rows: u16) -> Result<(), PtyError> {
        let inner = self.inner.lock().await;
        inner
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::IoError(std::io::Error::other(e)))
    }

    /// Append data to the scrollback buffer
    pub fn append_scrollback(&self, data: &[u8]) {
        if let Ok(mut scrollback) = self.scrollback.lock() {
            scrollback.append(data);
        }
    }

    /// Get all scrollback data for replay
    pub fn get_scrollback(&self) -> Vec<u8> {
        self.scrollback
            .lock()
            .map(|s| s.get_all())
            .unwrap_or_default()
    }
}

pub(crate) struct PtySessionInner {
    pub(crate) master: Box<dyn portable_pty::MasterPty + Send>,
    pub(crate) child: Box<dyn portable_pty::Child + Send + Sync>,
    // Note: reader and writer are now stored separately on PtySessionHandle
    // to allow independent locking for concurrent read/write operations
}

/// A PTY session wrapping Claude
pub struct PtySession {
    pub id: String,
    pub name: Option<String>,
    pub state: PtyState,
    pub handle: PtySessionHandle,
    /// When this session was created
    pub created_at: DateTime<Utc>,
}

// Note: PtySession creation is now handled by PtyBackend implementations
// See backend.rs for RealPtyBackend and MockPtyBackend

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pty::backend::RealPtyBackend;
    use crate::pty::{PtyBackend, PtyConfig};

    fn test_config() -> PtyConfig {
        PtyConfig {
            claude_path: "cat".into(),
            ..Default::default()
        }
    }

    #[test]
    fn backend_creates_running_session() {
        let backend = RealPtyBackend::new(test_config());
        let session = backend.create_session("test-id".to_string(), None, None);
        assert!(session.is_ok());

        let session = session.unwrap();
        assert_eq!(session.id, "test-id");
        assert_eq!(session.state, PtyState::Running);
    }

    #[test]
    fn backend_creates_session_with_name() {
        let backend = RealPtyBackend::new(test_config());
        let session = backend
            .create_session("test-id".to_string(), Some("my-session".to_string()), None)
            .unwrap();

        assert_eq!(session.name, Some("my-session".to_string()));
    }

    #[test]
    fn backend_invalid_command_fails() {
        let config = PtyConfig {
            claude_path: "/nonexistent/binary".into(),
            ..Default::default()
        };
        let backend = RealPtyBackend::new(config);
        let result = backend.create_session("test-id".to_string(), None, None);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn write_and_read_data() {
        let backend = RealPtyBackend::new(test_config());
        let session = backend
            .create_session("test-id".to_string(), None, None)
            .unwrap();

        // Write some data
        session.handle.write(b"hello\n").await.unwrap();

        // Give cat time to echo
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Read it back
        let data = session.handle.read().await.unwrap();
        assert!(!data.is_empty());
    }

    #[tokio::test]
    async fn resize_pty() {
        let backend = RealPtyBackend::new(test_config());
        let session = backend
            .create_session("test-id".to_string(), None, None)
            .unwrap();

        // Resize should not error
        let result = session.handle.resize(80, 24).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn handle_provides_scrollback_access() {
        let backend = RealPtyBackend::new(test_config());
        let session = backend
            .create_session("test-id".to_string(), None, None)
            .unwrap();

        // Write data and read it back (cat echoes)
        session.handle.write(b"test\n").await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Read the output
        let data = session.handle.read().await.unwrap();
        assert!(!data.is_empty());

        // Append to scrollback
        session.handle.append_scrollback(&data);

        // Verify scrollback contains data
        let scrollback = session.handle.get_scrollback();
        assert!(!scrollback.is_empty());
    }
}
