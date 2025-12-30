//! Session event collection and buffering

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

/// Events collected for a session
#[derive(Debug, Clone)]
pub struct ToolEvent {
    /// Name of the tool used
    pub tool_name: String,
    /// Input to the tool (JSON string)
    pub input: String,
    /// Output from the tool (if completed)
    pub output: Option<String>,
    /// Whether the tool succeeded
    pub success: bool,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
}

impl ToolEvent {
    /// Create a new pre-tool-use event (incomplete)
    pub fn pre_use(tool_name: String, input: String) -> Self {
        Self {
            tool_name,
            input,
            output: None,
            success: false,
            duration_ms: None,
            timestamp: Utc::now(),
        }
    }

    /// Create a completed tool event
    pub fn completed(
        tool_name: String,
        input: String,
        output: String,
        success: bool,
        duration_ms: u64,
    ) -> Self {
        Self {
            tool_name,
            input,
            output: Some(output),
            success,
            duration_ms: Some(duration_ms),
            timestamp: Utc::now(),
        }
    }
}

/// Buffer holding events for an active session
#[derive(Debug)]
pub struct SessionBuffer {
    /// Session identifier
    pub session_id: String,
    /// Project path where session is active
    pub project_path: Option<PathBuf>,
    /// Tool events recorded during session
    pub tool_events: Vec<ToolEvent>,
    /// User prompts submitted during session
    pub prompts: Vec<String>,
    /// When the session started
    pub start_time: DateTime<Utc>,
}

impl SessionBuffer {
    /// Create a new session buffer
    pub fn new(session_id: String, project_path: Option<PathBuf>) -> Self {
        Self {
            session_id,
            project_path,
            tool_events: Vec::new(),
            prompts: Vec::new(),
            start_time: Utc::now(),
        }
    }

    /// Record a tool event
    pub fn record_tool(&mut self, event: ToolEvent) {
        self.tool_events.push(event);
    }

    /// Record a user prompt
    pub fn record_prompt(&mut self, prompt: String) {
        self.prompts.push(prompt);
    }

    /// Get the number of events recorded
    pub fn event_count(&self) -> usize {
        self.tool_events.len()
    }
}

/// Collects and buffers session events
pub struct SessionCollector {
    /// Active session buffers keyed by session ID
    sessions: Arc<RwLock<HashMap<String, SessionBuffer>>>,
}

impl SessionCollector {
    /// Create a new session collector
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start tracking a new session
    pub async fn start_session(&self, session_id: String, project_path: Option<PathBuf>) {
        let buffer = SessionBuffer::new(session_id.clone(), project_path);
        self.sessions.write().await.insert(session_id, buffer);
    }

    /// Record a tool event for a session
    pub async fn record_tool_event(&self, session_id: &str, event: ToolEvent) {
        let mut sessions = self.sessions.write().await;
        if let Some(buffer) = sessions.get_mut(session_id) {
            buffer.record_tool(event);
        }
    }

    /// Record a user prompt for a session
    pub async fn record_prompt(&self, session_id: &str, prompt: String) {
        let mut sessions = self.sessions.write().await;
        if let Some(buffer) = sessions.get_mut(session_id) {
            buffer.record_prompt(prompt);
        }
    }

    /// End a session and return its buffer for processing
    pub async fn end_session(&self, session_id: &str) -> Option<SessionBuffer> {
        self.sessions.write().await.remove(session_id)
    }

    /// Get the number of active sessions
    pub async fn active_session_count(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Check if a session is being tracked
    pub async fn has_session(&self, session_id: &str) -> bool {
        self.sessions.read().await.contains_key(session_id)
    }
}

impl Default for SessionCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_start_session() {
        let collector = SessionCollector::new();

        collector
            .start_session("sess-123".to_string(), Some(PathBuf::from("/project")))
            .await;

        assert!(collector.has_session("sess-123").await);
        assert_eq!(collector.active_session_count().await, 1);
    }

    #[tokio::test]
    async fn test_record_tool_event() {
        let collector = SessionCollector::new();
        collector.start_session("sess-1".to_string(), None).await;

        let event = ToolEvent::completed(
            "Bash".to_string(),
            r#"{"command":"ls"}"#.to_string(),
            "file.txt".to_string(),
            true,
            150,
        );
        collector.record_tool_event("sess-1", event).await;

        let buffer = collector.end_session("sess-1").await.unwrap();
        assert_eq!(buffer.tool_events.len(), 1);
        assert_eq!(buffer.tool_events[0].tool_name, "Bash");
    }

    #[tokio::test]
    async fn test_record_prompt() {
        let collector = SessionCollector::new();
        collector.start_session("sess-1".to_string(), None).await;

        collector
            .record_prompt("sess-1", "Help me with Rust".to_string())
            .await;

        let buffer = collector.end_session("sess-1").await.unwrap();
        assert_eq!(buffer.prompts.len(), 1);
        assert_eq!(buffer.prompts[0], "Help me with Rust");
    }

    #[tokio::test]
    async fn test_end_session_removes_from_active() {
        let collector = SessionCollector::new();
        collector.start_session("sess-1".to_string(), None).await;

        assert!(collector.has_session("sess-1").await);

        let buffer = collector.end_session("sess-1").await;
        assert!(buffer.is_some());
        assert!(!collector.has_session("sess-1").await);
    }

    #[tokio::test]
    async fn test_end_session_nonexistent_returns_none() {
        let collector = SessionCollector::new();
        assert!(collector.end_session("nonexistent").await.is_none());
    }

    #[tokio::test]
    async fn test_multiple_sessions() {
        let collector = SessionCollector::new();

        collector.start_session("sess-1".to_string(), None).await;
        collector.start_session("sess-2".to_string(), None).await;
        collector.start_session("sess-3".to_string(), None).await;

        assert_eq!(collector.active_session_count().await, 3);

        collector.end_session("sess-2").await;
        assert_eq!(collector.active_session_count().await, 2);
        assert!(!collector.has_session("sess-2").await);
    }

    #[test]
    fn test_tool_event_pre_use() {
        let event = ToolEvent::pre_use("Read".to_string(), r#"{"path":"file.rs"}"#.to_string());

        assert_eq!(event.tool_name, "Read");
        assert!(event.output.is_none());
        assert!(!event.success);
    }

    #[test]
    fn test_session_buffer_event_count() {
        let mut buffer = SessionBuffer::new("sess-1".to_string(), None);

        assert_eq!(buffer.event_count(), 0);

        buffer.record_tool(ToolEvent::pre_use("Test".to_string(), "{}".to_string()));
        assert_eq!(buffer.event_count(), 1);
    }
}
