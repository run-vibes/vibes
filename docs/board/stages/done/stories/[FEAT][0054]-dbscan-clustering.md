---
id: FEAT0054
title: Incremental DBSCAN clustering
type: feat
status: done
priority: high
scope: plugin-system
depends: [FEAT0053]
estimate: 3h
created: 2026-01-09
---

# Incremental DBSCAN clustering

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement online clustering for novel patterns using incremental DBSCAN.

## Context

Novel patterns are clustered to identify recurring unknown situations. Traditional DBSCAN requires all data upfront; incremental DBSCAN processes new points as they arrive. This enables real-time pattern discovery. See [design.md](../../../milestones/34-open-world-adaptation/design.md).

## Tasks

### Task 1: Implement distance functions

**Files:**
- Create: `plugins/vibes-groove/src/openworld/clustering.rs`

**Steps:**
1. Implement distance metrics:
   ```rust
   /// Euclidean distance in embedding space
   pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
       a.iter()
           .zip(b.iter())
           .map(|(x, y)| (x - y).powi(2))
           .sum::<f32>()
           .sqrt()
   }

   /// Cosine similarity (1 - cosine = distance)
   pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
       1.0 - cosine_similarity(a, b)
   }
   ```
2. Add configuration for distance metric choice
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add clustering distance functions`

### Task 2: Implement region query

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/clustering.rs`

**Steps:**
1. Implement `region_query()`:
   ```rust
   /// Find all points within eps distance of query point
   fn region_query(
       query: &[f32],
       points: &[PatternFingerprint],
       eps: f32,
   ) -> Vec<usize> {
       points
           .iter()
           .enumerate()
           .filter(|(_, p)| euclidean_distance(query, &p.embedding) <= eps)
           .map(|(i, _)| i)
           .collect()
   }
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `feat(groove): add region query`

### Task 3: Implement incremental DBSCAN

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/clustering.rs`

**Steps:**
1. Implement `incremental_dbscan()`:
   ```rust
   /// Process new outliers against existing clusters
   pub fn incremental_dbscan(
       pending: &[PatternFingerprint],
       existing_clusters: &mut Vec<AnomalyCluster>,
       eps: f32,
       min_points: usize,
   ) -> Vec<AnomalyCluster> {
       let mut new_clusters = Vec::new();

       for point in pending {
           // Try to add to existing cluster
           if let Some(cluster) = find_nearest_cluster(point, existing_clusters, eps) {
               cluster.add_member(point.clone());
               continue;
           }

           // Try to form new cluster from pending
           let neighbors = region_query(&point.embedding, pending, eps);
           if neighbors.len() >= min_points {
               let cluster = expand_cluster(point, &neighbors, pending, eps, min_points);
               new_clusters.push(cluster);
           }
       }

       new_clusters
   }
   ```
2. Implement `expand_cluster()` for cluster growth
3. Implement centroid recalculation
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): implement incremental DBSCAN`

### Task 4: Integrate with NoveltyDetector

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/novelty.rs`

**Steps:**
1. Add `maybe_recluster()` method:
   ```rust
   pub async fn maybe_recluster(&mut self) -> Result<Vec<AnomalyCluster>> {
       if self.pending_outliers.len() < self.config.min_pending_for_cluster {
           return Ok(vec![]);
       }

       let new_clusters = incremental_dbscan(
           &self.pending_outliers,
           &mut self.clusters,
           self.config.eps,
           self.config.min_points,
       );

       // Persist new clusters
       for cluster in &new_clusters {
           self.store.save_cluster(cluster).await?;
       }

       // Clear clustered points from pending
       self.clear_clustered_pending(&new_clusters);

       Ok(new_clusters)
   }
   ```
2. Call from detection flow when appropriate
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): integrate DBSCAN with NoveltyDetector`

### Task 5: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/clustering.rs`

**Steps:**
1. Write tests with synthetic embeddings:
   - Test distance functions
   - Test region query
   - Test cluster formation
   - Test cluster merging
   - Test incremental updates
2. Run: `cargo test -p vibes-groove openworld::clustering`
3. Commit: `test(groove): add clustering tests`

## Acceptance Criteria

- [ ] Distance functions work correctly
- [ ] Region query finds neighbors within epsilon
- [ ] Incremental DBSCAN forms clusters from dense regions
- [ ] New points can join existing clusters
- [ ] Centroid recalculation works
- [ ] Integration with NoveltyDetector complete
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0054`
3. Commit, push, and create PR
