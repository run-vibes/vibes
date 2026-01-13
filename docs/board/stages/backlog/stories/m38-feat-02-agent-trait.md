---
id: m38-feat-02
title: Agent trait definition
type: feat
status: backlog
priority: high
epics: [agents]
depends: [m38-feat-01]
estimate: 3h
milestone: 38-agent-core
---

# Agent trait definition

## Summary

Define the core `Agent` trait that all agent implementations must satisfy. This is the primary abstraction for autonomous entities in vibes.

## Features

### Agent Trait

```rust
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
    async fn run(&mut self, task: Task) -> Result<TaskResult>;

    /// Pause execution (if supported)
    async fn pause(&mut self) -> Result<()>;

    /// Resume from paused state
    async fn resume(&mut self) -> Result<()>;

    /// Cancel current task
    async fn cancel(&mut self) -> Result<()>;
}
```

### AgentType Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    /// User-triggered, interactive agent
    AdHoc,
    /// Long-running, autonomous agent
    Background,
    /// Spawned by another agent
    Subagent,
    /// Real-time user collaboration
    Interactive,
}
```

## Implementation

1. Add `async-trait` dependency to `vibes-core`
2. Define `AgentType` enum in `types.rs`
3. Define `Agent` trait in `traits.rs`
4. Write unit tests for `AgentType` serialization
5. Verify trait is object-safe (`Box<dyn Agent>`)

## Acceptance Criteria

- [ ] `Agent` trait defined with all methods
- [ ] `AgentType` enum with 4 variants
- [ ] Trait is object-safe (can use `Box<dyn Agent>`)
- [ ] Serialization tests pass
- [ ] `just build` passes
