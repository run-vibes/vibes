---
id: FEAT0207
title: Add just learn list command
type: feat
status: done
priority: low
scope: coherence-verification/05-learnings-capture
depends: [FEAT0203]
estimate: 1h
created: 2026-01-19
---

# Add just learn list command

## Summary

Create `just learn list` command to show all learnings with their applied/pending status.

## Acceptance Criteria

- [x] `just learn list` shows all learnings from all sources
- [x] Groups by source (stories, milestones, ad-hoc)
- [x] Shows status: applied / pending / reviewed
- [x] `just learn list --pending` filters to pending only
- [x] `just learn list --category <cat>` filters by category
- [x] Output formatted as table

## Implementation Notes

**SRS Requirements:** SRS-10

**Files:**
- Modify: `.justfiles/learn.just`

