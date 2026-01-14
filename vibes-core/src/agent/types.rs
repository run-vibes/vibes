//! Agent type definitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;
use uuid::Uuid;

/// Unique identifier for an agent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

/// Unique identifier for a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

/// Model identifier
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelId(pub String);

/// Tool identifier
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolId(pub String);

/// Agent type classification
///
/// Determines how an agent is spawned and managed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    /// User-triggered, interactive agent (e.g., Claude Code session)
    AdHoc,
    /// Long-running, autonomous agent (e.g., scheduled tasks)
    Background,
    /// Spawned by another agent for subtasks
    Subagent,
    /// Real-time user collaboration (e.g., pair programming)
    Interactive,
}

/// Agent execution status
///
/// Tracks the lifecycle state of an agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum AgentStatus {
    /// Ready to accept tasks
    #[serde(rename = "idle")]
    #[default]
    Idle,
    /// Actively executing a task
    #[serde(rename = "running")]
    Running {
        task: TaskId,
        started: DateTime<Utc>,
    },
    /// Execution paused
    #[serde(rename = "paused")]
    Paused { task: TaskId, reason: String },
    /// Waiting for user input
    #[serde(rename = "waiting_for_input")]
    WaitingForInput { prompt: String },
    /// Task failed
    #[serde(rename = "failed")]
    Failed { error: String },
}

/// Where an agent executes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ExecutionLocation {
    /// Same machine as vibes server
    #[default]
    Local,
    /// Remote vibes instance
    Remote { endpoint: Url },
}

/// Resource limits for agent execution
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum tokens to use
    pub max_tokens: Option<u64>,
    /// Maximum execution duration
    #[serde(with = "option_duration_serde")]
    pub max_duration: Option<Duration>,
    /// Maximum number of tool calls
    pub max_tool_calls: Option<u32>,
}

/// Agent permissions (stub - will be expanded)
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Permissions {
    /// Allow file system access
    pub filesystem: bool,
    /// Allow network access
    pub network: bool,
    /// Allow shell execution
    pub shell: bool,
}

/// Agent execution context
///
/// Configures how an agent executes tasks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentContext {
    /// Where the agent executes
    pub location: ExecutionLocation,
    /// Model to use for inference
    pub model: ModelId,
    /// Available tools
    pub tools: Vec<ToolId>,
    /// Permission boundaries
    pub permissions: Permissions,
    /// Resource limits
    pub resource_limits: ResourceLimits,
}

impl Default for AgentContext {
    fn default() -> Self {
        Self {
            location: ExecutionLocation::default(),
            model: ModelId("claude-sonnet-4-20250514".to_string()),
            tools: Vec::new(),
            permissions: Permissions::default(),
            resource_limits: ResourceLimits::default(),
        }
    }
}

/// Serde helper for Option<Duration>
mod option_duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    #[derive(Serialize, Deserialize)]
    struct DurationRepr {
        secs: u64,
        nanos: u32,
    }

    pub fn serialize<S>(duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match duration {
            Some(d) => {
                let repr = DurationRepr {
                    secs: d.as_secs(),
                    nanos: d.subsec_nanos(),
                };
                Some(repr).serialize(serializer)
            }
            None => None::<DurationRepr>.serialize(serializer),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<DurationRepr> = Option::deserialize(deserializer)?;
        Ok(opt.map(|repr| Duration::new(repr.secs, repr.nanos)))
    }
}

