---
id: CHORE0145
title: Migrate existing milestones
type: chore
status: backlog
priority: low
scope: coherence-verification/03-formal-planning-process
depends: [FEAT0191, FEAT0193]
estimate: 6h
created: 2026-01-18
---

# Migrate Existing Milestones

## Summary

Migrate all existing milestones to the new document structure: rename design.md → DESIGN.md, create SRS.md from implementation.md content.

## Acceptance Criteria

- [ ] All milestones have DESIGN.md (uppercase)
- [ ] All milestones have SRS.md
- [ ] implementation.md content migrated to SRS.md story list
- [ ] implementation.md files removed after migration
- [ ] Git history preserved where possible

## Implementation Notes

- For each milestone:
  1. `git mv design.md DESIGN.md`
  2. Create SRS.md with requirements + story list from implementation.md
  3. Update README to new navigation format
  4. Remove implementation.md
- Milestones without design.md get new DESIGN.md from template

## Requirements

- SRS-14: All existing milestones have SRS.md
- SRS-15: All existing `design.md` renamed to `DESIGN.md`
- SRS-16: All existing `implementation.md` content migrated to SRS.md
- SRS-NFR-03: Migration preserves git history where possible
