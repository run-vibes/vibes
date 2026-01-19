---
id: 38-autonomous-agents
title: Autonomous Agents
status: done
epics: [agents]
---

# Autonomous Agents

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

| # | Story | Description | Status |
|---|-------|-------------|--------|
| 1 | [FEAT0151](../../../../stages/done/stories/[FEAT][0151]-agent-module-skeleton.md) | Agent module skeleton in vibes-core | done |
| 2 | [FEAT0152](../../../../stages/done/stories/[FEAT][0152]-agent-trait.md) | Agent trait definition | done |
| 3 | [FEAT0153](../../../../stages/done/stories/[FEAT][0153]-agent-status.md) | AgentStatus and AgentContext types | done |
| 4 | [FEAT0154](../../../../stages/done/stories/[FEAT][0154]-task-types.md) | Task and TaskResult types | done |
| 5 | [FEAT0155](../../../../stages/done/stories/[FEAT][0155]-agent-lifecycle.md) | Agent lifecycle management | done |
| 6 | [FEAT0156](../../../../stages/done/stories/[FEAT][0156]-agent-cli.md) | Agent CLI commands | done |
| 7 | [FEAT0157](../../../../stages/done/stories/[FEAT][0157]-agent-web-ui.md) | Agent Web UI | done |

## Progress

**Requirements:** 0/0 verified
**Stories:** 7/7 complete

