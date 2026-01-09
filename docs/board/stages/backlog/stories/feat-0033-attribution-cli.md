---
id: FEAT0033
title: Attribution CLI commands
type: feat
status: pending
priority: high
epics: [plugin-system]
depends: [FEAT0031]
estimate: 2h
created: 2026-01-09
milestone: 31-attribution-engine
---

# Attribution CLI commands

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement `vibes groove attr` subcommands for querying and managing attribution data.

## Context

Users need CLI visibility into the attribution engine's decisions and learning values. Also need ability to manually enable/disable learnings. See [design.md](../../../milestones/31-attribution-engine/design.md).

## Tasks

### Task 1: Add attr subcommand scaffold

**Files:**
- Modify: `vibes-cli/src/commands/groove.rs`

**Steps:**
1. Add `attr` subcommand to groove:
   ```rust
   #[derive(Subcommand)]
   pub enum AttrCommands {
       /// Show attribution engine status
       Status,
       /// List learning values
       Values {
           #[arg(long, default_value = "value")]
           sort: String,
           #[arg(long, default_value = "20")]
           limit: usize,
       },
       /// Show detailed attribution for a learning
       Show {
           learning_id: String,
       },
       /// Explain attribution for a specific session
       Explain {
           learning_id: String,
           session_id: String,
       },
   }
   ```
2. Run: `cargo check -p vibes-cli`
3. Commit: `feat(cli): add attr subcommand scaffold`

### Task 2: Add HTTP routes for queries

**Files:**
- Modify: `plugins/vibes-groove/src/plugin.rs`

**Steps:**
1. Add routes to groove plugin:
   ```rust
   // GET /groove/attr/status
   // GET /groove/attr/values?sort=value&limit=20
   // GET /groove/attr/show/:learning_id
   // GET /groove/attr/explain/:learning_id/:session_id
   ```
2. Implement handlers that query attribution store
3. Add response types for JSON serialization
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): add attribution HTTP routes`

### Task 3: Implement status command

**Files:**
- Modify: `vibes-cli/src/commands/groove.rs`

**Steps:**
1. Implement `attr status`:
   - Query engine configuration
   - Show learning value summary (total, active, deprecated)
   - Show recent activity (last 24h attributions)
   - Format as readable table
2. Run: `cargo check -p vibes-cli`
3. Commit: `feat(cli): implement attr status`

### Task 4: Implement values command

**Files:**
- Modify: `vibes-cli/src/commands/groove.rs`

**Steps:**
1. Implement `attr values`:
   - Query learning values from store
   - Support sort options: value, confidence, sessions
   - Format as table with columns: ID, Value, Confidence, Sessions, Status
2. Run: `cargo check -p vibes-cli`
3. Commit: `feat(cli): implement attr values`

### Task 5: Implement show command

**Files:**
- Modify: `vibes-cli/src/commands/groove.rs`

**Steps:**
1. Implement `attr show`:
   - Full learning value breakdown
   - Per-source attribution (temporal, ablation)
   - Recent session attributions
   - Learning content preview
2. Run: `cargo check -p vibes-cli`
3. Commit: `feat(cli): implement attr show`

### Task 6: Implement explain command

**Files:**
- Modify: `vibes-cli/src/commands/groove.rs`

**Steps:**
1. Implement `attr explain`:
   - Load specific attribution record
   - Show activation signals with message indices
   - Show temporal correlation details
   - Show ablation status if applicable
2. Run: `cargo check -p vibes-cli`
3. Commit: `feat(cli): implement attr explain`

### Task 7: Add learn enable/disable commands

**Files:**
- Modify: `vibes-cli/src/commands/groove.rs`

**Steps:**
1. Add to existing `learn` subcommand:
   ```rust
   /// Enable a deprecated learning
   Enable { id: String },
   /// Manually deprecate a learning
   Disable { id: String, reason: Option<String> },
   ```
2. Implement enable: update status to Active
3. Implement disable: update status to Deprecated
4. Run: `cargo check -p vibes-cli`
5. Commit: `feat(cli): add learn enable/disable`

## Acceptance Criteria

- [ ] `vibes groove attr status` shows engine status
- [ ] `vibes groove attr values` lists learning values
- [ ] `vibes groove attr show <id>` shows detailed breakdown
- [ ] `vibes groove attr explain <learning> <session>` explains attribution
- [ ] `vibes groove learn enable <id>` re-enables deprecated learning
- [ ] `vibes groove learn disable <id>` manually deprecates learning
- [ ] All commands query via HTTP to daemon
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0033`
3. Commit, push, and create PR
