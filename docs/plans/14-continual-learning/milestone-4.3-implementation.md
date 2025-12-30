# Milestone 4.3: Capture & Inject Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the end-to-end learning pipeline that captures session signals, extracts learnings, and injects them into future Claude Code sessions.

**Architecture:** Hook events flow through HookReceiver → EventBus (new Hook variant) → SessionCollector (buffers per session) → on Stop: TranscriptParser + LearningExtractor → LearningStore. Injection flows from LearningStore → LearningFormatter → three channels (CLAUDE.md @import, SessionStart hook, UserPromptSubmit hook).

**Tech Stack:** Rust, tokio async, serde_json, regex, dirs crate for cross-platform paths

---

## Phase 1: Core Infrastructure

### Task 1.1: Add VibesEvent::Hook Variant

**Files:**
- Modify: `vibes-core/src/events/types.rs`
- Modify: `vibes-core/src/hooks/mod.rs` (re-export HookEvent)

**Step 1: Write the failing test**

Add to `vibes-core/src/events/types.rs` in the tests module:

```rust
#[test]
fn vibes_event_hook_serialization_roundtrip() {
    use crate::hooks::HookEvent;
    use crate::hooks::PreToolUseData;

    let hook_event = HookEvent::PreToolUse(PreToolUseData {
        tool_name: "Bash".to_string(),
        input: r#"{"command": "ls"}"#.to_string(),
        session_id: Some("sess-123".to_string()),
    });

    let event = VibesEvent::Hook {
        session_id: Some("sess-123".to_string()),
        event: hook_event,
    };

    let json = serde_json::to_string(&event).unwrap();
    let parsed: VibesEvent = serde_json::from_str(&json).unwrap();

    assert!(matches!(parsed, VibesEvent::Hook { session_id: Some(id), .. } if id == "sess-123"));
}

#[test]
fn vibes_event_hook_session_id_extraction() {
    use crate::hooks::HookEvent;
    use crate::hooks::StopData;

    let hook_event = HookEvent::Stop(StopData {
        transcript_path: None,
        reason: Some("user".to_string()),
        session_id: Some("sess-456".to_string()),
    });

    let event = VibesEvent::Hook {
        session_id: Some("sess-456".to_string()),
        event: hook_event,
    };

    assert_eq!(event.session_id(), Some("sess-456"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-core vibes_event_hook`
Expected: FAIL with "no variant named `Hook`"

**Step 3: Write minimal implementation**

First, update `vibes-core/src/hooks/mod.rs` to re-export HookEvent:

```rust
// Add to existing re-exports
pub use types::HookEvent;
```

Then add to `VibesEvent` enum in `vibes-core/src/events/types.rs`:

```rust
use crate::hooks::HookEvent;

// Add to VibesEvent enum:
    /// Hook event from Claude Code
    Hook {
        session_id: Option<String>,
        event: HookEvent,
    },
```

Update `session_id()` method:

```rust
impl VibesEvent {
    pub fn session_id(&self) -> Option<&str> {
        match self {
            // ... existing arms ...
            VibesEvent::Hook { session_id, .. } => session_id.as_deref(),
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p vibes-core vibes_event_hook`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/events/types.rs vibes-core/src/hooks/mod.rs
git commit -m "feat(events): add VibesEvent::Hook variant for unified hook event stream"
```

---

### Task 1.2: Add SessionStart and UserPromptSubmit Hook Types

**Files:**
- Modify: `vibes-core/src/hooks/types.rs`

**Step 1: Write the failing test**

Add to `vibes-core/src/hooks/types.rs` tests:

```rust
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-core test_session_start`
Expected: FAIL with "cannot find type `SessionStartData`"

**Step 3: Write minimal implementation**

Add to `vibes-core/src/hooks/types.rs`:

```rust
/// Type of hook event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookType {
    PreToolUse,
    PostToolUse,
    Stop,
    SessionStart,
    UserPromptSubmit,
}

/// Data from a SessionStart hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStartData {
    /// Optional session ID
    pub session_id: Option<String>,
    /// Project path where session started
    pub project_path: Option<String>,
}

/// Data from a UserPromptSubmit hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPromptSubmitData {
    /// Optional session ID
    pub session_id: Option<String>,
    /// The prompt being submitted
    pub prompt: String,
}

