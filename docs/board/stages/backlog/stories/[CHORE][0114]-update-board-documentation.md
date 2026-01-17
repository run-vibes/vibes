---
id: CHORE0114
title: Update board documentation
type: chore
status: backlog
priority: high
epics: [coherence-verification]
depends: [CHORE0113, FEAT0110]
milestone: 02-board-restructure
estimate: 30m
created: 2026-01-17
updated: 2026-01-17
---

# Update board documentation

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Update CONVENTIONS.md and CLAUDE.md to reflect the new board hierarchy and commands.

## Context

After migration is complete, documentation needs to match the new structure.

## Tasks

### Task 1: Update CONVENTIONS.md

**Files:**
- Modify: `docs/board/CONVENTIONS.md`

**Steps:**
1. Update directory structure diagram to show:
   - Milestones nested under epics
   - No root `milestones/` directory
2. Update story naming section with `[TYPE][NNNN]` format
3. Add `done-epic` command to command reference
4. Update any examples to use new paths
5. Remove references to old structure
6. Commit: `docs(board): update CONVENTIONS.md for new hierarchy`

### Task 2: Update CLAUDE.md

**Files:**
- Modify: `CLAUDE.md`

**Steps:**
1. Update board commands section with `done-epic`
2. Update any path references to milestones
3. Verify all board-related instructions still accurate
4. Commit: `docs: update CLAUDE.md for board restructure`

### Task 3: Verify coherence-verification epic

**Steps:**
1. Check success criteria in epic README
2. Mark completed criteria as done
3. If all criteria met, run `just board done-epic coherence-verification`
4. Commit if epic completed

## Acceptance Criteria

- [ ] CONVENTIONS.md reflects new hierarchy
- [ ] CLAUDE.md reflects new commands
- [ ] No references to old `milestones/` root directory
- [ ] All examples use new naming format

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done CHORE0114`
3. Commit, push, and create PR
