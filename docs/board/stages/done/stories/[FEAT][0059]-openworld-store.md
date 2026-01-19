---
id: FEAT0059
title: CozoDB schema and store
type: feat
status: done
priority: high
scope: plugin-system
depends: [FEAT0052]
estimate: 2h
created: 2026-01-09
---

# CozoDB schema and store

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement persistence for open-world data including fingerprints, clusters, gaps, and events.

## Context

The open-world system needs persistent storage for pattern fingerprints, anomaly clusters, capability gaps, and failure records. Uses CozoDB with HNSW index for embedding similarity search. See [design.md](../../../milestones/34-open-world-adaptation/design.md).

## Tasks

### Task 1: Define CozoDB schema

**Files:**
- Modify: `plugins/vibes-groove/src/store/schema.rs`

**Steps:**
1. Add open-world relations:
   ```datalog
   :create pattern_fingerprint {
       hash: Int =>
       embedding_json: String,     -- Vec<f32> serialized
       context_summary: String,
       associated_learning: String?,
       created_at: Int
   }

   :create anomaly_cluster {
       id: String =>
       centroid_json: String,      -- Vec<f32> serialized
       member_count: Int,
       created_at: Int,
       last_seen: Int
   }

   :create cluster_member {
       cluster_id: String,
       fingerprint_hash: Int =>
       added_at: Int
   }

   :create capability_gap {
       id: String =>
       category: String,
       severity: String,
       status: String,
       context_pattern: String,
       failure_count: Int,
       first_seen: Int,
       last_seen: Int,
       solutions_json: String      -- Vec<SuggestedSolution> serialized
   }

   :create failure_record {
       id: String =>
       session_id: String,
       failure_type: String,
       context_hash: Int,
       learning_ids_json: String,
       timestamp: Int
   }

   :create novelty_event {
       id: String =>
       event_type: String,
       event_data_json: String,
       timestamp: Int
   }
   ```
2. Add HNSW index for embedding search:
   ```datalog
   ::hnsw create pattern_fingerprint:embedding_idx {
       fields: [embedding_json],
       dim: 384,
       ef: 50,
       m: 16
   }
   ```
3. Add standard indexes
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): add openworld schema`

### Task 2: Implement OpenWorldStore trait

**Files:**
- Modify: `plugins/vibes-groove/src/store/cozo.rs`

**Steps:**
1. Implement fingerprint operations:
   ```rust
   async fn save_fingerprint(&self, fp: &PatternFingerprint) -> Result<()>;
   async fn load_fingerprints(&self) -> Result<Vec<PatternFingerprint>>;
   async fn find_similar_fingerprints(&self, embedding: &[f32], limit: usize) -> Result<Vec<PatternFingerprint>>;
   ```
2. Implement cluster operations:
   ```rust
   async fn save_cluster(&self, cluster: &AnomalyCluster) -> Result<()>;
   async fn load_clusters(&self) -> Result<Vec<AnomalyCluster>>;
   async fn add_cluster_member(&self, cluster_id: ClusterId, hash: u64) -> Result<()>;
   ```
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): implement fingerprint and cluster storage`

### Task 3: Implement gap and failure storage

**Files:**
- Modify: `plugins/vibes-groove/src/store/cozo.rs`

**Steps:**
1. Implement gap operations:
   ```rust
   async fn save_gap(&self, gap: &CapabilityGap) -> Result<()>;
   async fn load_gaps(&self) -> Result<Vec<CapabilityGap>>;
   async fn load_gap(&self, id: GapId) -> Result<Option<CapabilityGap>>;
   async fn update_gap_status(&self, id: GapId, status: GapStatus) -> Result<()>;
   async fn update_gap_solutions(&self, id: GapId, solutions: &[SuggestedSolution]) -> Result<()>;
   ```
2. Implement failure operations:
   ```rust
   async fn save_failure(&self, failure: &FailureRecord) -> Result<()>;
   async fn load_failures_by_context(&self, hash: u64) -> Result<Vec<FailureRecord>>;
   async fn load_failures_by_session(&self, session_id: SessionId) -> Result<Vec<FailureRecord>>;
   ```
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): implement gap and failure storage`

### Task 4: Implement event storage

**Files:**
- Modify: `plugins/vibes-groove/src/store/cozo.rs`

**Steps:**
1. Implement event operations:
   ```rust
   async fn save_novelty_event(&self, event: &OpenWorldEvent) -> Result<()>;
   async fn load_novelty_events(&self, since: DateTime<Utc>, limit: usize) -> Result<Vec<OpenWorldEvent>>;
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `feat(groove): implement event storage`

### Task 5: Add migration and tests

**Files:**
- Modify: `plugins/vibes-groove/src/store/migrations.rs`
- Modify: `plugins/vibes-groove/src/store/cozo.rs`

**Steps:**
1. Add migration for new relations
2. Write tests:
   - Test fingerprint CRUD
   - Test cluster CRUD and member management
   - Test gap CRUD and solution updates
   - Test failure storage and queries
   - Test HNSW similarity search
3. Run: `cargo test -p vibes-groove store::openworld`
4. Commit: `test(groove): add openworld store tests`

## Acceptance Criteria

- [ ] Schema defines all required relations
- [ ] HNSW index for embedding similarity
- [ ] Fingerprint save/load/search works
- [ ] Cluster save/load/member management works
- [ ] Gap CRUD and solution updates work
- [ ] Failure record storage works
- [ ] Event storage works
- [ ] Migration added
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0059`
3. Commit, push, and create PR
