---
id: FEAT0151
title: Agent module skeleton in vibes-core
type: feat
status: done
priority: high
scope: agents/01-autonomous-agents
depends: []
estimate: 2h
---

# Agent module skeleton in vibes-core

## Summary

Create the foundational module structure for agent support in `vibes-core`. This establishes the directory layout and exports without implementing functionality.

## Features

### Module Structure

Create `vibes-core/src/agent/` with:

- `mod.rs` - Module exports
- `types.rs` - Type definitions (placeholder)
- `traits.rs` - Trait definitions (placeholder)
- `task.rs` - Task system types (placeholder)

### Public Exports

Export from `vibes-core/src/lib.rs`:

```rust
pub mod agent;
pub use agent::{Agent, AgentId, AgentType, AgentStatus};
```

### AgentId Type

Define the agent identifier:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

impl AgentId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}
```

## Implementation

1. Create `vibes-core/src/agent/mod.rs`
2. Create `vibes-core/src/agent/types.rs` with `AgentId`
3. Create placeholder files for traits and task
4. Export module from `vibes-core/src/lib.rs`
5. Verify build passes

## Acceptance Criteria

- [ ] `vibes-core/src/agent/` directory exists
- [ ] `AgentId` type is defined and exported
- [ ] Module structure compiles
- [ ] `just build` passes
