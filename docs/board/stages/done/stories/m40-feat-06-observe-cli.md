---
id: m40-feat-06
title: vibes observe traces CLI command
type: feat
status: done
priority: medium
epics: [observability]
depends: [m40-feat-05]
estimate: 3h
milestone: 40-observability-tracing
---

# vibes observe traces CLI command

## Summary

Add CLI commands for viewing and managing traces. Users can tail traces, filter by session/agent, and configure export targets.

## Features

### Commands

```
vibes observe traces                  # Tail traces (live)
vibes observe traces <session>        # Filter by session
vibes observe traces --agent <id>     # Filter by agent
vibes observe traces --level info     # Filter by level
vibes observe config                  # Show current config
vibes observe config set <key> <val>  # Update config
```

### Traces Command

```
$ vibes observe traces

Trace: 019abc12... | Session: 019def34...
└─ server::handle_ws_message (2.3ms)
   └─ session::process_event (1.8ms)
      ├─ model::inference (1.2ms) tokens=450
      └─ tool::execute (0.4ms) tool=read_file

Trace: 019abc13... | Session: 019def34...
└─ agent::run_task (5.2s)
   ├─ model::inference (2.1s) tokens=1200
   ├─ tool::execute (0.8s) tool=write_file
   └─ model::inference (1.9s) tokens=850
```

### Session Filter

```
$ vibes observe traces 019def34

Showing traces for session 019def34...

[14:23:01] process_event (1.8ms)
[14:23:02] process_event (2.1ms)
[14:23:05] run_task started
[14:23:10] run_task completed (5.2s)
```

### Config Command

```
$ vibes observe config

Tracing Configuration
─────────────────────────────────────
Enabled: true
Sample rate: 100%

Exporters:
  1. Console (pretty)
  2. OTLP (http://localhost:4317)

$ vibes observe config set tracing.sample_rate 0.5
Updated: tracing.sample_rate = 0.5
```

## Implementation

1. Add `observe` subcommand to `vibes-cli`
2. Implement `traces` command with filters
3. Implement live trace streaming
4. Implement `config` command for settings
5. Format output with tree structure
6. Add color coding for span status

## Acceptance Criteria

- [ ] `vibes observe traces` shows live traces
- [ ] Session filter works correctly
- [ ] Agent filter works correctly
- [ ] Level filter works correctly
- [ ] `vibes observe config` shows settings
- [ ] Config updates persist
