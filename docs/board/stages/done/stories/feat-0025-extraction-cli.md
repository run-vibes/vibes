---
id: FEAT0025
title: Learning extraction CLI commands
type: feat
status: done
priority: medium
epics: [plugin-system]
depends: [FEAT0024]
estimate: 2h
created: 2026-01-08
updated: 2026-01-09
milestone: 30-learning-extraction
---

# Learning extraction CLI commands

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Add `vibes groove learn` subcommands for managing extracted learnings.

## Context

Users need CLI commands to view, manage, and export learnings. These commands query the learning store and provide visibility into what the system has learned. See [design.md](../../../milestones/30-learning-extraction/design.md).

## Tasks

### Task 1: Add learn subcommand structure

**Files:**
- Create: `plugins/vibes-groove/src/cli/learn.rs`
- Modify: `plugins/vibes-groove/src/cli/mod.rs`

**Steps:**
1. Create `learn` subcommand with clap:
   ```rust
   #[derive(Subcommand)]
   pub enum LearnCommands {
       /// Show extraction status and counts
       Status,
       /// List learnings with filters
       List {
           #[arg(long)]
           scope: Option<String>,
           #[arg(long)]
           category: Option<String>,
         },
       /// Show full learning details
       Show { id: String },
       /// Delete a learning
       Delete { id: String },
       /// Export learnings as JSON
       Export {
           #[arg(long)]
           scope: Option<String>,
       },
   }
   ```
2. Add to groove CLI module
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add learn subcommand structure`

### Task 2: Implement status command

**Files:**
- Modify: `plugins/vibes-groove/src/cli/learn.rs`

**Steps:**
1. Implement `status` command:
   - Query learning counts by scope (project, user, global)
   - Query learning counts by category
   - Show embedder status (model loaded, dimensions)
   - Show recent extraction activity (last 24h)
2. Format output:
   ```
   Learning Extraction Status

   Learnings by scope:
     Project: 12
     User: 5
     Global: 3

   Learnings by category:
     Correction: 8
     ErrorRecovery: 7
     Pattern: 3
     Preference: 2

   Embedder: gte-small (384 dims) - healthy
   Last extraction: 2h ago
   ```
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): implement learn status command`

### Task 3: Implement list command

**Files:**
- Modify: `plugins/vibes-groove/src/cli/learn.rs`

**Steps:**
1. Implement `list` command:
   - Support `--scope project|user|global` filter
   - Support `--category correction|error_recovery|pattern|preference` filter
2. Query store with filters
3. Format as table:
   ```
   ID       Category        Confidence  Description
   abc123   Correction      0.85        Prefer tabs over spaces
   def456   ErrorRecovery   0.72        Retry with --force on permission error
   ```
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): implement learn list command`

### Task 4: Implement show and delete commands

**Files:**
- Modify: `plugins/vibes-groove/src/cli/learn.rs`

**Steps:**
1. Implement `show` command:
   - Query learning by ID
   - Display all fields
   - Show source information
   - Show embedding metadata (dimensions, not raw values)
2. Implement `delete` command:
   - Confirm before deletion
   - Remove from store
   - Show success message
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): implement learn show and delete commands`

### Task 5: Implement export command

**Files:**
- Modify: `plugins/vibes-groove/src/cli/learn.rs`

**Steps:**
1. Implement `export` command:
   - Query all learnings (with optional scope filter)
   - Serialize to JSON
   - Output to stdout (pipe-friendly)
2. JSON format:
   ```json
   {
     "learnings": [
       {
         "id": "abc123",
         "description": "...",
         "category": "Correction",
         "confidence": 0.85,
         "scope": "project",
         "created_at": "2024-01-15T10:30:00Z"
       }
     ],
     "exported_at": "2024-01-15T12:00:00Z"
   }
   ```
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): implement learn export command`

### Task 6: Add HTTP routes

**Files:**
- Modify: `plugins/vibes-groove/src/routes.rs`

**Steps:**
1. Add HTTP routes for CLI to query:
   - `GET /api/plugins/groove/learnings` - list with filters
   - `GET /api/plugins/groove/learnings/:id` - get one
   - `DELETE /api/plugins/groove/learnings/:id` - delete
   - `GET /api/plugins/groove/learnings/status` - extraction status
2. Wire routes into plugin
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add learning HTTP routes`

## Acceptance Criteria

- [x] `vibes groove learn status` shows counts and health
- [x] `vibes groove learn list` shows learnings with filters
- [x] `vibes groove learn show <id>` shows full details
- [x] `vibes groove learn delete <id>` removes with confirmation
- [x] `vibes groove learn export` outputs JSON
- [x] HTTP routes support all operations
- [x] All commands work end-to-end

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0025`
3. Commit, push, and create PR
