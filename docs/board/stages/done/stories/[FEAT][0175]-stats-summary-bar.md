---
id: FEAT0175
title: Stats summary bar
type: feat
status: done
priority: high
scope: tui/42-terminal-dashboard
depends: [m41-feat-05]
estimate: 2h
---

# Stats Summary Bar

## Summary

Implement the stats summary bar that displays at the top of the dashboard. Shows aggregate metrics: active session count, running agent count, and total cost.

## Features

### Metrics Displayed

- **Sessions**: Count of active sessions (with status breakdown)
- **Agents**: Count of running agents
- **Cost**: Total accumulated cost in USD

### Visual Layout

```
┌─ vibes ─────────────────────────────────────────────┐
│ Sessions: 3 active   Agents: 7 running   Cost: $12  │
└─────────────────────────────────────────────────────┘
```

### Color Coding

- Session count uses accent color
- Agent count uses running status color (green)
- Cost uses warning color when above threshold

## Implementation

### Widget Structure

```rust
pub struct StatsBarWidget {
    session_count: u32,
    agent_count: u32,
    total_cost: f64,
}

impl StatsBarWidget {
    pub fn update(&mut self, state: &AppState);
}
```

### Steps

1. Create `src/widgets/stats_bar.rs`
2. Define `StatsBarWidget` with metric fields
3. Implement `Widget` trait for single-line rendering
4. Add update method to compute stats from AppState
5. Format cost with dollar sign and 2 decimal places
6. Add to dashboard layout as top bar
7. Add unit tests for formatting

## Acceptance Criteria

- [x] Bar displays session count with "active" label
- [x] Bar displays agent count with "running" label
- [x] Bar displays cost formatted as "$X.XX"
- [x] Metrics update when AppState changes
- [x] Layout fits in single line with proper spacing
- [x] Colors match theme system
- [x] Unit tests pass
