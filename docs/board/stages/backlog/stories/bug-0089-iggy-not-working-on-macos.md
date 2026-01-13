---
id: BUG0089
title: Iggy doesn't return events on macOS
type: bug
status: pending
priority: high
epics: [cross-platform]
depends: []
estimate: 4h
created: 2026-01-12
---

# Iggy doesn't return events on macOS

## Summary

Iggy event streaming doesn't work on macOS - no events are returned. The daemon appears to start but events are never delivered.

## Symptoms

- Iggy server starts without errors
- Events are sent but never received
- No errors in logs
- Works correctly on Linux

## Compilation Warnings

The following warnings appear when compiling on macOS, suggesting Linux-specific code paths:

```
warning: constant `MIN_MEMLOCK_BYTES` is never used
warning: function `format_bytes` is never used
warning: constant `MEMLOCK_HELP` is never used
```

These constants/functions are likely related to Linux-specific memory locking (io_uring) that doesn't apply to macOS.

## Investigation Tasks

### Task 1: Diagnose the Issue

**Steps:**
1. Check if Iggy uses io_uring on Linux (not available on macOS)
2. Verify what async runtime/backend Iggy uses on macOS
3. Check Iggy's platform-specific code paths
4. Add debug logging to trace event flow on macOS

### Task 2: Fix or Workaround

**Steps:**
1. If io_uring related: ensure fallback to kqueue/poll on macOS
2. If configuration issue: update Iggy config for macOS compatibility
3. If upstream bug: report to Iggy project and implement workaround
4. Suppress or fix the unused code warnings with `#[cfg(target_os = "linux")]`

### Task 3: Verify Fix

**Steps:**
1. Test event sending on macOS
2. Test event receiving on macOS
3. Verify no regressions on Linux
4. Update any platform-specific documentation

## Acceptance Criteria

- [ ] Events flow correctly on macOS
- [ ] No unused code warnings on macOS
- [ ] Linux functionality unchanged
- [ ] Tests pass on both platforms
