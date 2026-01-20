---
id: FEAT0207
title: Add just learn list command
type: feat
status: backlog
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

- [ ] `just learn list` shows all learnings from all sources
- [ ] Groups by source (stories, milestones, ad-hoc)
- [ ] Shows status: applied / pending / reviewed
- [ ] `just learn list --pending` filters to pending only
- [ ] `just learn list --category <cat>` filters by category
- [ ] Output formatted as table

## Implementation Notes

**SRS Requirements:** SRS-10

**Files:**
- Modify: `.justfiles/learn.just`