/// A hook event received from Claude Code
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HookEvent {
    PreToolUse(PreToolUseData),
    PostToolUse(PostToolUseData),
    Stop(StopData),
    SessionStart(SessionStartData),
    UserPromptSubmit(UserPromptSubmitData),
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
        }
    }

    /// Whether this hook type supports returning a response
    pub fn supports_response(&self) -> bool {
        matches!(
            self,
            HookEvent::SessionStart(_) | HookEvent::UserPromptSubmit(_)
        )
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p vibes-core test_session_start test_user_prompt test_hook_supports`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/hooks/types.rs
git commit -m "feat(hooks): add SessionStart and UserPromptSubmit hook types"
```

---

### Task 1.3: Add Hook Response Type

**Files:**
- Modify: `vibes-core/src/hooks/types.rs`
- Modify: `vibes-core/src/hooks/mod.rs`

**Step 1: Write the failing test**

Add to `vibes-core/src/hooks/types.rs` tests:

```rust
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-core test_hook_response`
Expected: FAIL with "cannot find type `HookResponse`"

**Step 3: Write minimal implementation**

Add to `vibes-core/src/hooks/types.rs`:

```rust
/// Response to send back to Claude Code for injection hooks
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
```

Update `vibes-core/src/hooks/mod.rs` to export:

```rust
pub use types::{HookEvent, HookResponse, HookType};
pub use types::{PreToolUseData, PostToolUseData, StopData, SessionStartData, UserPromptSubmitData};
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p vibes-core test_hook_response`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/hooks/types.rs vibes-core/src/hooks/mod.rs
git commit -m "feat(hooks): add HookResponse type for injection hooks"
```

---

### Task 1.4: Extend HookInstaller for SessionStart and UserPromptSubmit

**Files:**
- Create: `vibes-core/src/hooks/scripts/session-start.sh`
- Create: `vibes-core/src/hooks/scripts/user-prompt-submit.sh`
- Modify: `vibes-core/src/hooks/scripts.rs`
- Modify: `vibes-core/src/hooks/installer.rs`

**Step 1: Create new hook scripts**

Create `vibes-core/src/hooks/scripts/session-start.sh`:

```bash
#!/bin/bash
# Session start hook - calls vibes daemon for context injection
# Receives: session_id, cwd

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/vibes-hook-send.sh"

# Build JSON payload
JSON=$(cat <<EOF
{
    "type": "session_start",
    "session_id": "${CLAUDE_SESSION_ID:-}",
    "project_path": "${PWD}"
}
EOF
)

# Send to vibes and capture response
RESPONSE=$(send_hook "$JSON")

# Output response for Claude to use
if [ -n "$RESPONSE" ]; then
    echo "$RESPONSE"
fi
```

Create `vibes-core/src/hooks/scripts/user-prompt-submit.sh`:

```bash
#!/bin/bash
# User prompt submit hook - calls vibes daemon for context injection
# Receives: session_id, prompt content via stdin

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/vibes-hook-send.sh"

# Read prompt from stdin
PROMPT=$(cat)

# Build JSON payload (escape the prompt)
ESCAPED_PROMPT=$(echo "$PROMPT" | jq -Rs .)

JSON=$(cat <<EOF
{
    "type": "user_prompt_submit",
    "session_id": "${CLAUDE_SESSION_ID:-}",
    "prompt": $ESCAPED_PROMPT
}
EOF
)

# Send to vibes and capture response
RESPONSE=$(send_hook "$JSON")

# Output response for Claude to use
if [ -n "$RESPONSE" ]; then
    echo "$RESPONSE"
fi
```

**Step 2: Update scripts.rs**

Modify `vibes-core/src/hooks/scripts.rs`:

```rust
//! Embedded hook scripts

/// Pre-tool-use hook script
pub const PRE_TOOL_USE: &str = include_str!("scripts/pre-tool-use.sh");

/// Post-tool-use hook script
pub const POST_TOOL_USE: &str = include_str!("scripts/post-tool-use.sh");

/// Stop hook script
pub const STOP: &str = include_str!("scripts/stop.sh");

/// Session start hook script
pub const SESSION_START: &str = include_str!("scripts/session-start.sh");

/// User prompt submit hook script
pub const USER_PROMPT_SUBMIT: &str = include_str!("scripts/user-prompt-submit.sh");

/// Helper script for sending hook data to vibes
pub const VIBES_HOOK_SEND: &str = include_str!("scripts/vibes-hook-send.sh");

/// All scripts with their target filenames
pub const SCRIPTS: &[(&str, &str)] = &[
    ("pre-tool-use.sh", PRE_TOOL_USE),
    ("post-tool-use.sh", POST_TOOL_USE),
    ("stop.sh", STOP),
    ("session-start.sh", SESSION_START),
    ("user-prompt-submit.sh", USER_PROMPT_SUBMIT),
    ("vibes-hook-send.sh", VIBES_HOOK_SEND),
];
```

**Step 3: Update installer.rs**

In `vibes-core/src/hooks/installer.rs`, update the `vibes_hooks` array in `update_settings`:

```rust
// Hook configurations to add
let vibes_hooks = [
    ("PreToolUse", "pre-tool-use.sh"),
    ("PostToolUse", "post-tool-use.sh"),
    ("Stop", "stop.sh"),
    ("SessionStart", "session-start.sh"),
    ("UserPromptSubmit", "user-prompt-submit.sh"),
];
```

**Step 4: Write tests and run**

Add test to `vibes-core/src/hooks/scripts.rs`:

```rust
#[test]
fn test_script_count() {
    assert_eq!(SCRIPTS.len(), 6); // Updated from 4
}

#[test]
fn test_session_start_script_has_content() {
    assert!(SESSION_START.len() > 50, "session-start.sh should have content");
    assert!(SESSION_START.starts_with("#!/bin/bash"));
}

#[test]
fn test_user_prompt_submit_script_has_content() {
    assert!(USER_PROMPT_SUBMIT.len() > 50, "user-prompt-submit.sh should have content");
    assert!(USER_PROMPT_SUBMIT.starts_with("#!/bin/bash"));
}
```

Update test in `vibes-core/src/hooks/installer.rs`:

```rust
#[test]
fn test_update_settings_creates_new() {
    // ... existing setup ...

    let hooks = settings.get("hooks").unwrap().as_array().unwrap();
    assert_eq!(hooks.len(), 5); // Updated from 3: PreToolUse, PostToolUse, Stop, SessionStart, UserPromptSubmit
}
```

Run: `cargo test -p vibes-core hooks`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-core/src/hooks/scripts/ vibes-core/src/hooks/scripts.rs vibes-core/src/hooks/installer.rs
git commit -m "feat(hooks): add SessionStart and UserPromptSubmit hook scripts"
```

---

### Task 1.5: Create GroovePaths for Cross-Platform File Locations

**Files:**
- Create: `vibes-groove/src/paths.rs`
- Modify: `vibes-groove/src/lib.rs`

**Step 1: Write the failing test**

Create `vibes-groove/src/paths.rs`:

```rust
//! Cross-platform path management for groove

use std::path::PathBuf;

use crate::Scope;

/// Manages file paths for groove data across platforms
#[derive(Debug, Clone)]
pub struct GroovePaths {
    /// User data directory (platform-specific)
    user_data_dir: PathBuf,
    /// Current project root (if in a project)
    project_root: Option<PathBuf>,
}

impl GroovePaths {
    /// Create paths with auto-detected directories
    pub fn new(project_root: Option<PathBuf>) -> Option<Self> {
        let user_data_dir = dirs::data_dir()?.join("vibes").join("plugins").join("groove");
        Some(Self {
            user_data_dir,
            project_root,
        })
    }

    /// Create paths with explicit directories (for testing)
    pub fn with_dirs(user_data_dir: PathBuf, project_root: Option<PathBuf>) -> Self {
        Self {
            user_data_dir,
            project_root,
        }
    }

    /// Get the learnings.md file path for a scope
    pub fn learnings_file(&self, scope: &Scope) -> PathBuf {
        match scope {
            Scope::User => self.user_data_dir.join("learnings.md"),
            Scope::Project { path } => path.join(".vibes").join("plugins").join("groove").join("learnings.md"),
            Scope::Enterprise { org_id } => self.user_data_dir.join("orgs").join(org_id).join("learnings.md"),
        }
    }

    /// Get the user data directory
    pub fn user_data_dir(&self) -> &PathBuf {
        &self.user_data_dir
    }

    /// Get the project root, if set
    pub fn project_root(&self) -> Option<&PathBuf> {
        self.project_root.as_ref()
    }

    /// Get the project learnings path (convenience method)
    pub fn project_learnings(&self) -> Option<PathBuf> {
        self.project_root.as_ref().map(|root| {
            root.join(".vibes").join("plugins").join("groove").join("learnings.md")
        })
    }

    /// Get the user learnings path (convenience method)
    pub fn user_learnings(&self) -> PathBuf {
        self.user_data_dir.join("learnings.md")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_learnings_file_user_scope() {
        let paths = GroovePaths::with_dirs(
            PathBuf::from("/home/user/.local/share/vibes/plugins/groove"),
            None,
        );

        let result = paths.learnings_file(&Scope::User);
        assert_eq!(
            result,
            Path::new("/home/user/.local/share/vibes/plugins/groove/learnings.md")
        );
    }

    #[test]
    fn test_learnings_file_project_scope() {
        let paths = GroovePaths::with_dirs(
            PathBuf::from("/home/user/.local/share/vibes/plugins/groove"),
            Some(PathBuf::from("/home/user/my-project")),
        );

        let result = paths.learnings_file(&Scope::Project {
            path: PathBuf::from("/home/user/my-project"),
        });
        assert_eq!(
            result,
            Path::new("/home/user/my-project/.vibes/plugins/groove/learnings.md")
        );
    }

    #[test]
    fn test_learnings_file_enterprise_scope() {
        let paths = GroovePaths::with_dirs(
            PathBuf::from("/home/user/.local/share/vibes/plugins/groove"),
            None,
        );

        let result = paths.learnings_file(&Scope::Enterprise {
            org_id: "acme-corp".to_string(),
        });
        assert_eq!(
            result,
            Path::new("/home/user/.local/share/vibes/plugins/groove/orgs/acme-corp/learnings.md")
        );
    }

    #[test]
    fn test_user_learnings_convenience() {
        let paths = GroovePaths::with_dirs(
            PathBuf::from("/data/groove"),
            None,
        );

        assert_eq!(paths.user_learnings(), Path::new("/data/groove/learnings.md"));
    }

    #[test]
    fn test_project_learnings_convenience() {
        let paths = GroovePaths::with_dirs(
            PathBuf::from("/data/groove"),
            Some(PathBuf::from("/project")),
        );

        assert_eq!(
            paths.project_learnings(),
            Some(PathBuf::from("/project/.vibes/plugins/groove/learnings.md"))
        );
    }

    #[test]
    fn test_project_learnings_none_when_no_project() {
        let paths = GroovePaths::with_dirs(
            PathBuf::from("/data/groove"),
            None,
        );

        assert!(paths.project_learnings().is_none());
    }
}
```

**Step 2: Run test to verify it fails initially**

Run: `cargo test -p vibes-groove paths`
Expected: FAIL (module doesn't exist)

**Step 3: Add module to lib.rs**

Add to `vibes-groove/src/lib.rs`:

```rust
pub mod paths;
pub use paths::GroovePaths;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p vibes-groove paths`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-groove/src/paths.rs vibes-groove/src/lib.rs
git commit -m "feat(groove): add GroovePaths for cross-platform file locations"
```

---

## Phase 2: Capture Pipeline

### Task 2.1: Create SessionCollector

**Files:**
- Create: `vibes-groove/src/capture/mod.rs`
- Create: `vibes-groove/src/capture/collector.rs`
- Modify: `vibes-groove/src/lib.rs`

**Step 1: Write the failing test**

Create `vibes-groove/src/capture/collector.rs`:

```rust
//! Session event collection and buffering

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

use crate::store::LearningStore;

/// Events collected for a session
#[derive(Debug, Clone)]
pub struct ToolEvent {
    pub tool_name: String,
    pub input: String,
    pub output: Option<String>,
    pub success: bool,
    pub timestamp: DateTime<Utc>,
}

/// Buffer holding events for an active session
#[derive(Debug)]
pub struct SessionBuffer {
    pub session_id: String,
    pub project_path: Option<PathBuf>,
    pub tool_events: Vec<ToolEvent>,
    pub start_time: DateTime<Utc>,
}

impl SessionBuffer {
    pub fn new(session_id: String, project_path: Option<PathBuf>) -> Self {
        Self {
            session_id,
            project_path,
            tool_events: Vec::new(),
            start_time: Utc::now(),
        }
    }
}

/// Collects and buffers session events, processes on session end
pub struct SessionCollector {
    /// Active session buffers
    sessions: Arc<RwLock<HashMap<String, SessionBuffer>>>,
    /// Learning storage
    store: Arc<dyn LearningStore>,
}

impl SessionCollector {
    /// Create a new session collector
    pub fn new(store: Arc<dyn LearningStore>) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            store,
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
            buffer.tool_events.push(event);
        }
    }

    /// End a session and trigger processing
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::CozoStore;
    use tempfile::TempDir;

    async fn create_test_collector() -> (SessionCollector, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let store = CozoStore::open(temp_dir.path().join("test.db")).unwrap();
        let collector = SessionCollector::new(Arc::new(store));
        (collector, temp_dir)
    }

    #[tokio::test]
    async fn test_start_session() {
        let (collector, _tmp) = create_test_collector().await;

        collector.start_session("sess-1".to_string(), None).await;

        assert!(collector.has_session("sess-1").await);
        assert_eq!(collector.active_session_count().await, 1);
    }

    #[tokio::test]
    async fn test_record_tool_event() {
        let (collector, _tmp) = create_test_collector().await;

        collector.start_session("sess-1".to_string(), None).await;
        collector.record_tool_event("sess-1", ToolEvent {
            tool_name: "Bash".to_string(),
            input: "ls".to_string(),
            output: Some("file.txt".to_string()),
            success: true,
            timestamp: Utc::now(),
        }).await;

        let buffer = collector.end_session("sess-1").await.unwrap();
        assert_eq!(buffer.tool_events.len(), 1);
        assert_eq!(buffer.tool_events[0].tool_name, "Bash");
    }

    #[tokio::test]
    async fn test_end_session_removes_buffer() {
        let (collector, _tmp) = create_test_collector().await;

        collector.start_session("sess-1".to_string(), None).await;
        let buffer = collector.end_session("sess-1").await;

        assert!(buffer.is_some());
        assert!(!collector.has_session("sess-1").await);
    }

    #[tokio::test]
    async fn test_concurrent_sessions() {
        let (collector, _tmp) = create_test_collector().await;

        collector.start_session("sess-1".to_string(), Some(PathBuf::from("/project1"))).await;
        collector.start_session("sess-2".to_string(), Some(PathBuf::from("/project2"))).await;

        assert_eq!(collector.active_session_count().await, 2);

        collector.record_tool_event("sess-1", ToolEvent {
            tool_name: "Read".to_string(),
            input: "file.rs".to_string(),
            output: None,
            success: true,
            timestamp: Utc::now(),
        }).await;

        collector.record_tool_event("sess-2", ToolEvent {
            tool_name: "Write".to_string(),
            input: "file.py".to_string(),
            output: None,
            success: true,
            timestamp: Utc::now(),
        }).await;

        let buffer1 = collector.end_session("sess-1").await.unwrap();
        let buffer2 = collector.end_session("sess-2").await.unwrap();

        assert_eq!(buffer1.tool_events[0].tool_name, "Read");
        assert_eq!(buffer2.tool_events[0].tool_name, "Write");
    }

    #[tokio::test]
    async fn test_record_event_for_unknown_session_is_noop() {
        let (collector, _tmp) = create_test_collector().await;

        // Should not panic
        collector.record_tool_event("unknown", ToolEvent {
            tool_name: "Bash".to_string(),
            input: "ls".to_string(),
            output: None,
            success: true,
            timestamp: Utc::now(),
        }).await;
    }
}
```

**Step 2: Create module file**

Create `vibes-groove/src/capture/mod.rs`:

```rust
//! Capture pipeline for session events

