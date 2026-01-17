# Milestone 02: Board Restructure - Implementation Plan

> **For Claude:** Work through stories in order. Use superpowers:executing-plans for each story.

**Goal:** Restructure the board hierarchy to Epic > Milestone > Story with proper naming and auto-sync.

**Design:** See [design.md](design.md) for architecture decisions.

---

## Stories

| # | Story | Description |
|---|-------|-------------|
| 1 | [CHORE][0111]-create-design-system-epic | Create missing design-system epic |
| 2 | [CHORE][0112]-migrate-milestones-to-epics | Move 55 milestones into their parent epics |
| 3 | [CHORE][0113]-rename-stories-new-format | Rename all stories to [TYPE][NNNN] format |
| 4 | [FEAT][0109]-board-generator-grouped-layout | Rewrite board generator for grouped epic sections |
| 5 | [FEAT][0110]-board-state-command-sync | Add auto-sync and done-epic command |
| 6 | [CHORE][0114]-update-board-documentation | Update CONVENTIONS.md and CLAUDE.md |

> **Status:** Check story frontmatter or run `just board status` for current status.

## Dependencies

```
[CHORE][0111] ─┐
               ├─► [CHORE][0112] ─► [CHORE][0113] ─┐
               │                                    ├─► [CHORE][0114]
               └─► [FEAT][0109] ──► [FEAT][0110] ──┘
```

- Story 2 depends on Story 1 (epic must exist before milestones can move)
- Story 3 depends on Story 2 (milestone paths change affects symlinks)
- Stories 4-5 can run in parallel with Stories 2-3
- Story 6 depends on all others (documents final state)

## Completion Criteria

- [ ] All 55 milestones nested under epics
- [ ] All stories use [TYPE][NNNN] naming
- [ ] Board README shows grouped epic sections
- [ ] `just board done-epic` command works
- [ ] State commands auto-sync milestone/epic docs
- [ ] Documentation updated
