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

See [DESIGN.md](DESIGN.md) for architecture and implementation details.

## Stories

| # | Story | Description | Status |
|---|-------|-------------|--------|
| 1 | [CHORE0111](../../../../stages/done/stories/[CHORE][0111]-create-design-system-epic.md) | Create design-system epic | done |
| 2 | [CHORE0112](../../../../stages/done/stories/[CHORE][0112]-migrate-milestones-to-epics.md) | Migrate milestones to epics | done |
| 3 | [CHORE0113](../../../../stages/done/stories/[CHORE][0113]-rename-stories-new-format.md) | Rename stories to new format | done |
| 4 | [CHORE0114](../../../../stages/done/stories/[CHORE][0114]-update-board-documentation.md) | Update board documentation | done |
| 5 | [FEAT0109](../../../../stages/done/stories/[FEAT][0109]-board-generator-grouped-layout.md) | Board generator grouped layout | done |
| 6 | [FEAT0110](../../../../stages/done/stories/[FEAT][0110]-board-state-command-sync.md) | Board state command sync | done |

## Progress

**Requirements:** 0/0
0 verified
**Stories:** 6/6 complete

