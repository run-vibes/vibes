//! Server-side agent registry wrapper
//!
//! Wraps vibes_core::AgentRegistry with server-specific methods:
//! - AgentInfo conversion for WebSocket protocol
//! - Prefix-based ID matching for CLI convenience
//! - Spawn with automatic LocalAgent creation

use tracing::instrument;
use uuid::Uuid;
use vibes_core::agent::{
    Agent, AgentId, AgentRegistry, AgentStatus, AgentType, LocalAgent, Task, TaskMetrics,
};
use vibes_core::error::VibesResult;

use crate::ws::protocol::AgentInfo;

/// Server-side agent registry with CLI-friendly operations
pub struct ServerAgentRegistry {
    inner: AgentRegistry,
}

impl ServerAgentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            inner: AgentRegistry::new(),
        }
    }

    /// List all agents with their info (for WebSocket protocol)
    pub fn list_agent_info(&self) -> Vec<AgentInfo> {
        self.inner
            .list()
            .into_iter()
            .filter_map(|id| {
                let agent = self.inner.get(id)?;
                Some(agent_to_info(agent))
            })
            .collect()
    }

    /// Get agent info by ID or prefix
    ///
    /// Supports partial UUID matching for CLI convenience:
    /// - Full UUID: "019abc12-3456-7890-abcd-ef1234567890"
    /// - Prefix: "019abc12" or "019abc"
    pub fn get_agent_info(&self, id_or_prefix: &str) -> Option<AgentInfo> {
        let agent_id = self.resolve_agent_id(id_or_prefix)?;
        let agent = self.inner.get(agent_id)?;
        Some(agent_to_info(agent))
    }

    /// Spawn a new agent and optionally start a task
    #[instrument(name = "agent::spawn", skip(self), fields(agent_type = ?agent_type))]
    pub async fn spawn_agent(
        &mut self,
        agent_type: AgentType,
        name: Option<String>,
        task_description: Option<String>,
    ) -> VibesResult<AgentInfo> {
        // Create agent with name or generate one
        let agent_name = name.unwrap_or_else(|| format!("{:?}-{}", agent_type, short_id()));
        let mut agent = LocalAgent::new(&agent_name).with_type(agent_type);

        // If a task was provided, run it immediately
        if let Some(description) = task_description {
            let task = Task::new(description);
            // Run the task (LocalAgent completes immediately for now)
            let _ = agent.run(task).await?;
        }

        // Register and return info
        let info = agent_to_info(&agent);
        self.inner.register(Box::new(agent));
        Ok(info)
    }

    /// Pause an agent by ID or prefix
    #[instrument(name = "agent::pause", skip(self), fields(agent_id = %id_or_prefix))]
    pub async fn pause_agent(&mut self, id_or_prefix: &str) -> VibesResult<()> {
        let agent_id = self
            .resolve_agent_id(id_or_prefix)
            .ok_or_else(|| vibes_core::error::AgentError::NotFound(id_or_prefix.to_string()))?;
        self.inner.pause(agent_id).await
    }

    /// Resume an agent by ID or prefix
    #[instrument(name = "agent::resume", skip(self), fields(agent_id = %id_or_prefix))]
    pub async fn resume_agent(&mut self, id_or_prefix: &str) -> VibesResult<()> {
        let agent_id = self
            .resolve_agent_id(id_or_prefix)
            .ok_or_else(|| vibes_core::error::AgentError::NotFound(id_or_prefix.to_string()))?;
        self.inner.resume(agent_id).await
    }

    /// Cancel current task on an agent by ID or prefix
    #[instrument(name = "agent::cancel", skip(self), fields(agent_id = %id_or_prefix))]
    pub async fn cancel_agent(&mut self, id_or_prefix: &str) -> VibesResult<()> {
        let agent_id = self
            .resolve_agent_id(id_or_prefix)
            .ok_or_else(|| vibes_core::error::AgentError::NotFound(id_or_prefix.to_string()))?;
        self.inner.cancel(agent_id).await
    }

    /// Stop and remove an agent by ID or prefix
    #[instrument(name = "agent::stop", skip(self), fields(agent_id = %id_or_prefix))]
    pub async fn stop_agent(&mut self, id_or_prefix: &str) -> VibesResult<()> {
        let agent_id = self
            .resolve_agent_id(id_or_prefix)
            .ok_or_else(|| vibes_core::error::AgentError::NotFound(id_or_prefix.to_string()))?;
        self.inner.stop(agent_id).await
    }

    /// Resolve an ID or prefix to a full AgentId
    fn resolve_agent_id(&self, id_or_prefix: &str) -> Option<AgentId> {
        // Try exact UUID match first
        if let Ok(uuid) = Uuid::parse_str(id_or_prefix) {
            let id = AgentId(uuid);
            if self.inner.get(id).is_some() {
                return Some(id);
            }
        }

        // Try prefix matching
        let prefix = id_or_prefix.to_lowercase();
        let mut matches: Vec<AgentId> = self
            .inner
            .list()
            .into_iter()
            .filter(|id| id.to_string().to_lowercase().starts_with(&prefix))
            .collect();

        // Return only if exactly one match
        if matches.len() == 1 {
            matches.pop()
        } else {
            None
        }
    }
}

