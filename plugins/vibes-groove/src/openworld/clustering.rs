//! Incremental DBSCAN clustering for novel patterns
//!
//! Implements online clustering to group similar novel patterns for
//! eventual classification. Uses DBSCAN with incremental updates.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openworld::types::PatternFingerprint;
    use chrono::Utc;

    // Helper to create fingerprint with embedding
    fn fp(embedding: Vec<f32>) -> PatternFingerprint {
        PatternFingerprint {
            hash: 0,
            embedding,
            context_summary: "test".to_string(),
            created_at: Utc::now(),
        }
    }

    // =========================================================================
    // Distance function tests
    // =========================================================================

    #[test]
    fn test_euclidean_distance_identical() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        assert!((euclidean_distance(&a, &b) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_euclidean_distance_known_value() {
        // Distance from (0,0,0) to (3,4,0) = 5
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0];
        assert!((euclidean_distance(&a, &b) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_euclidean_distance_symmetric() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        assert!((euclidean_distance(&a, &b) - euclidean_distance(&b, &a)).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_distance_identical() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_distance(&a, &b) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_distance_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!((cosine_distance(&a, &b) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_distance_opposite() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![-1.0, 0.0, 0.0];
        assert!((cosine_distance(&a, &b) - 2.0).abs() < 1e-6);
    }

    // =========================================================================
    // Region query tests
    // =========================================================================

    #[test]
    fn test_region_query_finds_nearby_points() {
        let points = vec![
            fp(vec![0.0, 0.0]),
            fp(vec![0.5, 0.0]), // Within eps=1.0
            fp(vec![0.0, 0.5]), // Within eps=1.0
            fp(vec![5.0, 5.0]), // Outside eps=1.0
        ];

        let query = vec![0.0, 0.0];
        let neighbors = region_query(&query, &points, 1.0, DistanceMetric::Euclidean);

        assert_eq!(neighbors.len(), 3); // First 3 points
        assert!(neighbors.contains(&0));
        assert!(neighbors.contains(&1));
        assert!(neighbors.contains(&2));
        assert!(!neighbors.contains(&3));
    }

    #[test]
    fn test_region_query_empty_when_none_nearby() {
        let points = vec![fp(vec![10.0, 10.0]), fp(vec![20.0, 20.0])];

        let query = vec![0.0, 0.0];
        let neighbors = region_query(&query, &points, 1.0, DistanceMetric::Euclidean);

        assert!(neighbors.is_empty());
    }

    // =========================================================================
    // DBSCAN configuration tests
    // =========================================================================

    #[test]
    fn test_dbscan_config_defaults() {
        let config = DbscanConfig::default();
        assert!(config.eps > 0.0);
        assert!(config.min_points > 0);
    }

    // =========================================================================
    // Cluster formation tests
    // =========================================================================

    #[test]
    fn test_incremental_dbscan_forms_cluster_from_dense_region() {
        // Create 5 points close together - should form a cluster
        let pending = vec![
            fp(vec![0.0, 0.0]),
            fp(vec![0.1, 0.0]),
            fp(vec![0.0, 0.1]),
            fp(vec![0.1, 0.1]),
            fp(vec![0.05, 0.05]),
        ];

        let config = DbscanConfig {
            eps: 0.5,
            min_points: 3,
            metric: DistanceMetric::Euclidean,
        };

        let mut existing_clusters = Vec::new();
        let result = incremental_dbscan(&pending, &mut existing_clusters, &config);

        assert_eq!(result.new_clusters.len(), 1);
        assert!(result.new_clusters[0].members.len() >= 3);
    }

    #[test]
    fn test_incremental_dbscan_no_cluster_when_sparse() {
        // Create points far apart - should not form a cluster
        let pending = vec![
            fp(vec![0.0, 0.0]),
            fp(vec![10.0, 10.0]),
            fp(vec![20.0, 20.0]),
        ];

        let config = DbscanConfig {
            eps: 0.5,
            min_points: 3,
            metric: DistanceMetric::Euclidean,
        };

        let mut existing_clusters = Vec::new();
        let result = incremental_dbscan(&pending, &mut existing_clusters, &config);

        assert!(result.new_clusters.is_empty());
    }

    #[test]
    fn test_incremental_dbscan_joins_existing_cluster() {
        use crate::openworld::types::AnomalyCluster;
        use uuid::Uuid;

        // Existing cluster at (0,0)
        let existing = AnomalyCluster {
            id: Uuid::now_v7(),
            centroid: vec![0.0, 0.0],
            members: vec![fp(vec![0.0, 0.0])],
            created_at: Utc::now(),
            last_seen: Utc::now(),
        };

        let mut existing_clusters = vec![existing];

        // New point close to existing cluster
        let pending = vec![fp(vec![0.1, 0.1])];

        let config = DbscanConfig {
            eps: 0.5,
            min_points: 2,
            metric: DistanceMetric::Euclidean,
        };

        let result = incremental_dbscan(&pending, &mut existing_clusters, &config);

        // Should join existing cluster, not form new one
        assert!(result.new_clusters.is_empty());
        assert_eq!(existing_clusters[0].members.len(), 2);
    }

    #[test]
    fn test_centroid_calculation() {
        let points = vec![
            fp(vec![0.0, 0.0]),
            fp(vec![2.0, 0.0]),
            fp(vec![0.0, 2.0]),
            fp(vec![2.0, 2.0]),
        ];

        let centroid = compute_centroid(&points);
        assert_eq!(centroid, vec![1.0, 1.0]);
    }

    #[test]
    fn test_centroid_empty_returns_empty() {
        let points: Vec<PatternFingerprint> = vec![];
        let centroid = compute_centroid(&points);
        assert!(centroid.is_empty());
    }
}

// =============================================================================
// Implementation
// =============================================================================

use crate::extraction::embedder::cosine_similarity;
use crate::openworld::types::{AnomalyCluster, PatternFingerprint};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Distance metric for clustering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceMetric {
    /// Euclidean (L2) distance
    Euclidean,
    /// Cosine distance (1 - similarity)
    Cosine,
}

impl Default for DistanceMetric {
    fn default() -> Self {
        Self::Euclidean
    }
}

/// Configuration for DBSCAN clustering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbscanConfig {
    /// Maximum distance between points in a cluster
    pub eps: f32,
    /// Minimum points to form a dense region
    pub min_points: usize,
    /// Distance metric to use
    pub metric: DistanceMetric,
}

impl Default for DbscanConfig {
    fn default() -> Self {
        Self {
            eps: 0.3,
            min_points: 3,
            metric: DistanceMetric::Euclidean,
        }
    }
}

/// Result of incremental DBSCAN
#[derive(Debug)]
pub struct DbscanResult {
    /// Newly formed clusters
    pub new_clusters: Vec<AnomalyCluster>,
    /// Indices of points that were clustered (added to existing or new)
    pub clustered_indices: Vec<usize>,
}

/// Euclidean distance between two vectors
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "Vectors must have same length");
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

/// Cosine distance (1 - similarity)
pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    1.0 - cosine_similarity(a, b)
}

/// Calculate distance using specified metric
fn distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> f32 {
    match metric {
        DistanceMetric::Euclidean => euclidean_distance(a, b),
        DistanceMetric::Cosine => cosine_distance(a, b),
    }
}

/// Find all points within eps distance of query point
pub fn region_query(
    query: &[f32],
    points: &[PatternFingerprint],
    eps: f32,
    metric: DistanceMetric,
) -> Vec<usize> {
    points
        .iter()
        .enumerate()
        .filter(|(_, p)| distance(query, &p.embedding, metric) <= eps)
        .map(|(i, _)| i)
        .collect()
}

/// Compute centroid of pattern fingerprints
pub fn compute_centroid(points: &[PatternFingerprint]) -> Vec<f32> {
    if points.is_empty() {
        return Vec::new();
    }

    let dim = points[0].embedding.len();
    let mut centroid = vec![0.0f32; dim];
    let count = points.len() as f32;

    for point in points {
        for (i, &val) in point.embedding.iter().enumerate() {
            centroid[i] += val;
        }
    }

    for val in &mut centroid {
        *val /= count;
    }

    centroid
}

/// Find nearest existing cluster within eps distance
fn find_nearest_cluster<'a>(
    point: &PatternFingerprint,
    clusters: &'a mut [AnomalyCluster],
    config: &DbscanConfig,
) -> Option<&'a mut AnomalyCluster> {
    let mut best: Option<(usize, f32)> = None;

    for (i, cluster) in clusters.iter().enumerate() {
        let dist = distance(&point.embedding, &cluster.centroid, config.metric);
        if dist <= config.eps && (best.is_none() || dist < best.unwrap().1) {
            best = Some((i, dist));
        }
    }

    best.map(|(i, _)| &mut clusters[i])
}

