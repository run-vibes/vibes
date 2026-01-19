---
id: CHORE0144
title: Migrate existing epics
type: chore
status: done
priority: low
scope: coherence-verification/03-formal-planning-process
depends: [FEAT0191, FEAT0192]
estimate: 4h
created: 2026-01-18
---

# Migrate Existing Epics

## Summary

Migrate all existing epics to the new document structure by creating PRD.md files from README content.

## Acceptance Criteria

- [x] All epics have PRD.md files
- [x] PRD.md contains requirements extracted from README
- [x] README.md updated to navigation format
- [x] Git history preserved where possible
- [x] Epic list: core, web-ui, cli, tui, coherence-verification, groove, etc.

## Implementation Notes

- For each epic:
  1. Extract requirements/goals from README to PRD.md
  2. Update README to new navigation format
  3. Verify links work
- Use `git mv` where applicable
- May need manual content restructuring

## Requirements

- SRS-13: All existing epics have PRD.md (migrated from README content)
- SRS-NFR-03: Migration preserves git history where possible