impl Default for ServerAgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert an Agent trait object to AgentInfo for protocol
fn agent_to_info(agent: &dyn Agent) -> AgentInfo {
    AgentInfo {
        id: agent.id().to_string(),
        name: agent.name().to_string(),
        agent_type: agent.agent_type(),
        status: agent.status(),
        context: agent.context().clone(),
        current_task_metrics: extract_current_metrics(&agent.status()),
    }
}

/// Extract current task metrics if agent is running
fn extract_current_metrics(status: &AgentStatus) -> Option<TaskMetrics> {
    match status {
        AgentStatus::Running { started, .. } => {
            // Calculate duration from started time
            let duration = chrono::Utc::now()
                .signed_duration_since(*started)
                .to_std()
                .unwrap_or_default();
            Some(TaskMetrics {
                duration,
                tokens_used: 0, // Would need to track this during execution
                tool_calls: 0,  // Would need to track this during execution
                iterations: 0,  // Would need to track this during execution
            })
        }
        _ => None,
    }
}

/// Generate a short random ID suffix
fn short_id() -> String {
    let uuid = Uuid::now_v7();
    // Take first 8 chars of the UUID
    uuid.to_string()[..8].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_registry_is_empty() {
        let registry = ServerAgentRegistry::new();
        assert!(registry.list_agent_info().is_empty());
    }

    #[tokio::test]
    async fn spawn_agent_creates_agent() {
        let mut registry = ServerAgentRegistry::new();
        let info = registry
            .spawn_agent(AgentType::AdHoc, Some("test-agent".to_string()), None)
            .await
            .unwrap();

        assert_eq!(info.name, "test-agent");
        assert_eq!(info.agent_type, AgentType::AdHoc);
    }

    #[tokio::test]
    async fn spawn_agent_with_task() {
        let mut registry = ServerAgentRegistry::new();
        let info = registry
            .spawn_agent(
                AgentType::AdHoc,
                Some("worker".to_string()),
                Some("Do something".to_string()),
            )
            .await
            .unwrap();

        // Agent should be created (task completes immediately for LocalAgent)
        assert_eq!(info.name, "worker");
    }

    #[tokio::test]
    async fn get_agent_info_by_full_id() {
        let mut registry = ServerAgentRegistry::new();
        let info = registry
            .spawn_agent(AgentType::Background, Some("bg-agent".to_string()), None)
            .await
            .unwrap();

        let retrieved = registry.get_agent_info(&info.id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "bg-agent");
    }

    #[tokio::test]
    async fn get_agent_info_by_prefix() {
        let mut registry = ServerAgentRegistry::new();
        let info = registry
            .spawn_agent(AgentType::AdHoc, Some("prefix-test".to_string()), None)
            .await
            .unwrap();

        // Use first 8 chars as prefix
        let prefix = &info.id[..8];
        let retrieved = registry.get_agent_info(prefix);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "prefix-test");
    }

    #[tokio::test]
    async fn stop_agent_removes_it() {
        let mut registry = ServerAgentRegistry::new();
        let info = registry
            .spawn_agent(AgentType::AdHoc, Some("to-stop".to_string()), None)
            .await
            .unwrap();

        registry.stop_agent(&info.id).await.unwrap();

        assert!(registry.get_agent_info(&info.id).is_none());
    }
}
