---
id: FEAT0053
title: NoveltyDetector with embedding reuse
type: feat
status: in-progress
priority: high
epics: [plugin-system]
depends: [FEAT0052]
estimate: 4h
created: 2026-01-09
milestone: 34-open-world-adaptation
---

# NoveltyDetector with embedding reuse

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement novelty detection using M30's embedder to identify unknown patterns.

## Context

The NoveltyDetector identifies patterns the system hasn't seen before. It reuses the embedder from M30 (learning extraction) to compute semantic similarity. Fast context hashing provides pre-filtering before expensive embedding comparisons. See [design.md](../../../milestones/34-open-world-adaptation/design.md).

## Tasks

### Task 1: Create NoveltyDetector struct

**Files:**
- Create: `plugins/vibes-groove/src/openworld/novelty.rs`

**Steps:**
1. Implement `NoveltyDetector` struct:
   ```rust
   pub struct NoveltyDetector {
       embedder: Arc<dyn Embedder>,
       store: Arc<dyn OpenWorldStore>,
       config: NoveltyConfig,

       // In-memory caches
       known_hashes: HashSet<u64>,
       pending_outliers: Vec<PatternFingerprint>,
       clusters: Vec<AnomalyCluster>,

       // Adaptive threshold
       similarity_threshold: AdaptiveParam,
   }
   ```
2. Implement constructor:
   ```rust
   impl NoveltyDetector {
       pub fn new(
           embedder: Arc<dyn Embedder>,
           store: Arc<dyn OpenWorldStore>,
           config: NoveltyConfig,
       ) -> Self { ... }
   }
   ```
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add NoveltyDetector struct`

### Task 2: Implement detection logic

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/novelty.rs`

**Steps:**
1. Implement `detect()` method:
   ```rust
   pub async fn detect(&mut self, context: &SessionContext) -> Result<NoveltyResult> {
       // Step 1: Fast hash check
       let hash = self.hash_context(context);
       if self.known_hashes.contains(&hash) {
           return Ok(NoveltyResult::Known { ... });
       }

       // Step 2: Compute embedding
       let embedding = self.embedder.embed(&context.to_text()).await?;

       // Step 3: Find nearest cluster
       if let Some(cluster) = self.find_nearest_cluster(&embedding) {
           if self.cosine_similarity(&embedding, &cluster.centroid) > self.similarity_threshold.value() {
               return Ok(NoveltyResult::Known { ... });
           }
       }

       // Step 4: Novel pattern
       Ok(NoveltyResult::Novel { ... })
   }
   ```
2. Implement helper methods:
   - `hash_context()` for fast pre-filtering
   - `cosine_similarity()` for embedding comparison
   - `find_nearest_cluster()` for cluster lookup
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): implement novelty detection logic`

### Task 3: Implement pattern management

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/novelty.rs`

**Steps:**
1. Implement `mark_known()`:
   ```rust
   pub async fn mark_known(&mut self, context: &SessionContext) -> Result<()> {
       let hash = self.hash_context(context);
       self.known_hashes.insert(hash);

       let embedding = self.embedder.embed(&context.to_text()).await?;
       let fingerprint = PatternFingerprint {
           hash,
           embedding,
           context_summary: context.summary(),
           created_at: Utc::now(),
       };

       self.store.save_fingerprint(&fingerprint).await?;
       Ok(())
   }
   ```
2. Implement `add_to_pending()`:
   - Track novel patterns before clustering
   - Trigger re-clustering when threshold reached
3. Implement loading from store on startup
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): implement pattern management`

### Task 4: Wire embedder from plugin

**Files:**
- Modify: `plugins/vibes-groove/src/plugin.rs`
- Modify: `plugins/vibes-groove/src/openworld/mod.rs`

**Steps:**
1. Add `NoveltyDetector` to `GroovePlugin`
2. Initialize with shared embedder from M30
3. Add configuration loading
4. Export from openworld module
5. Run: `cargo check -p vibes-groove`
6. Commit: `feat(groove): wire NoveltyDetector into plugin`

### Task 5: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/novelty.rs`

**Steps:**
1. Create mock embedder for tests
2. Write tests:
   - Test known pattern detection via hash
   - Test known pattern detection via embedding
   - Test novel pattern detection
   - Test mark_known persistence
   - Test threshold adaptation
3. Run: `cargo test -p vibes-groove openworld::novelty`
4. Commit: `test(groove): add novelty detector tests`

## Acceptance Criteria

- [ ] `NoveltyDetector` uses M30's embedder
- [ ] Fast hash pre-filtering works
- [ ] Embedding similarity comparison works
- [ ] Novel patterns correctly identified
- [ ] `mark_known()` updates hashes and store
- [ ] Similarity threshold is adaptive
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0053`
3. Commit, push, and create PR
