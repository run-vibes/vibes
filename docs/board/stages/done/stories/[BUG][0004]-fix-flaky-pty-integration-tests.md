---
id: BUG0004
title: Fix flaky PTY integration tests
type: bug
status: done
priority: medium
epics: []
depends: []
estimate: 2h
created: 2026-01-08
updated: 2026-01-08
---

# Fix flaky PTY integration tests

## Summary

Two PTY integration tests in `vibes-server` fail intermittently:

1. **`ctrl_c_terminates_pty_process`** - Timeout waiting for process termination
2. **`session_id_mismatch_regression`** - Output mismatch: expected "test", got "tes"

These tests pass when run individually but fail sporadically in full test suite runs.

## Root Cause

PTY output can arrive in multiple WebSocket messages (e.g., "tes" + "t\n"). The original `expect_pty_output` helper returned only the first message, causing test failures when the expected string was split across messages.

## Solution

Added `expect_pty_output_containing` helper that accumulates output until the expected content is found, making tests robust against message splitting.

## Acceptance Criteria

- [x] Both tests pass reliably in full test suite runs
- [x] Tests pass when run with `cargo nextest run` (parallel execution)
- [x] No timing-dependent failures over 10 consecutive runs
