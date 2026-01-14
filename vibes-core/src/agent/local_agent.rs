//! Local agent implementation for testing
//!
//! A minimal agent implementation that runs locally, useful for tests.

use async_trait::async_trait;
use tracing::instrument;

use super::task::{Task, TaskResult, TaskStatus};
use super::traits::Agent;
use super::types::{AgentContext, AgentId, AgentStatus, AgentType};
use crate::error::VibesResult;

/// A minimal local agent implementation for testing
pub struct LocalAgent {
    id: AgentId,
    name: String,
    agent_type: AgentType,
    status: AgentStatus,
    context: AgentContext,
}

impl LocalAgent {
    /// Create a new local agent with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: AgentId::new(),
            name: name.into(),
            agent_type: AgentType::AdHoc,
            status: AgentStatus::Idle,
            context: AgentContext::default(),
        }
    }

    /// Set the agent type
    pub fn with_type(mut self, agent_type: AgentType) -> Self {
        self.agent_type = agent_type;
        self
    }

    /// Set the agent context
    pub fn with_context(mut self, context: AgentContext) -> Self {
        self.context = context;
        self
    }
}

#[async_trait]
impl Agent for LocalAgent {
    fn id(&self) -> AgentId {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn agent_type(&self) -> AgentType {
        self.agent_type
    }

    fn status(&self) -> AgentStatus {
        self.status.clone()
    }

    fn context(&self) -> &AgentContext {
        &self.context
    }

    #[instrument(name = "agent::run", skip(self, task), fields(agent_id = %self.id, task_id = %task.id))]
    async fn run(&mut self, task: Task) -> VibesResult<TaskResult> {
        use chrono::Utc;
        use std::time::Duration;

        self.status = AgentStatus::Running {
            task: task.id,
            started: Utc::now(),
        };

        // Stub: immediately complete with no output
        self.status = AgentStatus::Idle;

        Ok(TaskResult {
            task_id: task.id,
            status: TaskStatus::Completed,
            output: None,
            artifacts: vec![],
            metrics: super::task::TaskMetrics {
                duration: Duration::ZERO,
                tokens_used: 0,
                tool_calls: 0,
                iterations: 0,
            },
        })
    }

    #[instrument(name = "agent::pause", skip(self), fields(agent_id = %self.id))]
    async fn pause(&mut self) -> VibesResult<()> {
        if let AgentStatus::Running { task, .. } = &self.status {
            self.status = AgentStatus::Paused {
                task: *task,
                reason: "User requested pause".to_string(),
            };
        }
        Ok(())
    }

    #[instrument(name = "agent::resume", skip(self), fields(agent_id = %self.id))]
    async fn resume(&mut self) -> VibesResult<()> {
        if let AgentStatus::Paused { task, .. } = &self.status {
            self.status = AgentStatus::Running {
                task: *task,
                started: chrono::Utc::now(),
            };
        }
        Ok(())
    }

    #[instrument(name = "agent::cancel", skip(self), fields(agent_id = %self.id))]
    async fn cancel(&mut self) -> VibesResult<()> {
        self.status = AgentStatus::Idle;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_agent_new_has_required_fields() {
        let agent = LocalAgent::new("test-agent");

        assert!(!agent.name().is_empty());
        assert_eq!(agent.agent_type(), AgentType::AdHoc);
        assert_eq!(agent.status(), AgentStatus::Idle);
    }

    #[test]
    fn local_agent_has_unique_id() {
        let agent1 = LocalAgent::new("agent-1");
        let agent2 = LocalAgent::new("agent-2");

        assert_ne!(agent1.id(), agent2.id());
    }

    #[test]
    fn local_agent_can_set_type() {
        let agent = LocalAgent::new("background-agent").with_type(AgentType::Background);

        assert_eq!(agent.agent_type(), AgentType::Background);
    }

    #[test]
    fn local_agent_can_set_context() {
        let ctx = AgentContext::default();
        let agent = LocalAgent::new("agent").with_context(ctx.clone());

        assert_eq!(*agent.context(), ctx);
    }
}
