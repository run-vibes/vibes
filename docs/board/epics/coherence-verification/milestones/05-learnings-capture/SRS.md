# Learnings Capture — Software Requirements Specification

> Capture and propagate learnings from completed work to improve process, architecture, and verification.

**Epic:** [Coherence Verification](../../PRD.md)
**Status:** backlog

## Scope

Add a system to capture learnings at multiple points (story completion, milestone completion, ad-hoc reflection) and propagate them to improve CLAUDE.md, templates, conventions, and code patterns.

## Requirements

### Functional Requirements

| ID | Requirement | Source | Verification |
|----|-------------|--------|--------------|
| SRS-01 | Story template includes `## Learnings` section | FR-06 | file exists |
| SRS-02 | `just board done` prompts for learnings before completing story | FR-06 | manual |
| SRS-03 | Learnings use structured template (category, context, insight, action, applies-to) | FR-06 | unit test |
| SRS-04 | `just board done-milestone` aggregates story learnings into LEARNINGS.md | FR-06 | file exists |
| SRS-05 | Milestone completion prompts for synthesis learnings | FR-06 | manual |
| SRS-06 | `just learn reflect` enables ad-hoc learning capture | FR-06 | manual |
| SRS-07 | Ad-hoc learnings saved to `docs/learnings/YYYY-MM-DD-topic.md` | FR-06 | file exists |
| SRS-08 | `just learn apply` suggests propagation targets for unapplied learnings | FR-06 | manual |
| SRS-09 | User can accept/reject/edit suggested changes | FR-06 | manual |
| SRS-10 | `just learn list` shows all learnings with applied/pending status | FR-06 | manual |

### Non-Functional Requirements

| ID | Requirement | Source | Verification |
|----|-------------|--------|--------------|
| SRS-NFR-01 | Learning capture is optional (can skip if no learnings) | NFR-03 | manual |
| SRS-NFR-02 | AI-assisted prompts guide reflection without being prescriptive | NFR-03 | manual |
| SRS-NFR-03 | Implementation in bash/just (consistent with board tooling) | NFR-03 | code review |

## Stories

| Story | Requirements | Status |
|-------|--------------|--------|
| [CHORE0202](../../../../stages/backlog/stories/[CHORE][0202]-add-learnings-template.md) | SRS-01 | backlog |
| [FEAT0203](../../../../stages/backlog/stories/[FEAT][0203]-implement-story-learning-capture.md) | SRS-02, SRS-03 | backlog |
| [FEAT0204](../../../../stages/backlog/stories/[FEAT][0204]-implement-milestone-learnings-aggregation.md) | SRS-04, SRS-05 | backlog |
| [FEAT0205](../../../../stages/backlog/stories/[FEAT][0205]-add-learn-reflect-command.md) | SRS-06, SRS-07 | backlog |
| [FEAT0206](../../../../stages/backlog/stories/[FEAT][0206]-add-learn-apply-command.md) | SRS-08, SRS-09 | backlog |
| [FEAT0207](../../../../stages/backlog/stories/[FEAT][0207]-add-learn-list-command.md) | SRS-10 | backlog |

## Traceability

- **Source:** PRD FR-06 (Generate verification reports), extended for process improvement
- **Implements:** Stories listed above
- **Verified by:** Manual testing of commands, file existence checks

