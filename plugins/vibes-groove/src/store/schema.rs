//! CozoDB schema definitions for the groove storage layer
//!
//! This module contains the Datalog schema for CozoDB including tables,
//! indexes, and the HNSW vector index for semantic search.

/// Current schema version
pub const CURRENT_SCHEMA_VERSION: u32 = 2;

/// Initial schema creation script (Datalog)
///
/// This script creates all the base tables, indexes, and the HNSW index
/// for semantic search with 384-dimensional embeddings (GteSmall).
pub const INITIAL_SCHEMA: &str = r#"
{
    :create schema_version {
        version: Int =>
        applied_at: Int,
        description: String
    }
}
{
    :create learning {
        id: String =>
        scope: String,
        category: String,
        description: String,
        pattern_json: String?,
        insight: String,
        confidence: Float,
        created_at: Int,
        updated_at: Int,
        source_type: String,
        source_json: String
    }
}
{
    :create usage_stats {
        learning_id: String =>
        times_injected: Int,
        times_helpful: Int,
        times_ignored: Int,
        times_contradicted: Int,
        last_used: Int?,
        confidence_alpha: Float,
        confidence_beta: Float
    }
}
{
    :create learning_embeddings {
        learning_id: String =>
        embedding: <F32; 384>
    }
}
{
    :create learning_relations {
        from_id: String,
        relation_type: String,
        to_id: String =>
        weight: Float,
        created_at: Int
    }
}
{
    :create adaptive_params {
        param_name: String =>
        value: Float,
        uncertainty: Float,
        observations: Int,
        prior_alpha: Float,
        prior_beta: Float,
        updated_at: Int
    }
}
{
    ::index create learning:by_scope { scope }
}
{
    ::index create learning:by_category { category }
}
{
    ::index create learning_relations:by_from { from_id }
}
{
    ::index create learning_relations:by_to { to_id }
}
{
    ::hnsw create learning_embeddings:semantic_idx {
        dim: 384,
        m: 16,
        ef_construction: 200,
        fields: [embedding]
    }
}
"#;

/// Schema migration definition
#[derive(Debug, Clone)]
pub struct Migration {
    /// Version number for this migration
    pub version: u32,
    /// Human-readable description of what this migration does
    pub description: &'static str,
    /// The Datalog script to execute for this migration
    pub script: &'static str,
}

/// Open-world adaptation schema (Migration v2)
///
/// Adds tables for pattern fingerprints, anomaly clusters, capability gaps,
/// failure records, and novelty events.
pub const OPENWORLD_SCHEMA: &str = r#"
{
    :create pattern_fingerprint {
        hash: Int =>
        embedding_json: String,
        context_summary: String,
        created_at: Int
    }
}
{
    :create anomaly_cluster {
        id: String =>
        centroid_json: String,
        members_json: String,
        created_at: Int,
        last_seen: Int
    }
}
{
    :create capability_gap {
        id: String =>
        category: String,
        severity: String,
        status: String,
        context_pattern: String,
        failure_count: Int,
        first_seen: Int,
        last_seen: Int,
        solutions_json: String
    }
}
{
    :create failure_record {
        id: String =>
        session_id: String,
        failure_type: String,
        context_hash: Int,
        learning_ids_json: String,
        timestamp: Int
    }
}
{
    :create novelty_event {
        id: String =>
        event_type: String,
        event_data_json: String,
        timestamp: Int
    }
}
{
    ::index create pattern_fingerprint:by_hash { hash }
}
{
    ::index create capability_gap:by_status { status }
}
{
    ::index create capability_gap:by_category { category }
}
{
    ::index create failure_record:by_context { context_hash }
}
{
    ::index create failure_record:by_session { session_id }
}
{
    ::index create novelty_event:by_timestamp { timestamp }
}
"#;

/// All migrations in order
pub static MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "Initial schema",
        script: INITIAL_SCHEMA,
    },
    Migration {
        version: 2,
        description: "Open-world adaptation schema",
        script: OPENWORLD_SCHEMA,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version_constant() {
        assert_eq!(CURRENT_SCHEMA_VERSION, 2);
    }

    #[test]
    fn test_migrations_count() {
        assert_eq!(MIGRATIONS.len(), 2);
    }

    #[test]
    fn test_initial_schema_contains_learning_table() {
        assert!(INITIAL_SCHEMA.contains(":create learning {"));
    }

    #[test]
    fn test_initial_schema_contains_usage_stats() {
        assert!(INITIAL_SCHEMA.contains(":create usage_stats {"));
    }

    #[test]
    fn test_initial_schema_contains_embeddings() {
        assert!(INITIAL_SCHEMA.contains(":create learning_embeddings {"));
    }

    #[test]
    fn test_initial_schema_contains_relations() {
        assert!(INITIAL_SCHEMA.contains(":create learning_relations {"));
    }

    #[test]
    fn test_initial_schema_contains_params() {
        assert!(INITIAL_SCHEMA.contains(":create adaptive_params {"));
    }

    #[test]
    fn test_initial_schema_contains_version_table() {
        assert!(INITIAL_SCHEMA.contains(":create schema_version {"));
    }

    // ==========================================================================
    // Open-world schema tests
    // ==========================================================================

    #[test]
    fn test_openworld_schema_contains_fingerprint() {
        assert!(OPENWORLD_SCHEMA.contains(":create pattern_fingerprint {"));
    }

    #[test]
    fn test_openworld_schema_contains_cluster() {
        assert!(OPENWORLD_SCHEMA.contains(":create anomaly_cluster {"));
    }

    #[test]
    fn test_openworld_schema_contains_gap() {
        assert!(OPENWORLD_SCHEMA.contains(":create capability_gap {"));
    }

    #[test]
    fn test_openworld_schema_contains_failure() {
        assert!(OPENWORLD_SCHEMA.contains(":create failure_record {"));
    }

    #[test]
    fn test_openworld_schema_contains_event() {
        assert!(OPENWORLD_SCHEMA.contains(":create novelty_event {"));
    }

    #[test]
    fn test_openworld_schema_contains_indexes() {
        assert!(OPENWORLD_SCHEMA.contains("::index create pattern_fingerprint:by_hash"));
        assert!(OPENWORLD_SCHEMA.contains("::index create capability_gap:by_status"));
        assert!(OPENWORLD_SCHEMA.contains("::index create failure_record:by_context"));
    }
}
