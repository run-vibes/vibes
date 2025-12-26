//! PrintMode backend using `claude -p`
//!
//! PrintModeBackend spawns Claude Code in print mode (`-p`) and communicates
//! via stream-json on stdout. This is the primary production backend.

use std::io::BufRead;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use tokio::sync::broadcast;
use uuid::Uuid;

use super::traits::{BackendState, ClaudeBackend};
use crate::error::BackendError;
use crate::events::ClaudeEvent;
use crate::parser::stream_json::{parse_line, to_claude_event};

/// Configuration for PrintModeBackend
#[derive(Debug, Clone, Default)]
pub struct PrintModeConfig {
    /// Path to claude binary (defaults to "claude")
    pub claude_path: Option<String>,
    /// Allowed tools (if empty, uses Claude's defaults)
    pub allowed_tools: Vec<String>,
    /// Working directory for the Claude process
    pub working_dir: Option<String>,
}

/// Backend that spawns Claude Code in print mode
///
/// Uses `claude -p --output-format stream-json` for communication.
/// Each `send()` spawns a new subprocess for the turn.
pub struct PrintModeBackend {
    /// Claude session ID for continuity
    claude_session_id: String,
    /// Current state
    state: BackendState,
    /// Configuration
    config: PrintModeConfig,
    /// Event broadcast channel
    tx: broadcast::Sender<ClaudeEvent>,
    /// Currently running process (if any)
    child: Option<Child>,
    /// Shutdown flag
    shutdown_requested: Arc<AtomicBool>,
}

impl PrintModeBackend {
    /// Create a new PrintModeBackend
    pub fn new(config: PrintModeConfig) -> Self {
        Self::with_session_id(Uuid::new_v4().to_string(), config)
    }

    /// Create a new PrintModeBackend with a specific session ID
    pub fn with_session_id(claude_session_id: String, config: PrintModeConfig) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            claude_session_id,
            state: BackendState::Idle,
            config,
            tx,
            child: None,
            shutdown_requested: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Build the command for spawning Claude
    ///
    /// This is extracted for testability - we can verify command construction
    /// without actually spawning processes.
    pub fn build_command(&self, input: &str) -> Command {
        let claude_path = self.config.claude_path.as_deref().unwrap_or("claude");

        let mut cmd = Command::new(claude_path);

        // Print mode with stream-json output (requires --verbose)
        cmd.arg("-p")
            .arg("--verbose")
            .arg("--output-format")
            .arg("stream-json");

        // Session ID for continuity
        cmd.arg("--session-id").arg(&self.claude_session_id);

        // Allowed tools (if specified)
        if !self.config.allowed_tools.is_empty() {
            cmd.arg("--allowedTools")
                .arg(self.config.allowed_tools.join(","));
        }

        // Working directory
        if let Some(ref working_dir) = self.config.working_dir {
            cmd.current_dir(working_dir);
        }

        // The prompt comes last
        cmd.arg(input);

        // Configure stdio
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::null());

