---
created: 2026-01-03
status: pending
---

# Feature: Support All Claude Code Hooks

> **For Claude:** Standard implementation task.

## Problem

vibes only supports a subset of Claude Code hooks. Missing hooks means we can't capture all session events for assessment and learning.

## Current vs Required

| Hook | Currently Supported | Response Type |
|------|---------------------|---------------|
| `PreToolUse` | Yes | Can block/modify |
| `PostToolUse` | Yes | Fire-and-forget |
| `Stop` | Yes | Fire-and-forget |
| `SessionStart` | Yes | Can inject context |
| `UserPromptSubmit` | Yes | Can inject context |
| `PermissionRequest` | **No** | Can block/modify |
| `Notification` | **No** | Fire-and-forget |
| `SubagentStop` | **No** | Fire-and-forget |
| `PreCompact` | **No** | Fire-and-forget |
| `SessionEnd` | **No** | Fire-and-forget |

## Changes Required

### 1. Type Definitions (`vibes-core/src/hooks/types.rs`)

Add new variants to `HookType` enum:
```rust
pub enum HookType {
    // existing...
    PermissionRequest,
    Notification,
    SubagentStop,
    PreCompact,
    SessionEnd,
}
```

Add corresponding data structs:
```rust
pub struct PermissionRequestData {
    pub session_id: Option<String>,
    pub tool_name: String,
    pub input: String,
}

pub struct NotificationData {
    pub session_id: Option<String>,
    pub title: String,
    pub message: String,
}

pub struct SubagentStopData {
    pub session_id: Option<String>,
    pub subagent_id: String,
    pub reason: Option<String>,
}

pub struct PreCompactData {
    pub session_id: Option<String>,
}

pub struct SessionEndData {
    pub session_id: Option<String>,
    pub reason: Option<String>,
}
```

### 2. Hook Scripts (`vibes-core/src/hooks/scripts/`)

Create shell scripts for each new hook type.

### 3. Hook Installer (`vibes-core/src/hooks/installer.rs`)

Update to register new hooks in Claude Code settings.

### 4. Plugin Handler (`plugins/vibes-groove/src/plugin.rs`)

Update `on_hook()` to handle new hook types for assessment.

## Tasks

- [ ] Add new variants to `HookType` enum
- [ ] Add data structs for each new hook type
- [ ] Add variants to `HookEvent` enum
- [ ] Create shell scripts for new hooks
- [ ] Update hook installer to register new hooks
- [ ] Update `GroovePlugin.on_hook()` to handle new types
- [ ] Add tests for new hook serialization/deserialization
- [ ] Test with Claude Code to verify hooks fire correctly

## Acceptance Criteria

- [ ] All 10 Claude Code hooks are supported
- [ ] Hook events flow through to Iggy event log
- [ ] `vibes groove init` installs all hook scripts
- [ ] Assessment processor can consume all hook types
- [ ] `just pre-commit` passes
