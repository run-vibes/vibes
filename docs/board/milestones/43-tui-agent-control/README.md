---
id: 43-tui-agent-control
title: TUI Agent Control
status: done
epics: [tui]
---

# TUI Agent Control

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

| ID | Title | Status |
|----|-------|--------|
| m43-feat-01 | Agent detail view layout | backlog |
| m43-feat-02 | Output stream panel | backlog |
| m43-feat-03 | Permission approval widget | backlog |
| m43-feat-04 | Agent control actions | backlog |

## Design

See [../../epics/tui/README.md](../../epics/tui/README.md) for architecture.
