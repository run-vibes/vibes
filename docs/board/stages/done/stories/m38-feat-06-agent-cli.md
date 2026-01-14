---
id: m38-feat-06
title: Agent CLI commands
type: feat
status: done
priority: medium
epics: [agents]
depends: [m38-feat-05]
estimate: 3h
milestone: 38-agent-core
---

# Agent CLI commands

## Summary

Add CLI commands for managing agents. Users can list, spawn, inspect, and control agents from the command line.

## Features

### Commands

```
vibes agent list                     # List agents in current session
vibes agent spawn <type> [--task]    # Spawn new agent
vibes agent status <id>              # Check agent status
vibes agent pause <id>               # Pause agent
vibes agent resume <id>              # Resume agent
vibes agent cancel <id>              # Cancel current task
vibes agent stop <id>                # Stop and remove agent
```

### List Command

```
$ vibes agent list

ID                                    TYPE        STATUS    NAME
────────────────────────────────────────────────────────────────────
019abc12-...                          adhoc       running   code-review
019abc13-...                          background  idle      file-watcher

Total: 2 agents (1 running, 1 idle)
```

### Status Command

```
$ vibes agent status 019abc12

Agent: code-review (019abc12-...)
Type: AdHoc
Status: Running
  Task: 019def34-...
  Started: 2 minutes ago

Context:
  Location: Local
  Model: claude-3-5-sonnet
  Tools: read, write, bash

Metrics (current task):
  Duration: 2m 15s
  Tokens: 12,450
  Tool calls: 8
```

### Spawn Command

```
$ vibes agent spawn adhoc --task "Review the authentication module"

Spawned agent 019abc14-... (adhoc)
Task started: Review the authentication module
```

## Implementation

1. Add `agent` subcommand to `vibes-cli`
2. Implement `list`, `spawn`, `status` commands
3. Implement `pause`, `resume`, `cancel`, `stop` commands
4. Add WebSocket messages for agent operations
5. Format output with consistent styling

## Acceptance Criteria

- [ ] `vibes agent list` shows all agents
- [ ] `vibes agent spawn` creates new agent
- [ ] `vibes agent status` shows detailed info
- [ ] `vibes agent pause/resume/cancel/stop` work
- [ ] Commands connect via existing WebSocket
- [ ] Error messages are clear and helpful
