//! Hook event types
//!
//! These types represent the structured data received from Claude Code hooks.
//! Field names and types match Claude Code's actual hook payloads.
//!
//! All hook data types derive `PartialEq` for consistency and testability.
//! This allows comparing events in tests and building equality-based logic.

use serde::{Deserialize, Serialize};
use serde_json::Value;

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
///
/// Example payload from Claude Code:
/// ```json
/// {
///   "session_id": "3535585c-1cb7-459f-8917-1df1802ce7f8",
///   "transcript_path": "/home/user/.claude/projects/.../session.jsonl",
///   "cwd": "/home/user/project",
///   "permission_mode": "default",
///   "hook_event_name": "PreToolUse",
///   "tool_name": "Bash",
///   "tool_input": {"command": "ls -la", "description": "List files"},
///   "tool_use_id": "toolu_01ABC..."
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreToolUseData {
    /// Session ID
    #[serde(default)]
    pub session_id: Option<String>,
    /// Path to the transcript JSONL file
    #[serde(default)]
    pub transcript_path: Option<String>,
    /// Current working directory
    #[serde(default)]
    pub cwd: Option<String>,
    /// Permission mode (e.g., "default")
    #[serde(default)]
    pub permission_mode: Option<String>,
    /// Hook event name (always "PreToolUse")
    #[serde(default)]
    pub hook_event_name: Option<String>,
    /// The tool being called
    pub tool_name: String,
    /// Tool input parameters as a JSON object
    pub tool_input: Value,
    /// Unique tool use ID
    #[serde(default)]
    pub tool_use_id: Option<String>,
}

/// Data from a PostToolUse hook
///
/// Example payload from Claude Code:
/// ```json
/// {
///   "session_id": "3535585c-1cb7-459f-8917-1df1802ce7f8",
///   "transcript_path": "/home/user/.claude/projects/.../session.jsonl",
///   "cwd": "/home/user/project",
///   "permission_mode": "default",
///   "hook_event_name": "PostToolUse",
///   "tool_name": "Bash",
///   "tool_input": {"command": "ls -la"},
///   "tool_response": {"stdout": "...", "stderr": "", "interrupted": false, "isImage": false},
///   "tool_use_id": "toolu_01ABC..."
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostToolUseData {
    /// Session ID
    #[serde(default)]
    pub session_id: Option<String>,
    /// Path to the transcript JSONL file
    #[serde(default)]
    pub transcript_path: Option<String>,
    /// Current working directory
    #[serde(default)]
    pub cwd: Option<String>,
    /// Permission mode (e.g., "default")
    #[serde(default)]
    pub permission_mode: Option<String>,
    /// Hook event name (always "PostToolUse")
    #[serde(default)]
    pub hook_event_name: Option<String>,
    /// The tool that was called
    pub tool_name: String,
    /// Tool input parameters as a JSON object
    #[serde(default)]
    pub tool_input: Option<Value>,
    /// Tool response/output as a JSON object
    pub tool_response: Value,
    /// Unique tool use ID
    #[serde(default)]
    pub tool_use_id: Option<String>,
}

/// Data from a Stop hook
///
/// Example payload from Claude Code:
/// ```json
/// {
///   "session_id": "3535585c-1cb7-459f-8917-1df1802ce7f8",
///   "transcript_path": "/home/user/.claude/projects/.../session.jsonl",
///   "cwd": "/home/user/project",
///   "permission_mode": "default",
///   "hook_event_name": "Stop",
///   "stop_hook_active": false
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StopData {
    /// Session ID
    #[serde(default)]
    pub session_id: Option<String>,
    /// Path to the transcript JSONL file
    #[serde(default)]
    pub transcript_path: Option<String>,
    /// Current working directory
    #[serde(default)]
    pub cwd: Option<String>,
    /// Permission mode
    #[serde(default)]
    pub permission_mode: Option<String>,
    /// Hook event name (always "Stop")
    #[serde(default)]
    pub hook_event_name: Option<String>,
    /// Whether a stop hook is currently active
    #[serde(default)]
    pub stop_hook_active: Option<bool>,
}

/// Data from a SessionStart hook
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionStartData {
    /// Session ID
    #[serde(default)]
    pub session_id: Option<String>,
    /// Path to the transcript JSONL file
    #[serde(default)]
    pub transcript_path: Option<String>,
    /// Current working directory
    #[serde(default)]
    pub cwd: Option<String>,
    /// Hook event name (always "SessionStart")
    #[serde(default)]
    pub hook_event_name: Option<String>,
    /// Project path where session started (may be same as cwd)
    #[serde(default)]
    pub project_path: Option<String>,
}

