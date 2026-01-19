---
id: 45-customizable-themes
title: Customizable Themes
status: in-progress
epics: [tui]
---

# Customizable Themes

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

| # | Story | Description | Status |
|---|-------|-------------|--------|
| 1 | [FEAT0186](../../../../stages/done/stories/[FEAT][0186]-theme-config-loader.md) | Theme config file loader | done |
| 2 | [FEAT0187](../../../../stages/done/stories/[FEAT][0187]-builtin-theme-variants.md) | Built-in theme variants | done |
| 3 | [FEAT0188](../../../../stages/done/stories/[FEAT][0188]-runtime-theme-switching.md) | Runtime theme switching | done |
| 4 | [FEAT0189](../../../../stages/done/stories/[FEAT][0189]-theme-preview-settings.md) | Theme preview in settings | done |

## Progress

**Requirements:** 0/0 verified
**Stories:** 4/4 complete

