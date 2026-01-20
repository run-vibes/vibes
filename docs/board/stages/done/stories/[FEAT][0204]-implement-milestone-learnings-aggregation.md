---
id: FEAT0204
title: Implement milestone learnings aggregation
type: feat
status: done
priority: high
scope: coherence-verification/05-learnings-capture
depends: [FEAT0203]
estimate: 2h
created: 2026-01-19
---

# Implement milestone learnings aggregation

## Summary

Modify `just board done-milestone` to aggregate story learnings and prompt for synthesis.

## Acceptance Criteria

- [x] `just board done-milestone <id>` collects learnings from all stories in milestone
- [x] Creates `LEARNINGS.md` in milestone directory
- [x] Aggregates story learnings under "## Story Learnings"
- [x] Prompts for milestone-level synthesis insights
- [x] Synthesis uses ML001, ML002 format
- [x] Milestone still completes if synthesis skipped

## Implementation Notes

**SRS Requirements:** SRS-04, SRS-05

**Files:**
- Modify: `.justfiles/board.just` (done-milestone recipe)

See [DESIGN.md](../../epics/coherence-verification/milestones/05-learnings-capture/DESIGN.md) for LEARNINGS.md format.


## Learnings

### L001: Aggregation extracts and combines learnings from multiple st

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Story completion reflection |
| **Insight** | **What went well:** Aggregation extracts and combines learnings from multiple stories | **Harder than expected:** Finding the right sed patterns for extraction | **Would do differently:** Consider using a dedicated parsing library |
| **Suggested Action** | Consider using a dedicated parsing library |
| **Applies To** | (to be determined) |
| **Applied** | |
