# Formal Planning Process — Software Requirements Specification

> Establish traceable document hierarchy: VISION → PRD → SRS → DESIGN → Stories → Verification

**Epic:** [Coherence Verification](../../README.md)
**Status:** in-progress

## Scope

Refactor the planning documentation structure to follow formal software engineering practices. Create clear traceability from product vision through implementation to verification.

## Requirements

### Document Structure Requirements

| ID | Requirement | Source | Verification |
|----|-------------|--------|--------------|
| SRS-01 | Product vision document exists at `docs/VISION.md` | FR-01 | file exists |
| SRS-02 | Each epic directory contains `PRD.md` with requirements | FR-02 | glob: `epics/*/PRD.md` |
| SRS-03 | Each milestone directory contains `SRS.md` | FR-03 | glob: `epics/*/milestones/*/SRS.md` |
| SRS-04 | Each milestone directory contains `DESIGN.md` | FR-04 | glob: `epics/*/milestones/*/DESIGN.md` |
| SRS-05 | README.md files provide navigation and status summaries | FR-05 | manual review |

### Tooling Requirements

| ID | Requirement | Source | Verification |
|----|-------------|--------|--------------|
| SRS-06 | `just board new epic` creates directory with README.md + PRD.md | FR-06 | manual |
| SRS-07 | `just board new milestone` creates directory with README.md + SRS.md + DESIGN.md | FR-07 | manual |
| SRS-08 | `just board generate` updates epic README with milestone progress | FR-08 | manual |
| SRS-09 | `just board generate` updates milestone README with story progress | FR-09 | manual |
| SRS-10 | Story state changes update milestone README | FR-10 | manual |
| SRS-11 | Milestone state changes update epic README | FR-11 | manual |

### Migration Requirements

| ID | Requirement | Source | Verification |
|----|-------------|--------|--------------|
| SRS-12 | `docs/PRD.md` renamed to `docs/VISION.md` | FR-12 | file exists |
| SRS-13 | All existing epics have PRD.md (migrated from README content) | FR-13 | glob check |
| SRS-14 | All existing milestones have SRS.md | FR-14 | glob check |
| SRS-15 | All existing `design.md` renamed to `DESIGN.md` | FR-15 | glob check |
| SRS-16 | All existing `implementation.md` content migrated to SRS.md | FR-16 | manual |

### Non-Functional Requirements

| ID | Requirement | Source | Verification |
|----|-------------|--------|--------------|
| SRS-NFR-01 | Documents follow consistent templates | NFR-01 | manual review |
| SRS-NFR-02 | README progress updates are idempotent | NFR-02 | run twice, same result |
| SRS-NFR-03 | Migration preserves git history where possible | NFR-03 | git log check |

## Stories

| Story | Requirements | Status |
|-------|--------------|--------|
| [CHORE0143](../../../../stages/backlog/stories/[CHORE][0143]-rename-prd-to-vision.md) | SRS-01, SRS-12 | backlog |
| [FEAT0191](../../../../stages/backlog/stories/[FEAT][0191]-create-document-templates.md) | SRS-NFR-01 | backlog |
| [FEAT0192](../../../../stages/backlog/stories/[FEAT][0192]-update-new-epic-command.md) | SRS-06 | backlog |
| [FEAT0193](../../../../stages/backlog/stories/[FEAT][0193]-update-new-milestone-command.md) | SRS-07 | backlog |
| [FEAT0194](../../../../stages/backlog/stories/[FEAT][0194]-update-generate-for-epic-readme.md) | SRS-08, SRS-NFR-02 | backlog |
| [FEAT0195](../../../../stages/backlog/stories/[FEAT][0195]-update-generate-for-milestone-readme.md) | SRS-09, SRS-NFR-02 | backlog |
| [FEAT0196](../../../../stages/backlog/stories/[FEAT][0196]-update-story-commands-for-milestone-sync.md) | SRS-10 | backlog |
| [CHORE0144](../../../../stages/backlog/stories/[CHORE][0144]-migrate-existing-epics.md) | SRS-13, SRS-NFR-03 | backlog |
| [CHORE0145](../../../../stages/backlog/stories/[CHORE][0145]-migrate-existing-milestones.md) | SRS-14, SRS-15, SRS-16, SRS-NFR-03 | backlog |

## Traceability

- **Source:** Coherence Verification epic goal of traceability from planning → execution → verification
- **Verified by:** Glob checks, manual review, `just board generate` output
