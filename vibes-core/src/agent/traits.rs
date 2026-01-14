//! Agent trait definitions
//!
//! The Agent trait is the primary abstraction for autonomous entities in vibes.

use async_trait::async_trait;

use super::task::{Task, TaskResult};
use super::types::{AgentContext, AgentId, AgentStatus, AgentType};
use crate::error::VibesResult;

/// Core trait for all agent implementations
///
/// Agents are autonomous entities that can execute tasks. They have:
/// - Identity (id, name, type)
/// - State (status, context)
/// - Lifecycle operations (run, pause, resume, cancel)
///
/// # Object Safety
///
/// This trait is designed to be object-safe, allowing `Box<dyn Agent>`.
///
/// # Example
///
/// ```ignore
/// use vibes_core::agent::{Agent, AgentId, AgentType, AgentStatus, AgentContext};
///
/// struct MyAgent {
///     id: AgentId,
///     name: String,
///     status: AgentStatus,
///     context: AgentContext,
/// }
///
/// #[async_trait]
/// impl Agent for MyAgent {
///     fn id(&self) -> AgentId { self.id }
///     fn name(&self) -> &str { &self.name }
///     // ... implement other methods
/// }
/// ```
#[async_trait]
pub trait Agent: Send + Sync {
    /// Unique identifier for this agent
    fn id(&self) -> AgentId;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Agent type classification
    fn agent_type(&self) -> AgentType;

    /// Current execution status
    fn status(&self) -> AgentStatus;

    /// Execution configuration
    fn context(&self) -> &AgentContext;

    /// Run a task to completion
    ///
    /// This is the main entry point for agent execution. The agent
    /// processes the task and returns the result.
    async fn run(&mut self, task: Task) -> VibesResult<TaskResult>;

    /// Pause execution (if supported)
    ///
    /// Not all agents support pausing. Returns an error if not supported.
    async fn pause(&mut self) -> VibesResult<()>;

    /// Resume from paused state
    ///
    /// Returns an error if the agent is not paused or doesn't support resume.
    async fn resume(&mut self) -> VibesResult<()>;

    /// Cancel current task
    ///
    /// Signals the agent to stop execution as soon as possible.
    async fn cancel(&mut self) -> VibesResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that the Agent trait is object-safe
    fn _assert_object_safe(_: Box<dyn Agent>) {}
}
