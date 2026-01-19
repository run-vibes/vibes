---
id: FEAT0193
title: Update new milestone command
type: feat
status: backlog
priority: high
scope: coherence-verification/03-formal-planning-process
depends: [FEAT0191]
estimate: 2h
created: 2026-01-18
---

# Update New Milestone Command

## Summary

Update `just board new milestone` to create the formal document structure: README.md + SRS.md + DESIGN.md using the new templates.

## Acceptance Criteria

- [ ] `just board new milestone "name" <epic>` creates directory with README.md
- [ ] Command creates SRS.md from template
- [ ] Command creates DESIGN.md from template
- [ ] All files have correct frontmatter
- [ ] README.md links to SRS.md and DESIGN.md
- [ ] Command outputs created file paths

## Implementation Notes

- Modify `_new-milestone` in `.justfiles/board.just`
- Read templates from `docs/board/templates/`
- Substitute variables: `${TITLE}`, `${ID}`, `${EPIC}`, `${DATE}`

## Requirements

- SRS-07: `just board new milestone` creates directory with README.md + SRS.md + DESIGN.md
