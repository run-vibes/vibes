//! Hook event types
//!
//! These types represent the structured data received from Claude Code hooks.

use serde::{Deserialize, Serialize};

/// Type of hook event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookType {
    PreToolUse,
    PostToolUse,
    Stop,
}

/// Data from a PreToolUse hook
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreToolUseData {
    /// The tool being called
    pub tool_name: String,
    /// Input parameters as JSON string
    pub input: String,
    /// Optional session ID (from VIBES_SESSION_ID env var)
    pub session_id: Option<String>,
}

/// Data from a PostToolUse hook
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostToolUseData {
    /// The tool that was called
    pub tool_name: String,
    /// Tool output
    pub output: String,
    /// Whether the tool succeeded
    pub success: bool,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Optional session ID
    pub session_id: Option<String>,
}

/// Data from a Stop hook
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopData {
    /// Path to the transcript JSONL file
    pub transcript_path: Option<String>,
    /// Reason for stopping (user, error, etc.)
    pub reason: Option<String>,
    /// Optional session ID
    pub session_id: Option<String>,
}

/// A hook event received from Claude Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HookEvent {
    PreToolUse(PreToolUseData),
    PostToolUse(PostToolUseData),
    Stop(StopData),
}

impl HookEvent {
    /// Get the session ID from this event, if available
    pub fn session_id(&self) -> Option<&str> {
        match self {
            HookEvent::PreToolUse(data) => data.session_id.as_deref(),
            HookEvent::PostToolUse(data) => data.session_id.as_deref(),
            HookEvent::Stop(data) => data.session_id.as_deref(),
        }
    }

    /// Get the hook type
    pub fn hook_type(&self) -> HookType {
        match self {
            HookEvent::PreToolUse(_) => HookType::PreToolUse,
            HookEvent::PostToolUse(_) => HookType::PostToolUse,
            HookEvent::Stop(_) => HookType::Stop,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_tool_use_serialization() {
        let data = PreToolUseData {
            tool_name: "Bash".to_string(),
            input: r#"{"command": "ls -la"}"#.to_string(),
            session_id: Some("sess-123".to_string()),
        };
        let event = HookEvent::PreToolUse(data);

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("pre_tool_use"));
        assert!(json.contains("Bash"));

        let parsed: HookEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.session_id(), Some("sess-123"));
    }

    #[test]
    fn test_post_tool_use_serialization() {
        let data = PostToolUseData {
            tool_name: "Read".to_string(),
            output: "file contents...".to_string(),
            success: true,
            duration_ms: 150,
            session_id: None,
        };
        let event = HookEvent::PostToolUse(data);

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("post_tool_use"));
        assert!(json.contains("Read"));

        let parsed: HookEvent = serde_json::from_str(&json).unwrap();
        assert!(parsed.session_id().is_none());
    }

    #[test]
    fn test_stop_serialization() {
        let data = StopData {
            transcript_path: Some("/tmp/transcript.jsonl".to_string()),
            reason: Some("user".to_string()),
            session_id: Some("sess-456".to_string()),
        };
        let event = HookEvent::Stop(data);

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("stop"));
        assert!(json.contains("transcript.jsonl"));
    }

    #[test]
    fn test_hook_type() {
        let pre = HookEvent::PreToolUse(PreToolUseData {
            tool_name: "Test".to_string(),
            input: "{}".to_string(),
            session_id: None,
        });
        assert_eq!(pre.hook_type(), HookType::PreToolUse);
    }
}
