---
id: m44-feat-02
title: Agent progress bars
type: feat
status: backlog
priority: high
epics: [tui]
depends: [m44-feat-01]
estimate: 2h
milestone: 44-tui-swarm-visualization
---

# Agent Progress Bars

## Summary

Implement progress bar widgets for visualizing individual agent task completion within the swarm view. Each agent card displays a visual progress indicator with percentage and completion state.

## Features

### AgentCard Widget

```rust
pub struct AgentCard {
    pub agent_id: AgentId,
    pub name: String,
    pub task: String,
    pub progress: f32,      // 0.0 to 1.0
    pub status: AgentStatus,
    pub selected: bool,
}

pub enum AgentStatus {
    Running,
    Completed,
    Failed,
    Waiting,
}
```

### Progress Bar Rendering

```
┌──────────┐
│ agent-1  │ ──── Security review (45%)
│ ████░░░░ │
└──────────┘
```

Components:
- Agent name in bordered box
- Task description with percentage
- ASCII progress bar: `█` for filled, `░` for empty
- Completion checkmark `✓` when 100%
- Error indicator when failed

### Progress Calculation

- Map agent token usage or step count to progress percentage
- Handle indeterminate progress (show spinner animation)
- Clamp values between 0-100%

### Status Colors

Using the theme system:
- Running: `theme.running` (phosphor green)
- Completed: `theme.completed` with `✓`
- Failed: `theme.error` with `✗`
- Waiting: `theme.dim`

## Implementation

1. Create `src/widgets/agent_card.rs` in vibes-tui
2. Implement `AgentCard` struct with progress tracking
3. Implement Widget trait for rendering
4. Create progress bar helper using `Gauge` or custom rendering
5. Apply status-based coloring from theme
6. Add selection highlight when `selected: true`
7. Integrate into SwarmView agent grid layout
8. Handle dynamic updates when agent progress changes

## Acceptance Criteria

- [ ] AgentCard widget renders agent name in bordered box
- [ ] Progress bar shows visual fill based on percentage
- [ ] Percentage displayed next to task description
- [ ] Completed agents show checkmark indicator
- [ ] Failed agents show error indicator with theme.error color
- [ ] Selected card has highlight background
- [ ] Progress updates reflect real-time agent state
- [ ] Works with varying agent counts in swarm
