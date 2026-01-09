---
id: FEAT0023
title: Error recovery detector
type: feat
status: done
priority: high
epics: [plugin-system]
depends: []
estimate: 2h
created: 2026-01-08
updated: 2026-01-09
milestone: 30-learning-extraction
---

# Error recovery detector

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Detect tool failure to fix sequences to extract error recovery learnings.

## Context

When Claude encounters an error (compilation failure, test failure, command error) and successfully recovers, this pattern is worth learning. The detector identifies failure → fix sequences in transcripts. See [design.md](../../../milestones/30-learning-extraction/design.md).

## Tasks

### Task 1: Define error recovery types

**Files:**
- Create: `plugins/vibes-groove/src/extraction/patterns/error_recovery.rs`

**Steps:**
1. Define `ErrorType` enum:
   ```rust
   pub enum ErrorType {
       CompilationError,
       TestFailure,
       CommandError,
       RuntimeError,
       Other(String),
   }
   ```
2. Define `ErrorRecoveryCandidate` struct:
   - Error type
   - Failed tool call
   - Recovery tool call(s)
   - Error message
   - Recovery strategy
   - Confidence
3. Add module to patterns/mod.rs
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): add error recovery types`

### Task 2: Implement failure detection

**Files:**
- Modify: `plugins/vibes-groove/src/extraction/patterns/error_recovery.rs`

**Steps:**
1. Create `ErrorRecoveryDetector` struct
2. Implement failure detection patterns:
   - Bash tool with non-zero exit code
   - Output containing "error:", "Error:", "ERROR"
   - Compilation errors (rustc, tsc, etc.)
   - Test failures ("FAILED", "FAIL")
3. Extract error details:
   - Error message
   - File/line if available
   - Error type classification
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): implement failure detection`

### Task 3: Implement recovery detection

**Files:**
- Modify: `plugins/vibes-groove/src/extraction/patterns/error_recovery.rs`

**Steps:**
1. Implement recovery detection:
   - Find successful tool call after failure
   - Same or related file/command
   - Within N turns of failure
2. Extract recovery strategy:
   - What changed between failure and success
   - Files modified
   - Commands run
3. Calculate confidence:
   - Higher if same file modified
   - Higher if error message addressed
   - Lower if many turns between failure and success
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): implement recovery detection`

### Task 4: Implement PatternDetector trait

**Files:**
- Modify: `plugins/vibes-groove/src/extraction/patterns/error_recovery.rs`

**Steps:**
1. Implement `detect()`:
   - Scan transcript for failures
   - For each failure, look for recovery
   - Filter by minimum confidence
2. Implement `to_learning()`:
   - Category: `ErrorRecovery`
   - Description: "When {error_type} occurs, {recovery_strategy}"
   - Pattern: error context for matching
   - Source: session ID, tool call IDs
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): implement ErrorRecoveryDetector`

### Task 5: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/extraction/patterns/error_recovery.rs`

**Steps:**
1. Create test transcripts:
   - Compilation error → fix → success
   - Test failure → code change → pass
   - Command error → different command → success
2. Write tests:
   - Test failure detection by type
   - Test recovery matching
   - Test confidence calculation
   - Test learning extraction
3. Run: `cargo test -p vibes-groove extraction::patterns::error_recovery`
4. Commit: `test(groove): add error recovery tests`

## Acceptance Criteria

- [ ] Detects compilation errors
- [ ] Detects test failures
- [ ] Detects command errors
- [ ] Matches failures with recoveries
- [ ] Extracts meaningful recovery strategies
- [ ] Calculates reasonable confidence scores
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0023`
3. Commit, push, and create PR
