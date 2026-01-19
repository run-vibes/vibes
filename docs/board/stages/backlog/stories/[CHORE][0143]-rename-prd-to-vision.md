---
id: CHORE0143
title: Rename PRD to VISION
type: chore
status: backlog
priority: high
scope: coherence-verification/03-formal-planning-process
depends: []
estimate: 1h
created: 2026-01-18
---

# Rename PRD to VISION

## Summary

Rename `docs/PRD.md` to `docs/VISION.md` and update all references throughout the codebase. The VISION document serves as the product-level north star, distinct from epic-level PRDs.

## Acceptance Criteria

- [ ] `docs/VISION.md` exists with content from PRD.md <!-- verify: snapshot:vision-exists -->
- [ ] `docs/PRD.md` no longer exists
- [ ] All links to PRD.md updated to VISION.md
- [ ] CLAUDE.md references updated

## Implementation Notes

- Use `git mv` to preserve history
- Search for PRD.md references: `grep -r "PRD.md" docs/`
- Update CLAUDE.md Architecture section reference

## Requirements

- SRS-01: Product vision document exists at `docs/VISION.md`
- SRS-12: `docs/PRD.md` renamed to `docs/VISION.md`