mod collector;

pub use collector::{SessionBuffer, SessionCollector, ToolEvent};
```

**Step 3: Add to lib.rs**

```rust
pub mod capture;
pub use capture::{SessionBuffer, SessionCollector, ToolEvent};
```

**Step 4: Run tests**

Run: `cargo test -p vibes-groove capture`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-groove/src/capture/
git commit -m "feat(groove): add SessionCollector for buffering session events"
```

---

### Task 2.2: Create TranscriptParser

**Files:**
- Create: `vibes-groove/src/capture/parser.rs`
- Modify: `vibes-groove/src/capture/mod.rs`

**Step 1: Write the failing test**

Create `vibes-groove/src/capture/parser.rs`:

```rust
//! Claude Code transcript JSONL parser

use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::GrooveError;

/// A message from a transcript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptMessage {
    pub role: String,
    pub content: String,
    pub timestamp: Option<DateTime<Utc>>,
}

/// A tool use from a transcript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptToolUse {
    pub tool_name: String,
    pub input: Value,
    pub output: Option<String>,
    pub success: bool,
}

/// Metadata about a transcript
#[derive(Debug, Clone, Default)]
pub struct TranscriptMetadata {
    pub total_messages: usize,
    pub user_messages: usize,
    pub assistant_messages: usize,
    pub tool_uses: usize,
}

/// Parsed transcript data
#[derive(Debug, Clone)]
pub struct ParsedTranscript {
    pub session_id: String,
    pub messages: Vec<TranscriptMessage>,
    pub tool_uses: Vec<TranscriptToolUse>,
    pub metadata: TranscriptMetadata,
}

/// Parser for Claude Code JSONL transcripts
pub struct TranscriptParser {
    /// Supported transcript versions
    supported_versions: Vec<String>,
}

impl Default for TranscriptParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TranscriptParser {
    pub fn new() -> Self {
        Self {
            supported_versions: vec!["1".to_string()],
        }
    }

    /// Parse a transcript from a file path
    pub fn parse_file(&self, path: &Path) -> Result<ParsedTranscript, GrooveError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| GrooveError::Io(e.to_string()))?;
        self.parse(&content, path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown"))
    }

    /// Parse transcript content
    pub fn parse(&self, content: &str, session_id: &str) -> Result<ParsedTranscript, GrooveError> {
        let mut messages = Vec::new();
        let mut tool_uses = Vec::new();
        let mut user_count = 0;
        let mut assistant_count = 0;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            match serde_json::from_str::<Value>(line) {
                Ok(value) => {
                    if let Some(role) = value.get("role").and_then(|r| r.as_str()) {
                        let content_text = value.get("content")
                            .map(|c| {
                                if let Some(s) = c.as_str() {
                                    s.to_string()
                                } else {
                                    c.to_string()
                                }
                            })
                            .unwrap_or_default();

                        match role {
                            "user" => user_count += 1,
                            "assistant" => assistant_count += 1,
                            _ => {}
                        }

                        messages.push(TranscriptMessage {
                            role: role.to_string(),
                            content: content_text,
                            timestamp: None,
                        });
                    }

                    // Check for tool use
                    if let Some(tool_name) = value.get("tool_name").and_then(|t| t.as_str()) {
                        tool_uses.push(TranscriptToolUse {
                            tool_name: tool_name.to_string(),
                            input: value.get("input").cloned().unwrap_or(Value::Null),
                            output: value.get("output").and_then(|o| o.as_str()).map(String::from),
                            success: value.get("success").and_then(|s| s.as_bool()).unwrap_or(true),
                        });
                    }
                }
                Err(_) => {
                    // Skip malformed lines silently
                    continue;
                }
            }
        }

        let metadata = TranscriptMetadata {
            total_messages: messages.len(),
            user_messages: user_count,
            assistant_messages: assistant_count,
            tool_uses: tool_uses.len(),
        };

        Ok(ParsedTranscript {
            session_id: session_id.to_string(),
            messages,
            tool_uses,
            metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_jsonl() {
        let content = r#"{"role": "user", "content": "Hello"}
{"role": "assistant", "content": "Hi there!"}
{"role": "user", "content": "Help me code"}"#;

        let parser = TranscriptParser::new();
        let result = parser.parse(content, "test-session").unwrap();

        assert_eq!(result.session_id, "test-session");
        assert_eq!(result.messages.len(), 3);
        assert_eq!(result.metadata.user_messages, 2);
        assert_eq!(result.metadata.assistant_messages, 1);
    }

    #[test]
    fn test_parse_extracts_tool_uses() {
        let content = r#"{"role": "user", "content": "Run ls"}
{"tool_name": "Bash", "input": {"command": "ls"}, "output": "file.txt", "success": true}
{"role": "assistant", "content": "Done"}"#;

        let parser = TranscriptParser::new();
        let result = parser.parse(content, "test").unwrap();

        assert_eq!(result.tool_uses.len(), 1);
        assert_eq!(result.tool_uses[0].tool_name, "Bash");
        assert!(result.tool_uses[0].success);
    }

    #[test]
    fn test_parse_handles_malformed_lines() {
        let content = r#"{"role": "user", "content": "Hello"}
this is not valid json
{"role": "assistant", "content": "Hi"}"#;

        let parser = TranscriptParser::new();
        let result = parser.parse(content, "test").unwrap();

        // Should skip the malformed line
        assert_eq!(result.messages.len(), 2);
    }

    #[test]
    fn test_parse_empty_content() {
        let parser = TranscriptParser::new();
        let result = parser.parse("", "test").unwrap();

        assert_eq!(result.messages.len(), 0);
        assert_eq!(result.metadata.total_messages, 0);
    }

    #[test]
    fn test_parse_whitespace_lines() {
        let content = r#"
{"role": "user", "content": "Hello"}


{"role": "assistant", "content": "Hi"}
"#;

        let parser = TranscriptParser::new();
        let result = parser.parse(content, "test").unwrap();

        assert_eq!(result.messages.len(), 2);
    }
}
```

