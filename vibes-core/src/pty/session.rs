//! PTY session management

use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{PtyConfig, PtyError};

/// State of a PTY session
#[derive(Debug, Clone, PartialEq)]
pub enum PtyState {
    Running,
    Exited(i32),
}

/// Handle to interact with a PTY session
#[derive(Clone)]
pub struct PtySessionHandle {
    pub(crate) inner: Arc<Mutex<PtySessionInner>>,
    /// Separate mutex for the reader to avoid blocking writes while reading
    reader: Arc<std::sync::Mutex<Box<dyn std::io::Read + Send>>>,
    /// Separate mutex for the writer to avoid blocking reads while writing
    writer: Arc<std::sync::Mutex<Box<dyn std::io::Write + Send>>>,
}

impl PtySessionHandle {
    /// Write data to the PTY
    pub async fn write(&self, data: &[u8]) -> Result<(), PtyError> {
        let data = data.to_vec();
        let writer = Arc::clone(&self.writer);

        tokio::task::spawn_blocking(move || {
            let mut guard = writer
                .lock()
                .map_err(|_| PtyError::IoError(std::io::Error::other("mutex poisoned")))?;
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
    pub async fn read(&self) -> Result<Vec<u8>, PtyError> {
        let reader = Arc::clone(&self.reader);

        tokio::task::spawn_blocking(move || {
            let mut guard = reader
                .lock()
                .map_err(|_| PtyError::IoError(std::io::Error::other("mutex poisoned")))?;
            let mut buf = vec![0u8; 4096];

            use std::io::Read;
            match guard.read(&mut buf) {
                Ok(n) if n > 0 => {
                    buf.truncate(n);
                    Ok(buf)
                }
                Ok(_) => Ok(vec![]),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(vec![]),
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
}

impl PtySession {
    /// Spawn a new PTY session
    pub fn spawn(id: String, name: Option<String>, config: &PtyConfig) -> Result<Self, PtyError> {
        let pty_system = native_pty_system();

        let pair = pty_system
            .openpty(PtySize {
                rows: config.initial_rows,
                cols: config.initial_cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::CreateFailed(e.to_string()))?;

        let cmd = CommandBuilder::new(&config.claude_path);

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
        };

        Ok(Self {
            id,
            name,
            state: PtyState::Running,
            handle,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_creates_running_session() {
        // Use 'cat' for testing - it will wait for input
        let config = PtyConfig {
            claude_path: "cat".into(),
            ..Default::default()
        };

        let session = PtySession::spawn("test-id".to_string(), None, &config);
        assert!(session.is_ok());

        let session = session.unwrap();
        assert_eq!(session.id, "test-id");
        assert_eq!(session.state, PtyState::Running);
    }

    #[test]
    fn spawn_with_name() {
        let config = PtyConfig {
            claude_path: "cat".into(),
            ..Default::default()
        };

        let session = PtySession::spawn(
            "test-id".to_string(),
            Some("my-session".to_string()),
            &config,
        )
        .unwrap();

        assert_eq!(session.name, Some("my-session".to_string()));
    }

    #[test]
    fn spawn_invalid_command_fails() {
        let config = PtyConfig {
            claude_path: "/nonexistent/binary".into(),
            ..Default::default()
        };

        let result = PtySession::spawn("test-id".to_string(), None, &config);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn write_and_read_data() {
        // Use 'cat' - it echoes input back
        let config = PtyConfig {
            claude_path: "cat".into(),
            ..Default::default()
        };

        let session = PtySession::spawn("test-id".to_string(), None, &config).unwrap();

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
        let config = PtyConfig {
            claude_path: "cat".into(),
            ..Default::default()
        };

        let session = PtySession::spawn("test-id".to_string(), None, &config).unwrap();

        // Resize should not error
        let result = session.handle.resize(80, 24).await;
        assert!(result.is_ok());
    }
}
