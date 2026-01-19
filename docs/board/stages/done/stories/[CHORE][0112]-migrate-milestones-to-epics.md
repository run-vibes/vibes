---
id: CHORE0112
title: Migrate milestones to epics
type: chore
status: done
priority: high
scope: coherence-verification/02-epic-based-project-hierarchy
depends: [CHORE0111]
estimate: 1h
created: 2026-01-17
---

# Migrate milestones to epics

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Move all 55 milestones from `milestones/` into their parent epic directories at `epics/<epic>/milestones/`.

## Context

See [design.md](../../epics/coherence-verification/milestones/02-board-restructure/design.md) for the epic-to-milestone mapping.

Each milestone's first epic in frontmatter is the primary owner. Milestones move to `epics/<primary-epic>/milestones/<milestone-id>/`.

## Tasks

### Task 1: Create migration script

**Files:**
- Create: `scripts/migrate-milestones.sh`

**Steps:**
1. Write bash script that:
   - Reads each milestone's README.md
   - Extracts first epic from `epics:` frontmatter
   - Creates `epics/<epic>/milestones/` if needed
   - Moves milestone directory
   - Updates any relative paths in design.md/implementation.md
2. Make script executable
3. Commit: `chore(board): add milestone migration script`

### Task 2: Run migration

**Steps:**
1. Run `./scripts/migrate-milestones.sh`
2. Verify all 55 milestones moved correctly
3. Verify milestone README/design.md files still valid
4. Remove empty `milestones/` directory
5. Commit: `chore(board): migrate milestones into parent epics`

### Task 3: Cleanup

**Steps:**
1. Remove migration script (one-time use)
2. Commit: `chore(board): remove migration script`

## Acceptance Criteria

- [ ] All 55 milestones exist under `epics/<epic>/milestones/`
- [ ] No milestones remain in root `milestones/` directory
- [ ] All milestone design.md/implementation.md files valid
- [ ] `just board generate` succeeds

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done CHORE0112`
3. Commit, push, and create PR
