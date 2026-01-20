---
id: BUG0130
title: "Fix: Assessment Page Multi-Select Bug"
type: fix
status: done
priority: medium
scope: web-ui/03-infinite-event-stream
depends: []
estimate:
created: 2026-01-03
---

# Fix: Assessment Page Multi-Select Bug

## Problem

Clicking on an assessment event row sometimes selects multiple adjacent rows (e.g., a medium and a lightweight event). This is the same class of bug that was fixed in the firehose (commit `1eec252`).

## Root Cause

When processing a single raw event, the `SyncAssessmentProcessor` could generate both a lightweight and checkpoint result sharing the same `event_id`. Since the frontend uses `event_id` for selection, clicking one would match both.

## Solution

Append result type suffix to each assessment result's event_id:
- Lightweight results: `{uuid}-lightweight`
- Checkpoint results: `{uuid}-checkpoint`

Also updated `extractTimestampFromUuidv7()` to strip suffixes when parsing timestamps.

## Tasks

- [x] Trace assessment WebSocket message format
- [x] Identify that lightweight + checkpoint share same event_id
- [x] Add unique suffix per result type in sync_processor.rs
- [x] Update frontend uuidv7 extraction to handle suffixes
- [x] Add regression tests for unique IDs

## Acceptance Criteria

- [x] Clicking any assessment row selects only that row
- [x] Selection works correctly across all tiers (lightweight, medium, heavy)
- [x] No regression in firehose selection behavior
- [x] `just pre-commit` passes
