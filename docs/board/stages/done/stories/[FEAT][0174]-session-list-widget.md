---
id: FEAT0174
title: Session list widget
type: feat
status: done
priority: high
scope: tui/42-terminal-dashboard
depends: [m41-feat-05]
estimate: 3h
---

# Session List Widget

## Summary

Implement the session list widget for the dashboard view. Displays all active sessions with status indicators, agent counts, and branch information. Supports keyboard navigation and selection.

## Features

### Session Row

Each session displays:

- Status indicator (bullet color: green=running, yellow=paused, gray=completed, red=failed)
- Session ID (truncated)
- Agent count
- Branch name (from session metadata)
- Status text

### Keyboard Navigation

- `j/k` - Move selection up/down
- `Enter` - Navigate to session detail view
- `n` - Create new session (future milestone)

### Visual Layout

```
┌─ Sessions ──────────────────────────────────────────┐
│ ● session-abc   2 agents   feature/auth   Running   │
│   session-def   1 agent    bugfix/leak    Paused    │
│   session-ghi   4 agents   swarm          Active    │
└─────────────────────────────────────────────────────┘
```

## Implementation

### Widget Structure

```rust
pub struct SessionListWidget {
    sessions: Vec<SessionInfo>,
    selected: usize,
    scroll_offset: usize,
}

pub struct SessionInfo {
    pub id: SessionId,
    pub status: SessionStatus,
    pub agent_count: usize,
    pub branch: Option<String>,
    pub name: Option<String>,
}
```

### Steps

1. Create `src/widgets/mod.rs` and `src/widgets/session_list.rs`
2. Define `SessionListWidget` struct with session data
3. Implement `Widget` trait for rendering
4. Add selection state and scroll offset
5. Handle keyboard events for navigation
6. Connect to `DashboardView` layout
7. Add unit tests for rendering and navigation

## Acceptance Criteria

- [x] Widget renders session list with status indicators
- [x] Status colors match theme (running=green, paused=yellow, etc.)
- [x] j/k navigation moves selection
- [x] Enter key triggers navigation event
- [x] List scrolls when selection exceeds visible area
- [x] Empty state shows "No sessions" message
- [x] Unit tests pass
