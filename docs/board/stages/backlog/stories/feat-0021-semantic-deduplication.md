---
id: FEAT0021
title: Semantic deduplication
type: feat
status: backlog
priority: high
epics: [plugin-system]
depends: [FEAT0020]
estimate: 2h
created: 2026-01-08
updated: 2026-01-08
milestone: 30-learning-extraction
---

# Semantic deduplication

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement configurable semantic deduplication to prevent storing duplicate learnings.

## Context

When extracting learnings, we may encounter the same insight multiple times (e.g., user corrects the same mistake repeatedly). Semantic deduplication uses embeddings to find and merge similar learnings. See [design.md](../../../milestones/30-learning-extraction/design.md).

## Tasks

### Task 1: Create DeduplicationStrategy trait

**Files:**
- Create: `plugins/vibes-groove/src/extraction/dedup.rs`

**Steps:**
1. Define `DeduplicationStrategy` trait:
   ```rust
   #[async_trait]
   pub trait DeduplicationStrategy: Send + Sync {
       /// Check if a candidate learning is a duplicate of an existing one
       async fn find_duplicate(
           &self,
           candidate: &Learning,
           store: &dyn LearningStore,
       ) -> Result<Option<Learning>>;

       /// Merge a duplicate into an existing learning
       async fn merge(
           &self,
           existing: &Learning,
           duplicate: &Learning,
       ) -> Result<Learning>;
   }
   ```
2. Add module to `extraction/mod.rs`
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add DeduplicationStrategy trait`

### Task 2: Implement SemanticDedup

**Files:**
- Modify: `plugins/vibes-groove/src/extraction/dedup.rs`

**Steps:**
1. Create `SemanticDedup` struct:
   - `similarity_threshold: f64` (default 0.9)
   - `embedder: Arc<dyn Embedder>`
2. Implement `find_duplicate()`:
   - Embed candidate description
   - Query store for similar learnings (using HNSW)
   - Return first match above threshold (if any)
3. Implement `merge()`:
   - Keep original description
   - Average confidence scores
   - Update timestamp to latest
   - Combine sources (if tracking multiple)
   - Increment usage count
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): implement SemanticDedup`

### Task 3: Add configuration

**Files:**
- Modify: `plugins/vibes-groove/src/config.rs`

**Steps:**
1. Add deduplication config:
   ```rust
   pub struct DeduplicationConfig {
       pub enabled: bool,
       pub similarity_threshold: f64,
   }
   ```
2. Add defaults (enabled=true, threshold=0.9)
3. Wire into extraction config
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): add deduplication config`

### Task 4: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/extraction/dedup.rs`

**Steps:**
1. Write tests:
   - Test exact duplicate detection (similarity = 1.0)
   - Test near-duplicate detection (similarity > threshold)
   - Test non-duplicate (similarity < threshold)
   - Test merge behavior (confidence averaging)
   - Test threshold configuration
2. Run: `cargo test -p vibes-groove extraction::dedup`
3. Commit: `test(groove): add deduplication tests`

## Acceptance Criteria

- [ ] `DeduplicationStrategy` trait defined
- [ ] `SemanticDedup` uses embeddings for similarity
- [ ] Configurable similarity threshold
- [ ] Merge preserves original description
- [ ] Merge averages confidence scores
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0021`
3. Commit, push, and create PR
