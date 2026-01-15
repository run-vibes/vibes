---
id: 42-tui-dashboard
title: TUI Dashboard
status: planned
epics: [tui]
---

# TUI Dashboard

## Overview

Second milestone of the TUI epic. Implements the main dashboard view showing an overview of all activity: sessions, agents, costs, and recent events.

## Goals

- Session list with status indicators
- Agent count and status summary
- Cost tracking display
- Recent activity feed
- Navigation to session/agent details

## Key Deliverables

- `DashboardView` implementation with real content
- Session list widget with selection
- Activity feed widget
- Stats summary bar (sessions, agents, cost)
- WebSocket subscription for real-time updates

## Wireframe

```
┌─ vibes ─────────────────────────────────────────────┐
│ Sessions: 3 active   Agents: 7 running   Cost: $12  │
├─────────────────────────────────────────────────────┤
│ ● session-abc   2 agents   feature/auth   Running   │
│   session-def   1 agent    bugfix/leak    Paused    │
│   session-ghi   4 agents   swarm          Active    │
├─────────────────────────────────────────────────────┤
│ Recent Activity                                      │
│ 14:32 agent-1 completed task "implement login"      │
│ 14:31 agent-2 waiting for permission                │
│ 14:30 swarm-1 started parallel execution            │
├─────────────────────────────────────────────────────┤
│ [j/k] Navigate  [Enter] Select  [n] New  [?] Help   │
└─────────────────────────────────────────────────────┘
```

## Epics

- [tui](../../epics/tui)

## Stories

| ID | Title | Status |
|----|-------|--------|
| m42-feat-01 | Session list widget | backlog |
| m42-feat-02 | Stats summary bar | backlog |
| m42-feat-03 | Activity feed widget | backlog |
| m42-feat-04 | WebSocket real-time updates | backlog |

## Design

See [../../epics/tui/README.md](../../epics/tui/README.md) for architecture.
