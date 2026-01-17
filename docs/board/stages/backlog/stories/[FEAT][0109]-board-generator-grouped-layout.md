---
id: FEAT0109
title: Board generator grouped layout
type: feat
status: backlog
priority: high
epics: [coherence-verification]
depends: []
milestone: 02-board-restructure
estimate: 2h
created: 2026-01-17
updated: 2026-01-17
---

# Board generator grouped layout

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Rewrite the `just board generate` command to produce a grouped layout where milestones are nested under their parent epics.

## Context

See [design.md](../../epics/coherence-verification/milestones/02-board-restructure/design.md) for the target README layout.

## Tasks

### Task 1: Update generate command

**Files:**
- Modify: `.justfiles/board.just` (generate task)

**Steps:**
1. Update the generate task to:
   - Show In Progress, Backlog, Icebox sections for stories
   - Group epics as H3 sections with milestone tables
   - Show epic status and milestone count in header
   - Show milestone progress (stories done/total)
   - Add Done section with completed milestones/epics
   - Link milestones and epics to their directories
2. Test with `just board generate`
3. Verify README.md matches expected layout
4. Commit: `feat(board): grouped layout for board generator`

### Task 2: Add Icebox section

**Steps:**
1. Add Icebox section between Backlog and Epics
2. Show stories with their blocking reason (from depends field)
3. Commit: `feat(board): add icebox section to board`

### Task 3: Add Done section

**Steps:**
1. Add Done section at bottom (collapsed by default)
2. Group by epic
3. Show completed milestones with dates (from `completed:` frontmatter)
4. Link epic names to their README
5. Link milestone names to their directories
6. Commit: `feat(board): add done milestones section`

## Acceptance Criteria

- [ ] README shows In Progress, Backlog, Icebox, Epics, Done sections
- [ ] Epics section groups milestones under H3 headers
- [ ] Epic headers show status and milestone count
- [ ] Milestone rows show status and story progress
- [ ] Done section shows completed milestones with links and dates
- [ ] All links resolve correctly

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0109`
3. Commit, push, and create PR
