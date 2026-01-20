---
id: FEAT0171
title: Views and ViewStack navigation
type: feat
status: done
priority: high
scope: tui/01-terminal-ui-framework
depends: [m41-feat-02]
estimate: 4h
---

# Views and ViewStack Navigation

## Summary

Implement the view abstraction and stack-based navigation model. Views are the primary user interaction surface, and the stack enables natural "drill down and back" navigation patterns like lazygit.

## Features

### View Enum

```rust
pub enum View {
    Dashboard,              // Overview of all activity
    Session(SessionId),     // Single session detail
    Agent(AgentId),         // Single agent detail
    Swarm(SwarmId),         // Swarm visualization
    Models,                 // Model registry
    Observe,                // Observability dashboard
    Evals,                  // Evaluation results
    Settings,               // Configuration
}
```

### ViewStack

```rust
pub struct ViewStack {
    pub current: View,
    pub history: Vec<View>,
}

impl ViewStack {
    pub fn new() -> Self {
        Self {
            current: View::Dashboard,
            history: Vec::new(),
        }
    }

    /// Push new view, saving current to history
    pub fn push(&mut self, view: View) {
        self.history.push(std::mem::replace(&mut self.current, view));
    }

    /// Pop to previous view, returns false if at root
    pub fn pop(&mut self) -> bool {
        if let Some(prev) = self.history.pop() {
            self.current = prev;
            true
        } else {
            false
        }
    }

    /// Replace current view without affecting history
    pub fn replace(&mut self, view: View) {
        self.current = view;
    }

    /// Check if we can go back
    pub fn can_pop(&self) -> bool {
        !self.history.is_empty()
    }
}
```

### ViewRenderer Trait

```rust
pub trait ViewRenderer {
    fn render(&self, frame: &mut Frame, area: Rect, app: &App);
    fn title(&self) -> &str;
}
```

### Placeholder Dashboard

```rust
impl ViewRenderer for DashboardView {
    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        let block = Block::default()
            .title(" vibes ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border));

        let text = Paragraph::new("Dashboard - Coming in Milestone 42")
            .style(Style::default().fg(app.theme.fg))
            .block(block);

        frame.render_widget(text, area);
    }

    fn title(&self) -> &str {
        "Dashboard"
    }
}
```

## Implementation

1. Create `src/views/mod.rs` with View enum
2. Create `src/views/stack.rs` with ViewStack
3. Create `src/views/traits.rs` with ViewRenderer trait
4. Create `src/views/dashboard.rs` with placeholder DashboardView
5. Integrate ViewStack into App struct
6. Wire App::render() to delegate to current view
7. Add unit tests for ViewStack push/pop/replace

## Acceptance Criteria

- [x] View enum has all variants from epic spec
- [x] ViewStack starts with Dashboard as default
- [x] `push()` saves current to history, sets new current
- [x] `pop()` restores previous view from history
- [x] `pop()` returns false when history empty (at root)
- [x] `replace()` changes current without affecting history
- [x] `can_pop()` returns correct state
- [x] Dashboard placeholder renders with title and border
- [x] Unit tests for ViewStack operations pass
