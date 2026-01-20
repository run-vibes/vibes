---
id: FEAT0204
title: Implement milestone learnings aggregation
type: feat
status: backlog
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

- [ ] `just board done-milestone <id>` collects learnings from all stories in milestone
- [ ] Creates `LEARNINGS.md` in milestone directory
- [ ] Aggregates story learnings under "## Story Learnings"
- [ ] Prompts for milestone-level synthesis insights
- [ ] Synthesis uses ML001, ML002 format
- [ ] Milestone still completes if synthesis skipped

## Implementation Notes

**SRS Requirements:** SRS-04, SRS-05

**Files:**
- Modify: `.justfiles/board.just` (done-milestone recipe)

See [DESIGN.md](../../epics/coherence-verification/milestones/05-learnings-capture/DESIGN.md) for LEARNINGS.md format.

