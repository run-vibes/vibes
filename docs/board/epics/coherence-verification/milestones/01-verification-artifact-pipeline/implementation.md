# Milestone 01: Verification Artifact Pipeline - Implementation Plan

> **For Claude:** Work through stories in order. Use superpowers:executing-plans for each story.

**Goal:** Complete the verification pipeline so `just verify all` captures artifacts at all three tiers.

**Design:** See [design.md](design.md) for architecture decisions.

---

## Stories

| # | Story | Description |
|---|-------|-------------|
| 1 | [FEAT][0191]-verification-definition-files | Create snapshots.json and checkpoints.json definition files |
| 2 | [FEAT][0192]-snapshots-spec | Playwright spec to capture snapshots from definition file |
| 3 | [FEAT][0193]-checkpoints-spec | Playwright spec to capture interaction checkpoints |
| 4 | [FEAT][0194]-videos-spec | Playwright spec for web video recording |
| 5 | [CHORE][0195]-precommit-integration | Add verification to pre-commit workflow |
| 6 | [DOCS][0196]-verification-documentation | Document verification commands in CLAUDE.md |

> **Status:** Check story frontmatter or run `just board status` for current status.

## Dependencies

```
[FEAT][0191] ─► [FEAT][0192] ─┐
             └► [FEAT][0193] ─┼─► [CHORE][0195] ─► [DOCS][0196]
             └► [FEAT][0194] ─┘
```

- Stories 2-4 depend on Story 1 (definition files must exist)
- Story 5 depends on Stories 2-4 (all capture specs needed)
- Story 6 depends on Story 5 (document final state)

## Completion Criteria

- [x] `just verify snapshots` captures PNGs from snapshots.json
- [x] `just verify checkpoints` captures interaction sequences
- [x] `just verify videos` records web videos via Playwright
- [x] `just verify all` runs all tiers and generates report
- [ ] `just pre-commit` includes verification step (deferred - adds 7+ min)
- [x] CLAUDE.md documents verification commands
