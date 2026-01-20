---
id: 03-terminal-agent-control
title: Terminal Agent Control
status: done
epics: [tui]
---

# Terminal Agent Control

## Overview

Third milestone of the TUI epic. Implements the agent detail view with permission approval, output viewing, and agent control (pause/resume/cancel).

## Goals

- Agent detail view with output stream
- Permission request handling (approve/deny/view diff)
- Agent control actions (pause, resume, cancel)
- Context panel (files, tokens, tools, duration)
- Real-time output streaming

## Key Deliverables

- `AgentView` implementation
- Permission approval widget
- Output stream panel
- Context stats panel
- Agent control keybindings

## Wireframe

```
┌─ Agent: agent-1 ────────────────────────────────────┐
│ Session: session-abc   Model: claude-sonnet-4-20250514       │
│ Status: Running   Task: implement login flow        │
├──────────────────────────┬──────────────────────────┤
│ Output                   │ Context                  │
│                          │                          │
│ > Analyzing codebase...  │ Files: 12                │
│ > Found auth module      │ Tokens: 45,231           │
│ > Implementing login     │ Tools: 8 calls           │
│   handler...             │ Duration: 4m 32s         │
│                          │                          │
├──────────────────────────┴──────────────────────────┤
│ ⚠ Permission Request: Write to src/auth/login.rs    │
│ [y] Approve  [n] Deny  [v] View diff  [e] Edit      │
├─────────────────────────────────────────────────────┤
│ [p] Pause  [c] Cancel  [r] Restart  [Esc] Back      │
└─────────────────────────────────────────────────────┘
```

## Epics

- [tui](../../epics/tui)

## Stories

| # | Story | Description | Status |
|---|-------|-------------|--------|
| 1 | [FEAT0178](../../../../stages/done/stories/[FEAT][0178]-agent-detail-view-layout.md) | Agent detail view layout | done |
| 2 | [FEAT0179](../../../../stages/done/stories/[FEAT][0179]-output-stream-panel.md) | Output stream panel | done |
| 3 | [FEAT0180](../../../../stages/done/stories/[FEAT][0180]-permission-approval-widget.md) | Permission approval widget | done |
| 4 | [FEAT0181](../../../../stages/done/stories/[FEAT][0181]-agent-control-actions.md) | Agent control actions | done |

## Progress

**Requirements:** 0/0 verified
**Stories:** 4/4 complete

