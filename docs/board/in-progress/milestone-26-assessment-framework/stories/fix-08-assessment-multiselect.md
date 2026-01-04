---
created: 2026-01-03
status: pending
---

# Fix: Assessment Page Multi-Select Bug

> **For Claude:** Use superpowers:systematic-debugging to trace the issue.

## Problem

Clicking on an assessment event row sometimes selects multiple adjacent rows (e.g., a medium and a lightweight event). This is the same class of bug that was fixed in the firehose (commit `1eec252`).

## Root Cause Hypothesis

Assessment uses multiple Iggy topics (lightweight, medium, heavy). Each topic assigns offsets independently. If the WebSocket is returning offsets instead of globally unique event IDs, two events from different topics could share the same offset value, causing both to be selected when clicking one.

## Investigation Steps

1. Check what the assessment WebSocket endpoint returns (offset vs event_id)
2. Verify assessment events have unique UUIDv7 event_ids
3. Trace from backend event generation to frontend selection

## Likely Fix

Ensure assessment WebSocket messages use `event_id` (UUIDv7) as the unique identifier, not partition offsets. The frontend already uses `event.event_id` in `toDisplayEvent()`, so the fix is likely in the backend WebSocket serialization.

## Tasks

- [ ] Reproduce the bug (click assessment rows, observe multi-select)
- [ ] Trace assessment WebSocket message format
- [ ] Compare with firehose fix (commit `1eec252`)
- [ ] Apply same UUIDv7 event_id pattern if needed
- [ ] Add test case for unique selection

## Acceptance Criteria

- [ ] Clicking any assessment row selects only that row
- [ ] Selection works correctly across all tiers (lightweight, medium, heavy)
- [ ] No regression in firehose selection behavior
- [ ] `just pre-commit` passes
