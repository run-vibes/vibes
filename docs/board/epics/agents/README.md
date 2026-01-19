---
id: agents
title: Agent Orchestration
status: planned
description: Agent lifecycle, swarms, local/remote execution, session-agent relationship
---

# Agent Orchestration

Orchestration layer where sessions contain agents. Supports ad-hoc, background, and subagents. Swarms coordinate multiple agents on same or different tasks. Local and remote execution.

## Overview

Sessions and Agents are separate concepts:
- A **Session** is a workspace containing state, event log, and agents
- An **Agent** is an autonomous entity that performs work within a session
- A **Swarm** coordinates multiple agents (same or different tasks)

## Session-Agent Relationship

```rust
pub struct Session {
    pub id: SessionId,
    pub agents: HashMap<AgentId, Box<dyn Agent>>,
    pub primary_agent: Option<AgentId>,
    pub swarms: Vec<Swarm>,
    pub event_log: EventLog,
    pub state: SessionState,
}

pub trait Agent: Send + Sync {
    fn id(&self) -> AgentId;
    fn name(&self) -> &str;
    fn agent_type(&self) -> AgentType;

    async fn run(&mut self, task: Task) -> Result<TaskResult>;
    async fn pause(&mut self) -> Result<()>;
    async fn resume(&mut self) -> Result<()>;
    async fn cancel(&mut self) -> Result<()>;

    fn status(&self) -> AgentStatus;
    fn context(&self) -> &AgentContext;
}
```

## Agent Types

```rust
pub enum AgentType {
    AdHoc,         // User-triggered, interactive
    Background,    // Long-running, autonomous
    Subagent,      // Spawned by another agent
    Interactive,   // Real-time user collaboration
}

pub enum AgentStatus {
    Idle,
    Running { task: TaskId, started: DateTime },
    Paused { task: TaskId, reason: String },
    WaitingForInput { prompt: String },
    Completed { result: TaskResult },
    Failed { error: String },
}
```

## Swarm Architecture

```rust
pub struct Swarm {
    pub id: SwarmId,
    pub agents: Vec<AgentId>,
    pub strategy: SwarmStrategy,
    pub coordinator: Option<AgentId>,
    pub status: SwarmStatus,
}

pub enum SwarmStrategy {
    Parallel { merge: MergeStrategy },
    Pipeline { stages: Vec<AgentId> },
    Supervisor { supervisor: AgentId, workers: Vec<AgentId> },
    Debate { rounds: u32, judge: Option<AgentId> },
}

pub enum MergeStrategy {
    First,              // First to complete wins
    All,                // Wait for all, combine results
    Vote,               // Consensus voting
    Best { scorer: Scorer },  // Score and select best
}
```

### Swarm Strategies

| Strategy | Use Case | Example |
|----------|----------|---------|
| **Parallel** | Same task, multiple approaches | Code review by 3 agents, merge findings |
| **Pipeline** | Sequential processing | Research -> Draft -> Edit -> Review |
| **Supervisor** | Delegated work | Lead agent assigns subtasks to workers |
| **Debate** | Consensus building | Agents debate approach, judge decides |

## Execution Locations

```rust
pub enum ExecutionLocation {
    Local,                              // Same machine as vibes
    Remote { endpoint: Url },           // Remote vibes instance
    Cloud { provider: CloudProvider },  // Cloud execution
}

pub struct AgentContext {
    pub location: ExecutionLocation,
    pub model: ModelId,
    pub tools: Vec<ToolId>,
    pub permissions: Permissions,
    pub resource_limits: ResourceLimits,
}
```

## Agent Lifecycle

```
┌─────────┐     ┌─────────┐     ┌─────────┐
│  Idle   │────▶│ Running │────▶│Complete │
└─────────┘     └────┬────┘     └─────────┘
     ▲               │
     │          ┌────▼────┐
     └──────────│ Paused  │
                └─────────┘
```

## Task System

```rust
pub struct Task {
    pub id: TaskId,
    pub description: String,
    pub context: TaskContext,
    pub constraints: TaskConstraints,
    pub parent: Option<TaskId>,  // For subtasks
}

pub struct TaskResult {
    pub task_id: TaskId,
    pub status: TaskStatus,
    pub output: Option<Value>,
    pub artifacts: Vec<Artifact>,
    pub metrics: TaskMetrics,
}

pub struct TaskMetrics {
    pub duration: Duration,
    pub tokens_used: u64,
    pub tool_calls: u32,
    pub iterations: u32,
}
```

## Communication

```rust
pub enum AgentMessage {
    Task(Task),
    Result(TaskResult),
    Query { from: AgentId, question: String },
    Response { to: AgentId, answer: String },
    Handoff { to: AgentId, context: Value },
    Broadcast { message: String },
}

impl Agent {
    async fn send(&self, to: AgentId, msg: AgentMessage) -> Result<()>;
    async fn receive(&mut self) -> Option<AgentMessage>;
    async fn broadcast(&self, msg: AgentMessage) -> Result<()>;
}
```

## CLI Commands

```
vibes agent list                     # List agents in current session
vibes agent spawn <type> <task>      # Spawn new agent
vibes agent status <id>              # Check agent status
vibes agent pause <id>               # Pause agent
vibes agent resume <id>              # Resume agent
vibes agent cancel <id>              # Cancel agent

vibes swarm create <strategy>        # Create swarm
vibes swarm add <swarm> <agent>      # Add agent to swarm
vibes swarm run <swarm> <task>       # Run task on swarm
vibes swarm status <swarm>           # Swarm status
```

<!-- BEGIN GENERATED -->
## Milestones

**Progress:** 1/1 milestones complete, 7/7 stories done

| ID | Milestone | Stories | Status |
|----|-----------|---------|--------|
| 38 | [Autonomous Agents](milestones/38-autonomous-agents/) | 7/7 | done |
<!-- END GENERATED -->
