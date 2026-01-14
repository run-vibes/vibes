//! Agent registry for managing agent instances
//!
//! The AgentRegistry is responsible for:
//! - Registering and tracking agent instances
//! - Providing lifecycle operations (spawn, pause, resume, cancel, stop)
//! - Querying agents by type and status

use std::collections::HashMap;

use super::task::{Task, TaskResult};
use super::traits::Agent;
use super::types::{AgentId, AgentStatus, AgentType};
use crate::error::{AgentError, VibesResult};

/// Variant discriminator for AgentStatus (for filtering)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatusVariant {
    Idle,
    Running,
    Paused,
    WaitingForInput,
    Failed,
}

impl From<&AgentStatus> for AgentStatusVariant {
    fn from(status: &AgentStatus) -> Self {
        match status {
            AgentStatus::Idle => AgentStatusVariant::Idle,
            AgentStatus::Running { .. } => AgentStatusVariant::Running,
            AgentStatus::Paused { .. } => AgentStatusVariant::Paused,
            AgentStatus::WaitingForInput { .. } => AgentStatusVariant::WaitingForInput,
            AgentStatus::Failed { .. } => AgentStatusVariant::Failed,
        }
    }
}

/// Registry for managing agent instances
pub struct AgentRegistry {
    agents: HashMap<AgentId, Box<dyn Agent>>,
}

