---
id: BUG0004
title: Fix flaky PTY integration tests
type: bug
status: backlog
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

## Failing Tests

**File:** `vibes-server/tests/pty_integration.rs`

### ctrl_c_terminates_pty_process
- **Error:** Timeout waiting for PTY process to terminate after Ctrl+C
- **Line:** ~line 120-140

### session_id_mismatch_regression
- **Error:** `Expected output to contain 'test', got: "tes". This may indicate a session ID mismatch bug!`
- **Line:** 161

## Acceptance Criteria

- [ ] Both tests pass reliably in full test suite runs
- [ ] Tests pass when run with `cargo nextest run` (parallel execution)
- [ ] No timing-dependent failures over 10 consecutive runs

## Implementation Notes

Possible causes:
- Race conditions in PTY output buffering
- Insufficient timeouts for process termination
- Session state pollution between tests
- Output buffer not being fully flushed before assertion

Investigation approach:
1. Run tests with `--nocapture` to see timing
2. Check if tests share any mutable state
3. Add longer timeouts or explicit waits for output completion
4. Consider using test isolation (separate PTY instances)
