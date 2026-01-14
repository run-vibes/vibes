//! Agent type definitions

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for an agent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

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
/// Stub type - will be expanded in m38-feat-03.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AgentStatus {
    /// Agent is idle, waiting for work
    #[default]
    Idle,
    /// Agent is actively executing a task
    Running,
    /// Agent execution is paused
    Paused,
    /// Agent has completed all work
    Completed,
    /// Agent encountered an error
    Failed,
}

/// Agent execution context
///
/// Stub type - will be expanded in m38-feat-03.
#[derive(Debug, Clone, Default)]
pub struct AgentContext {
    /// Maximum execution time in seconds (0 = unlimited)
    pub timeout_secs: u64,
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
