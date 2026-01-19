# ${MILESTONE_TITLE} — Software Requirements Specification

> ${MILESTONE_GOAL}

<!--
Template: SRS.md (Milestone-level Software Requirements Specification)
Purpose: Define requirements with verification criteria for a milestone
Location: docs/board/epics/${EPIC_NAME}/milestones/${MILESTONE_ID}/SRS.md

Placeholders:
  ${MILESTONE_TITLE} - Human-readable milestone title
  ${MILESTONE_GOAL} - One-line goal
  ${EPIC_NAME} - Parent epic directory name
-->

**Epic:** [${EPIC_TITLE}](../PRD.md)
**Status:** backlog

## Scope

<!-- What this milestone delivers -->

## Requirements

### Functional Requirements

| ID | Requirement | Source | Verification |
|----|-------------|--------|--------------|
| SRS-01 | ${REQUIREMENT_1} | FR-01 | ${VERIFICATION_1} |
| SRS-02 | ${REQUIREMENT_2} | FR-02 | ${VERIFICATION_2} |

### Non-Functional Requirements

| ID | Requirement | Source | Verification |
|----|-------------|--------|--------------|
| SRS-NFR-01 | ${NFR_REQUIREMENT_1} | NFR-01 | ${NFR_VERIFICATION_1} |

## Stories

| Story | Requirements | Status |
|-------|--------------|--------|
| [${STORY_ID}](${STORY_LINK}) | SRS-01 | backlog |

## Traceability

- **Source:** Epic PRD requirements
- **Implements:** Stories
- **Verified by:** `just verify story <ID>`