impl AgentId {
    /// Create a new agent ID using UUID v7 (time-ordered)
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TaskId {
    /// Create a new task ID using UUID v7 (time-ordered)
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn agent_id_is_unique() {
        let id1 = AgentId::new();
        let id2 = AgentId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn agent_id_display() {
        let id = AgentId::new();
        let display = format!("{}", id);
        assert!(!display.is_empty());
    }

    #[test]
    fn agent_type_serialization_roundtrip() {
        let types = [
            AgentType::AdHoc,
            AgentType::Background,
            AgentType::Subagent,
            AgentType::Interactive,
        ];

        for agent_type in types {
            let json = serde_json::to_string(&agent_type).unwrap();
            let deserialized: AgentType = serde_json::from_str(&json).unwrap();
            assert_eq!(agent_type, deserialized);
        }
    }

    #[test]
    fn agent_type_json_format() {
        assert_eq!(
            serde_json::to_string(&AgentType::AdHoc).unwrap(),
            "\"AdHoc\""
        );
        assert_eq!(
            serde_json::to_string(&AgentType::Background).unwrap(),
            "\"Background\""
        );
        assert_eq!(
            serde_json::to_string(&AgentType::Subagent).unwrap(),
            "\"Subagent\""
        );
        assert_eq!(
            serde_json::to_string(&AgentType::Interactive).unwrap(),
            "\"Interactive\""
        );
    }

    #[test]
    fn agent_status_default() {
        assert_eq!(AgentStatus::default(), AgentStatus::Idle);
    }

    // ===== NEW TESTS FOR m38-feat-03 =====

    #[test]
    fn task_id_is_unique() {
        let id1 = TaskId::new();
        let id2 = TaskId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn agent_status_running_has_task_and_started() {
        let task_id = TaskId::new();
        let started = Utc::now();
        let status = AgentStatus::Running {
            task: task_id,
            started,
        };

        match status {
            AgentStatus::Running { task, started: s } => {
                assert_eq!(task, task_id);
                assert_eq!(s, started);
            }
            _ => panic!("Expected Running variant"),
        }
    }

    #[test]
    fn agent_status_paused_has_task_and_reason() {
        let task_id = TaskId::new();
        let reason = "User requested pause".to_string();
        let status = AgentStatus::Paused {
            task: task_id,
            reason: reason.clone(),
        };

        match status {
            AgentStatus::Paused { task, reason: r } => {
                assert_eq!(task, task_id);
                assert_eq!(r, reason);
            }
            _ => panic!("Expected Paused variant"),
        }
    }

    #[test]
    fn agent_status_waiting_for_input_has_prompt() {
        let prompt = "Please confirm the action".to_string();
        let status = AgentStatus::WaitingForInput {
            prompt: prompt.clone(),
        };

        match status {
            AgentStatus::WaitingForInput { prompt: p } => {
                assert_eq!(p, prompt);
            }
            _ => panic!("Expected WaitingForInput variant"),
        }
    }

    #[test]
    fn agent_status_failed_has_error() {
        let error = "Connection timeout".to_string();
        let status = AgentStatus::Failed {
            error: error.clone(),
        };

        match status {
            AgentStatus::Failed { error: e } => {
                assert_eq!(e, error);
            }
            _ => panic!("Expected Failed variant"),
        }
    }

    #[test]
    fn agent_status_serialization_roundtrip() {
        let statuses = [
            AgentStatus::Idle,
            AgentStatus::Running {
                task: TaskId::new(),
                started: Utc::now(),
            },
            AgentStatus::Paused {
                task: TaskId::new(),
                reason: "test".to_string(),
            },
            AgentStatus::WaitingForInput {
                prompt: "test?".to_string(),
            },
            AgentStatus::Failed {
                error: "oops".to_string(),
            },
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: AgentStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, deserialized);
        }
    }

    #[test]
    fn execution_location_local() {
        let loc = ExecutionLocation::Local;
        assert_eq!(loc, ExecutionLocation::Local);
    }

    #[test]
    fn execution_location_remote_has_endpoint() {
        let endpoint = url::Url::parse("https://remote.vibes.dev").unwrap();
        let loc = ExecutionLocation::Remote {
            endpoint: endpoint.clone(),
        };

        match loc {
            ExecutionLocation::Remote { endpoint: e } => {
                assert_eq!(e, endpoint);
            }
            _ => panic!("Expected Remote variant"),
        }
    }

    #[test]
    fn execution_location_serialization_roundtrip() {
        let locations = [
            ExecutionLocation::Local,
            ExecutionLocation::Remote {
                endpoint: url::Url::parse("https://remote.vibes.dev").unwrap(),
            },
        ];

        for loc in locations {
            let json = serde_json::to_string(&loc).unwrap();
            let deserialized: ExecutionLocation = serde_json::from_str(&json).unwrap();
            assert_eq!(loc, deserialized);
        }
    }

    #[test]
    fn resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_tokens, None);
        assert_eq!(limits.max_duration, None);
        assert_eq!(limits.max_tool_calls, None);
    }

    #[test]
    fn resource_limits_serialization_roundtrip() {
        let limits = ResourceLimits {
            max_tokens: Some(100_000),
            max_duration: Some(std::time::Duration::from_secs(3600)),
            max_tool_calls: Some(50),
        };

        let json = serde_json::to_string(&limits).unwrap();
        let deserialized: ResourceLimits = serde_json::from_str(&json).unwrap();
        assert_eq!(limits, deserialized);
    }

    #[test]
    fn agent_context_has_required_fields() {
        let ctx = AgentContext {
            location: ExecutionLocation::Local,
            model: ModelId("claude-3-opus".to_string()),
            tools: vec![ToolId("bash".to_string()), ToolId("read".to_string())],
            permissions: Permissions::default(),
            resource_limits: ResourceLimits::default(),
        };

        assert_eq!(ctx.location, ExecutionLocation::Local);
        assert_eq!(ctx.model.0, "claude-3-opus");
        assert_eq!(ctx.tools.len(), 2);
    }

    #[test]
    fn agent_context_default() {
        let ctx = AgentContext::default();
        assert_eq!(ctx.location, ExecutionLocation::Local);
        assert!(ctx.tools.is_empty());
    }

    #[test]
    fn agent_context_serialization_roundtrip() {
        let ctx = AgentContext {
            location: ExecutionLocation::Local,
            model: ModelId("claude-3-opus".to_string()),
            tools: vec![ToolId("bash".to_string())],
            permissions: Permissions::default(),
            resource_limits: ResourceLimits {
                max_tokens: Some(50_000),
                max_duration: None,
                max_tool_calls: Some(100),
            },
        };

        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: AgentContext = serde_json::from_str(&json).unwrap();
        assert_eq!(ctx, deserialized);
    }
}
