//! PTY session management

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
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
}

pub(crate) struct PtySessionInner {
    #[allow(dead_code)]
    pub(crate) master: Box<dyn portable_pty::MasterPty + Send>,
    pub(crate) child: Box<dyn portable_pty::Child + Send + Sync>,
    #[allow(dead_code)]
    pub(crate) reader: Box<dyn std::io::Read + Send>,
    #[allow(dead_code)]
    pub(crate) writer: Box<dyn std::io::Write + Send>,
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
    pub fn spawn(
        id: String,
        name: Option<String>,
        config: &PtyConfig,
    ) -> Result<Self, PtyError> {
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
            reader,
            writer,
        };

        let handle = PtySessionHandle {
            inner: Arc::new(Mutex::new(inner)),
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
        let mut config = PtyConfig::default();
        // Use 'cat' for testing - it will wait for input
        config.claude_path = "cat".into();

        let session = PtySession::spawn("test-id".to_string(), None, &config);
        assert!(session.is_ok());

        let session = session.unwrap();
        assert_eq!(session.id, "test-id");
        assert_eq!(session.state, PtyState::Running);
    }

    #[test]
    fn spawn_with_name() {
        let mut config = PtyConfig::default();
        config.claude_path = "cat".into();

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
        let mut config = PtyConfig::default();
        config.claude_path = "/nonexistent/binary".into();

        let result = PtySession::spawn("test-id".to_string(), None, &config);
        assert!(result.is_err());
    }
}
