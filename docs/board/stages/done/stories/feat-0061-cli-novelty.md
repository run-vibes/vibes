---
id: FEAT0061
title: CLI commands (novelty)
type: feat
status: done
priority: medium
epics: [plugin-system]
depends: [FEAT0053]
estimate: 2h
created: 2026-01-09
milestone: 34-open-world-adaptation
---

# CLI commands (novelty)

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement novelty detection CLI commands for viewing and managing patterns and clusters.

## Context

Power users need CLI access to novelty detection data. Commands allow viewing clusters, fingerprints, and marking patterns as known. See [design.md](../../../milestones/34-open-world-adaptation/design.md).

## Tasks

### Task 1: Add novelty subcommand structure

**Files:**
- Modify: `plugins/vibes-groove/src/plugin.rs`

**Steps:**
1. Add `novelty` subcommand registration:
   ```rust
   ["novelty", "status"] => self.cmd_novelty_status(args),
   ["novelty", "clusters"] => self.cmd_novelty_clusters(args),
   ["novelty", "cluster"] => self.cmd_novelty_cluster(args),
   ["novelty", "fingerprints"] => self.cmd_novelty_fingerprints(args),
   ["novelty", "mark-known"] => self.cmd_novelty_mark_known(args),
   ["novelty", "reset"] => self.cmd_novelty_reset(args),
   ```
2. Register command definitions
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add novelty subcommand structure`

### Task 2: Implement status and listing commands

**Files:**
- Modify: `plugins/vibes-groove/src/plugin.rs`

**Steps:**
1. Implement `cmd_novelty_status()`:
   ```rust
   fn cmd_novelty_status(&self, _args: &[String]) -> Result<CommandResult> {
       // Return summary: known patterns, pending outliers, cluster count
       let data = NoveltyStatusResponse {
           known_patterns: self.novelty_detector.known_count(),
           pending_outliers: self.novelty_detector.pending_count(),
           cluster_count: self.novelty_detector.cluster_count(),
           threshold: self.novelty_detector.similarity_threshold(),
           last_detection: self.novelty_detector.last_detection_time(),
       };
       Ok(CommandResult::json(data))
   }
   ```
2. Implement `cmd_novelty_clusters()`:
   - List all clusters with member counts
   - Show created/last seen times
3. Implement `cmd_novelty_fingerprints()`:
   - List known fingerprints
   - Support filtering by context
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): implement novelty listing commands`

### Task 3: Implement detail and management commands

**Files:**
- Modify: `plugins/vibes-groove/src/plugin.rs`

**Steps:**
1. Implement `cmd_novelty_cluster()`:
   ```rust
   fn cmd_novelty_cluster(&self, args: &[String]) -> Result<CommandResult> {
       let id = args.first().ok_or(GrooveError::MissingArg("cluster_id"))?;
       let cluster = self.novelty_detector.get_cluster(id)?;
       // Return full cluster details with members
       Ok(CommandResult::json(cluster))
   }
   ```
2. Implement `cmd_novelty_mark_known()`:
   - Mark a fingerprint hash as known
   - Move from pending to known
3. Implement `cmd_novelty_reset()`:
   - Reset all novelty detection data
   - Require confirmation
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): implement novelty management commands`

### Task 4: Add HTTP routes

**Files:**
- Modify: `plugins/vibes-groove/src/plugin.rs`

**Steps:**
1. Add route registrations:
   ```rust
   (HttpMethod::Get, "/novelty/status") => self.route_novelty_status(),
   (HttpMethod::Get, "/novelty/clusters") => self.route_novelty_clusters(&request),
   (HttpMethod::Get, "/novelty/clusters/:id") => self.route_novelty_cluster(&request),
   (HttpMethod::Get, "/novelty/fingerprints") => self.route_novelty_fingerprints(&request),
   (HttpMethod::Post, "/novelty/mark-known/:hash") => self.route_novelty_mark_known(&request),
   (HttpMethod::Post, "/novelty/reset") => self.route_novelty_reset(&request),
   ```
2. Implement route handlers
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add novelty HTTP routes`

### Task 5: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/plugin.rs`

**Steps:**
1. Write tests:
   - Test command registration
   - Test route registration
   - Test status response format
   - Test cluster listing
   - Test mark-known operation
2. Run: `cargo test -p vibes-groove -- novelty`
3. Commit: `test(groove): add novelty CLI tests`

## Acceptance Criteria

- [ ] `vibes groove novelty status` shows summary
- [ ] `vibes groove novelty clusters` lists clusters
- [ ] `vibes groove novelty cluster <id>` shows details
- [ ] `vibes groove novelty fingerprints` lists known patterns
- [ ] `vibes groove novelty mark-known <hash>` marks as known
- [ ] `vibes groove novelty reset` resets data
- [ ] HTTP routes work for all commands
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0061`
3. Commit, push, and create PR
