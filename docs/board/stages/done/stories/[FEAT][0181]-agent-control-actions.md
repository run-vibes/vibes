---
id: FEAT0181
title: Agent control actions
type: feat
status: done
priority: high
scope: tui/03-terminal-agent-control
depends: [m43-feat-01]
estimate: 3h
---

# Agent Control Actions

## Summary

Implement the agent control bar with pause, resume, cancel, and restart actions. These controls allow the user to manage agent execution state from the TUI.

## Features

### Control Bar

```
├─────────────────────────────────────────────────────┤
│ [p] Pause  [c] Cancel  [r] Restart  [Esc] Back      │
└─────────────────────────────────────────────────────┘
```

Context-sensitive display:
- When running: `[p] Pause  [c] Cancel  [r] Restart  [Esc] Back`
- When paused: `[p] Resume  [c] Cancel  [r] Restart  [Esc] Back`
- When completed/failed: `[r] Restart  [Esc] Back`

### Control Actions

| Key | Action | Description |
|-----|--------|-------------|
| `p` | Pause/Resume | Toggle pause state |
| `c` | Cancel | Stop agent execution |
| `r` | Restart | Restart agent with same task |
| `Esc` | Back | Return to previous view |

### Confirmation Dialogs

For destructive actions, show confirmation:
- Cancel: "Cancel agent execution? [y/n]"
- Restart: "Restart agent? Current progress will be lost. [y/n]"

### State Transitions

```rust
pub enum AgentState {
    Running,
    Paused,
    WaitingForPermission,
    Completed,
    Failed,
    Cancelled,
}
```

Control availability by state:
- **Running**: pause ✓, cancel ✓, restart ✓
- **Paused**: resume ✓, cancel ✓, restart ✓
- **WaitingForPermission**: pause ✓, cancel ✓, restart ✓
- **Completed**: restart ✓ only
- **Failed**: restart ✓ only
- **Cancelled**: restart ✓ only

### Visual Feedback

- Show brief status message after action: "Agent paused", "Agent cancelled"
- Disable unavailable actions visually (dimmed)
- Highlight currently applicable actions

## Implementation

1. Create `src/widgets/control_bar.rs` with ControlBar widget
2. Implement context-sensitive action rendering
3. Add keyboard handlers for p/c/r/Esc
4. Create confirmation dialog component
5. Implement pause/resume command via WebSocket
6. Implement cancel command via WebSocket
7. Implement restart command via WebSocket
8. Add visual feedback for action results
9. Handle state-dependent action availability

## Acceptance Criteria

- [x] Control bar renders at bottom of AgentView
- [x] Actions shown are context-sensitive to agent state
- [x] `p` toggles pause/resume for running agents
- [x] `c` shows confirmation dialog (cancel command pending server integration)
- [x] `r` shows confirmation dialog (restart command pending server integration)
- [x] `Esc` returns to previous view
- [x] Unavailable actions are visually dimmed
- [ ] Commands sent via WebSocket (TODOs in place, requires server integration)
- [ ] Brief status feedback shown after actions (requires command completion)
- [x] Confirmation dialogs work correctly