        cmd
    }

    /// Spawn the Claude process and process its output
    fn spawn_and_process(&mut self, input: &str) -> Result<(), BackendError> {
        let mut cmd = self.build_command(input);

        // Spawn the process
        let mut child = cmd.spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                BackendError::ClaudeNotFound
            } else {
                BackendError::SpawnFailed(e)
            }
        })?;

        // Take stdout for reading
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| BackendError::ClaudeError("Failed to capture stdout".to_string()))?;

        // Store child for potential shutdown
        self.child = Some(child);

        // Read and process stdout line by line
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines() {
            // Check shutdown flag
            if self.shutdown_requested.load(Ordering::SeqCst) {
                break;
            }

            match line {
                Ok(line_str) => {
                    // Parse the line
                    if let Some(msg) = parse_line(&line_str) {
                        // Convert to ClaudeEvent
                        if let Some(event) = to_claude_event(msg) {
                            // Broadcast the event
                            let _ = self.tx.send(event);
                        }
                    }
                }
                Err(e) => {
                    // Log but don't fail - stream may have ended
                    eprintln!("Error reading stdout: {}", e);
                    break;
                }
            }
        }

        // Wait for process to complete and get exit status
        if let Some(mut child) = self.child.take() {
            match child.wait() {
                Ok(status) => {
                    if status.success() {
                        // Clean exit - transition to Idle
                        self.state = BackendState::Idle;
                    } else {
                        // Non-zero exit - transition to Failed
                        self.state = BackendState::Failed {
                            message: format!("Claude exited with code {:?}", status.code()),
                            recoverable: true,
                        };
                        // Emit error event
                        let _ = self.tx.send(ClaudeEvent::Error {
                            message: format!("Process exited with code {:?}", status.code()),
                            recoverable: true,
                        });
                    }
                }
                Err(e) => {
                    self.state = BackendState::Failed {
                        message: format!("Failed to wait for process: {}", e),
                        recoverable: true,
                    };
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl ClaudeBackend for PrintModeBackend {
    async fn send(&mut self, input: &str) -> Result<(), BackendError> {
        // Transition to Processing
        self.state = BackendState::Processing;

        // Spawn and process (blocking, but in async context)
        // Note: In a real implementation, this would use tokio::process
        // For now, we use std::process for simplicity
        self.spawn_and_process(input)?;

        Ok(())
    }

    fn subscribe(&self) -> broadcast::Receiver<ClaudeEvent> {
        self.tx.subscribe()
    }

    async fn respond_permission(
        &mut self,
        _request_id: &str,
        _approved: bool,
    ) -> Result<(), BackendError> {
        // PrintMode doesn't support interactive permission responses
        // This would require stdin communication
        Err(BackendError::ClaudeError(
            "PrintMode backend does not support interactive permissions".to_string(),
        ))
    }

    fn claude_session_id(&self) -> &str {
        &self.claude_session_id
    }

    fn state(&self) -> BackendState {
        self.state.clone()
    }

    async fn shutdown(&mut self) -> Result<(), BackendError> {
        self.shutdown_requested.store(true, Ordering::SeqCst);

        // Kill any running process
        if let Some(ref mut child) = self.child {
            let _ = child.kill();
        }

        self.state = BackendState::Finished;
        Ok(())
    }
}

/// Factory for creating PrintModeBackend instances
pub struct PrintModeBackendFactory {
    config: PrintModeConfig,
}

impl PrintModeBackendFactory {
    /// Create a new factory with the given config
    pub fn new(config: PrintModeConfig) -> Self {
        Self { config }
    }
}

impl super::traits::BackendFactory for PrintModeBackendFactory {
    fn create(&self, claude_session_id: Option<String>) -> Box<dyn ClaudeBackend> {
        match claude_session_id {
            Some(id) => Box::new(PrintModeBackend::with_session_id(id, self.config.clone())),
            None => Box::new(PrintModeBackend::new(self.config.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::traits::BackendFactory;

    // ==================== Config Tests ====================

    #[test]
    fn config_default_is_empty() {
        let config = PrintModeConfig::default();
        assert!(config.claude_path.is_none());
        assert!(config.allowed_tools.is_empty());
        assert!(config.working_dir.is_none());
    }

    // ==================== Creation Tests ====================

    #[test]
    fn new_creates_with_generated_session_id() {
        let backend = PrintModeBackend::new(PrintModeConfig::default());
        assert!(!backend.claude_session_id().is_empty());
    }

    #[test]
    fn new_with_session_id_uses_provided_id() {
        let backend =
            PrintModeBackend::with_session_id("my-session".to_string(), PrintModeConfig::default());
        assert_eq!(backend.claude_session_id(), "my-session");
    }

    #[test]
    fn new_starts_in_idle_state() {
        let backend = PrintModeBackend::new(PrintModeConfig::default());
        assert_eq!(backend.state(), BackendState::Idle);
    }

    // ==================== Command Building Tests ====================

    #[test]
    fn build_command_includes_print_mode_flag() {
        let backend = PrintModeBackend::new(PrintModeConfig::default());
        let cmd = backend.build_command("Hello");

        let args: Vec<_> = cmd.get_args().collect();
        assert!(args.contains(&std::ffi::OsStr::new("-p")));
    }

    #[test]
    fn build_command_includes_verbose_flag() {
        let backend = PrintModeBackend::new(PrintModeConfig::default());
        let cmd = backend.build_command("Hello");

        let args: Vec<_> = cmd.get_args().collect();
        assert!(args.contains(&std::ffi::OsStr::new("--verbose")));
    }

    #[test]
    fn build_command_includes_stream_json_format() {
        let backend = PrintModeBackend::new(PrintModeConfig::default());
        let cmd = backend.build_command("Hello");

        let args: Vec<_> = cmd.get_args().collect();
        assert!(args.contains(&std::ffi::OsStr::new("--output-format")));
        assert!(args.contains(&std::ffi::OsStr::new("stream-json")));
    }

    #[test]
    fn build_command_includes_session_id() {
        let backend = PrintModeBackend::with_session_id(
            "test-session-123".to_string(),
            PrintModeConfig::default(),
        );
        let cmd = backend.build_command("Hello");

        let args: Vec<_> = cmd.get_args().collect();
        assert!(args.contains(&std::ffi::OsStr::new("--session-id")));
        assert!(args.contains(&std::ffi::OsStr::new("test-session-123")));
    }

    #[test]
    fn build_command_includes_allowed_tools() {
        let config = PrintModeConfig {
            allowed_tools: vec!["Read".to_string(), "Write".to_string()],
            ..Default::default()
        };
        let backend = PrintModeBackend::new(config);
        let cmd = backend.build_command("Hello");

        let args: Vec<_> = cmd.get_args().collect();
        assert!(args.contains(&std::ffi::OsStr::new("--allowedTools")));
        assert!(args.contains(&std::ffi::OsStr::new("Read,Write")));
    }

    #[test]
    fn build_command_omits_allowed_tools_when_empty() {
        let backend = PrintModeBackend::new(PrintModeConfig::default());
        let cmd = backend.build_command("Hello");

        let args: Vec<_> = cmd.get_args().collect();
        assert!(!args.contains(&std::ffi::OsStr::new("--allowedTools")));
    }

    #[test]
    fn build_command_uses_custom_claude_path() {
        let config = PrintModeConfig {
            claude_path: Some("/custom/path/claude".to_string()),
            ..Default::default()
        };
        let backend = PrintModeBackend::new(config);
        let cmd = backend.build_command("Hello");

        assert_eq!(cmd.get_program(), "/custom/path/claude");
    }

    #[test]
    fn build_command_uses_default_claude_path() {
        let backend = PrintModeBackend::new(PrintModeConfig::default());
        let cmd = backend.build_command("Hello");

        assert_eq!(cmd.get_program(), "claude");
    }

    #[test]
    fn build_command_includes_prompt_as_last_arg() {
        let backend = PrintModeBackend::new(PrintModeConfig::default());
        let cmd = backend.build_command("What is 2+2?");

        let args: Vec<_> = cmd.get_args().collect();
        assert_eq!(args.last(), Some(&std::ffi::OsStr::new("What is 2+2?")));
    }

    // ==================== Subscribe Tests ====================

    #[test]
    fn subscribe_returns_receiver() {
        let backend = PrintModeBackend::new(PrintModeConfig::default());
        let _rx = backend.subscribe();
        // Just verify we can subscribe without panicking
    }

    // ==================== Factory Tests ====================

    #[tokio::test]
    async fn factory_creates_backend_with_config() {
        let config = PrintModeConfig {
            allowed_tools: vec!["Bash".to_string()],
            ..Default::default()
        };
        let factory = PrintModeBackendFactory::new(config);

        let backend = factory.create(None);
        assert!(!backend.claude_session_id().is_empty());
    }

    #[tokio::test]
    async fn factory_uses_provided_session_id() {
        let factory = PrintModeBackendFactory::new(PrintModeConfig::default());

        let backend = factory.create(Some("custom-session".to_string()));
        assert_eq!(backend.claude_session_id(), "custom-session");
    }

    // ==================== Shutdown Tests ====================

    #[tokio::test]
    async fn shutdown_transitions_to_finished() {
        let mut backend = PrintModeBackend::new(PrintModeConfig::default());

        backend.shutdown().await.unwrap();

        assert_eq!(backend.state(), BackendState::Finished);
    }

    // Note: Integration tests that actually spawn Claude would go in tests/integration/
    // These unit tests verify the command construction logic without spawning processes.
}
