---
id: CHORE0113
title: Rename stories to new format
type: chore
status: done
priority: high
scope: coherence-verification/02-epic-based-project-hierarchy
depends: [CHORE0112]
estimate: 1h
created: 2026-01-17
---

# Rename stories to new format

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Rename all ~200 stories from legacy formats to `[TYPE][NNNN]-verb-phrase.md`.

## Context

See [design.md](../../epics/coherence-verification/milestones/02-board-restructure/design.md) for naming conventions.

**Migration rules:**
- `feat-0042-name.md` → `[FEAT][0042]-name.md`
- `m26-feat-01-name.md` → `[FEAT][NNNN]-name.md` (new ID, `milestone: 26` in frontmatter)
- Update frontmatter `id:` field to match new format

## Tasks

### Task 1: Create rename script

**Files:**
- Create: `scripts/rename-stories.sh`

**Steps:**
1. Write bash script that:
   - Scans `stages/*/stories/*.md`
   - Handles legacy patterns: `feat-NNNN-*`, `m*-feat-*-*`, etc.
   - For milestone-prefixed: extracts milestone, assigns new global ID
   - Updates frontmatter `id:` and adds `milestone:` field
   - Renames file to `[TYPE][NNNN]-name.md`
   - Updates symlinks in epics
2. Make script executable
3. Commit: `chore(board): add story rename script`

### Task 2: Run rename

**Steps:**
1. Run `./scripts/rename-stories.sh`
2. Verify all stories renamed correctly
3. Verify all symlinks in epics still work
4. Verify frontmatter updated correctly
5. Commit: `chore(board): rename stories to [TYPE][NNNN] format`

### Task 3: Cleanup

**Steps:**
1. Remove rename script
2. Commit: `chore(board): remove rename script`

## Acceptance Criteria

- [x] All stories use `[TYPE][NNNN]-verb-phrase.md` format
- [x] All frontmatter `id:` fields updated
- [x] Milestone-prefixed stories have `milestone:` in frontmatter
- [x] All epic symlinks resolve correctly
- [x] `just board generate` succeeds
- [x] `just board status` shows correct counts

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done CHORE0113`
3. Commit, push, and create PR