/// Expand cluster from seed point
fn expand_cluster(
    seed_idx: usize,
    points: &[PatternFingerprint],
    assigned: &mut [bool],
    config: &DbscanConfig,
) -> Vec<PatternFingerprint> {
    let mut cluster_members = vec![points[seed_idx].clone()];
    assigned[seed_idx] = true;

    let mut queue: Vec<usize> = region_query(
        &points[seed_idx].embedding,
        points,
        config.eps,
        config.metric,
    );

    while let Some(idx) = queue.pop() {
        if assigned[idx] {
            continue;
        }

        assigned[idx] = true;
        cluster_members.push(points[idx].clone());

        // Find neighbors of this point
        let neighbors = region_query(&points[idx].embedding, points, config.eps, config.metric);
        if neighbors.len() >= config.min_points {
            // Core point - add its neighbors to queue
            for n in neighbors {
                if !assigned[n] {
                    queue.push(n);
                }
            }
        }
    }

    cluster_members
}

/// Incremental DBSCAN clustering
///
/// Process pending outliers against existing clusters:
/// 1. Try to add each point to nearest existing cluster
/// 2. Form new clusters from dense regions in pending points
pub fn incremental_dbscan(
    pending: &[PatternFingerprint],
    existing_clusters: &mut [AnomalyCluster],
    config: &DbscanConfig,
) -> DbscanResult {
    let mut new_clusters = Vec::new();
    let mut clustered_indices = Vec::new();
    let mut assigned = vec![false; pending.len()];

    // Phase 1: Try to add points to existing clusters
    for (i, point) in pending.iter().enumerate() {
        if let Some(cluster) = find_nearest_cluster(point, existing_clusters, config) {
            cluster.members.push(point.clone());
            cluster.centroid = compute_centroid(&cluster.members);
            cluster.last_seen = Utc::now();
            assigned[i] = true;
            clustered_indices.push(i);
        }
    }

    // Phase 2: Form new clusters from remaining points
    for i in 0..pending.len() {
        if assigned[i] {
            continue;
        }

        let neighbors = region_query(&pending[i].embedding, pending, config.eps, config.metric);

        // Filter to only unassigned neighbors
        let unassigned_neighbors: Vec<usize> = neighbors
            .iter()
            .filter(|&&idx| !assigned[idx])
            .copied()
            .collect();

        if unassigned_neighbors.len() >= config.min_points {
            // Form new cluster
            let members = expand_cluster(i, pending, &mut assigned, config);

            // Record clustered indices
            for (j, is_assigned) in assigned.iter().enumerate() {
                if *is_assigned && !clustered_indices.contains(&j) {
                    clustered_indices.push(j);
                }
            }

            let cluster = AnomalyCluster {
                id: uuid::Uuid::now_v7(),
                centroid: compute_centroid(&members),
                members,
                created_at: Utc::now(),
                last_seen: Utc::now(),
            };

            new_clusters.push(cluster);
        }
    }

    DbscanResult {
        new_clusters,
        clustered_indices,
    }
}
