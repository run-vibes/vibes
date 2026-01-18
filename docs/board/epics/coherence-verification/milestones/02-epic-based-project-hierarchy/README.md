---
id: 02-epic-based-project-hierarchy
title: Epic-Based Project Hierarchy
status: done
completed: 2026-01-18
epic: coherence-verification
created: 2026-01-17
---

# Milestone 02: Board Restructure

> Reorganize board hierarchy to Epic > Milestone > Story with clearer lifecycles.

## Value Statement

The board hierarchy matches mental model (Epic > Milestone > Story), stories have consistent naming, and blocked work is clearly separated in an icebox stage.

## Done Criteria

- [x] Epic structure is `epics/<epic>/milestones/<NN-value>/`
- [x] Stories use `[TYPE][NNNN]-verb-phrase` naming
- [x] Icebox stage exists at `stages/icebox/stories/`
- [x] `just board ice` and `just board thaw` commands work
- [x] CONVENTIONS.md reflects new hierarchy
- [x] Existing stories migrated to new format

## Design

See [design.md](design.md) for architecture and implementation details.

## Stories

| ID | Story | Status |
|----|-------|--------|
| CHORE0111 | Create design-system epic | done |
| CHORE0112 | Migrate milestones to epics | done |
| CHORE0113 | Rename stories to new format | done |
| FEAT0109 | Board generator grouped layout | done |
| FEAT0110 | Board state command sync | done |
| CHORE0114 | Update board documentation | done |
