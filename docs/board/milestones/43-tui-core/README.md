---
id: 43-tui-core
title: TUI Core
status: planned
epics: [tui]
---

# TUI Core

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

## Design

See [../../epics/tui/README.md](../../epics/tui/README.md) for architecture.
