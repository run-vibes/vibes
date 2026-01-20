---
id: FEAT0176
title: Activity feed widget
type: feat
status: done
priority: medium
scope: tui/02-terminal-dashboard
depends: [m42-feat-01]
estimate: 3h
---

# Activity Feed Widget

## Summary

Implement the activity feed widget that shows recent events across all sessions. Displays timestamped entries for agent actions, status changes, and permission requests.

## Features

### Event Types

- Agent task completed
- Agent waiting for permission
- Agent started/paused/resumed
- Swarm coordination events
- Session lifecycle events

### Visual Layout

```
┌─ Recent Activity ───────────────────────────────────┐
│ 14:32 agent-1 completed task "implement login"      │
│ 14:31 agent-2 waiting for permission                │
│ 14:30 swarm-1 started parallel execution            │
│ 14:28 session-abc created                           │
└─────────────────────────────────────────────────────┘
```

### Entry Format

- Timestamp (HH:MM format)
- Source identifier (agent ID, swarm ID, or session ID)
- Action description
- Truncated if too long for display width

## Implementation

### Widget Structure

```rust
pub struct ActivityFeedWidget {
    entries: VecDeque<ActivityEntry>,
    max_entries: usize,
    scroll_offset: usize,
}

pub struct ActivityEntry {
    pub timestamp: DateTime<Utc>,
    pub source: ActivitySource,
    pub message: String,
    pub level: ActivityLevel,
}

pub enum ActivitySource {
    Agent(AgentId),
    Swarm(SwarmId),
    Session(SessionId),
    System,
}

pub enum ActivityLevel {
    Info,
    Warning,
    Error,
}
```

### Steps

1. Create `src/widgets/activity_feed.rs`
2. Define `ActivityFeedWidget` with bounded entry list
3. Implement `Widget` trait for multi-line rendering
4. Add `push_entry` method with max size enforcement
5. Format timestamps in local time
6. Color-code entries by level (info=dim, warning=yellow, error=red)
7. Add scroll support for long feeds
8. Add unit tests for entry management and rendering

## Acceptance Criteria

- [x] Feed displays entries with timestamps
- [x] Entries show source and message
- [x] Timestamps formatted as HH:MM
- [x] Scroll works when entries exceed visible area
- [x] Unit tests pass
- [x] Widget integrated into DashboardView

Note: Entry level coloring and max entries enforcement deferred to m42-feat-04 when real events are wired.
