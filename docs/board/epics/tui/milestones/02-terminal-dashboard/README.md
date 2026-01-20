---
id: 02-terminal-dashboard
title: Terminal Dashboard
status: done
epics: [tui]
---

# Terminal Dashboard

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

| # | Story | Description | Status |
|---|-------|-------------|--------|
| 1 | [FEAT0174](../../../../stages/done/stories/[FEAT][0174]-session-list-widget.md) | Session list widget | done |
| 2 | [FEAT0175](../../../../stages/done/stories/[FEAT][0175]-stats-summary-bar.md) | Stats summary bar | done |
| 3 | [FEAT0176](../../../../stages/done/stories/[FEAT][0176]-activity-feed-widget.md) | Activity feed widget | done |
| 4 | [FEAT0177](../../../../stages/done/stories/[FEAT][0177]-websocket-real-time-updates.md) | WebSocket real-time updates | done |

## Progress

**Requirements:** 0/0 verified
**Stories:** 4/4 complete