**Step 2: Update mod.rs**

```rust
mod collector;
mod parser;

pub use collector::{SessionBuffer, SessionCollector, ToolEvent};
pub use parser::{ParsedTranscript, TranscriptMessage, TranscriptMetadata, TranscriptParser, TranscriptToolUse};
```

**Step 3: Run tests**

Run: `cargo test -p vibes-groove parser`
Expected: PASS

**Step 4: Commit**

```bash
git add vibes-groove/src/capture/parser.rs vibes-groove/src/capture/mod.rs
git commit -m "feat(groove): add TranscriptParser for JSONL transcript parsing"
```

---

### Task 2.3: Create LearningExtractor

**Files:**
- Create: `vibes-groove/src/capture/extractor.rs`
- Modify: `vibes-groove/src/capture/mod.rs`

**Step 1: Write the failing test**

Create `vibes-groove/src/capture/extractor.rs`:

```rust
//! Learning extraction from parsed transcripts

use regex::Regex;

use crate::{Learning, LearningCategory, LearningContent, LearningSource, Scope};

use super::{ParsedTranscript, TranscriptMessage};

/// A signal detected in transcript content
#[derive(Debug, Clone)]
pub enum Signal {
    /// Explicit preference statement ("always use X", "prefer Y")
    ExplicitPreference {
        statement: String,
        message_index: usize,
    },
    /// Positive feedback ("perfect", "exactly", "great")
    PositiveFeedback {
        context: String,
        message_index: usize,
    },
    /// Tool pattern (consistent tool usage)
    ToolPattern {
        tool_name: String,
        success_rate: f64,
    },
}

impl Signal {
    /// Compute confidence score for this signal
    pub fn confidence(&self) -> f64 {
        match self {
            Signal::ExplicitPreference { .. } => 0.85,
            Signal::PositiveFeedback { .. } => 0.70,
            Signal::ToolPattern { success_rate, .. } => *success_rate,
        }
    }
}

/// Extracts learnings from parsed transcripts
pub struct LearningExtractor {
    /// Pattern for explicit preferences
    preference_pattern: Regex,
    /// Pattern for positive feedback
    feedback_pattern: Regex,
}

impl Default for LearningExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl LearningExtractor {
    pub fn new() -> Self {
        Self {
            preference_pattern: Regex::new(r"(?i)\b(always|never|prefer|use)\s+([^.!?\n]{5,50})").unwrap(),
            feedback_pattern: Regex::new(r"(?i)^(perfect|exactly|great|thanks|awesome|wonderful)[\s!.,]*$").unwrap(),
        }
    }

    /// Extract signals from a parsed transcript
    pub fn extract_signals(&self, transcript: &ParsedTranscript) -> Vec<Signal> {
        let mut signals = Vec::new();

        // Extract from user messages
        for (idx, msg) in transcript.messages.iter().enumerate() {
            if msg.role == "user" {
                signals.extend(self.extract_from_message(msg, idx));
            }
        }

        // Extract tool patterns
        signals.extend(self.extract_tool_patterns(transcript));

        signals
    }

    /// Extract signals from a single message
    fn extract_from_message(&self, msg: &TranscriptMessage, index: usize) -> Vec<Signal> {
        let mut signals = Vec::new();

        // Check for explicit preferences
        for cap in self.preference_pattern.captures_iter(&msg.content) {
            if let Some(statement) = cap.get(0) {
                signals.push(Signal::ExplicitPreference {
                    statement: statement.as_str().to_string(),
                    message_index: index,
                });
            }
        }

        // Check for positive feedback
        if self.feedback_pattern.is_match(msg.content.trim()) {
            signals.push(Signal::PositiveFeedback {
                context: msg.content.clone(),
                message_index: index,
            });
        }

        signals
    }

    /// Extract tool usage patterns
    fn extract_tool_patterns(&self, transcript: &ParsedTranscript) -> Vec<Signal> {
        use std::collections::HashMap;

        let mut tool_stats: HashMap<String, (usize, usize)> = HashMap::new(); // (success, total)

        for tool_use in &transcript.tool_uses {
            let entry = tool_stats.entry(tool_use.tool_name.clone()).or_insert((0, 0));
            if tool_use.success {
                entry.0 += 1;
            }
            entry.1 += 1;
        }

        tool_stats
            .into_iter()
            .filter(|(_, (success, total))| *total >= 3 && *success as f64 / *total as f64 >= 0.8)
            .map(|(name, (success, total))| Signal::ToolPattern {
                tool_name: name,
                success_rate: success as f64 / total as f64,
            })
            .collect()
    }

    /// Convert signals to Learning objects
    pub fn signals_to_learnings(
        &self,
        signals: &[Signal],
        session_id: &str,
        scope: Scope,
    ) -> Vec<Learning> {
        signals
            .iter()
            .map(|signal| {
                let (category, content, source) = match signal {
                    Signal::ExplicitPreference { statement, message_index } => (
                        LearningCategory::Preference,
                        LearningContent {
                            description: format!("User expressed preference: {}", statement),
                            pattern: None,
                            insight: statement.clone(),
                        },
                        LearningSource::Transcript {
                            session_id: session_id.to_string(),
                            message_index: *message_index,
                        },
                    ),
                    Signal::PositiveFeedback { context, message_index } => (
                        LearningCategory::Preference,
                        LearningContent {
                            description: format!("Positive feedback: {}", context),
                            pattern: None,
                            insight: format!("This approach was well-received: {}", context),
                        },
                        LearningSource::Transcript {
                            session_id: session_id.to_string(),
                            message_index: *message_index,
                        },
                    ),
                    Signal::ToolPattern { tool_name, success_rate } => (
                        LearningCategory::ToolUsage,
                        LearningContent {
                            description: format!("Tool {} used successfully {:.0}% of the time", tool_name, success_rate * 100.0),
                            pattern: Some(serde_json::json!({
                                "tool": tool_name,
                                "success_rate": success_rate,
                            })),
                            insight: format!("User prefers using {} tool", tool_name),
                        },
                        LearningSource::Transcript {
                            session_id: session_id.to_string(),
                            message_index: 0,
                        },
                    ),
                };

                let mut learning = Learning::new(scope.clone(), category, content, source);
                learning.confidence = signal.confidence();
                learning
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::TranscriptToolUse;

    fn make_transcript(messages: Vec<(&str, &str)>, tool_uses: Vec<TranscriptToolUse>) -> ParsedTranscript {
        ParsedTranscript {
            session_id: "test".to_string(),
            messages: messages
                .into_iter()
                .map(|(role, content)| TranscriptMessage {
                    role: role.to_string(),
                    content: content.to_string(),
                    timestamp: None,
                })
                .collect(),
            tool_uses,
            metadata: Default::default(),
        }
    }

    #[test]
    fn test_extract_always_use_preference() {
        let transcript = make_transcript(
            vec![("user", "always use pytest instead of unittest")],
            vec![],
        );

        let extractor = LearningExtractor::new();
        let signals = extractor.extract_signals(&transcript);

        assert_eq!(signals.len(), 1);
        assert!(matches!(&signals[0], Signal::ExplicitPreference { statement, .. } if statement.contains("pytest")));
    }

    #[test]
    fn test_extract_prefer_statement() {
        let transcript = make_transcript(
            vec![("user", "I prefer using Rust over Python")],
            vec![],
        );

        let extractor = LearningExtractor::new();
        let signals = extractor.extract_signals(&transcript);

        assert_eq!(signals.len(), 1);
        assert!(matches!(&signals[0], Signal::ExplicitPreference { statement, .. } if statement.contains("Rust")));
    }

    #[test]
    fn test_extract_positive_feedback() {
        let transcript = make_transcript(
            vec![
                ("assistant", "Here's the code"),
                ("user", "Perfect!"),
            ],
            vec![],
        );

        let extractor = LearningExtractor::new();
        let signals = extractor.extract_signals(&transcript);

        assert_eq!(signals.len(), 1);
        assert!(matches!(&signals[0], Signal::PositiveFeedback { .. }));
    }

    #[test]
    fn test_extract_tool_pattern() {
        let transcript = make_transcript(
            vec![],
            vec![
                TranscriptToolUse { tool_name: "Bash".into(), input: serde_json::json!({}), output: None, success: true },
                TranscriptToolUse { tool_name: "Bash".into(), input: serde_json::json!({}), output: None, success: true },
                TranscriptToolUse { tool_name: "Bash".into(), input: serde_json::json!({}), output: None, success: true },
                TranscriptToolUse { tool_name: "Bash".into(), input: serde_json::json!({}), output: None, success: true },
            ],
        );

        let extractor = LearningExtractor::new();
        let signals = extractor.extract_signals(&transcript);

        assert_eq!(signals.len(), 1);
        assert!(matches!(&signals[0], Signal::ToolPattern { tool_name, success_rate } if tool_name == "Bash" && *success_rate == 1.0));
    }

    #[test]
    fn test_tool_pattern_requires_threshold() {
        // Only 2 uses - not enough
        let transcript = make_transcript(
            vec![],
            vec![
                TranscriptToolUse { tool_name: "Bash".into(), input: serde_json::json!({}), output: None, success: true },
                TranscriptToolUse { tool_name: "Bash".into(), input: serde_json::json!({}), output: None, success: true },
            ],
        );

        let extractor = LearningExtractor::new();
        let signals = extractor.extract_signals(&transcript);

        assert!(signals.is_empty());
    }

    #[test]
    fn test_confidence_scores() {
        let pref = Signal::ExplicitPreference { statement: "test".into(), message_index: 0 };
        assert_eq!(pref.confidence(), 0.85);

        let feedback = Signal::PositiveFeedback { context: "great".into(), message_index: 0 };
        assert_eq!(feedback.confidence(), 0.70);

        let tool = Signal::ToolPattern { tool_name: "Bash".into(), success_rate: 0.9 };
        assert_eq!(tool.confidence(), 0.9);
    }

    #[test]
    fn test_signals_to_learnings() {
        let signals = vec![
            Signal::ExplicitPreference {
                statement: "always use pytest".into(),
                message_index: 0,
            },
        ];

        let extractor = LearningExtractor::new();
        let learnings = extractor.signals_to_learnings(&signals, "sess-1", Scope::User);

        assert_eq!(learnings.len(), 1);
        assert_eq!(learnings[0].category, LearningCategory::Preference);
        assert_eq!(learnings[0].confidence, 0.85);
        assert!(learnings[0].content.insight.contains("pytest"));
    }
}
```

