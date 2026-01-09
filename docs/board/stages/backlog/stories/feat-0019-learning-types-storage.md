---
id: FEAT0019
title: Learning types and storage
type: feat
status: backlog
priority: high
epics: [plugin-system]
depends: []
estimate: 2h
created: 2026-01-08
updated: 2026-01-08
milestone: 30-learning-extraction
---

# Learning types and storage

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Define core types and CozoDB storage layer for the learning extraction system.

## Context

This is the foundation story for M30 Learning Extraction. All other stories in this milestone depend on these types. See [design.md](../../../milestones/30-learning-extraction/design.md) for architecture.

## Tasks

### Task 1: Create extraction module structure

**Files:**
- Create: `plugins/vibes-groove/src/extraction/mod.rs`
- Create: `plugins/vibes-groove/src/extraction/types.rs`

**Steps:**
1. Create `extraction/` directory in vibes-groove
2. Create `mod.rs` with submodule declarations
3. Create `types.rs` with core type definitions:
   - `Learning` struct with all fields from design
   - `LearningId` newtype
   - `LearningScope` enum (Project, User, Global)
   - `LearningCategory` enum (Correction, ErrorRecovery, Pattern, Preference)
   - `LearningPattern` struct
   - `LearningSource` struct
   - `ExtractionMethod` enum
   - `ExtractionEvent` enum for Iggy
4. Add serde derives for all types
5. Run: `cargo check -p vibes-groove`
6. Commit: `feat(groove): add learning extraction types`

### Task 2: Define LearningStore trait

**Files:**
- Create: `plugins/vibes-groove/src/extraction/store.rs`

**Steps:**
1. Create `store.rs` with `LearningStore` trait
2. Define async methods:
   - `save(&self, learning: &Learning) -> Result<()>`
   - `get(&self, id: LearningId) -> Result<Option<Learning>>`
   - `update(&self, learning: &Learning) -> Result<()>`
   - `delete(&self, id: LearningId) -> Result<()>`
   - `find_by_scope(&self, scope: &LearningScope) -> Result<Vec<Learning>>`
   - `find_similar(&self, embedding: &[f32], threshold: f64) -> Result<Vec<Learning>>`
   - `find_for_injection(&self, context: &SessionContext) -> Result<Vec<Learning>>`
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add LearningStore trait`

### Task 3: Implement CozoDB storage

**Files:**
- Modify: `plugins/vibes-groove/src/store/schema.rs`
- Modify: `plugins/vibes-groove/src/store/cozo.rs`

**Steps:**
1. Add schema definitions for learning tables:
   - `learning` relation with all fields
   - `learning_embedding` relation with HNSW index
   - Standard indexes for scope, category, created_at
2. Implement `LearningStore` for `CozoStore`:
   - CRUD operations using CozoScript queries
   - `find_similar` using HNSW search
   - `find_for_injection` with scope + status filtering
3. Add migration to create tables on init
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): implement CozoDB learning storage`

### Task 4: Add tests

**Files:**
- Create: `plugins/vibes-groove/src/extraction/tests.rs`

**Steps:**
1. Write tests for type serialization
2. Write tests for store CRUD operations
3. Write tests for similarity search (with mock embeddings)
4. Run: `cargo test -p vibes-groove extraction`
5. Commit: `test(groove): add learning storage tests`

## Acceptance Criteria

- [ ] All learning types defined with serde derives
- [ ] LearningStore trait defined with all methods
- [ ] CozoDB implementation passes all tests
- [ ] HNSW index created for embedding similarity
- [ ] `cargo test -p vibes-groove extraction::store` passes

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0019`
3. Commit, push, and create PR
