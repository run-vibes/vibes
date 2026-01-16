---
id: m44-feat-01
title: Swarm view layout
type: feat
status: in-progress
priority: high
epics: [tui]
depends: [m41-feat-03]
estimate: 3h
milestone: 44-tui-swarm-visualization
---

# Swarm View Layout

## Summary

Implement the basic `SwarmView` struct and layout rendering. This provides the container for swarm visualization, including the header, agent grid area, and footer with keybindings.

## Features

### SwarmView Struct

```rust
pub struct SwarmView {
    pub swarm_id: SwarmId,
    pub agents: Vec<AgentId>,
    pub selected_index: usize,
}

impl SwarmView {
    pub fn new(swarm_id: SwarmId) -> Self;
    pub fn select_next(&mut self);
    pub fn select_prev(&mut self);
    pub fn selected_agent(&self) -> Option<AgentId>;
}
```

### Layout Regions

```
┌─ Swarm: <name> ─────────────────────────────────┐
│ Strategy: <strategy>   Status: <status>         │  <- Header
│ Task: <task description>                        │
├─────────────────────────────────────────────────┤
│                                                 │
│     (Agent cards rendered here)                 │  <- Agent Grid
│                                                 │
├─────────────────────────────────────────────────┤
│ [Enter] Agent detail  [m] Merge results  [Esc] Back│ <- Footer
└─────────────────────────────────────────────────┘
```

### View Trait Implementation

```rust
impl View for SwarmView {
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme);
    fn handle_key(&mut self, key: KeyEvent) -> ViewAction;
    fn title(&self) -> &str;
}
```

### Navigation Integration

- Add `View::Swarm(SwarmId)` variant to the View enum (if not already present)
- Navigate from Dashboard when selecting a swarm
- Enter key on agent card navigates to Agent view
- Esc returns to previous view

## Implementation

1. Create `src/views/swarm.rs` in vibes-tui
2. Add `SwarmView` struct with selection state
3. Implement layout using ratatui's `Layout::vertical`
4. Render header block with swarm name, strategy, status
5. Render footer block with keybinding hints
6. Implement `handle_key` for j/k navigation and Enter/Esc
7. Register view in ViewStack navigation
8. Add tests for layout and selection logic

## Acceptance Criteria

- [ ] `SwarmView` struct created with selection tracking
- [ ] Layout renders header, agent area, and footer regions
- [ ] j/k keys navigate agent selection
- [ ] Enter navigates to selected agent detail
- [ ] Esc returns to previous view
- [ ] Swarm name, strategy, and status display in header
- [ ] Footer shows contextual keybindings
- [ ] Theme colors applied consistently