**Step 2: Update mod.rs**

```rust
mod collector;
mod extractor;
mod parser;

pub use collector::{SessionBuffer, SessionCollector, ToolEvent};
pub use extractor::{LearningExtractor, Signal};
pub use parser::{ParsedTranscript, TranscriptMessage, TranscriptMetadata, TranscriptParser, TranscriptToolUse};
```

**Step 3: Run tests**

Run: `cargo test -p vibes-groove extractor`
Expected: PASS

**Step 4: Commit**

```bash
git add vibes-groove/src/capture/extractor.rs vibes-groove/src/capture/mod.rs
git commit -m "feat(groove): add LearningExtractor for MVP pattern extraction"
```

---

## Phase 3: Injection Pipeline

### Task 3.1: Create LearningFormatter

**Files:**
- Create: `vibes-groove/src/inject/mod.rs`
- Create: `vibes-groove/src/inject/formatter.rs`
- Modify: `vibes-groove/src/lib.rs`

**Step 1: Write the failing test**

Create `vibes-groove/src/inject/formatter.rs`:

```rust
//! Learning formatter with HTML comment markers

use regex::Regex;

use crate::{Learning, LearningCategory, Scope};

/// Formats learnings with HTML comment markers for injection
pub struct LearningFormatter {
    /// Pattern to parse existing markers
    marker_pattern: Regex,
}

impl Default for LearningFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl LearningFormatter {
    pub fn new() -> Self {
        Self {
            marker_pattern: Regex::new(
                r"<!--\s*groove:([A-Za-z0-9]+)\s+confidence:([0-9.]+)\s+scope:(\w+)\s+category:(\w+)\s*-->\n([\s\S]*?)<!--\s*/groove:\1\s*-->"
            ).unwrap(),
        }
    }

    /// Format a single learning with markers
    pub fn format_one(&self, learning: &Learning) -> String {
        let id = learning.id.to_string();
        // Use first 11 chars for brevity (like base32 representation)
        let short_id = &id[..11.min(id.len())];

        format!(
            "<!-- groove:{} confidence:{:.2} scope:{} category:{} -->\n{}\n<!-- /groove:{} -->",
            short_id,
            learning.confidence,
            self.scope_str(&learning.scope),
            learning.category.as_str(),
            learning.content.insight,
            short_id,
        )
    }

    /// Format multiple learnings, grouped by category
    pub fn format_all(&self, learnings: &[Learning]) -> String {
        if learnings.is_empty() {
            return String::new();
        }

        let mut output = String::from("## groove Learnings\n\n");

        // Group by category
        let mut by_category: std::collections::HashMap<&LearningCategory, Vec<&Learning>> =
            std::collections::HashMap::new();

        for learning in learnings {
            by_category.entry(&learning.category).or_default().push(learning);
        }

        // Sort categories for consistent output
        let mut categories: Vec<_> = by_category.keys().collect();
        categories.sort_by_key(|c| c.as_str());

        for category in categories {
            let items = &by_category[category];
            output.push_str(&format!("### {}\n\n", self.category_title(category)));

            for learning in *items {
                output.push_str(&self.format_one(learning));
                output.push_str("\n\n");
            }
        }

        output.trim_end().to_string()
    }

    /// Parse learnings from formatted content (for reading back)
    pub fn parse(&self, content: &str) -> Vec<ParsedMarker> {
        self.marker_pattern
            .captures_iter(content)
            .filter_map(|cap| {
                Some(ParsedMarker {
                    id: cap.get(1)?.as_str().to_string(),
                    confidence: cap.get(2)?.as_str().parse().ok()?,
                    scope: cap.get(3)?.as_str().to_string(),
                    category: cap.get(4)?.as_str().to_string(),
                    insight: cap.get(5)?.as_str().trim().to_string(),
                })
            })
            .collect()
    }

    fn scope_str(&self, scope: &Scope) -> &str {
        match scope {
            Scope::User => "user",
            Scope::Project { .. } => "project",
            Scope::Enterprise { .. } => "enterprise",
        }
    }

    fn category_title(&self, category: &LearningCategory) -> &str {
        match category {
            LearningCategory::CodePattern => "Code Patterns",
            LearningCategory::Preference => "Preferences",
            LearningCategory::Solution => "Solutions",
            LearningCategory::ErrorRecovery => "Error Recovery",
            LearningCategory::ToolUsage => "Tool Usage",
            LearningCategory::HarnessKnowledge => "Harness Knowledge",
        }
    }
}

/// A marker parsed from formatted content
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedMarker {
    pub id: String,
    pub confidence: f64,
    pub scope: String,
    pub category: String,
    pub insight: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LearningContent, LearningSource};

    fn make_learning(insight: &str, category: LearningCategory) -> Learning {
        Learning::new(
            Scope::User,
            category,
            LearningContent {
                description: "Test".into(),
                pattern: None,
                insight: insight.into(),
            },
            LearningSource::UserCreated,
        )
    }

    #[test]
    fn test_format_one_produces_correct_markers() {
        let learning = make_learning("Always use pytest", LearningCategory::Preference);
        let formatter = LearningFormatter::new();

        let result = formatter.format_one(&learning);

        assert!(result.starts_with("<!-- groove:"));
        assert!(result.contains("confidence:0.50"));
        assert!(result.contains("scope:user"));
        assert!(result.contains("category:preference"));
        assert!(result.contains("Always use pytest"));
        assert!(result.ends_with("-->"));
    }

    #[test]
    fn test_format_all_groups_by_category() {
        let learnings = vec![
            make_learning("Use pytest", LearningCategory::Preference),
            make_learning("Handle errors", LearningCategory::ErrorRecovery),
            make_learning("Prefer rust", LearningCategory::Preference),
        ];
        let formatter = LearningFormatter::new();

        let result = formatter.format_all(&learnings);

        assert!(result.starts_with("## groove Learnings"));
        assert!(result.contains("### Preferences"));
        assert!(result.contains("### Error Recovery"));
        // Preferences should appear before Error Recovery (alphabetical by category key)
        let pref_pos = result.find("### Error Recovery").unwrap();
        let err_pos = result.find("### Preferences").unwrap();
        assert!(err_pos < pref_pos || pref_pos < err_pos); // Just verify both exist
    }

    #[test]
    fn test_parse_extracts_markers() {
        let content = r#"## groove Learnings

### Preferences

<!-- groove:abc12345678 confidence:0.85 scope:user category:preference -->
Always use pytest
<!-- /groove:abc12345678 -->

<!-- groove:def12345678 confidence:0.70 scope:project category:tool_usage -->
Use cargo watch
<!-- /groove:def12345678 -->
"#;

        let formatter = LearningFormatter::new();
        let markers = formatter.parse(content);

        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0].id, "abc12345678");
        assert_eq!(markers[0].confidence, 0.85);
        assert_eq!(markers[0].insight, "Always use pytest");
        assert_eq!(markers[1].id, "def12345678");
        assert_eq!(markers[1].category, "tool_usage");
    }

    #[test]
    fn test_format_and_parse_roundtrip() {
        let learning = make_learning("Test insight", LearningCategory::Preference);
        let formatter = LearningFormatter::new();

        let formatted = formatter.format_one(&learning);
        let parsed = formatter.parse(&formatted);

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].insight, "Test insight");
        assert_eq!(parsed[0].scope, "user");
        assert_eq!(parsed[0].category, "preference");
    }

    #[test]
    fn test_format_all_empty() {
        let formatter = LearningFormatter::new();
        let result = formatter.format_all(&[]);
        assert!(result.is_empty());
    }
}
```

