---
id: m44-feat-04
title: Swarm coordination status
type: feat
status: backlog
priority: medium
epics: [tui]
depends: [m44-feat-01]
estimate: 2h
milestone: 44-tui-swarm-visualization
---

# Swarm Coordination Status

## Summary

Implement swarm-level status indicators showing execution strategy, overall coordination state, and aggregate metrics. This gives users a high-level view of swarm health without drilling into individual agents.

## Features

### Strategy Display

Show the swarm execution strategy in the header:

```
Strategy: Parallel   Status: Running
```

Supported strategies:
- **Parallel**: All agents run simultaneously
- **Sequential**: Agents run one after another
- **Pipeline**: Output of one feeds into next
- **Voting**: Multiple agents, consensus on result

### Coordination Indicators

Visual indicators for swarm-level state:

```
┌─ Swarm: code-review ────────────────────────────┐
│ Strategy: Parallel   Status: Running   ⟳        │
│ Task: Review PR #123                            │
│ Agents: 2/3 running  1 completed                │
│ Progress: ████████░░░░░░░░ 53%                 │
├─────────────────────────────────────────────────┤
```

Components:
- Strategy badge with icon
- Overall status (Pending, Running, Completed, Failed, Partial)
- Spinner animation while active
- Agent count breakdown
- Aggregate progress bar

### Status States

```rust
pub enum SwarmStatus {
    Pending,    // Not yet started
    Running,    // At least one agent active
    Completed,  // All agents completed successfully
    Failed,     // At least one agent failed
    Partial,    // Some completed, some failed
    Cancelled,  // User cancelled
}
```

### Aggregate Metrics

```
┌─ Metrics ───────────────────────────────────────┐
│ Total Tokens: 125,432   Cost: $1.87             │
│ Duration: 4m 32s        Efficiency: 94%         │
└─────────────────────────────────────────────────┘
```

## Implementation

1. Add strategy and status fields to SwarmView state
2. Create status badge widget with strategy icons
3. Implement aggregate progress calculation
4. Add agent count breakdown display
5. Create spinner animation for running state
6. Implement metrics aggregation across agents
7. Handle real-time status updates via WebSocket
8. Apply theme colors for different states

## Acceptance Criteria

- [ ] Strategy displays with appropriate label
- [ ] Status indicator updates in real-time
- [ ] Spinner animates while swarm is running
- [ ] Agent count shows running/completed breakdown
- [ ] Aggregate progress bar reflects overall completion
- [ ] Metrics display total tokens and cost
- [ ] Failed state shows error styling
- [ ] Partial completion shows warning styling
- [ ] All indicators use theme colors
