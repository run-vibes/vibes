---
id: FEAT0206
title: Add just learn apply propagation engine
type: feat
status: done
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

- [x] `just learn apply` scans all learnings (stories, milestones, ad-hoc)
- [x] Filters to unapplied learnings only
- [x] For each learning, AI suggests concrete changes to "Applies To" targets
- [x] Shows diff-style preview of suggested changes
- [x] User can accept/reject/edit each suggestion
- [x] Accepted changes applied to target files
- [x] Learning marked as applied with date and targets

## Implementation Notes

**SRS Requirements:** SRS-08, SRS-09

**Files:**
- Modify: `.justfiles/learn.just`
- Create: `verification/ai/lib/learnings.ts` - Learning scanner and parser
- Create: `verification/ai/learn-apply.ts` - Main entry point script

See [DESIGN.md](../../epics/coherence-verification/milestones/05-learnings-capture/DESIGN.md) for propagation workflow.

## Learnings

### L001: Apply command provides a clean interface for propagating lea

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Story completion reflection |
| **Insight** | **What went well:** Apply command provides a clean interface for propagating learnings | **Harder than expected:** Matching learning IDs with their target files | **Would do differently:** Add a preview mode to see what would change |
| **Suggested Action** | Add a preview mode to see what would change |
| **Applies To** | (to be determined) |
| **Applied** | |

<!-- No learnings captured for this story -->