**Step 2: Create mod.rs**

Create `vibes-groove/src/inject/mod.rs`:

```rust
//! Injection pipeline for learnings

mod formatter;

pub use formatter::{LearningFormatter, ParsedMarker};
```

**Step 3: Update lib.rs**

```rust
pub mod inject;
pub use inject::{LearningFormatter, ParsedMarker};
```

**Step 4: Run tests**

Run: `cargo test -p vibes-groove formatter`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-groove/src/inject/
git commit -m "feat(groove): add LearningFormatter with HTML comment markers"
```

---

### Task 3.2: Create ClaudeCodeInjector

**Files:**
- Create: `vibes-groove/src/inject/claude_code.rs`
- Modify: `vibes-groove/src/inject/mod.rs`

**Step 1: Write the failing test**

Create `vibes-groove/src/inject/claude_code.rs`:

```rust
//! Claude Code injector - syncs learnings to learnings.md

use std::fs;
use std::path::Path;

use crate::{GrooveError, Learning};

use super::LearningFormatter;

/// Syncs learnings to the Claude Code environment
pub struct ClaudeCodeInjector {
    formatter: LearningFormatter,
}

impl Default for ClaudeCodeInjector {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaudeCodeInjector {
    pub fn new() -> Self {
        Self {
            formatter: LearningFormatter::new(),
        }
    }

    /// Sync learnings to a learnings.md file
    pub fn sync_to_file(&self, learnings: &[Learning], path: &Path) -> Result<(), GrooveError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| GrooveError::Io(format!("Failed to create directory: {}", e)))?;
        }

        let content = self.formatter.format_all(learnings);
        fs::write(path, content)
            .map_err(|e| GrooveError::Io(format!("Failed to write learnings file: {}", e)))?;

        Ok(())
    }

    /// Read existing learnings from a file
    pub fn read_from_file(&self, path: &Path) -> Result<Vec<super::ParsedMarker>, GrooveError> {
        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(path)
            .map_err(|e| GrooveError::Io(format!("Failed to read learnings file: {}", e)))?;

        Ok(self.formatter.parse(&content))
    }

    /// Generate hook response with formatted learnings
    pub fn generate_hook_response(&self, learnings: &[Learning]) -> Option<String> {
        if learnings.is_empty() {
            return None;
        }
        Some(self.formatter.format_all(learnings))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LearningCategory, LearningContent, LearningSource, Scope};
    use tempfile::TempDir;

    fn make_learning(insight: &str) -> Learning {
        Learning::new(
            Scope::User,
            LearningCategory::Preference,
            LearningContent {
                description: "Test".into(),
                pattern: None,
                insight: insight.into(),
            },
            LearningSource::UserCreated,
        )
    }

    #[test]
    fn test_sync_to_file_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("learnings.md");

        let injector = ClaudeCodeInjector::new();
        let learnings = vec![make_learning("Use pytest")];

        injector.sync_to_file(&learnings, &path).unwrap();

        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("pytest"));
    }

    #[test]
    fn test_sync_to_file_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("nested").join("dir").join("learnings.md");

        let injector = ClaudeCodeInjector::new();
        let learnings = vec![make_learning("Test")];

        injector.sync_to_file(&learnings, &path).unwrap();

        assert!(path.exists());
    }

    #[test]
    fn test_sync_preserves_on_update() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("learnings.md");
        let injector = ClaudeCodeInjector::new();

        // First sync
        let learnings1 = vec![make_learning("First insight")];
        injector.sync_to_file(&learnings1, &path).unwrap();

        // Second sync (overwrites)
        let learnings2 = vec![
            make_learning("First insight"),
            make_learning("Second insight"),
        ];
        injector.sync_to_file(&learnings2, &path).unwrap();

        let markers = injector.read_from_file(&path).unwrap();
        assert_eq!(markers.len(), 2);
    }

    #[test]
    fn test_read_from_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("nonexistent.md");

        let injector = ClaudeCodeInjector::new();
        let markers = injector.read_from_file(&path).unwrap();

        assert!(markers.is_empty());
    }

    #[test]
    fn test_generate_hook_response() {
        let injector = ClaudeCodeInjector::new();
        let learnings = vec![make_learning("Use pytest")];

        let response = injector.generate_hook_response(&learnings);

        assert!(response.is_some());
        let content = response.unwrap();
        assert!(content.contains("groove Learnings"));
        assert!(content.contains("pytest"));
    }

    #[test]
    fn test_generate_hook_response_empty() {
        let injector = ClaudeCodeInjector::new();
        let response = injector.generate_hook_response(&[]);

        assert!(response.is_none());
    }
}
```

**Step 2: Update mod.rs**

```rust
mod claude_code;
mod formatter;

