---
id: FEAT0178
title: Agent detail view layout
type: feat
status: done
priority: high
scope: tui/03-terminal-agent-control
depends: [m41-feat-03]
estimate: 3h
---

# Agent Detail View Layout

## Summary

Implement the base layout for the agent detail view. This is the primary view for monitoring and interacting with a single agent, showing header info, a two-column main area for output and context, and footer areas for permissions and controls.

## Features

### AgentView Struct

```rust
pub struct AgentView {
    agent_id: AgentId,
}

impl AgentView {
    pub fn new(agent_id: AgentId) -> Self {
        Self { agent_id }
    }
}
```

### Layout Structure

```
┌─ Agent: agent-1 ────────────────────────────────────┐
│ Session: session-abc   Model: claude-sonnet-4-20250514       │
│ Status: Running   Task: implement login flow        │
├──────────────────────────┬──────────────────────────┤
│ Output                   │ Context                  │
│ (placeholder)            │ (placeholder)            │
│                          │                          │
├──────────────────────────┴──────────────────────────┤
│ Permission area (placeholder)                       │
├─────────────────────────────────────────────────────┤
│ Control bar (placeholder)                           │
└─────────────────────────────────────────────────────┘
```

### Header Section

Display agent metadata:
- Agent ID (in title)
- Session ID
- Model name
- Current status (Running, Paused, Waiting, Completed, Failed)
- Current task description (truncated if long)

### Two-Column Main Area

Split horizontally:
- Left: Output panel (60% width) - placeholder for m43-feat-02
- Right: Context panel (40% width) - shows files, tokens, tools, duration

### Context Panel

Display agent context statistics:
- Files: count of files in context
- Tokens: current token usage
- Tools: number of tool calls
- Duration: elapsed time since agent started

## Implementation

1. Create `src/views/agent.rs` with AgentView struct
2. Implement ViewRenderer trait for AgentView
3. Add layout using ratatui::layout::{Layout, Constraint, Direction}
4. Create header section with agent info
5. Create two-column split for output/context
6. Create context panel with stats display
7. Add placeholder sections for permission and control areas
8. Register AgentView in view routing

## Acceptance Criteria

- [ ] AgentView struct created with agent_id
- [ ] ViewRenderer trait implemented
- [ ] Header displays agent ID, session, model, status, task
- [ ] Two-column layout with 60/40 split renders correctly
- [ ] Context panel displays placeholder stats
- [ ] Permission area placeholder visible
- [ ] Control bar placeholder visible
- [ ] Navigation from dashboard to agent view works (via View::Agent)
- [ ] Esc returns to previous view in stack
