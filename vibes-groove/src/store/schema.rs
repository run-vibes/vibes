//! CozoDB schema definitions for the groove storage layer
//!
//! This module contains the Datalog schema for CozoDB including tables,
//! indexes, and the HNSW vector index for semantic search.

/// Current schema version
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Initial schema creation script (Datalog)
///
/// This script creates all the base tables, indexes, and the HNSW index
/// for semantic search with 384-dimensional embeddings (GteSmall).
pub const INITIAL_SCHEMA: &str = r#"
# Schema version tracking
:create schema_version {
    version: Int =>
    applied_at: Int,
    description: String
}

# Core learning entity
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

# Per-learning usage statistics (updated frequently)
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

# Embeddings for semantic search (384-dim GteSmall)
:create learning_embeddings {
    learning_id: String =>
    embedding: <F32; 384>
}

# Learning relationships
:create learning_relations {
    from_id: String,
    relation_type: String,
    to_id: String =>
    weight: Float,
    created_at: Int
}

# System-wide adaptive parameters
:create adaptive_params {
    param_name: String =>
    value: Float,
    uncertainty: Float,
    observations: Int,
    prior_alpha: Float,
    prior_beta: Float,
    updated_at: Int
}

# Indexes
::index create learning:by_scope { scope }
::index create learning:by_category { category }
::index create learning_relations:by_from { from_id }
::index create learning_relations:by_to { to_id }

# HNSW index for semantic search
::hnsw create learning_embeddings:semantic_idx {
    dim: 384,
    m: 16,
    ef_construction: 200,
    fields: [embedding]
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

/// All migrations in order
pub static MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    description: "Initial schema",
    script: INITIAL_SCHEMA,
}];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version_constant() {
        assert_eq!(CURRENT_SCHEMA_VERSION, 1);
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
}
