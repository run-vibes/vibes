---
id: m38-feat-07
title: Agent Web UI
type: feat
status: done
priority: medium
epics: [agents]
depends: [m38-feat-06]
estimate: 4h
milestone: 38-agent-core
---

# Agent Web UI

## Summary

Add agent management to the web UI. Users can view, spawn, and control agents from the dashboard.

## Features

### Agents Page

A dedicated `/agents` route showing all agents in the current session:

- Agent list with ID, type, status, and current task
- Status indicators (running, idle, paused, completed)
- Quick actions (pause, resume, cancel, stop)
- Spawn new agent button

### Agent Detail View

Clicking an agent shows detailed information:

- Agent metadata (ID, type, name, created time)
- Current status and task progress
- Context configuration (model, tools, location)
- Metrics: duration, tokens, tool calls
- Task history

### Agent Spawning

Modal or drawer for creating new agents:

- Select agent type (adhoc, background, subagent, interactive)
- Optional task description
- Context configuration
- Spawn button with feedback

### Real-time Updates

- WebSocket subscription for agent status changes
- Live task progress updates
- Automatic list refresh on spawn/stop

## Implementation

1. Add `/agents` route to web-ui
2. Create `AgentList` component with status badges
3. Create `AgentDetail` component for expanded view
4. Create `SpawnAgentModal` component
5. Add WebSocket handlers for agent events
6. Integrate with existing session context

## Acceptance Criteria

- [x] Agents page shows all session agents
- [x] Agent status updates in real-time
- [x] Spawn modal creates new agents
- [x] Pause/resume/cancel/stop actions work
- [x] Detail view shows agent metrics
- [x] Responsive design for mobile
