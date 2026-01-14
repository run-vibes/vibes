---
id: m39-feat-05
title: vibes eval study CLI commands
type: feat
status: in-progress
priority: medium
epics: [evals]
depends: [m39-feat-04]
estimate: 3h
milestone: 39-eval-core
---

# vibes eval study CLI commands

## Summary

Add CLI commands for managing longitudinal studies. Users can start, stop, and monitor studies from the command line.

## Features

### Commands

```
vibes eval study start <name>         # Start longitudinal study
vibes eval study stop <id>            # Stop study
vibes eval study status               # Current study status
vibes eval study list                 # List all studies
vibes eval study checkpoint <id>      # Force checkpoint now
vibes eval study report <id>          # Generate summary report
```

### Start Command

```
$ vibes eval study start "weekly-performance" --period weeks:2

Started longitudinal study: weekly-performance
ID: 019abc12-...
Period: 2 weeks
Checkpoint interval: 1 hour

Tracking metrics:
  - Session success rate
  - First attempt success rate
  - Cost per successful task
  - Learning effectiveness
```

### Status Command

```
$ vibes eval study status

Active Studies
──────────────────────────────────────────────────────────────────
ID                   NAME                    STARTED      CHECKPOINTS
019abc12-...         weekly-performance      2 days ago   48

Latest Checkpoint (1 hour ago):
  Sessions completed: 24
  Success rate: 87.5%
  Avg iterations: 1.4
  Cost per task: $0.12
```

### Report Command

```
$ vibes eval study report 019abc12

Study Report: weekly-performance
Period: Jan 1 - Jan 14, 2026
────────────────────────────────────────────────────────────────────

Summary:
  Total sessions: 156
  Success rate: 85.2% (+3.1% from baseline)
  Cost: $18.45

Trends:
  ↑ First attempt success: 72% → 81%
  ↑ Learning effectiveness: 65% → 78%
  → Avg iterations: stable at 1.4

Insights:
  - Groove learnings improving success rate
  - Tool efficiency improving over time
```

## Implementation

**Note:** Commands use the event-sourced `StudyManager` from m39-feat-04.

1. Add `eval` subcommand to `vibes-cli`
2. Add `study` subcommand with operations
3. Implement commands using `StudyManager`:
   - `start` → `StudyManager::create_study()` + `start_study()` (emits events)
   - `stop` → `StudyManager::stop_study()` (emits event)
   - `status`, `list` → `StudyManager::get_study()`, `list_studies()` (reads projection)
   - `checkpoint` → `StudyManager::record_checkpoint()` (emits event)
   - `report` → Query projection, format results
4. Add WebSocket messages for study operations
5. Format output with consistent styling

## Acceptance Criteria

- [ ] `vibes eval study start` creates study
- [ ] `vibes eval study stop` finalizes study
- [ ] `vibes eval study status` shows active studies
- [ ] `vibes eval study report` generates summary
- [ ] Error messages are clear and helpful
- [ ] Output formatting is consistent
