---
created: 2026-01-01
status: pending
---

# Chore: Assessment Framework Integration Testing

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Goal

Validate the full assessment pipeline works end-to-end, from event emission through consumer processing to CLI output.

## Context

With EventLog migration complete and processor wired up, we need to verify:
1. Events flow from vibes-server through Iggy to groove consumers
2. Assessment processing produces expected signals
3. CLI commands show real assessment data

## Tasks

### Task 1: Test Event Flow

**Steps:**
1. Start vibes server with Iggy
2. Emit test events via the server
3. Verify assessment consumer receives them
4. Check events appear in Iggy topics

**Verification:**
- Events logged by consumer
- No errors in server logs

### Task 2: Test Assessment Processing

**Steps:**
1. Send events that should trigger lightweight signals
2. Verify LightweightEvents emitted
3. Send events that should trigger circuit breaker
4. Verify intervention triggered (or logged)

**Verification:**
- Assessment log contains expected events
- Signal detection working

### Task 3: Test CLI Commands

**Steps:**
1. Run `vibes groove assess status`
2. Verify real data displayed (not hardcoded)
3. Run `vibes groove assess history`
4. Verify session data from actual assessments

**Current state:** CLI commands show hardcoded dummy data. After processor wiring, they should query actual assessment state.

### Task 4: Document Findings

**Steps:**
1. Note any issues discovered
2. Create follow-up stories for bugs/gaps
3. Update milestone status

## Acceptance Criteria

- [ ] Can trace event from emission to consumer processing
- [ ] Assessment signals detected correctly
- [ ] CLI shows real assessment data
- [ ] No silent failures in pipeline
- [ ] Known issues documented
