---
created: 2026-01-03
status: done
---

# Feature: Support All Claude Code Hooks

> **For Claude:** Standard implementation task.

## Problem

vibes only supports a subset of Claude Code hooks. Missing hooks means we can't capture all session events for assessment and learning.

## Current vs Required

| Hook | Supported | Response Type |
|------|-----------|---------------|
| `PreToolUse` | Yes | Can block/modify |
| `PostToolUse` | Yes | Fire-and-forget |
| `Stop` | Yes | Fire-and-forget |
| `SessionStart` | Yes | Can inject context |
| `UserPromptSubmit` | Yes | Can inject context |
| `PermissionRequest` | Yes | Can block/modify |
| `Notification` | Yes | Fire-and-forget |
| `SubagentStop` | Yes | Fire-and-forget |
| `PreCompact` | Yes | Fire-and-forget |
| `SessionEnd` | Yes | Fire-and-forget |

## Changes Made

### 1. Type Definitions (`vibes-core/src/hooks/types.rs`)

Added 5 new variants to `HookType` enum and corresponding data structs:
- `PermissionRequestData` - tool requesting permission
- `NotificationData` - notification title and message
- `SubagentStopData` - subagent ID and reason
- `PreCompactData` - pre-compaction event
- `SessionEndData` - session ending with reason

### 2. Hook Scripts (`vibes-core/src/hooks/scripts/`)

Created shell scripts for each new hook type:
- `permission-request.sh` - uses vibes-hook-inject.sh (can respond)
- `notification.sh` - uses vibes-hook-send.sh (fire-and-forget)
- `subagent-stop.sh` - uses vibes-hook-send.sh
- `pre-compact.sh` - uses vibes-hook-send.sh
- `session-end.sh` - uses vibes-hook-send.sh

### 3. Hook Installer (`vibes-core/src/hooks/installer.rs`)

Updated to register all 10 hooks in Claude Code settings.

### 4. Plugin Handler (`plugins/vibes-groove/src/plugin.rs`)

Updated `on_hook()` to handle all new hook types with appropriate logging.

## Tasks

- [x] Add new variants to `HookType` enum
- [x] Add data structs for each new hook type
- [x] Add variants to `HookEvent` enum
- [x] Create shell scripts for new hooks
- [x] Update hook installer to register new hooks
- [x] Update `GroovePlugin.on_hook()` to handle new types
- [x] Add tests for new hook serialization/deserialization
- [x] Test with Claude Code to verify hooks fire correctly (verified: 283 hook events in Iggy including session_end, notification)

## Acceptance Criteria

- [x] All 10 Claude Code hooks are supported
- [x] Hook events flow through to Iggy event log
- [x] `vibes groove init` installs all hook scripts
- [x] Assessment processor can consume all hook types
- [x] `just pre-commit` passes
