---
id: FEAT0192
title: Update new epic command
type: feat
status: backlog
priority: high
scope: coherence-verification/03-formal-planning-process
depends: [FEAT0191]
estimate: 2h
created: 2026-01-18
---

# Update New Epic Command

## Summary

Update `just board new epic` to create the formal document structure: README.md + PRD.md using the new templates.

## Acceptance Criteria

- [ ] `just board new epic "name"` creates directory with README.md
- [ ] `just board new epic "name"` creates PRD.md from template
- [ ] PRD.md has correct frontmatter (id, title, status)
- [ ] README.md links to PRD.md
- [ ] Command outputs created file paths

## Implementation Notes

- Modify `_new-epic` in `.justfiles/board.just`
- Read templates from `docs/board/templates/`
- Substitute variables: `${TITLE}`, `${ID}`, `${DATE}`

## Requirements

- SRS-06: `just board new epic` creates directory with README.md + PRD.md
