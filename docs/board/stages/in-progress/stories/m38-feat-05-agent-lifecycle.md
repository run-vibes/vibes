---
id: m38-feat-05
title: Agent lifecycle management
type: feat
status: in-progress
priority: high
epics: [agents]
depends: [m38-feat-03, m38-feat-04]
estimate: 4h
milestone: 38-agent-core
---

# Agent lifecycle management

## Summary

Implement the agent registry and lifecycle management in vibes-core. This enables spawning, tracking, and controlling agents within a session context.

## Features

### AgentRegistry

```rust
pub struct AgentRegistry {
    agents: HashMap<AgentId, Box<dyn Agent>>,
}

impl AgentRegistry {
    pub fn new() -> Self;

    /// Register a new agent
    pub fn register(&mut self, agent: Box<dyn Agent>) -> AgentId;

    /// Get agent by ID
    pub fn get(&self, id: AgentId) -> Option<&dyn Agent>;

    /// Get mutable agent by ID
    pub fn get_mut(&mut self, id: AgentId) -> Option<&mut Box<dyn Agent>>;

    /// Remove agent from registry
    pub fn remove(&mut self, id: AgentId) -> Option<Box<dyn Agent>>;

    /// List all agents
    pub fn list(&self) -> Vec<AgentId>;

    /// Get agents by type
    pub fn by_type(&self, agent_type: AgentType) -> Vec<AgentId>;

    /// Get agents by status
    pub fn by_status(&self, status: AgentStatus) -> Vec<AgentId>;
}
```

### Lifecycle Operations

```rust
impl AgentRegistry {
    /// Spawn a new agent with configuration
    pub async fn spawn(
        &mut self,
        agent_type: AgentType,
        context: AgentContext,
    ) -> Result<AgentId>;

    /// Run a task on an agent
    pub async fn run_task(
        &mut self,
        agent_id: AgentId,
        task: Task,
    ) -> Result<TaskResult>;

    /// Pause an agent
    pub async fn pause(&mut self, agent_id: AgentId) -> Result<()>;

    /// Resume an agent
    pub async fn resume(&mut self, agent_id: AgentId) -> Result<()>;

    /// Cancel an agent's current task
    pub async fn cancel(&mut self, agent_id: AgentId) -> Result<()>;

    /// Stop and remove an agent
    pub async fn stop(&mut self, agent_id: AgentId) -> Result<()>;
}
```

### Basic Agent Implementation

Create a minimal `LocalAgent` implementation for testing:

```rust
pub struct LocalAgent {
    id: AgentId,
    name: String,
    agent_type: AgentType,
    status: AgentStatus,
    context: AgentContext,
}
```

## Implementation

1. Create `vibes-core/src/agent/registry.rs`
2. Implement `AgentRegistry` struct
3. Implement lifecycle methods
4. Create `LocalAgent` stub implementation
5. Write integration tests for lifecycle transitions

## Acceptance Criteria

- [x] `AgentRegistry` manages agent instances
- [x] Register adds agents and returns their ID
- [x] Run task executes tasks on agents
- [x] Pause/resume/cancel work correctly
- [x] Stop removes agents from registry
- [x] Filter by type and status variant
- [x] LocalAgent stub implements Agent trait
- [x] Tests cover all lifecycle transitions (22 tests)
