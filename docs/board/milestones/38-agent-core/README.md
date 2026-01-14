---
id: 38-agent-core
title: Agent Core
status: planned
epics: [agents]
---

# Agent Core

## Overview

First milestone of the Agents epic. Establishes the foundation for agent orchestration: the Agent trait, agent types, lifecycle management, and task system.

## Goals

- Agent trait and lifecycle (idle, running, paused, completed)
- Agent types: Ad-hoc, Background, Subagent, Interactive
- Task system with metrics (duration, tokens, tool calls)
- Agent status tracking and control

## Key Deliverables

- Agent trait in `vibes-core`
- `AgentType` and `AgentStatus` enums
- `Task` and `TaskResult` types
- `AgentContext` for execution configuration
- Basic agent spawning and control

## Stories

| ID | Title | Priority | Estimate |
|----|-------|----------|----------|
| m38-feat-01 | Agent module skeleton in vibes-core | high | 2h |
| m38-feat-02 | Agent trait definition | high | 3h |
| m38-feat-03 | AgentStatus and AgentContext types | high | 2h |
| m38-feat-04 | Task and TaskResult types | high | 3h |
| m38-feat-05 | Agent lifecycle management | high | 4h |
| m38-feat-06 | Agent CLI commands | medium | 3h |
| m38-feat-07 | Agent Web UI | medium | 4h |

## Epics

- [agents](../../epics/agents)

## Design

See [../../epics/agents/README.md](../../epics/agents/README.md) for architecture.