impl AgentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    /// Register a new agent and return its ID
    pub fn register(&mut self, agent: Box<dyn Agent>) -> AgentId {
        let id = agent.id();
        self.agents.insert(id, agent);
        id
    }

    /// Get an agent by ID
    pub fn get(&self, id: AgentId) -> Option<&dyn Agent> {
        self.agents.get(&id).map(|a| a.as_ref())
    }

    /// Get a mutable reference to an agent by ID
    pub fn get_mut(&mut self, id: AgentId) -> Option<&mut Box<dyn Agent>> {
        self.agents.get_mut(&id)
    }

    /// Remove an agent from the registry
    pub fn remove(&mut self, id: AgentId) -> Option<Box<dyn Agent>> {
        self.agents.remove(&id)
    }

    /// List all agent IDs
    pub fn list(&self) -> Vec<AgentId> {
        self.agents.keys().copied().collect()
    }

    /// Get agents by type
    pub fn by_type(&self, agent_type: AgentType) -> Vec<AgentId> {
        self.agents
            .iter()
            .filter(|(_, agent)| agent.agent_type() == agent_type)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get agents by status variant
    pub fn by_status_variant(&self, variant: AgentStatusVariant) -> Vec<AgentId> {
        self.agents
            .iter()
            .filter(|(_, agent)| AgentStatusVariant::from(&agent.status()) == variant)
            .map(|(id, _)| *id)
            .collect()
    }

    // ===== Lifecycle Operations =====

    /// Run a task on an agent
    pub async fn run_task(&mut self, agent_id: AgentId, task: Task) -> VibesResult<TaskResult> {
        let agent = self
            .agents
            .get_mut(&agent_id)
            .ok_or_else(|| AgentError::NotFound(format!("Agent {} not found", agent_id)))?;

        agent.run(task).await
    }

    /// Pause an agent
    pub async fn pause(&mut self, agent_id: AgentId) -> VibesResult<()> {
        let agent = self
            .agents
            .get_mut(&agent_id)
            .ok_or_else(|| AgentError::NotFound(format!("Agent {} not found", agent_id)))?;

        agent.pause().await
    }

    /// Resume an agent
    pub async fn resume(&mut self, agent_id: AgentId) -> VibesResult<()> {
        let agent = self
            .agents
            .get_mut(&agent_id)
            .ok_or_else(|| AgentError::NotFound(format!("Agent {} not found", agent_id)))?;

        agent.resume().await
    }

    /// Cancel an agent's current task
    pub async fn cancel(&mut self, agent_id: AgentId) -> VibesResult<()> {
        let agent = self
            .agents
            .get_mut(&agent_id)
            .ok_or_else(|| AgentError::NotFound(format!("Agent {} not found", agent_id)))?;

        agent.cancel().await
    }

    /// Stop and remove an agent
    pub async fn stop(&mut self, agent_id: AgentId) -> VibesResult<()> {
        // First cancel any running work
        {
            let agent = self
                .agents
                .get_mut(&agent_id)
                .ok_or_else(|| AgentError::NotFound(format!("Agent {} not found", agent_id)))?;

            agent.cancel().await?;
        }

        // Then remove from registry
        self.agents.remove(&agent_id);
        Ok(())
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::local_agent::LocalAgent;
    use crate::agent::{Agent, AgentType};

    // ===== Basic CRUD Tests =====

    #[test]
    fn registry_new_is_empty() {
        let registry = AgentRegistry::new();
        assert!(registry.list().is_empty());
    }

    #[test]
    fn registry_register_returns_agent_id() {
        let mut registry = AgentRegistry::new();
        let agent = LocalAgent::new("test-agent");
        let expected_id = agent.id();

        let id = registry.register(Box::new(agent));

        assert_eq!(id, expected_id);
    }

    #[test]
    fn registry_get_returns_agent() {
        let mut registry = AgentRegistry::new();
        let agent = LocalAgent::new("test-agent");
        let id = registry.register(Box::new(agent));

        let retrieved = registry.get(id);

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "test-agent");
    }

    #[test]
    fn registry_get_returns_none_for_unknown_id() {
        let registry = AgentRegistry::new();
        let unknown_id = crate::agent::AgentId::new();

        assert!(registry.get(unknown_id).is_none());
    }

    #[test]
    fn registry_get_mut_returns_mutable_agent() {
        let mut registry = AgentRegistry::new();
        let agent = LocalAgent::new("original-name");
        let id = registry.register(Box::new(agent));

        let agent_mut = registry.get_mut(id);
        assert!(agent_mut.is_some());
    }

    #[test]
    fn registry_remove_returns_agent() {
        let mut registry = AgentRegistry::new();
        let agent = LocalAgent::new("to-remove");
        let id = registry.register(Box::new(agent));

        let removed = registry.remove(id);

        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name(), "to-remove");
        assert!(registry.get(id).is_none());
    }

    #[test]
    fn registry_remove_returns_none_for_unknown_id() {
        let mut registry = AgentRegistry::new();
        let unknown_id = crate::agent::AgentId::new();

        assert!(registry.remove(unknown_id).is_none());
    }

    #[test]
    fn registry_list_returns_all_agent_ids() {
        let mut registry = AgentRegistry::new();
        let id1 = registry.register(Box::new(LocalAgent::new("agent-1")));
        let id2 = registry.register(Box::new(LocalAgent::new("agent-2")));
        let id3 = registry.register(Box::new(LocalAgent::new("agent-3")));

        let ids = registry.list();

        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
        assert!(ids.contains(&id3));
    }

    // ===== Filter Tests =====

    #[test]
    fn registry_by_type_filters_correctly() {
        let mut registry = AgentRegistry::new();
        let adhoc = LocalAgent::new("adhoc").with_type(AgentType::AdHoc);
        let background = LocalAgent::new("background").with_type(AgentType::Background);
        let subagent = LocalAgent::new("subagent").with_type(AgentType::Subagent);

        let adhoc_id = registry.register(Box::new(adhoc));
        let _bg_id = registry.register(Box::new(background));
        let _sub_id = registry.register(Box::new(subagent));

        let adhoc_agents = registry.by_type(AgentType::AdHoc);

        assert_eq!(adhoc_agents.len(), 1);
        assert!(adhoc_agents.contains(&adhoc_id));
    }

    #[test]
    fn registry_by_status_filters_correctly() {
        let mut registry = AgentRegistry::new();

        // All agents start as Idle
        let id1 = registry.register(Box::new(LocalAgent::new("idle-agent")));
        let _id2 = registry.register(Box::new(LocalAgent::new("another-idle")));

        let idle_agents = registry.by_status_variant(AgentStatusVariant::Idle);

        assert_eq!(idle_agents.len(), 2);
        assert!(idle_agents.contains(&id1));
    }

    // ===== Lifecycle Operation Tests =====

    #[tokio::test]
    async fn registry_run_task_executes_on_agent() {
        use crate::agent::{Task, TaskStatus};

        let mut registry = AgentRegistry::new();
        let id = registry.register(Box::new(LocalAgent::new("worker")));

        let task = Task::new("Test task");
        let result = registry.run_task(id, task).await.unwrap();

        assert_eq!(result.status, TaskStatus::Completed);
    }

    #[tokio::test]
    async fn registry_run_task_returns_error_for_unknown_agent() {
        let mut registry = AgentRegistry::new();
        let unknown_id = crate::agent::AgentId::new();

        let task = crate::agent::Task::new("Test task");
        let result = registry.run_task(unknown_id, task).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn registry_pause_pauses_agent() {
        let mut registry = AgentRegistry::new();
        let id = registry.register(Box::new(LocalAgent::new("pausable")));

        // Agent needs to be running to pause
        let task = crate::agent::Task::new("Long task");
        // Start a task (LocalAgent completes immediately, but we can still test the API)
        let _ = registry.run_task(id, task).await;

        let result = registry.pause(id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn registry_pause_returns_error_for_unknown_agent() {
        let mut registry = AgentRegistry::new();
        let unknown_id = crate::agent::AgentId::new();

        let result = registry.pause(unknown_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn registry_resume_resumes_agent() {
        let mut registry = AgentRegistry::new();
        let id = registry.register(Box::new(LocalAgent::new("resumable")));

        let result = registry.resume(id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn registry_cancel_cancels_agent() {
        let mut registry = AgentRegistry::new();
        let id = registry.register(Box::new(LocalAgent::new("cancellable")));

        let result = registry.cancel(id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn registry_stop_removes_agent() {
        let mut registry = AgentRegistry::new();
        let id = registry.register(Box::new(LocalAgent::new("stoppable")));

        let result = registry.stop(id).await;
        assert!(result.is_ok());

        // Agent should be removed after stop
        assert!(registry.get(id).is_none());
    }

    #[tokio::test]
    async fn registry_stop_returns_error_for_unknown_agent() {
        let mut registry = AgentRegistry::new();
        let unknown_id = crate::agent::AgentId::new();

        let result = registry.stop(unknown_id).await;
        assert!(result.is_err());
    }
}
