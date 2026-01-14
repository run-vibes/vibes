//! Agent system for vibes
//!
//! This module provides the foundation for agent orchestration:
//! - Agent trait and lifecycle management
//! - Agent types (Ad-hoc, Background, Subagent, Interactive)
//! - Task system with metrics

pub mod local_agent;
pub mod registry;
pub mod task;
pub mod traits;
pub mod types;

pub use local_agent::LocalAgent;
pub use registry::{AgentRegistry, AgentStatusVariant};
pub use task::{
    Artifact, ArtifactType, Task, TaskBuilder, TaskConstraints, TaskContext, TaskMetrics,
    TaskResult, TaskStatus,
};
pub use traits::Agent;
pub use types::{
    AgentContext, AgentId, AgentStatus, AgentType, ExecutionLocation, ModelId, Permissions,
    ResourceLimits, TaskId, ToolId,
};
