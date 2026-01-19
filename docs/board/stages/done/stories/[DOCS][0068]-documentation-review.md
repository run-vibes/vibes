---
id: DOCS0068
title: Documentation review
type: docs
status: done
priority: low
scope: verification
depends: []
estimate: 2h
created: 2026-01-09
---

# Documentation review

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Audit and update project documentation to ensure accuracy after the groove-dashboard milestone.

## Context

Documentation drifts as features are added. This story reviews all docs to ensure commands work, descriptions are accurate, and nothing is missing.

## Tasks

### Task 1: Audit README.md

**Files:**
- Modify: `README.md`

**Steps:**
1. Verify setup instructions work:
   - Clone command
   - Nix/direnv setup
   - `just setup` command
2. Check that listed features match current state
3. Update screenshots if UI has changed significantly
4. Fix any broken links
5. Commit: `docs: update README.md`

### Task 2: Audit CLAUDE.md

**Files:**
- Modify: `CLAUDE.md`

**Steps:**
1. Verify all `just` commands listed actually exist:
   ```bash
   just --list | grep -E "^[a-z]"
   ```
2. Check workflow instructions are accurate
3. Verify architecture table matches current crates
4. Update any outdated sections
5. Commit: `docs: update CLAUDE.md`

### Task 3: Audit docs/PRD.md

**Files:**
- Modify: `docs/PRD.md`

**Steps:**
1. Review feature descriptions against implementation
2. Mark completed features
3. Update any changed scope
4. Remove obsolete sections
5. Commit: `docs: update PRD.md`

### Task 4: Audit board conventions

**Files:**
- Modify: `docs/board/CONVENTIONS.md`

**Steps:**
1. Verify story/epic/milestone formats match actual files
2. Check that `just board` commands are documented correctly
3. Update any changed workflows
4. Commit: `docs: update board CONVENTIONS.md`

### Task 5: Identify documentation gaps

**Steps:**
1. Check for missing READMEs in key directories:
   - `plugins/vibes-groove/`
   - `design-system/`
   - `web-ui/`
2. List any undocumented features or APIs
3. Create placeholder stories for missing docs (if significant)
4. Add findings to story completion notes
5. No commit (research output)

## Acceptance Criteria

- [ ] README.md setup instructions verified working
- [ ] CLAUDE.md commands all exist and work
- [ ] PRD.md reflects current implementation status
- [ ] Board conventions match actual process
- [ ] Documentation gaps identified and noted
- [ ] No broken links in docs

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done DOCS0068`
3. Commit, push, and create PR
