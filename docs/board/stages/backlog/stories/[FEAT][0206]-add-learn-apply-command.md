---
id: FEAT0206
title: Add just learn apply propagation engine
type: feat
status: backlog
priority: medium
scope: coherence-verification/05-learnings-capture
depends: [FEAT0203, FEAT0204, FEAT0205]
estimate: 3h
created: 2026-01-19
---

# Add just learn apply propagation engine

## Summary

Create `just learn apply` command that uses AI to suggest where learnings should be propagated.

## Acceptance Criteria

- [ ] `just learn apply` scans all learnings (stories, milestones, ad-hoc)
- [ ] Filters to unapplied learnings only
- [ ] For each learning, AI suggests concrete changes to "Applies To" targets
- [ ] Shows diff-style preview of suggested changes
- [ ] User can accept/reject/edit each suggestion
- [ ] Accepted changes applied to target files
- [ ] Learning marked as applied with date and targets

## Implementation Notes

**SRS Requirements:** SRS-08, SRS-09

**Files:**
- Modify: `.justfiles/learn.just`

See [DESIGN.md](../../epics/coherence-verification/milestones/05-learnings-capture/DESIGN.md) for propagation workflow.

