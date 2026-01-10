---
id: 40-agent-core
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

## Epics

- [agents](../../epics/agents)

## Design

See [../../epics/agents/README.md](../../epics/agents/README.md) for architecture.