pub use claude_code::ClaudeCodeInjector;
pub use formatter::{LearningFormatter, ParsedMarker};
```

**Step 3: Update lib.rs exports**

```rust
pub use inject::{ClaudeCodeInjector, LearningFormatter, ParsedMarker};
```

**Step 4: Run tests**

Run: `cargo test -p vibes-groove claude_code`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-groove/src/inject/claude_code.rs vibes-groove/src/inject/mod.rs
git commit -m "feat(groove): add ClaudeCodeInjector for learnings.md sync"
```

---

## Phase 4: Setup & CLI

### Task 4.1: Add groove init Command

**Files:**
- Modify: `vibes-groove/src/plugin.rs`

**Step 1: Add command registration**

Add to `register_commands` in `vibes-groove/src/plugin.rs`:

```rust
// init
ctx.register_command(CommandSpec {
    path: vec!["init".into()],
    description: "Initialize groove for the current project".into(),
    args: vec![ArgSpec {
        name: "user".into(),
        description: "Initialize for user scope instead of project".into(),
        required: false,
    }],
})?;
```

**Step 2: Add command handler**

Add to `handle_command` match:

```rust
["init"] => self.cmd_init(args),
```

**Step 3: Implement handler**

```rust
fn cmd_init(&self, args: &vibes_plugin_api::CommandArgs) -> Result<CommandOutput, PluginError> {
    use crate::{GroovePaths, Scope};
    use std::fs;

    let is_user_scope = args.args.iter().any(|a| a == "--user" || a == "-u");

    let paths = GroovePaths::new(if is_user_scope {
        None
    } else {
        Some(std::env::current_dir().unwrap_or_default())
    }).ok_or_else(|| PluginError::Config("Cannot determine data directory".into()))?;

    let learnings_path = if is_user_scope {
        paths.user_learnings()
    } else {
        paths.project_learnings()
            .ok_or_else(|| PluginError::Config("Not in a project directory".into()))?
    };

    // Create directory and empty learnings file
    if let Some(parent) = learnings_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| PluginError::Io(format!("Failed to create directory: {}", e)))?;
    }

    if !learnings_path.exists() {
        fs::write(&learnings_path, "")
            .map_err(|e| PluginError::Io(format!("Failed to create learnings file: {}", e)))?;
    }

    let scope_str = if is_user_scope { "user" } else { "project" };
    let mut output = String::new();
    output.push_str(&format!("Initialized groove for {} scope\n", scope_str));
    output.push_str(&format!("Learnings file: {}\n\n", learnings_path.display()));

    if !is_user_scope {
        output.push_str("To enable injection, add to your CLAUDE.md:\n");
        output.push_str("  @.vibes/plugins/groove/learnings.md\n");
    }

    Ok(CommandOutput::Text(output))
}
```

**Step 4: Add test**

```rust
#[test]
fn test_cmd_init_registers() {
    let mut plugin = GroovePlugin;
    let mut ctx = create_test_context();
    plugin.on_load(&mut ctx).unwrap();

    let commands = ctx.pending_commands();
    let paths: Vec<_> = commands.iter().map(|c| c.path.join(" ")).collect();
    assert!(paths.contains(&"init".to_string()));
}
```

**Step 5: Run tests and commit**

Run: `cargo test -p vibes-groove plugin`
Expected: PASS

```bash
git add vibes-groove/src/plugin.rs
git commit -m "feat(groove): add init command for project/user setup"
```

---

### Task 4.2: Add groove list Command

**Files:**
- Modify: `vibes-groove/src/plugin.rs`

**Step 1: Add command registration**

```rust
// list
ctx.register_command(CommandSpec {
    path: vec!["list".into()],
    description: "List current learnings".into(),
    args: vec![],
})?;
```

**Step 2: Add handler**

```rust
["list"] => self.cmd_list(),

// Implementation:
fn cmd_list(&self) -> Result<CommandOutput, PluginError> {
    use crate::{ClaudeCodeInjector, GroovePaths};

    let paths = GroovePaths::new(Some(std::env::current_dir().unwrap_or_default()))
        .ok_or_else(|| PluginError::Config("Cannot determine data directory".into()))?;

    let injector = ClaudeCodeInjector::new();
    let mut output = String::new();

    // Check project learnings
    if let Some(project_path) = paths.project_learnings() {
        if project_path.exists() {
            let markers = injector.read_from_file(&project_path)
                .map_err(|e| PluginError::Io(e.to_string()))?;

            output.push_str(&format!("Project learnings ({}):\n", project_path.display()));
            if markers.is_empty() {
                output.push_str("  (none)\n");
            } else {
                for m in &markers {
                    output.push_str(&format!(
                        "  [{:.2}] {} - {}\n",
                        m.confidence,
                        m.category,
                        truncate(&m.insight, 50)
                    ));
                }
            }
            output.push('\n');
        }
    }

    // Check user learnings
    let user_path = paths.user_learnings();
    if user_path.exists() {
        let markers = injector.read_from_file(&user_path)
            .map_err(|e| PluginError::Io(e.to_string()))?;

        output.push_str(&format!("User learnings ({}):\n", user_path.display()));
        if markers.is_empty() {
            output.push_str("  (none)\n");
        } else {
            for m in &markers {
                output.push_str(&format!(
                    "  [{:.2}] {} - {}\n",
                    m.confidence,
                    m.category,
                    truncate(&m.insight, 50)
                ));
            }
        }
    }

    if output.is_empty() {
        output = "No learnings found. Run 'vibes groove init' to get started.\n".into();
    }

    Ok(CommandOutput::Text(output))
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
```

**Step 3: Run tests and commit**

```bash
git add vibes-groove/src/plugin.rs
git commit -m "feat(groove): add list command to show learnings"
```

---

### Task 4.3: Add groove status Command

**Files:**
- Modify: `vibes-groove/src/plugin.rs`

**Step 1: Add command registration and handler**

```rust
// status
ctx.register_command(CommandSpec {
    path: vec!["status".into()],
    description: "Show groove injection status".into(),
    args: vec![],
})?;

// Handler:
["status"] => self.cmd_status(),

fn cmd_status(&self) -> Result<CommandOutput, PluginError> {
    use crate::GroovePaths;

    let paths = GroovePaths::new(Some(std::env::current_dir().unwrap_or_default()))
        .ok_or_else(|| PluginError::Config("Cannot determine data directory".into()))?;

    let mut output = String::new();
    output.push_str("groove Status\n");
    output.push_str("=============\n\n");

    // Check injection channels
    output.push_str("Injection Channels:\n");

    // CLAUDE.md check
    let claude_md = std::env::current_dir()
        .ok()
        .map(|p| p.join("CLAUDE.md"));

    let has_import = claude_md.as_ref().map(|p| {
        std::fs::read_to_string(p)
            .map(|content| content.contains("@.vibes/plugins/groove"))
            .unwrap_or(false)
    }).unwrap_or(false);

    output.push_str(&format!(
        "  CLAUDE.md @import:  {}\n",
        if has_import { "Y Enabled" } else { "N Not configured" }
    ));

    // Hook check (simplified - would need to actually check settings.json)
    output.push_str("  SessionStart hook:   ? (check ~/.claude/settings.json)\n");
    output.push_str("  UserPromptSubmit:    ? (check ~/.claude/settings.json)\n\n");

    // Learnings files
    output.push_str("Learnings Files:\n");

    if let Some(project_path) = paths.project_learnings() {
        let exists = project_path.exists();
        output.push_str(&format!(
            "  Project: {} {}\n",
            if exists { "Y" } else { "N" },
            project_path.display()
        ));
    }

    let user_path = paths.user_learnings();
    let user_exists = user_path.exists();
    output.push_str(&format!(
        "  User:    {} {}\n",
        if user_exists { "Y" } else { "N" },
        user_path.display()
    ));

    Ok(CommandOutput::Text(output))
}
```

