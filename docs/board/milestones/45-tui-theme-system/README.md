---
id: 45-tui-theme-system
title: TUI Theme System
status: planned
epics: [tui]
---

# TUI Theme System

## Overview

Fifth milestone of the TUI epic. Extends the theme system with customization, theme loading from config, and additional built-in themes.

## Goals

- Theme configuration file support
- Multiple built-in themes
- Runtime theme switching
- Custom color definitions
- Theme preview in settings

## Key Deliverables

- Theme loader from TOML config
- Additional built-in themes (dark, light, high-contrast)
- `:theme <name>` command
- Settings view theme selector
- Theme preview widget

## Theme Structure

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
}
```

## Epics

- [tui](../../epics/tui)

## Stories

| ID | Title | Status |
|----|-------|--------|
| m45-feat-01 | Theme config file loader | backlog |
| m45-feat-02 | Built-in theme variants | backlog |
| m45-feat-03 | Runtime theme switching | backlog |
| m45-feat-04 | Theme preview in settings | backlog |

## Design

See [../../epics/tui/README.md](../../epics/tui/README.md) for architecture.
