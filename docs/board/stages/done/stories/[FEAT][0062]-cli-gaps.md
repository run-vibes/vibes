---
id: FEAT0062
title: CLI commands (gaps)
type: feat
status: done
priority: medium
epics: [plugin-system]
depends: [FEAT0055, FEAT0057]
estimate: 2h
created: 2026-01-09
milestone: 34-open-world-adaptation
---

# CLI commands (gaps)

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement capability gap CLI commands for viewing and resolving gaps.

## Context

Users need CLI access to view capability gaps and manage their resolution. Commands allow listing gaps, viewing solutions, and marking gaps as resolved or dismissed. See [design.md](../../../milestones/34-open-world-adaptation/design.md).

## Tasks

### Task 1: Add gaps subcommand structure

**Files:**
- Modify: `plugins/vibes-groove/src/plugin.rs`

**Steps:**
1. Add `gaps` subcommand registration:
   ```rust
   ["gaps", "status"] => self.cmd_gaps_status(args),
   ["gaps", "list"] => self.cmd_gaps_list(args),
   ["gaps", "show"] => self.cmd_gaps_show(args),
   ["gaps", "dismiss"] => self.cmd_gaps_dismiss(args),
   ["gaps", "resolve"] => self.cmd_gaps_resolve(args),
   ["gaps", "apply"] => self.cmd_gaps_apply(args),
   ```
2. Register command definitions
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add gaps subcommand structure`

### Task 2: Implement status and listing commands

**Files:**
- Modify: `plugins/vibes-groove/src/plugin.rs`

**Steps:**
1. Implement `cmd_gaps_status()`:
   ```rust
   fn cmd_gaps_status(&self, _args: &[String]) -> Result<CommandResult> {
       let data = GapsStatusResponse {
           total_gaps: self.gap_detector.total_count(),
           detected: self.gap_detector.count_by_status(GapStatus::Detected),
           confirmed: self.gap_detector.count_by_status(GapStatus::Confirmed),
           in_progress: self.gap_detector.count_by_status(GapStatus::InProgress),
           resolved: self.gap_detector.count_by_status(GapStatus::Resolved),
           by_severity: self.gap_detector.count_by_severity(),
       };
       Ok(CommandResult::json(data))
   }
   ```
2. Implement `cmd_gaps_list()`:
   - List gaps with filters (status, severity, category)
   - Show summary: ID, category, severity, failure count
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): implement gaps listing commands`

### Task 3: Implement detail and management commands

**Files:**
- Modify: `plugins/vibes-groove/src/plugin.rs`

**Steps:**
1. Implement `cmd_gaps_show()`:
   ```rust
   fn cmd_gaps_show(&self, args: &[String]) -> Result<CommandResult> {
       let id = args.first().ok_or(GrooveError::MissingArg("gap_id"))?;
       let gap = self.gap_detector.get_gap(id)?;
       // Return full gap with solutions and failure history
       Ok(CommandResult::json(GapDetailResponse {
           gap,
           failures: self.gap_detector.get_failures_for_gap(id)?,
       }))
   }
   ```
2. Implement `cmd_gaps_dismiss()`:
   - Mark gap as dismissed
   - Record reason
3. Implement `cmd_gaps_resolve()`:
   - Mark gap as resolved
   - Record resolution notes
4. Implement `cmd_gaps_apply()`:
   - Apply a suggested solution
   - Execute solution action
5. Run: `cargo check -p vibes-groove`
6. Commit: `feat(groove): implement gaps management commands`

### Task 4: Add openworld combined commands

**Files:**
- Modify: `plugins/vibes-groove/src/plugin.rs`

**Steps:**
1. Add combined `openworld` commands:
   ```rust
   ["openworld", "status"] => self.cmd_openworld_status(args),
   ["openworld", "history"] => self.cmd_openworld_history(args),
   ```
2. Implement `cmd_openworld_status()`:
   - Combined status of novelty + gaps
   - System health summary
3. Implement `cmd_openworld_history()`:
   - Recent events from all topics
   - Filterable by type
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): add openworld combined commands`

### Task 5: Add HTTP routes and tests

**Files:**
- Modify: `plugins/vibes-groove/src/plugin.rs`

**Steps:**
1. Add route registrations for gaps and openworld
2. Implement route handlers
3. Write tests:
   - Test command registration
   - Test route registration
   - Test status responses
   - Test gap management operations
4. Run: `cargo test -p vibes-groove -- gaps`
5. Commit: `test(groove): add gaps CLI tests`

## Acceptance Criteria

- [ ] `vibes groove gaps status` shows summary
- [ ] `vibes groove gaps list` lists with filters
- [ ] `vibes groove gaps show <id>` shows details and solutions
- [ ] `vibes groove gaps dismiss <id>` dismisses gap
- [ ] `vibes groove gaps resolve <id>` marks as resolved
- [ ] `vibes groove gaps apply <gap> <solution>` applies solution
- [ ] `vibes groove openworld status` shows combined status
- [ ] `vibes groove openworld history` shows recent events
- [ ] HTTP routes work for all commands
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0062`
3. Commit, push, and create PR