**Step 2: Run tests and commit**

```bash
git add vibes-groove/src/plugin.rs
git commit -m "feat(groove): add status command for injection overview"
```

---

## Phase 5: Integration & Documentation

### Task 5.1: Wire Plugin to EventBus for Hook Events

**Files:**
- Modify: `vibes-groove/src/plugin.rs`

This task integrates the plugin with the daemon's EventBus to receive hook events. The full implementation requires daemon-side changes that are beyond the scope of the plugin crate alone.

**Step 1: Add hook event handling stubs**

Add to `GroovePlugin`:

```rust
/// Handle incoming hook events (called by daemon when subscribed to EventBus)
pub fn handle_hook_event(&mut self, event: &crate::hooks::HookEvent) -> Option<String> {
    use crate::{ClaudeCodeInjector, GroovePaths, Scope};

    match event {
        crate::hooks::HookEvent::SessionStart(data) => {
            // Load and return learnings for injection
            let paths = GroovePaths::new(
                data.project_path.as_ref().map(std::path::PathBuf::from)
            )?;

            let injector = ClaudeCodeInjector::new();

            // Collect learnings from all scopes
            let mut all_learnings = Vec::new();

            // Read user learnings
            if let Ok(markers) = injector.read_from_file(&paths.user_learnings()) {
                // Convert markers back to learnings (simplified for now)
                // Full implementation would query LearningStore
            }

            // For now, return None until we have storage integration
            None
        }
        crate::hooks::HookEvent::UserPromptSubmit(data) => {
            // Could filter learnings by prompt relevance (future: semantic search)
            None
        }
        _ => None,
    }
}
```

**Step 2: Add imports**

```rust
// Note: Will need vibes-core dependency or shared types
```

**Step 3: Commit**

```bash
git add vibes-groove/src/plugin.rs
git commit -m "feat(groove): add hook event handling stubs for EventBus integration"
```

---

### Task 5.2: End-to-End Integration Test

**Files:**
- Create: `vibes-groove/tests/integration.rs`

**Step 1: Write integration test**

```rust
//! End-to-end integration tests for groove capture → store → inject pipeline

use std::sync::Arc;
use tempfile::TempDir;

use vibes_groove::{
    capture::{LearningExtractor, SessionCollector, ToolEvent, TranscriptParser},
    inject::ClaudeCodeInjector,
    store::CozoStore,
    Learning, LearningCategory, Scope,
};

/// Test the full pipeline: capture session → extract learnings → store → inject
#[tokio::test]
async fn test_capture_to_injection_pipeline() {
    let temp_dir = TempDir::new().unwrap();

    // 1. Create storage
    let store = Arc::new(CozoStore::open(temp_dir.path().join("test.db")).unwrap());

    // 2. Simulate transcript with a preference
    let transcript_content = r#"{"role": "user", "content": "always use pytest instead of unittest"}
{"role": "assistant", "content": "I'll use pytest for testing."}
{"tool_name": "Bash", "input": {"command": "pytest"}, "output": "1 passed", "success": true}
{"role": "user", "content": "Perfect!"}"#;

    // 3. Parse transcript
    let parser = TranscriptParser::new();
    let transcript = parser.parse(transcript_content, "test-session").unwrap();

    assert_eq!(transcript.messages.len(), 2);
    assert_eq!(transcript.tool_uses.len(), 1);

    // 4. Extract learnings
    let extractor = LearningExtractor::new();
    let signals = extractor.extract_signals(&transcript);

    // Should find: explicit preference + positive feedback
    assert!(signals.len() >= 2);

    // 5. Convert to learnings
    let learnings = extractor.signals_to_learnings(&signals, "test-session", Scope::User);
    assert!(!learnings.is_empty());

    // 6. Store learnings
    for learning in &learnings {
        store.store(learning).await.unwrap();
    }

    // 7. Verify storage
    let stored = store.find_by_scope(&Scope::User).await.unwrap();
    assert!(!stored.is_empty());

    // 8. Inject to file
    let injector = ClaudeCodeInjector::new();
    let learnings_path = temp_dir.path().join("learnings.md");
    injector.sync_to_file(&stored, &learnings_path).unwrap();

    // 9. Verify injection file
    let content = std::fs::read_to_string(&learnings_path).unwrap();
    assert!(content.contains("groove:"));
    assert!(content.contains("pytest"));

    // 10. Parse back and verify
    let markers = injector.read_from_file(&learnings_path).unwrap();
    assert!(!markers.is_empty());
    assert!(markers.iter().any(|m| m.insight.contains("pytest")));
}

#[tokio::test]
async fn test_preference_appears_in_hook_response() {
    let temp_dir = TempDir::new().unwrap();
    let store = Arc::new(CozoStore::open(temp_dir.path().join("test.db")).unwrap());

    // Create a learning
    let learning = Learning::new(
        Scope::User,
        LearningCategory::Preference,
        vibes_groove::LearningContent {
            description: "User prefers pytest".into(),
            pattern: None,
            insight: "Always use pytest for testing".into(),
        },
        vibes_groove::LearningSource::UserCreated,
    );
    store.store(&learning).await.unwrap();

    // Get learnings for hook response
    let learnings = store.find_by_scope(&Scope::User).await.unwrap();
    let injector = ClaudeCodeInjector::new();
    let response = injector.generate_hook_response(&learnings);

    assert!(response.is_some());
    let content = response.unwrap();
    assert!(content.contains("pytest"));
    assert!(content.contains("groove:"));
}
```

**Step 2: Run integration test**

Run: `cargo test -p vibes-groove --test integration`
Expected: PASS

**Step 3: Commit**

```bash
git add vibes-groove/tests/integration.rs
git commit -m "test(groove): add end-to-end integration test for capture-store-inject"
```

---

### Task 5.3: Update Documentation

**Files:**
- Modify: `vibes-groove/README.md`
- Modify: `docs/PROGRESS.md`

**Step 1: Update README with implementation status**

Update `vibes-groove/README.md` crate structure section to reflect actual implementation.

**Step 2: Update PROGRESS.md**

Mark milestone 4.3 tasks as complete.

**Step 3: Commit**

```bash
git add vibes-groove/README.md docs/PROGRESS.md
git commit -m "docs: update groove README and PROGRESS for milestone 4.3"
```

---

## Summary

| Phase | Tasks | Key Deliverables |
|-------|-------|-----------------|
| 1. Core Infrastructure | 1.1-1.5 | VibesEvent::Hook, new hook types, HookResponse, hook scripts, GroovePaths |
| 2. Capture Pipeline | 2.1-2.3 | SessionCollector, TranscriptParser, LearningExtractor |
| 3. Injection Pipeline | 3.1-3.2 | LearningFormatter, ClaudeCodeInjector |
| 4. Setup & CLI | 4.1-4.3 | init, list, status commands |
| 5. Integration | 5.1-5.3 | EventBus integration, E2E tests, docs |

**Total: ~20 tasks across 5 phases**

After completing all tasks, run the full test suite:

```bash
just test
just clippy
```

---

*Plan created: 2025-12-29*
