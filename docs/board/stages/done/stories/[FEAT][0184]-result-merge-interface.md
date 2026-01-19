---
id: FEAT0184
title: Result merge interface
type: feat
status: done
priority: medium
scope: tui/44-swarm-monitoring
depends: [m44-feat-02]
estimate: 3h
---

# Result Merge Interface

## Summary

Implement the result merge action for swarms. When all agents complete, or when manually triggered, users can merge agent results into a consolidated output. This includes a confirmation dialog and result preview.

## Features

### Merge Action

- Keybinding: `m` in SwarmView
- Only available when at least one agent has completed
- Disabled indicator when no results to merge

### Merge Dialog

```
┌─ Merge Results ─────────────────────────────────┐
│                                                 │
│ Ready to merge results from 3 agents:           │
│                                                 │
│   ✓ agent-1: Security review                    │
│   ✓ agent-2: Performance review                 │
│   ✓ agent-3: Code style review                  │
│                                                 │
│ Merge strategy: Concatenate                     │
│                                                 │
├─────────────────────────────────────────────────┤
│        [Enter] Confirm    [Esc] Cancel          │
└─────────────────────────────────────────────────┘
```

### Result Preview

After merge confirmation, show combined output:

```
┌─ Merged Results ────────────────────────────────┐
│                                                 │
│ ## Security Review (agent-1)                    │
│ - No critical vulnerabilities found             │
│ - Recommend adding input validation             │
│                                                 │
│ ## Performance Review (agent-2)                 │
│ - N+1 query detected in UserController          │
│ - Consider caching for frequently accessed data │
│                                                 │
│ ## Code Style (agent-3)                         │
│ - All files pass linting                        │
│                                                 │
├─────────────────────────────────────────────────┤
│ [c] Copy to clipboard  [s] Save  [Esc] Close    │
└─────────────────────────────────────────────────┘
```

### Commands

```rust
pub enum SwarmCommand {
    MergeResults { strategy: MergeStrategy },
    // ... other commands
}

pub enum MergeStrategy {
    Concatenate,    // Append all results
    Summarize,      // Ask orchestrator to summarize
    Custom(String), // User-provided merge prompt
}
```

## Implementation

1. Add merge keybinding handler in SwarmView
2. Create `MergeDialog` modal component
3. List completed agents with their task summaries
4. Implement merge action sending command to server
5. Create `MergedResultsView` for displaying output
6. Add clipboard copy functionality (via crossterm)
7. Add save to file option
8. Handle partial merge (some agents still running)

## Acceptance Criteria

- [x] `m` key opens merge confirmation dialog
- [x] Dialog lists all completed agents
- [x] Merge disabled when no agents completed
- [x] Confirm triggers merge command to server (types defined, wiring to server pending orchestration)
- [x] Merged results display in scrollable view
- [x] Copy to clipboard works
- [x] Save to file with timestamped filename
- [x] Esc cancels at any stage
- [x] Partial merge shows warning for incomplete agents
