---
id: 41-terminal-ui-framework
title: Terminal UI Framework
status: done
epics: [tui]
---

# Terminal UI Framework

## Overview

First milestone of the TUI epic. Establishes the foundation for the terminal user interface: app structure, view system, navigation, and keybindings.

## Goals

- ratatui-based application structure
- View stack with push/pop navigation
- Vim-style keybindings (j/k/h/l navigation)
- Theme system with CRT-inspired default
- WebSocket client for server communication

## Key Deliverables

- `vibes-tui` crate
- `App` struct with state, views, keybindings
- `View` enum and `ViewStack`
- `KeyBindings` with global and view-specific bindings
- `Theme` struct with vibes default theme
- `vibes tui` CLI command to launch

## Epics

- [tui](../../epics/tui)

## Stories

| # | Story | Description | Status |
|---|-------|-------------|--------|
| 1 | [FEAT0169](../../../../stages/done/stories/[FEAT][0169]-crate-theme.md) | vibes-tui crate scaffold and theme system | done |
| 2 | [FEAT0170](../../../../stages/done/stories/[FEAT][0170]-app-event-loop.md) | App struct and event loop | done |
| 3 | [FEAT0171](../../../../stages/done/stories/[FEAT][0171]-views-viewstack.md) | Views and ViewStack navigation | done |
| 4 | [FEAT0172](../../../../stages/done/stories/[FEAT][0172]-keybindings.md) | KeyBindings system | done |
| 5 | [FEAT0173](../../../../stages/done/stories/[FEAT][0173]-cli-integration.md) | CLI command and WebSocket integration | done |

## Progress

**Requirements:** 0/0 verified
**Stories:** 5/5 complete

