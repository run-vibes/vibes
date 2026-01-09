---
id: FEAT0041
title: Strategy CLI commands
type: feat
status: in-progress
priority: high
epics: [plugin-system]
depends: [FEAT0039]
estimate: 2h
created: 2026-01-09
milestone: 32-adaptive-strategies
---

# Strategy CLI commands

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement `vibes groove strategy` subcommands for querying and managing strategy distributions.

## Context

Users need CLI visibility into how strategies are being learned and selected. Also need ability to reset distributions for debugging. See [design.md](../../../milestones/32-adaptive-strategies/design.md).

## Tasks

### Task 1: Add strategy subcommand scaffold

**Files:**
- Modify: `vibes-cli/src/commands/groove.rs`

**Steps:**
1. Add `strategy` subcommand to groove:
   ```rust
   #[derive(Subcommand)]
   pub enum StrategyCommands {
       /// Show strategy learner status
       Status,
       /// List category distributions
       Distributions,
       /// Detailed distribution breakdown
       Show { category: String },
       /// Show learning's strategy override
       Learning { id: String },
       /// Strategy selection history for a learning
       History {
           learning_id: String,
           #[arg(long, default_value = "20")]
           limit: usize,
       },
       /// Reset category to default priors
       Reset {
           category: String,
           #[arg(long)]
           confirm: bool,
       },
       /// Clear learning specialization
       ResetLearning {
           id: String,
           #[arg(long)]
           confirm: bool,
       },
   }
   ```
2. Run: `cargo check -p vibes-cli`
3. Commit: `feat(cli): add strategy subcommand scaffold`

### Task 2: Add HTTP routes for queries

**Files:**
- Modify: `plugins/vibes-groove/src/plugin.rs`

**Steps:**
1. Add routes to groove plugin:
   ```rust
   // GET /groove/strategy/status
   // GET /groove/strategy/distributions
   // GET /groove/strategy/show/:category/:context_type
   // GET /groove/strategy/learning/:id
   // GET /groove/strategy/history/:learning_id?limit=20
   // POST /groove/strategy/reset/:category/:context_type
   // POST /groove/strategy/reset-learning/:id
   ```
2. Implement handlers that query strategy store
3. Add response types for JSON serialization
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): add strategy HTTP routes`

### Task 3: Implement status command

**Files:**
- Modify: `vibes-cli/src/commands/groove.rs`

**Steps:**
1. Implement `strategy status`:
   - Query distribution counts by category
   - Show top performing strategies overall
   - Show recent activity (strategy selections)
   - Format as readable summary
2. Run: `cargo check -p vibes-cli`
3. Commit: `feat(cli): implement strategy status`

### Task 4: Implement distributions command

**Files:**
- Modify: `vibes-cli/src/commands/groove.rs`

**Steps:**
1. Implement `strategy distributions`:
   - List all category/context combinations
   - Show session count for each
   - Show leading strategy variant
   - Format as table
2. Run: `cargo check -p vibes-cli`
3. Commit: `feat(cli): implement strategy distributions`

### Task 5: Implement show command

**Files:**
- Modify: `vibes-cli/src/commands/groove.rs`

**Steps:**
1. Implement `strategy show`:
   - Full distribution breakdown with Beta params (alpha, beta)
   - Visual bar chart of current weights
   - List specialized learnings in this category
   - Show strategy parameters
2. Run: `cargo check -p vibes-cli`
3. Commit: `feat(cli): implement strategy show`

### Task 6: Implement learning command

**Files:**
- Modify: `vibes-cli/src/commands/groove.rs`

**Steps:**
1. Implement `strategy learning`:
   - Show override status (inheriting vs specialized)
   - Show effective weights
   - Show session count and specialization threshold
   - Compare to category distribution
2. Run: `cargo check -p vibes-cli`
3. Commit: `feat(cli): implement strategy learning`

### Task 7: Implement history and reset commands

**Files:**
- Modify: `vibes-cli/src/commands/groove.rs`

**Steps:**
1. Implement `strategy history`:
   - Query strategy events for learning
   - Show selection timeline with outcomes
   - Format as table
2. Implement `strategy reset`:
   - Require --confirm flag
   - Reset category to default priors
   - Show confirmation message
3. Implement `strategy reset-learning`:
   - Require --confirm flag
   - Clear specialization, revert to category
   - Show confirmation message
4. Run: `cargo check -p vibes-cli`
5. Commit: `feat(cli): implement history and reset commands`

## Acceptance Criteria

- [ ] `vibes groove strategy status` shows learner status
- [ ] `vibes groove strategy distributions` lists all distributions
- [ ] `vibes groove strategy show <cat>` shows detailed breakdown
- [ ] `vibes groove strategy learning <id>` shows override status
- [ ] `vibes groove strategy history <id>` shows selection timeline
- [ ] `vibes groove strategy reset` requires confirmation
- [ ] All commands query via HTTP to daemon
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0041`
3. Commit, push, and create PR
