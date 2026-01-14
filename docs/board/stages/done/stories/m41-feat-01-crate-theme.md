---
id: m41-feat-01
title: vibes-tui crate scaffold and theme system
type: feat
status: done
priority: high
epics: [tui]
depends: []
estimate: 3h
milestone: 41-tui-core
---

# vibes-tui Crate Scaffold and Theme System

## Summary

Create the foundation `vibes-tui` crate and implement the CRT-inspired theme system. These are independent pieces that don't depend on each other, grouped because theme is needed before any view can render.

## Features

### Crate Structure

```
vibes-tui/
├── Cargo.toml
└── src/
    ├── lib.rs
    └── theme.rs
```

### Dependencies

```toml
[dependencies]
ratatui = "0.29"
crossterm = "0.28"
tokio = { workspace = true }
```

### Theme Struct

```rust
pub struct Theme {
    pub name: String,

    // Colors
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,

    // Status colors
    pub running: Color,
    pub paused: Color,
    pub completed: Color,
    pub failed: Color,

    // UI elements
    pub border: Color,
    pub selection: Color,
    pub highlight: Color,

    // Text styles
    pub bold: Style,
    pub dim: Style,
    pub italic: Style,
}
```

### Default Theme

CRT phosphor green theme matching the design system:

```rust
pub fn vibes_default() -> Theme {
    Theme {
        name: "vibes".into(),
        bg: Color::Rgb(18, 18, 18),
        fg: Color::Rgb(0, 255, 136),        // Phosphor green #00ff88
        accent: Color::Rgb(0, 200, 255),    // Cyan accent #00c8ff
        success: Color::Rgb(0, 255, 136),
        warning: Color::Rgb(255, 200, 0),
        error: Color::Rgb(255, 85, 85),
        running: Color::Rgb(0, 255, 136),
        paused: Color::Rgb(255, 200, 0),
        completed: Color::Rgb(100, 100, 100),
        failed: Color::Rgb(255, 85, 85),
        border: Color::Rgb(60, 60, 60),
        selection: Color::Rgb(40, 80, 40),
        highlight: Color::Rgb(0, 150, 100),
        // ...styles
    }
}
```

## Implementation

1. Create `vibes-tui/Cargo.toml` with dependencies
2. Add `vibes-tui` to workspace `Cargo.toml`
3. Create `src/lib.rs` with module exports
4. Create `src/theme.rs` with Theme struct and vibes_default()
5. Add unit tests for theme construction

## Acceptance Criteria

- [ ] `cargo build -p vibes-tui` succeeds
- [ ] Crate added to workspace members
- [ ] Theme struct has all fields from epic spec
- [ ] `vibes_default()` returns CRT phosphor theme
- [ ] Colors match design system (#00ff88 phosphor green, #00c8ff cyan)
- [ ] Unit tests pass
