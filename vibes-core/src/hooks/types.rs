//! Hook event types
//!
//! These types represent the structured data received from Claude Code hooks.
//!
//! All hook data types derive `PartialEq` for consistency and testability.
//! This allows comparing events in tests and building equality-based logic.

use serde::{Deserialize, Serialize};

/// Type of hook event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookType {
    PreToolUse,
    PostToolUse,
    Stop,
    SessionStart,
    UserPromptSubmit,
    PermissionRequest,
    Notification,
    SubagentStop,
    PreCompact,
    SessionEnd,
}

impl HookType {
    /// Get the hook type as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            HookType::PreToolUse => "PreToolUse",
            HookType::PostToolUse => "PostToolUse",
            HookType::Stop => "Stop",
            HookType::SessionStart => "SessionStart",
            HookType::UserPromptSubmit => "UserPromptSubmit",
            HookType::PermissionRequest => "PermissionRequest",
            HookType::Notification => "Notification",
            HookType::SubagentStop => "SubagentStop",
            HookType::PreCompact => "PreCompact",
            HookType::SessionEnd => "SessionEnd",
        }
    }
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

/// Data from a SessionStart hook
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionStartData {
    /// Optional session ID
    pub session_id: Option<String>,
    /// Project path where session started
    pub project_path: Option<String>,
}

/// Data from a UserPromptSubmit hook
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserPromptSubmitData {
    /// Optional session ID
    pub session_id: Option<String>,
    /// The prompt being submitted
    pub prompt: String,
}

/// Data from a PermissionRequest hook
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermissionRequestData {
    /// Optional session ID
    pub session_id: Option<String>,
    /// The tool requesting permission
    pub tool_name: String,
    /// Tool input as JSON string
    pub input: String,
}

/// Data from a Notification hook
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationData {
    /// Optional session ID
    pub session_id: Option<String>,
    /// Notification title
    pub title: String,
    /// Notification message
    pub message: String,
}

/// Data from a SubagentStop hook
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubagentStopData {
    /// Optional session ID
    pub session_id: Option<String>,
    /// ID of the subagent that stopped
    pub subagent_id: String,
    /// Reason for stopping
    pub reason: Option<String>,
}

/// Data from a PreCompact hook
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreCompactData {
    /// Optional session ID
    pub session_id: Option<String>,
}

/// Data from a SessionEnd hook
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionEndData {
    /// Optional session ID
    pub session_id: Option<String>,
    /// Reason for ending
    pub reason: Option<String>,
}

/// A hook event received from Claude Code
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HookEvent {
    PreToolUse(PreToolUseData),
    PostToolUse(PostToolUseData),
    Stop(StopData),
    SessionStart(SessionStartData),
    UserPromptSubmit(UserPromptSubmitData),
    PermissionRequest(PermissionRequestData),
    Notification(NotificationData),
    SubagentStop(SubagentStopData),
    PreCompact(PreCompactData),
    SessionEnd(SessionEndData),
}

impl HookEvent {
    /// Get the session ID from this event, if available
    pub fn session_id(&self) -> Option<&str> {
        match self {
            HookEvent::PreToolUse(data) => data.session_id.as_deref(),
            HookEvent::PostToolUse(data) => data.session_id.as_deref(),
            HookEvent::Stop(data) => data.session_id.as_deref(),
            HookEvent::SessionStart(data) => data.session_id.as_deref(),
            HookEvent::UserPromptSubmit(data) => data.session_id.as_deref(),
            HookEvent::PermissionRequest(data) => data.session_id.as_deref(),
            HookEvent::Notification(data) => data.session_id.as_deref(),
            HookEvent::SubagentStop(data) => data.session_id.as_deref(),
            HookEvent::PreCompact(data) => data.session_id.as_deref(),
            HookEvent::SessionEnd(data) => data.session_id.as_deref(),
        }
    }

    /// Get the hook type
    pub fn hook_type(&self) -> HookType {
        match self {
            HookEvent::PreToolUse(_) => HookType::PreToolUse,
            HookEvent::PostToolUse(_) => HookType::PostToolUse,
            HookEvent::Stop(_) => HookType::Stop,
            HookEvent::SessionStart(_) => HookType::SessionStart,
            HookEvent::UserPromptSubmit(_) => HookType::UserPromptSubmit,
            HookEvent::PermissionRequest(_) => HookType::PermissionRequest,
            HookEvent::Notification(_) => HookType::Notification,
            HookEvent::SubagentStop(_) => HookType::SubagentStop,
            HookEvent::PreCompact(_) => HookType::PreCompact,
            HookEvent::SessionEnd(_) => HookType::SessionEnd,
        }
    }

    /// Whether this hook type supports returning a response
    pub fn supports_response(&self) -> bool {
        matches!(
            self,
            HookEvent::SessionStart(_)
                | HookEvent::UserPromptSubmit(_)
                | HookEvent::PermissionRequest(_)
        )
    }

    /// Get the project path from this event, if available
    pub fn project_path(&self) -> Option<String> {
        match self {
            HookEvent::SessionStart(data) => data.project_path.clone(),
            // Other hook types don't carry project path
            _ => None,
        }
    }
}

/// Response to send back to Claude Code for injection hooks
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookResponse {
    /// Additional context to inject into Claude's conversation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_context: Option<String>,
}

