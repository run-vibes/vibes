---
id: FEAT0110
title: Board state command sync
type: feat
status: done
priority: high
scope: coherence-verification/02-epic-based-project-hierarchy
depends: [FEAT0109]
estimate: 2h
created: 2026-01-17
---

# Board state command sync

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Add automatic doc sync to state transition commands and implement `done-epic` command.

## Context

See [design.md](../../epics/coherence-verification/milestones/02-board-restructure/design.md) for sync behavior.

## Tasks

### Task 1: Add done-epic command

**Files:**
- Modify: `.justfiles/board.just`

**Steps:**
1. Add `done-epic ID` command that:
   - Finds epic by ID or partial match
   - Updates epic README frontmatter `status: done`
   - Adds `completed: <date>` to frontmatter
   - Regenerates board README
2. Test with a sample epic
3. Commit: `feat(board): add done-epic command`

### Task 2: Sync milestone docs on story done

**Files:**
- Modify: `.justfiles/board.just` (done task)

**Steps:**
1. After moving story to done:
   - Check if story has `milestone:` in frontmatter
   - Find milestone's implementation.md
   - Update story checkbox from `[ ]` to `[x]`
   - If all stories checked, prompt to complete milestone
2. Test with a milestone-linked story
3. Commit: `feat(board): sync milestone docs on story completion`

### Task 3: Add completed date to milestone done

**Files:**
- Modify: `.justfiles/board.just` (done-milestone task)

**Steps:**
1. When completing milestone:
   - Add `completed: <date>` to milestone README frontmatter
   - Check if all milestones in epic are done
   - If so, prompt to complete epic
2. Test with a sample milestone
3. Commit: `feat(board): add completion dates to milestones`

### Task 4: Sync epic docs on milestone done

**Files:**
- Modify: `.justfiles/board.just` (done-milestone task)

**Steps:**
1. After marking milestone done:
   - Find parent epic from milestone path
   - Update epic README if it has a milestone table
2. Commit: `feat(board): sync epic docs on milestone completion`

## Acceptance Criteria

- [ ] `just board done-epic <id>` marks epic as done with date
- [ ] `just board done <story>` updates milestone implementation.md
- [ ] `just board done-milestone <id>` adds completion date
- [ ] Completion prompts shown when all items done
- [ ] All state transitions update related docs

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0110`
3. Commit, push, and create PR