/// Data from a UserPromptSubmit hook
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserPromptSubmitData {
    /// Session ID
    #[serde(default)]
    pub session_id: Option<String>,
    /// Path to the transcript JSONL file
    #[serde(default)]
    pub transcript_path: Option<String>,
    /// Current working directory
    #[serde(default)]
    pub cwd: Option<String>,
    /// Hook event name (always "UserPromptSubmit")
    #[serde(default)]
    pub hook_event_name: Option<String>,
    /// The prompt being submitted
    #[serde(default)]
    pub prompt: Option<String>,
}

/// Data from a PermissionRequest hook
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermissionRequestData {
    /// Session ID
    #[serde(default)]
    pub session_id: Option<String>,
    /// Path to the transcript JSONL file
    #[serde(default)]
    pub transcript_path: Option<String>,
    /// Current working directory
    #[serde(default)]
    pub cwd: Option<String>,
    /// Hook event name (always "PermissionRequest")
    #[serde(default)]
    pub hook_event_name: Option<String>,
    /// The tool requesting permission
    #[serde(default)]
    pub tool_name: Option<String>,
    /// Tool input as JSON object
    #[serde(default)]
    pub tool_input: Option<Value>,
}

/// Data from a Notification hook
///
/// Example payload from Claude Code:
/// ```json
/// {
///   "session_id": "3535585c-1cb7-459f-8917-1df1802ce7f8",
///   "transcript_path": "/home/user/.claude/projects/.../session.jsonl",
///   "cwd": "/home/user/project",
///   "hook_event_name": "Notification",
///   "message": "Claude needs your permission to use Bash",
///   "notification_type": "permission_prompt"
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationData {
    /// Session ID
    #[serde(default)]
    pub session_id: Option<String>,
    /// Path to the transcript JSONL file
    #[serde(default)]
    pub transcript_path: Option<String>,
    /// Current working directory
    #[serde(default)]
    pub cwd: Option<String>,
    /// Hook event name (always "Notification")
    #[serde(default)]
    pub hook_event_name: Option<String>,
    /// Notification message
    pub message: String,
    /// Type of notification (e.g., "permission_prompt", "idle_prompt")
    #[serde(default)]
    pub notification_type: Option<String>,
}

/// Data from a SubagentStop hook
///
/// Example payload from Claude Code:
/// ```json
/// {
///   "session_id": "3535585c-1cb7-459f-8917-1df1802ce7f8",
///   "transcript_path": "/home/user/.claude/projects/.../session.jsonl",
///   "cwd": "/home/user/project",
///   "permission_mode": "default",
///   "hook_event_name": "SubagentStop",
///   "stop_hook_active": false,
///   "agent_id": "ac6e1e6",
///   "agent_transcript_path": "/home/user/.claude/projects/.../agent-ac6e1e6.jsonl"
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubagentStopData {
    /// Session ID
    #[serde(default)]
    pub session_id: Option<String>,
    /// Path to the transcript JSONL file
    #[serde(default)]
    pub transcript_path: Option<String>,
    /// Current working directory
    #[serde(default)]
    pub cwd: Option<String>,
    /// Permission mode
    #[serde(default)]
    pub permission_mode: Option<String>,
    /// Hook event name (always "SubagentStop")
    #[serde(default)]
    pub hook_event_name: Option<String>,
    /// Whether a stop hook is currently active
    #[serde(default)]
    pub stop_hook_active: Option<bool>,
    /// ID of the subagent that stopped
    pub agent_id: String,
    /// Path to the subagent's transcript
    #[serde(default)]
    pub agent_transcript_path: Option<String>,
}

/// Data from a PreCompact hook
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreCompactData {
    /// Session ID
    #[serde(default)]
    pub session_id: Option<String>,
    /// Path to the transcript JSONL file
    #[serde(default)]
    pub transcript_path: Option<String>,
    /// Current working directory
    #[serde(default)]
    pub cwd: Option<String>,
    /// Hook event name (always "PreCompact")
    #[serde(default)]
    pub hook_event_name: Option<String>,
}

