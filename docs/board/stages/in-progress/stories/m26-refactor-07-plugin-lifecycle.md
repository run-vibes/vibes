---
id: R002
title: Refactor: Add Plugin `on_ready()` Lifecycle Method
type: refactor
status: done
priority: medium
epics: [core, cli, plugin-system]
depends: []
estimate:
created: 2026-01-03
updated: 2026-01-07
milestone: 26-assessment-framework
---

# Refactor: Add Plugin `on_ready()` Lifecycle Method

## Problem

The assessment consumer was tightly coupled to `vibes-groove` internals. This meant:

1. `vibes-server` depended on `vibes-groove` internals
2. The groove plugin wasn't self-contained
3. Plugin enable/disable required modifying server code

## Solution

Implemented a callback-based pattern where:

1. Server's plugin consumer calls `dispatch_raw_event()` for each event
2. This invokes `on_event()` on each loaded plugin
3. Plugins return `PluginAssessmentResult` values
4. Server broadcasts results to WebSocket clients

This is cleaner than having plugins manage their own event consumers - the server owns the event loop, plugins just respond to callbacks.

Also added:
- `on_ready()` lifecycle method for post-initialization setup
- `event_id` field to `PluginAssessmentResult` for UI timestamp display
- Renamed `consumers/assessment.rs` → `consumers/plugin.rs` to reflect generic purpose

## Tasks

- [x] Add `on_ready()` method to `Plugin` trait with default no-op
- [x] Implement callback pattern via `dispatch_raw_event()` → `on_event()`
- [x] Add `event_id` to `PluginAssessmentResult` for UI display
- [x] Remove `vibes-groove` dependency from `vibes-server`
- [x] Rename `consumers/assessment.rs` → `consumers/plugin.rs`
- [x] Update frontend to transform wire format correctly

## Acceptance Criteria

- [x] `vibes-server` has no knowledge of groove internals
- [x] Events flow to plugins via callback pattern
- [x] Existing functionality unchanged (assessment events still flow to UI)
- [x] `just pre-commit` passes