impl HookResponse {
    /// Create an empty response (no injection)
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create a response with additional context
    pub fn with_context(context: impl Into<String>) -> Self {
        Self {
            additional_context: Some(context.into()),
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

    #[test]
    fn test_session_start_serialization() {
        let data = SessionStartData {
            session_id: Some("sess-789".to_string()),
            project_path: Some("/home/user/project".to_string()),
        };
        let event = HookEvent::SessionStart(data);

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("session_start"));

        let parsed: HookEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.session_id(), Some("sess-789"));
    }

    #[test]
    fn test_user_prompt_submit_serialization() {
        let data = UserPromptSubmitData {
            session_id: Some("sess-abc".to_string()),
            prompt: "Help me with Rust".to_string(),
        };
        let event = HookEvent::UserPromptSubmit(data);

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("user_prompt_submit"));

        let parsed: HookEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.session_id(), Some("sess-abc"));
    }

    #[test]
    fn test_hook_supports_response() {
        let session_start = HookEvent::SessionStart(SessionStartData {
            session_id: None,
            project_path: None,
        });
        assert!(session_start.supports_response());

        let user_prompt = HookEvent::UserPromptSubmit(UserPromptSubmitData {
            session_id: None,
            prompt: "test".to_string(),
        });
        assert!(user_prompt.supports_response());

        let stop = HookEvent::Stop(StopData {
            transcript_path: None,
            reason: None,
            session_id: None,
        });
        assert!(!stop.supports_response());
    }

    #[test]
    fn test_hook_response_serialization() {
        let response = HookResponse {
            additional_context: Some("## groove Learnings\n\n- Use pytest".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("additionalContext"));
        assert!(json.contains("groove"));

        let parsed: HookResponse = serde_json::from_str(&json).unwrap();
        assert!(parsed.additional_context.unwrap().contains("pytest"));
    }

    #[test]
    fn test_hook_response_empty() {
        let response = HookResponse::empty();
        let json = serde_json::to_string(&response).unwrap();
        // Empty response should still be valid JSON
        let parsed: HookResponse = serde_json::from_str(&json).unwrap();
        assert!(parsed.additional_context.is_none());
    }

    // TDD: Tests for new hook types (should fail until implemented)

    #[test]
    fn test_permission_request_serialization() {
        let data = PermissionRequestData {
            session_id: Some("sess-perm-123".to_string()),
            tool_name: "Bash".to_string(),
            input: r#"{"command": "rm -rf /"}"#.to_string(),
        };
        let event = HookEvent::PermissionRequest(data);

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("permission_request"));
        assert!(json.contains("Bash"));

        let parsed: HookEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.session_id(), Some("sess-perm-123"));
        assert_eq!(parsed.hook_type(), HookType::PermissionRequest);
    }

    #[test]
    fn test_notification_serialization() {
        let data = NotificationData {
            session_id: Some("sess-notif-456".to_string()),
            title: "Build Complete".to_string(),
            message: "Your build finished successfully".to_string(),
        };
        let event = HookEvent::Notification(data);

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("notification"));
        assert!(json.contains("Build Complete"));

        let parsed: HookEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.session_id(), Some("sess-notif-456"));
        assert_eq!(parsed.hook_type(), HookType::Notification);
    }

    #[test]
    fn test_subagent_stop_serialization() {
        let data = SubagentStopData {
            session_id: Some("sess-sub-789".to_string()),
            subagent_id: "agent-42".to_string(),
            reason: Some("completed".to_string()),
        };
        let event = HookEvent::SubagentStop(data);

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("subagent_stop"));
        assert!(json.contains("agent-42"));

        let parsed: HookEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.session_id(), Some("sess-sub-789"));
        assert_eq!(parsed.hook_type(), HookType::SubagentStop);
    }

    #[test]
    fn test_pre_compact_serialization() {
        let data = PreCompactData {
            session_id: Some("sess-compact-abc".to_string()),
        };
        let event = HookEvent::PreCompact(data);

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("pre_compact"));

        let parsed: HookEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.session_id(), Some("sess-compact-abc"));
        assert_eq!(parsed.hook_type(), HookType::PreCompact);
    }

    #[test]
    fn test_session_end_serialization() {
        let data = SessionEndData {
            session_id: Some("sess-end-xyz".to_string()),
            reason: Some("user_exit".to_string()),
        };
        let event = HookEvent::SessionEnd(data);

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("session_end"));
        assert!(json.contains("user_exit"));

        let parsed: HookEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.session_id(), Some("sess-end-xyz"));
        assert_eq!(parsed.hook_type(), HookType::SessionEnd);
    }

    #[test]
    fn test_permission_request_supports_response() {
        let event = HookEvent::PermissionRequest(PermissionRequestData {
            session_id: None,
            tool_name: "Write".to_string(),
            input: "{}".to_string(),
        });
        // PermissionRequest can block/modify, so it supports response
        assert!(event.supports_response());
    }

    #[test]
    fn test_fire_and_forget_hooks_no_response() {
        let notification = HookEvent::Notification(NotificationData {
            session_id: None,
            title: "test".to_string(),
            message: "test".to_string(),
        });
        assert!(!notification.supports_response());

        let subagent = HookEvent::SubagentStop(SubagentStopData {
            session_id: None,
            subagent_id: "sub-1".to_string(),
            reason: None,
        });
        assert!(!subagent.supports_response());

        let compact = HookEvent::PreCompact(PreCompactData { session_id: None });
        assert!(!compact.supports_response());

        let end = HookEvent::SessionEnd(SessionEndData {
            session_id: None,
            reason: None,
        });
        assert!(!end.supports_response());
    }
}