/// Data from a SessionEnd hook
///
/// Example payload from Claude Code:
/// ```json
/// {
///   "session_id": "3535585c-1cb7-459f-8917-1df1802ce7f8",
///   "transcript_path": "/home/user/.claude/projects/.../session.jsonl",
///   "cwd": "/home/user/project",
///   "hook_event_name": "SessionEnd",
///   "reason": "other"
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionEndData {
    /// Session ID
    #[serde(default)]
    pub session_id: Option<String>,
    /// Path to the transcript JSONL file
    #[serde(default)]
    pub transcript_path: Option<String>,
    /// Current working directory
    #[serde(default)]
    pub cwd: Option<String>,
    /// Hook event name (always "SessionEnd")
    #[serde(default)]
    pub hook_event_name: Option<String>,
    /// Reason for ending (e.g., "other", "user_exit")
    #[serde(default)]
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
    use serde_json::json;

    // Tests for parsing REAL Claude Code payloads (captured from actual hooks)

    #[test]
    fn test_parse_real_pre_tool_use_payload() {
        let payload = r#"{
            "type": "pre_tool_use",
            "session_id": "3535585c-1cb7-459f-8917-1df1802ce7f8",
            "transcript_path": "/home/alex/.claude/projects/test/session.jsonl",
            "cwd": "/home/alex/workspace/project",
            "permission_mode": "default",
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_input": {"command": "ls -la", "description": "List files"},
            "tool_use_id": "toolu_01ABC123"
        }"#;

        let event: HookEvent = serde_json::from_str(payload).unwrap();
        assert_eq!(event.hook_type(), HookType::PreToolUse);
        assert_eq!(
            event.session_id(),
            Some("3535585c-1cb7-459f-8917-1df1802ce7f8")
        );

        if let HookEvent::PreToolUse(data) = event {
            assert_eq!(data.tool_name, "Bash");
            assert_eq!(data.tool_input["command"], "ls -la");
            assert_eq!(data.cwd, Some("/home/alex/workspace/project".to_string()));
        } else {
            panic!("Expected PreToolUse");
        }
    }

    #[test]
    fn test_parse_real_post_tool_use_payload() {
        let payload = r#"{
            "type": "post_tool_use",
            "session_id": "3535585c-1cb7-459f-8917-1df1802ce7f8",
            "transcript_path": "/home/alex/.claude/projects/test/session.jsonl",
            "cwd": "/home/alex/workspace/project",
            "permission_mode": "default",
            "hook_event_name": "PostToolUse",
            "tool_name": "Bash",
            "tool_input": {"command": "echo hello"},
            "tool_response": {"stdout": "hello\n", "stderr": "", "interrupted": false, "isImage": false},
            "tool_use_id": "toolu_01ABC123"
        }"#;

        let event: HookEvent = serde_json::from_str(payload).unwrap();
        assert_eq!(event.hook_type(), HookType::PostToolUse);

        if let HookEvent::PostToolUse(data) = event {
            assert_eq!(data.tool_name, "Bash");
            assert_eq!(data.tool_response["stdout"], "hello\n");
            assert_eq!(data.tool_response["interrupted"], false);
        } else {
            panic!("Expected PostToolUse");
        }
    }

    #[test]
    fn test_parse_real_notification_payload() {
        let payload = r#"{
            "type": "notification",
            "session_id": "3535585c-1cb7-459f-8917-1df1802ce7f8",
            "transcript_path": "/home/alex/.claude/projects/test/session.jsonl",
            "cwd": "/home/alex/workspace/project",
            "hook_event_name": "Notification",
            "message": "Claude needs your permission to use Bash",
            "notification_type": "permission_prompt"
        }"#;

        let event: HookEvent = serde_json::from_str(payload).unwrap();
        assert_eq!(event.hook_type(), HookType::Notification);

        if let HookEvent::Notification(data) = event {
            assert_eq!(data.message, "Claude needs your permission to use Bash");
            assert_eq!(
                data.notification_type,
                Some("permission_prompt".to_string())
            );
        } else {
            panic!("Expected Notification");
        }
    }

    #[test]
    fn test_parse_real_stop_payload() {
        let payload = r#"{
            "type": "stop",
            "session_id": "3535585c-1cb7-459f-8917-1df1802ce7f8",
            "transcript_path": "/home/alex/.claude/projects/test/session.jsonl",
            "cwd": "/home/alex/workspace/project",
            "permission_mode": "default",
            "hook_event_name": "Stop",
            "stop_hook_active": false
        }"#;

        let event: HookEvent = serde_json::from_str(payload).unwrap();
        assert_eq!(event.hook_type(), HookType::Stop);

        if let HookEvent::Stop(data) = event {
            assert_eq!(data.stop_hook_active, Some(false));
            assert!(data.transcript_path.is_some());
        } else {
            panic!("Expected Stop");
        }
    }

    #[test]
    fn test_parse_real_subagent_stop_payload() {
        let payload = r#"{
            "type": "subagent_stop",
            "session_id": "3535585c-1cb7-459f-8917-1df1802ce7f8",
            "transcript_path": "/home/alex/.claude/projects/test/session.jsonl",
            "cwd": "/home/alex/workspace/project",
            "permission_mode": "default",
            "hook_event_name": "SubagentStop",
            "stop_hook_active": false,
            "agent_id": "ac6e1e6",
            "agent_transcript_path": "/home/alex/.claude/projects/test/agent-ac6e1e6.jsonl"
        }"#;

        let event: HookEvent = serde_json::from_str(payload).unwrap();
        assert_eq!(event.hook_type(), HookType::SubagentStop);

        if let HookEvent::SubagentStop(data) = event {
            assert_eq!(data.agent_id, "ac6e1e6");
            assert!(data.agent_transcript_path.is_some());
        } else {
            panic!("Expected SubagentStop");
        }
    }

    #[test]
    fn test_parse_real_session_end_payload() {
        let payload = r#"{
            "type": "session_end",
            "session_id": "3535585c-1cb7-459f-8917-1df1802ce7f8",
            "transcript_path": "/home/alex/.claude/projects/test/session.jsonl",
            "cwd": "/home/alex/workspace/project",
            "hook_event_name": "SessionEnd",
            "reason": "other"
        }"#;

        let event: HookEvent = serde_json::from_str(payload).unwrap();
        assert_eq!(event.hook_type(), HookType::SessionEnd);

        if let HookEvent::SessionEnd(data) = event {
            assert_eq!(data.reason, Some("other".to_string()));
        } else {
            panic!("Expected SessionEnd");
        }
    }

    // Basic struct construction tests

    #[test]
    fn test_pre_tool_use_construction() {
        let data = PreToolUseData {
            session_id: Some("sess-123".to_string()),
            transcript_path: None,
            cwd: Some("/project".to_string()),
            permission_mode: Some("default".to_string()),
            hook_event_name: Some("PreToolUse".to_string()),
            tool_name: "Bash".to_string(),
            tool_input: json!({"command": "ls"}),
            tool_use_id: Some("toolu_123".to_string()),
        };
        let event = HookEvent::PreToolUse(data);

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("pre_tool_use"));
        assert!(json.contains("Bash"));
    }

    #[test]
    fn test_post_tool_use_construction() {
        let data = PostToolUseData {
            session_id: None,
            transcript_path: None,
            cwd: None,
            permission_mode: None,
            hook_event_name: None,
            tool_name: "Read".to_string(),
            tool_input: Some(json!({"file_path": "/tmp/test"})),
            tool_response: json!({"content": "file contents..."}),
            tool_use_id: None,
        };
        let event = HookEvent::PostToolUse(data);

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("post_tool_use"));
        assert!(json.contains("Read"));
    }

    #[test]
    fn test_hook_supports_response() {
        let session_start = HookEvent::SessionStart(SessionStartData {
            session_id: None,
            transcript_path: None,
            cwd: None,
            hook_event_name: None,
            project_path: None,
        });
        assert!(session_start.supports_response());

        let user_prompt = HookEvent::UserPromptSubmit(UserPromptSubmitData {
            session_id: None,
            transcript_path: None,
            cwd: None,
            hook_event_name: None,
            prompt: Some("test".to_string()),
        });
        assert!(user_prompt.supports_response());

        let stop = HookEvent::Stop(StopData {
            session_id: None,
            transcript_path: None,
            cwd: None,
            permission_mode: None,
            hook_event_name: None,
            stop_hook_active: None,
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
        let parsed: HookResponse = serde_json::from_str(&json).unwrap();
        assert!(parsed.additional_context.is_none());
    }

    #[test]
    fn test_notification_construction() {
        let data = NotificationData {
            session_id: Some("sess-notif-456".to_string()),
            transcript_path: None,
            cwd: None,
            hook_event_name: None,
            message: "Your build finished successfully".to_string(),
            notification_type: Some("idle_prompt".to_string()),
        };
        let event = HookEvent::Notification(data);

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("notification"));
        assert!(json.contains("Your build finished"));
        assert!(!event.supports_response());
    }

    #[test]
    fn test_permission_request_supports_response() {
        let event = HookEvent::PermissionRequest(PermissionRequestData {
            session_id: None,
            transcript_path: None,
            cwd: None,
            hook_event_name: None,
            tool_name: Some("Write".to_string()),
            tool_input: Some(json!({})),
        });
        assert!(event.supports_response());
    }

    #[test]
    fn test_fire_and_forget_hooks_no_response() {
        let notification = HookEvent::Notification(NotificationData {
            session_id: None,
            transcript_path: None,
            cwd: None,
            hook_event_name: None,
            message: "test".to_string(),
            notification_type: None,
        });
        assert!(!notification.supports_response());

        let subagent = HookEvent::SubagentStop(SubagentStopData {
            session_id: None,
            transcript_path: None,
            cwd: None,
            permission_mode: None,
            hook_event_name: None,
            stop_hook_active: None,
            agent_id: "sub-1".to_string(),
            agent_transcript_path: None,
        });
        assert!(!subagent.supports_response());

        let compact = HookEvent::PreCompact(PreCompactData {
            session_id: None,
            transcript_path: None,
            cwd: None,
            hook_event_name: None,
        });
        assert!(!compact.supports_response());

        let end = HookEvent::SessionEnd(SessionEndData {
            session_id: None,
            transcript_path: None,
            cwd: None,
            hook_event_name: None,
            reason: None,
        });
        assert!(!end.supports_response());
    }
}
