---
id: coherence-02
title: Board Restructure
status: planned
epic: coherence-verification
created: 2026-01-17
---

# Milestone 02: Board Restructure

> Reorganize board hierarchy to Epic > Milestone > Story with clearer lifecycles.

## Value Statement

The board hierarchy matches mental model (Epic > Milestone > Story), stories have consistent naming, and blocked work is clearly separated in an icebox stage.

## Done Criteria

- [ ] Epic structure is `epics/<epic>/milestones/<NN-value>/`
- [ ] Stories use `[TYPE][NNNN]-verb-phrase` naming
- [ ] Icebox stage exists at `stages/icebox/stories/`
- [ ] `just board ice` and `just board thaw` commands work
- [ ] CONVENTIONS.md reflects new hierarchy
- [ ] Existing stories migrated to new format

## Design

See [design.md](design.md) for architecture and implementation details.

## Stories

| ID | Story | Status |
|----|-------|--------|
| TBD | Add icebox stage directory | planned |
| TBD | Implement ice and thaw commands | planned |
| TBD | Update story naming in board commands | planned |
| TBD | Create epic milestone directory structure | planned |
| TBD | Migrate existing stories to new format | planned |
| TBD | Update CONVENTIONS.md | planned |
