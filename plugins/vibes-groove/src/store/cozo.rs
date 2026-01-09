//! CozoDB-backed learning store implementation
//!
//! This module provides the main `CozoStore` struct that wraps CozoDB with
//! RocksDB backend. It handles database creation, schema initialization,
//! and migrations.

use std::collections::BTreeMap;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use cozo::{DataValue, DbInstance, NamedRows, Vector};
use ndarray::Array1;

use super::schema::MIGRATIONS;
use crate::{
    AdaptiveParam, GrooveError, Learning, LearningCategory, LearningContent, LearningId,
    LearningRelation, LearningSource, RelationType, Result, Scope, SystemParam, UsageStats,
};

/// CozoDB-backed learning store
pub struct CozoStore {
    db: Arc<DbInstance>,
    initialized: bool,
}

impl CozoStore {
    /// Open or create a groove database at the given path
    pub async fn open(path: &Path) -> Result<Self> {
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| GrooveError::Database(format!("Failed to create directory: {e}")))?;
        }

        let db = DbInstance::new("rocksdb", path, "")
            .map_err(|e| GrooveError::Database(format!("Failed to open database: {e}")))?;

        let db = Arc::new(db);
        let mut store = Self {
            db,
            initialized: false,
        };

        store.ensure_schema().await?;
        store.initialized = true;

        Ok(store)
    }

    /// Check if store is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get the underlying database instance
    ///
    /// Useful for sharing the DB with other stores (e.g., strategy store)
    pub fn db(&self) -> Arc<DbInstance> {
        self.db.clone()
    }

    /// Get current schema version from database
    pub async fn get_schema_version(&self) -> Result<u32> {
        let query = "?[version] := *schema_version{version}, version = max(version)";

        match self.run_query(query, Default::default()).await {
            Ok(rows) if !rows.rows.is_empty() => {
                let version = rows.rows[0][0]
                    .get_int()
                    .ok_or_else(|| GrooveError::Database("Invalid version type".into()))?;
                Ok(version as u32)
            }
            Ok(_) => Ok(0), // No version recorded
            Err(e) => {
                // Table might not exist yet
                let msg = e.to_string();
                if msg.contains("not found") || msg.contains("Cannot find") {
                    Ok(0)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Ensure schema is up to date
    async fn ensure_schema(&mut self) -> Result<()> {
        let current = self.get_schema_version().await?;

        for migration in MIGRATIONS.iter().filter(|m| m.version > current) {
            self.apply_migration(migration).await?;
        }

        Ok(())
    }

    /// Apply a single migration
    async fn apply_migration(&self, migration: &super::schema::Migration) -> Result<()> {
        // Run migration script
        self.db
            .run_script(
                migration.script,
                Default::default(),
                cozo::ScriptMutability::Mutable,
            )
            .map_err(|e| {
                GrooveError::Migration(format!("Migration {} failed: {e}", migration.version))
            })?;

        // Record migration
        let now = Utc::now().timestamp();
        let record_query = format!(
            "?[version, applied_at, description] <- [[{}, {}, '{}']] :put schema_version {{version => applied_at, description}}",
            migration.version,
            now,
            migration.description.replace('\'', "''")
        );

        self.db
            .run_script(
                &record_query,
                Default::default(),
                cozo::ScriptMutability::Mutable,
            )
            .map_err(|e| {
                GrooveError::Migration(format!(
                    "Failed to record migration {}: {e}",
                    migration.version
                ))
            })?;

        Ok(())
    }

    /// Run a query and return results
    async fn run_query(
        &self,
        query: &str,
        params: BTreeMap<String, DataValue>,
    ) -> Result<NamedRows> {
        self.db
            .run_script(query, params, cozo::ScriptMutability::Immutable)
            .map_err(|e| GrooveError::Database(format!("Query failed: {e}")))
    }

    /// Run a mutation query
    async fn run_mutation(
        &self,
        query: &str,
        params: BTreeMap<String, DataValue>,
    ) -> Result<NamedRows> {
        self.db
            .run_script(query, params, cozo::ScriptMutability::Mutable)
            .map_err(|e| GrooveError::Database(format!("Mutation failed: {e}")))
    }

    /// Store a new learning, returns its ID
    pub async fn store(&self, learning: &Learning) -> Result<LearningId> {
        let id_str = learning.id.to_string();
        let scope_str = learning.scope.to_db_string();
        let category_str = learning.category.as_str();
        let description = learning.content.description.replace('\'', "''");
        let pattern_json = learning
            .content
            .pattern
            .as_ref()
            .map(|p| format!("'{}'", p.to_string().replace('\'', "''")))
            .unwrap_or_else(|| "null".to_string());
        let insight = learning.content.insight.replace('\'', "''");
        let confidence = learning.confidence;
        let created_at = learning.created_at.timestamp();
        let updated_at = learning.updated_at.timestamp();
        let source_type = learning.source.source_type();
        let source_json = serde_json::to_string(&learning.source)
            .map_err(|e| GrooveError::Serialization(e.to_string()))?
            .replace('\'', "''");

        // Insert learning
        let query = format!(
            r#"?[id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json] <- [[
                '{}', '{}', '{}', '{}', {}, '{}', {}, {}, {}, '{}', '{}'
            ]]
            :put learning {{
                id => scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json
            }}"#,
            id_str,
            scope_str,
            category_str,
            description,
            pattern_json,
            insight,
            confidence,
            created_at,
            updated_at,
            source_type,
            source_json
        );

        self.run_mutation(&query, Default::default()).await?;

        // Create default usage stats
        let stats = UsageStats::default();
        let usage_query = format!(
            r#"?[learning_id, times_injected, times_helpful, times_ignored, times_contradicted, last_used, confidence_alpha, confidence_beta] <- [[
                '{}', {}, {}, {}, {}, null, {}, {}
            ]]
            :put usage_stats {{
                learning_id => times_injected, times_helpful, times_ignored, times_contradicted, last_used, confidence_alpha, confidence_beta
            }}"#,
            id_str,
            stats.times_injected,
            stats.times_helpful,
            stats.times_ignored,
            stats.times_contradicted,
            stats.confidence_alpha,
            stats.confidence_beta
        );

        self.run_mutation(&usage_query, Default::default()).await?;

        Ok(learning.id)
    }

    /// Retrieve a learning by ID
    pub async fn get(&self, id: LearningId) -> Result<Option<Learning>> {
        let query = format!(
            r#"?[id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json] :=
                *learning{{id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json}},
                id = '{}'"#,
            id
        );

        let rows = self.run_query(&query, Default::default()).await?;

        if rows.rows.is_empty() {
            return Ok(None);
        }

        self.row_to_learning(&rows.rows[0])
    }

    /// Retrieve multiple learnings by IDs in a single query (batch fetch)
    async fn get_many(&self, ids: &[LearningId]) -> Result<Vec<Learning>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        // Build IN clause with quoted UUIDs
        let id_list = ids
            .iter()
            .map(|id| format!("'{}'", id))
            .collect::<Vec<_>>()
            .join(", ");

        let query = format!(
            r#"?[id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json] :=
                *learning{{id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json}},
                id in [{}]"#,
            id_list
        );

        let rows = self.run_query(&query, Default::default()).await?;

        let mut learnings = Vec::new();
        for row in &rows.rows {
            if let Ok(Some(learning)) = self.row_to_learning(row) {
                learnings.push(learning);
            }
        }

        Ok(learnings)
    }

    /// Find all learnings in a scope
    pub async fn find_by_scope(&self, scope: &Scope) -> Result<Vec<Learning>> {
        let scope_str = scope.to_db_string();
        let query = format!(
            r#"?[id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json] :=
                *learning{{id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json}},
                scope = '{}'"#,
            scope_str
        );

        let rows = self.run_query(&query, Default::default()).await?;

        let mut learnings = Vec::new();
        for row in &rows.rows {
            if let Some(learning) = self.row_to_learning(row)? {
                learnings.push(learning);
            }
        }

        Ok(learnings)
    }

    /// Find learnings by category
    pub async fn find_by_category(&self, category: &LearningCategory) -> Result<Vec<Learning>> {
        let category_str = category.as_str();
        let query = format!(
            r#"?[id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json] :=
                *learning{{id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json}},
                category = '{}'"#,
            category_str
        );

        let rows = self.run_query(&query, Default::default()).await?;

        let mut learnings = Vec::new();
        for row in &rows.rows {
            if let Some(learning) = self.row_to_learning(row)? {
                learnings.push(learning);
            }
        }

        Ok(learnings)
    }

    /// Delete a learning and all related data
    pub async fn delete(&self, id: LearningId) -> Result<bool> {
        // Check if exists first
        if self.get(id).await?.is_none() {
            return Ok(false);
        }

        let id_str = id.to_string();

        // Delete from learning table
        let learning_query = format!("?[id] <- [['{}']]:rm learning {{id}}", id_str);
        self.run_mutation(&learning_query, Default::default())
            .await?;

        // Delete from usage_stats table
        let usage_query = format!(
            "?[learning_id] <- [['{}']]:rm usage_stats {{learning_id}}",
            id_str
        );
        self.run_mutation(&usage_query, Default::default()).await?;

        // Delete from learning_embeddings table
        let embeddings_query = format!(
            "?[learning_id] <- [['{}']]:rm learning_embeddings {{learning_id}}",
            id_str
        );
        self.run_mutation(&embeddings_query, Default::default())
            .await?;

        Ok(true)
    }

    /// Count learnings (for stats)
    pub async fn count(&self) -> Result<u64> {
        let query = "?[count(id)] := *learning{id}";

        let rows = self.run_query(query, Default::default()).await?;

        if rows.rows.is_empty() {
            return Ok(0);
        }

        let count = rows.rows[0][0]
            .get_int()
            .ok_or_else(|| GrooveError::Database("Invalid count type".into()))?;

        Ok(count as u64)
    }

    /// Update usage stats for a learning
    pub async fn update_usage(&self, id: LearningId, stats: &UsageStats) -> Result<()> {
        let id_str = id.to_string();
        let last_used = stats.last_used.map(|dt| dt.timestamp()).unwrap_or(-1);
        let last_used_str = if last_used < 0 {
            "null".to_string()
        } else {
            last_used.to_string()
        };

        let query = format!(
            r#"?[learning_id, times_injected, times_helpful, times_ignored, times_contradicted, last_used, confidence_alpha, confidence_beta] <- [[
                '{}', {}, {}, {}, {}, {}, {}, {}
            ]]
            :put usage_stats {{
                learning_id => times_injected, times_helpful, times_ignored, times_contradicted, last_used, confidence_alpha, confidence_beta
            }}"#,
            id_str,
            stats.times_injected,
            stats.times_helpful,
            stats.times_ignored,
            stats.times_contradicted,
            last_used_str,
            stats.confidence_alpha,
            stats.confidence_beta,
        );

        self.run_mutation(&query, Default::default()).await?;
        Ok(())
    }

    /// Retrieve usage stats for a learning
    pub async fn get_usage(&self, id: LearningId) -> Result<Option<UsageStats>> {
        let query = format!(
            r#"?[times_injected, times_helpful, times_ignored, times_contradicted, last_used, confidence_alpha, confidence_beta] :=
                *usage_stats{{learning_id, times_injected, times_helpful, times_ignored, times_contradicted, last_used, confidence_alpha, confidence_beta}},
                learning_id = '{}'"#,
            id
        );

        let rows = self.run_query(&query, Default::default()).await?;

        if rows.rows.is_empty() {
            return Ok(None);
        }

        let row = &rows.rows[0];

        let times_injected = row[0]
            .get_int()
            .ok_or_else(|| GrooveError::Database("Invalid times_injected".into()))?
            as u32;
        let times_helpful = row[1]
            .get_int()
            .ok_or_else(|| GrooveError::Database("Invalid times_helpful".into()))?
            as u32;
        let times_ignored = row[2]
            .get_int()
            .ok_or_else(|| GrooveError::Database("Invalid times_ignored".into()))?
            as u32;
        let times_contradicted = row[3]
            .get_int()
            .ok_or_else(|| GrooveError::Database("Invalid times_contradicted".into()))?
            as u32;

        let last_used = match &row[4] {
            cozo::DataValue::Null => None,
            val => val
                .get_int()
                .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0)),
        };

        let confidence_alpha = row[5]
            .get_float()
            .ok_or_else(|| GrooveError::Database("Invalid confidence_alpha".into()))?;
        let confidence_beta = row[6]
            .get_float()
            .ok_or_else(|| GrooveError::Database("Invalid confidence_beta".into()))?;

        Ok(Some(UsageStats {
            times_injected,
            times_helpful,
            times_ignored,
            times_contradicted,
            last_used,
            confidence_alpha,
            confidence_beta,
        }))
    }

    /// Store a relation between two learnings
    pub async fn store_relation(&self, relation: &LearningRelation) -> Result<()> {
        let query = format!(
            r#"?[from_id, relation_type, to_id, weight, created_at] <- [[
                '{}', '{}', '{}', {}, {}
            ]]
            :put learning_relations {{
                from_id, relation_type, to_id => weight, created_at
            }}"#,
            relation.from_id,
            relation.relation_type.as_str(),
            relation.to_id,
            relation.weight,
            relation.created_at.timestamp(),
        );

        self.run_mutation(&query, Default::default()).await?;
        Ok(())
    }

    /// Find learnings related to the given learning ID
    pub async fn find_related(
        &self,
        id: LearningId,
        relation_type: Option<&RelationType>,
    ) -> Result<Vec<Learning>> {
        // Build query based on whether relation_type filter is provided
        let base_query = if let Some(rt) = relation_type {
            format!(
                r#"?[to_id] := *learning_relations{{from_id, relation_type, to_id}}, from_id = '{}', relation_type = '{}'"#,
                id,
                rt.as_str()
            )
        } else {
            format!(
                r#"?[to_id] := *learning_relations{{from_id, to_id}}, from_id = '{}'"#,
                id
            )
        };

        let rows = self.run_query(&base_query, Default::default()).await?;

        // Collect all IDs first, then batch fetch (avoids N+1 queries)
        let ids: Vec<LearningId> = rows
            .rows
            .iter()
            .filter_map(|row| row[0].get_str().and_then(|s| uuid::Uuid::parse_str(s).ok()))
            .collect();

        self.get_many(&ids).await
    }

    /// Helper to convert a database row to a Learning struct
    fn row_to_learning(&self, row: &[DataValue]) -> Result<Option<Learning>> {
        if row.len() < 11 {
            return Ok(None);
        }

        // Extract values from row
        // [id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json]
        let id_str = row[0]
            .get_str()
            .ok_or_else(|| GrooveError::Database("Invalid id type".into()))?;
        let id = LearningId::parse_str(id_str)
            .map_err(|e| GrooveError::Database(format!("Invalid UUID: {e}")))?;

        let scope_str = row[1]
            .get_str()
            .ok_or_else(|| GrooveError::Database("Invalid scope type".into()))?;
        let scope = Scope::from_db_string(scope_str)?;

        let category_str = row[2]
            .get_str()
            .ok_or_else(|| GrooveError::Database("Invalid category type".into()))?;
        let category = LearningCategory::from_str(category_str)
            .map_err(|e| GrooveError::Database(format!("Invalid category: {e}")))?;

        let description = row[3]
            .get_str()
            .ok_or_else(|| GrooveError::Database("Invalid description type".into()))?
            .to_string();

        let pattern: Option<serde_json::Value> =
            match &row[4] {
                DataValue::Null => None,
                DataValue::Str(s) => Some(serde_json::from_str(s).map_err(|e| {
                    GrooveError::Serialization(format!("Invalid pattern JSON: {e}"))
                })?),
                _ => None,
            };

        let insight = row[5]
            .get_str()
            .ok_or_else(|| GrooveError::Database("Invalid insight type".into()))?
            .to_string();

        let confidence = row[6]
            .get_float()
            .ok_or_else(|| GrooveError::Database("Invalid confidence type".into()))?;

        let created_at_ts = row[7]
            .get_int()
            .ok_or_else(|| GrooveError::Database("Invalid created_at type".into()))?;
        let created_at = DateTime::from_timestamp(created_at_ts, 0)
            .ok_or_else(|| GrooveError::Database("Invalid created_at timestamp".into()))?;

        let updated_at_ts = row[8]
            .get_int()
            .ok_or_else(|| GrooveError::Database("Invalid updated_at type".into()))?;
        let updated_at = DateTime::from_timestamp(updated_at_ts, 0)
            .ok_or_else(|| GrooveError::Database("Invalid updated_at timestamp".into()))?;

        // source_type is at index 9, source_json is at index 10
        let source_json = row[10]
            .get_str()
            .ok_or_else(|| GrooveError::Database("Invalid source_json type".into()))?;
        let source: LearningSource = serde_json::from_str(source_json)
            .map_err(|e| GrooveError::Serialization(format!("Invalid source JSON: {e}")))?;

        Ok(Some(Learning {
            id,
            scope,
            category,
            content: LearningContent {
                description,
                pattern,
                insight,
            },
            confidence,
            created_at,
            updated_at,
            source,
        }))
    }

    // ===== ParamStore Implementation =====

    /// Store or update a system parameter
    pub async fn store_param(&self, param: &SystemParam) -> Result<()> {
        let name = param.name.replace('\'', "''");
        let updated_at = param.updated_at.timestamp();

        let query = format!(
            r#"?[param_name, value, uncertainty, observations, prior_alpha, prior_beta, updated_at] <- [[
                '{}', {}, {}, {}, {}, {}, {}
            ]]
            :put adaptive_params {{
                param_name => value, uncertainty, observations, prior_alpha, prior_beta, updated_at
            }}"#,
            name,
            param.param.value,
            param.param.uncertainty,
            param.param.observations,
            param.param.prior_alpha,
            param.param.prior_beta,
            updated_at,
        );

        self.run_mutation(&query, Default::default()).await?;
        Ok(())
    }

    /// Get a system parameter by name
    pub async fn get_param(&self, name: &str) -> Result<Option<SystemParam>> {
        let name_escaped = name.replace('\'', "''");
        let query = format!(
            r#"?[param_name, value, uncertainty, observations, prior_alpha, prior_beta, updated_at] :=
                *adaptive_params{{param_name, value, uncertainty, observations, prior_alpha, prior_beta, updated_at}},
                param_name = '{}'"#,
            name_escaped
        );

        let rows = self.run_query(&query, Default::default()).await?;

        if rows.rows.is_empty() {
            return Ok(None);
        }

        self.row_to_system_param(&rows.rows[0])
    }

    /// Get all system parameters
    pub async fn all_params(&self) -> Result<Vec<SystemParam>> {
        let query = r#"?[param_name, value, uncertainty, observations, prior_alpha, prior_beta, updated_at] :=
            *adaptive_params{param_name, value, uncertainty, observations, prior_alpha, prior_beta, updated_at}"#;

        let rows = self.run_query(query, Default::default()).await?;

        let mut params = Vec::new();
        for row in &rows.rows {
            if let Some(param) = self.row_to_system_param(row)? {
                params.push(param);
            }
        }

        Ok(params)
    }

    // ===== Embedding/Semantic Search Implementation =====

    /// Expected embedding dimension (matches GteSmall / all-MiniLM-L6-v2)
    const EMBEDDING_DIM: usize = 384;

    /// Store embedding vector for a learning
    ///
    /// # Errors
    /// Returns an error if the embedding dimension is not exactly 384
    pub async fn store_embedding(&self, learning_id: LearningId, embedding: &[f32]) -> Result<()> {
        // Validate embedding dimension
        if embedding.len() != Self::EMBEDDING_DIM {
            return Err(GrooveError::Database(format!(
                "Invalid embedding dimension: expected {}, got {}",
                Self::EMBEDDING_DIM,
                embedding.len()
            )));
        }

        // Use CozoDB parameters with proper Vector type
        let mut params = BTreeMap::new();
        params.insert(
            "learning_id".to_string(),
            DataValue::Str(learning_id.to_string().into()),
        );
        // Convert to ndarray Array1 for the Vector::F32 type
        let array: Array1<f32> = Array1::from_vec(embedding.to_vec());
        params.insert("embedding".to_string(), DataValue::Vec(Vector::F32(array)));

        let query = r#"?[learning_id, embedding] <- [[$learning_id, $embedding]]
            :put learning_embeddings { learning_id => embedding }"#;

        self.run_mutation(query, params).await?;
        Ok(())
    }

    /// Semantic search using HNSW index
    ///
    /// Returns learnings with their cosine distance (lower = more similar).
    /// Distance ranges from 0 (identical) to 2 (opposite).
    ///
    /// # Errors
    /// Returns an error if the embedding dimension is not exactly 384
    pub async fn semantic_search(
        &self,
        embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(Learning, f64)>> {
        // Validate embedding dimension
        if embedding.len() != Self::EMBEDDING_DIM {
            return Err(GrooveError::Database(format!(
                "Invalid embedding dimension: expected {}, got {}",
                Self::EMBEDDING_DIM,
                embedding.len()
            )));
        }

        // Use CozoDB parameters with proper Vector type
        let mut params = BTreeMap::new();
        let array: Array1<f32> = Array1::from_vec(embedding.to_vec());
        params.insert("query_vec".to_string(), DataValue::Vec(Vector::F32(array)));
        params.insert("k".to_string(), DataValue::from(limit as i64));

        // HNSW search query
        // ef: 50 provides a good balance of speed and accuracy
        // The distance is bound via the bind_distance parameter
        let query = r#"?[learning_id, distance] := ~learning_embeddings:semantic_idx {
                learning_id |
                query: $query_vec,
                k: $k,
                ef: 50,
                bind_distance: distance
            }"#;

        let rows = self.run_query(query, params).await?;

        // Collect IDs and distances first (avoids N+1 queries)
        let mut id_distances: Vec<(LearningId, f64)> = Vec::new();
        for row in &rows.rows {
            if let Some(learning_id_str) = row[0].get_str()
                && let Ok(learning_id) = uuid::Uuid::parse_str(learning_id_str)
            {
                let distance = row[1].get_float().unwrap_or(f64::MAX);
                id_distances.push((learning_id, distance));
            }
        }

        // Batch fetch all learnings
        let ids: Vec<LearningId> = id_distances.iter().map(|(id, _)| *id).collect();
        let learnings = self.get_many(&ids).await?;

        // Build a map for O(1) lookup
        let learning_map: std::collections::HashMap<LearningId, Learning> =
            learnings.into_iter().map(|l| (l.id, l)).collect();

        // Reassemble results with distances, preserving HNSW order
        let mut results: Vec<(Learning, f64)> = id_distances
            .into_iter()
            .filter_map(|(id, dist)| learning_map.get(&id).cloned().map(|l| (l, dist)))
            .collect();

        // Results should already be sorted by distance from HNSW, but ensure ordering
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results)
    }

    /// Helper to convert a database row to a SystemParam struct
    fn row_to_system_param(&self, row: &[DataValue]) -> Result<Option<SystemParam>> {
        if row.len() < 7 {
            return Ok(None);
        }

        // [param_name, value, uncertainty, observations, prior_alpha, prior_beta, updated_at]
        let name = row[0]
            .get_str()
            .ok_or_else(|| GrooveError::Database("Invalid param_name type".into()))?
            .to_string();

        let value = row[1]
            .get_float()
            .ok_or_else(|| GrooveError::Database("Invalid value type".into()))?;

        let uncertainty = row[2]
            .get_float()
            .ok_or_else(|| GrooveError::Database("Invalid uncertainty type".into()))?;

        let observations = row[3]
            .get_int()
            .ok_or_else(|| GrooveError::Database("Invalid observations type".into()))?
            as u64;

        let prior_alpha = row[4]
            .get_float()
            .ok_or_else(|| GrooveError::Database("Invalid prior_alpha type".into()))?;

        let prior_beta = row[5]
            .get_float()
            .ok_or_else(|| GrooveError::Database("Invalid prior_beta type".into()))?;

        let updated_at_ts = row[6]
            .get_int()
            .ok_or_else(|| GrooveError::Database("Invalid updated_at type".into()))?;
        let updated_at = DateTime::from_timestamp(updated_at_ts, 0)
            .ok_or_else(|| GrooveError::Database("Invalid updated_at timestamp".into()))?;

        Ok(Some(SystemParam {
            name,
            param: AdaptiveParam {
                value,
                uncertainty,
                observations,
                prior_alpha,
                prior_beta,
            },
            updated_at,
        }))
    }

    // ===== Export/Import Implementation =====

    /// Export all data from the store
    ///
    /// Exports all learnings (with usage stats), system parameters, and relations
    /// to a portable `GrooveExport` format for backup or migration.
    pub async fn export(&self) -> Result<crate::GrooveExport> {
        use crate::{GrooveExport, LearningExport};

        let mut export = GrooveExport::new();

        // Export all learnings with their usage stats
        let learnings_query = r#"
            ?[id, scope, category, description, pattern_json, insight, confidence,
              created_at, updated_at, source_type, source_json] :=
            *learning{id, scope, category, description, pattern_json, insight,
                     confidence, created_at, updated_at, source_type, source_json}
        "#;
        let rows = self.run_query(learnings_query, Default::default()).await?;

        for row in &rows.rows {
            if let Some(learning) = self.row_to_learning(row)? {
                // Get usage stats for this learning
                let usage_stats = self.get_usage(learning.id).await?.unwrap_or_default();

                export.learnings.push(LearningExport {
                    id: learning.id,
                    scope: learning.scope,
                    category: learning.category,
                    content: learning.content,
                    confidence: learning.confidence,
                    created_at: learning.created_at,
                    updated_at: learning.updated_at,
                    source: learning.source,
                    usage_stats,
                });
            }
        }

        // Export all params
        export.params = self.all_params().await?;

        // Export all relations
        let relations_query = r#"
            ?[from_id, relation_type, to_id, weight, created_at] :=
            *learning_relations{from_id, relation_type, to_id, weight, created_at}
        "#;
        let rows = self.run_query(relations_query, Default::default()).await?;

        for row in &rows.rows {
            if let Some(relation) = self.row_to_relation(row)? {
                export.relations.push(relation);
            }
        }

        Ok(export)
    }

    /// Import data into the store
    ///
    /// Imports learnings, parameters, and relations from a `GrooveExport`.
    /// Skips learnings with IDs that already exist (no overwrites).
    /// Returns statistics about the import operation.
    pub async fn import(&self, export: &crate::GrooveExport) -> Result<crate::ImportStats> {
        let mut stats = crate::ImportStats::default();

        // Import learnings (skip if ID already exists)
        for learning_export in &export.learnings {
            if self.get(learning_export.id).await?.is_some() {
                stats.learnings_skipped += 1;
                continue;
            }

            // Reconstruct Learning and store with original ID
            let learning = Learning {
                id: learning_export.id,
                scope: learning_export.scope.clone(),
                category: learning_export.category.clone(),
                content: learning_export.content.clone(),
                confidence: learning_export.confidence,
                created_at: learning_export.created_at,
                updated_at: learning_export.updated_at,
                source: learning_export.source.clone(),
            };

            // Use internal store that preserves ID
            self.store_with_id(&learning).await?;

            // Store usage stats
            self.update_usage(learning_export.id, &learning_export.usage_stats)
                .await?;

            stats.learnings_imported += 1;
            stats.embeddings_queued += 1; // Embeddings need regeneration
        }

        // Import params
        for param in &export.params {
            self.store_param(param).await?;
            stats.params_imported += 1;
        }

        // Import relations
        for relation in &export.relations {
            self.store_relation(relation).await?;
            stats.relations_imported += 1;
        }

        Ok(stats)
    }

    /// Internal method to store learning preserving its ID
    ///
    /// Unlike `store()` which uses the learning's pre-assigned ID anyway,
    /// this is explicit about preserving imported IDs.
    async fn store_with_id(&self, learning: &Learning) -> Result<()> {
        let id_str = learning.id.to_string();
        let scope_str = learning.scope.to_db_string();
        let category_str = learning.category.as_str();
        let description = learning.content.description.replace('\'', "''");
        let pattern_json = learning
            .content
            .pattern
            .as_ref()
            .map(|p| format!("'{}'", p.to_string().replace('\'', "''")))
            .unwrap_or_else(|| "null".to_string());
        let insight = learning.content.insight.replace('\'', "''");
        let confidence = learning.confidence;
        let created_at = learning.created_at.timestamp();
        let updated_at = learning.updated_at.timestamp();
        let source_type = learning.source.source_type();
        let source_json = serde_json::to_string(&learning.source)
            .map_err(|e| GrooveError::Serialization(e.to_string()))?
            .replace('\'', "''");

        // Insert learning
        let query = format!(
            r#"?[id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json] <- [[
                '{}', '{}', '{}', '{}', {}, '{}', {}, {}, {}, '{}', '{}'
            ]]
            :put learning {{
                id => scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json
            }}"#,
            id_str,
            scope_str,
            category_str,
            description,
            pattern_json,
            insight,
            confidence,
            created_at,
            updated_at,
            source_type,
            source_json
        );

        self.run_mutation(&query, Default::default()).await?;
        Ok(())
    }

    /// Helper to convert a database row to a LearningRelation struct
    fn row_to_relation(&self, row: &[DataValue]) -> Result<Option<LearningRelation>> {
        if row.len() < 5 {
            return Ok(None);
        }

        // [from_id, relation_type, to_id, weight, created_at]
        let from_id_str = row[0]
            .get_str()
            .ok_or_else(|| GrooveError::Database("Invalid from_id type".into()))?;
        let from_id = LearningId::parse_str(from_id_str)
            .map_err(|e| GrooveError::Database(format!("Invalid from_id UUID: {e}")))?;

        let relation_type_str = row[1]
            .get_str()
            .ok_or_else(|| GrooveError::Database("Invalid relation_type type".into()))?;
        let relation_type = RelationType::from_str(relation_type_str)
            .map_err(|e| GrooveError::Database(format!("Unknown relation type: {e}")))?;

        let to_id_str = row[2]
            .get_str()
            .ok_or_else(|| GrooveError::Database("Invalid to_id type".into()))?;
        let to_id = LearningId::parse_str(to_id_str)
            .map_err(|e| GrooveError::Database(format!("Invalid to_id UUID: {e}")))?;

        let weight = row[3]
            .get_float()
            .ok_or_else(|| GrooveError::Database("Invalid weight type".into()))?;

        let created_at_ts = row[4]
            .get_int()
            .ok_or_else(|| GrooveError::Database("Invalid created_at type".into()))?;
        let created_at = DateTime::from_timestamp(created_at_ts, 0)
            .ok_or_else(|| GrooveError::Database("Invalid created_at timestamp".into()))?;

        Ok(Some(LearningRelation {
            from_id,
            relation_type,
            to_id,
            weight,
            created_at,
        }))
    }

    /// Update an existing learning
    pub async fn update(&self, learning: &Learning) -> Result<()> {
        let id_str = learning.id.to_string();
        let scope_str = learning.scope.to_db_string();
        let category_str = learning.category.as_str();
        let description = learning.content.description.replace('\'', "''");
        let pattern_json = learning
            .content
            .pattern
            .as_ref()
            .map(|p| format!("'{}'", p.to_string().replace('\'', "''")))
            .unwrap_or_else(|| "null".to_string());
        let insight = learning.content.insight.replace('\'', "''");
        let confidence = learning.confidence;
        let created_at = learning.created_at.timestamp();
        let updated_at = Utc::now().timestamp();
        let source_type = learning.source.source_type();
        let source_json = serde_json::to_string(&learning.source)
            .map_err(|e| GrooveError::Serialization(e.to_string()))?
            .replace('\'', "''");

        let query = format!(
            r#"?[id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json] <- [[
                '{}', '{}', '{}', '{}', {}, '{}', {}, {}, {}, '{}', '{}'
            ]]
            :put learning {{
                id => scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json
            }}"#,
            id_str,
            scope_str,
            category_str,
            description,
            pattern_json,
            insight,
            confidence,
            created_at,
            updated_at,
            source_type,
            source_json
        );

        self.run_mutation(&query, Default::default()).await?;
        Ok(())
    }

    /// Find similar learnings above a similarity threshold
    ///
    /// Uses HNSW index for efficient similarity search. The threshold is
    /// specified as a cosine similarity value (0.0-1.0), where higher is more similar.
    /// Internally converts to distance (2 - 2*similarity) for HNSW search.
    pub async fn find_similar(
        &self,
        embedding: &[f32],
        threshold: f64,
        limit: usize,
    ) -> Result<Vec<(Learning, f64)>> {
        // Validate embedding dimension
        if embedding.len() != Self::EMBEDDING_DIM {
            return Err(GrooveError::Database(format!(
                "Invalid embedding dimension: expected {}, got {}",
                Self::EMBEDDING_DIM,
                embedding.len()
            )));
        }

        // Use semantic_search and filter by threshold
        // semantic_search returns cosine distance (0 = identical, 2 = opposite)
        // Convert threshold (similarity) to max_distance: distance = 2 - 2*similarity
        // For similarity >= threshold: distance <= 2 - 2*threshold
        let max_distance = 2.0 - 2.0 * threshold;

        let results = self.semantic_search(embedding, limit * 2).await?;

        // Filter by distance threshold and convert to similarity
        let filtered: Vec<(Learning, f64)> = results
            .into_iter()
            .filter(|(_, distance)| *distance <= max_distance)
            .map(|(learning, distance)| {
                // Convert distance back to similarity
                let similarity = 1.0 - distance / 2.0;
                (learning, similarity)
            })
            .take(limit)
            .collect();

        Ok(filtered)
    }

    /// Find learnings suitable for injection based on session context
    ///
    /// If a context embedding is provided, returns learnings ordered by relevance.
    /// Otherwise returns learnings by confidence and recency.
    pub async fn find_for_injection(
        &self,
        scope: &Scope,
        context_embedding: Option<&[f32]>,
        limit: usize,
    ) -> Result<Vec<Learning>> {
        if let Some(embedding) = context_embedding {
            // Use semantic search filtered by scope
            let results = self.semantic_search(embedding, limit * 3).await?;

            // Filter by scope and take limit
            let scope_str = scope.to_db_string();
            let filtered: Vec<Learning> = results
                .into_iter()
                .filter(|(l, _)| l.scope.to_db_string() == scope_str)
                .map(|(l, _)| l)
                .take(limit)
                .collect();

            Ok(filtered)
        } else {
            // Fallback to scope-based retrieval ordered by confidence
            let scope_str = scope.to_db_string();
            let query = format!(
                r#"?[id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json] :=
                    *learning{{id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json}},
                    scope == '{}'
                :order -confidence, -updated_at
                :limit {}"#,
                scope_str, limit
            );

            let rows = self.run_query(&query, Default::default()).await?;

            let mut learnings = Vec::new();
            for row in &rows.rows {
                if let Some(learning) = self.row_to_learning(row)? {
                    learnings.push(learning);
                }
            }

            Ok(learnings)
        }
    }

    /// Count learnings by scope
    pub async fn count_by_scope(&self, scope: &Scope) -> Result<u64> {
        let scope_str = scope.to_db_string();
        let query = format!(
            "?[count(id)] := *learning{{id, scope}}, scope == '{}'",
            scope_str
        );

        let rows = self.run_query(&query, Default::default()).await?;

        if rows.rows.is_empty() {
            return Ok(0);
        }

        let count = rows.rows[0][0]
            .get_int()
            .ok_or_else(|| GrooveError::Database("Invalid count type".into()))?;

        Ok(count as u64)
    }

    /// Count learnings by category
    pub async fn count_by_category(&self, category: &LearningCategory) -> Result<u64> {
        let category_str = category.as_str();
        let query = format!(
            "?[count(id)] := *learning{{id, category}}, category == '{}'",
            category_str
        );

        let rows = self.run_query(&query, Default::default()).await?;

        if rows.rows.is_empty() {
            return Ok(0);
        }

        let count = rows.rows[0][0]
            .get_int()
            .ok_or_else(|| GrooveError::Database("Invalid count type".into()))?;

        Ok(count as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::super::schema::CURRENT_SCHEMA_VERSION;
    use super::*;
    use crate::{
        Learning, LearningCategory, LearningContent, LearningRelation, LearningSource, Outcome,
        RelationType, Scope, SystemParam,
    };
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_cozo_store() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();
        assert!(store.is_initialized());
    }

    #[tokio::test]
    async fn test_schema_version_recorded() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();
        let version = store.get_schema_version().await.unwrap();
        assert_eq!(version, CURRENT_SCHEMA_VERSION);
    }

    #[tokio::test]
    async fn test_reopen_existing_db() {
        let tmp = TempDir::new().unwrap();

        // Create and close
        {
            let store = CozoStore::open(tmp.path()).await.unwrap();
            assert_eq!(
                store.get_schema_version().await.unwrap(),
                CURRENT_SCHEMA_VERSION
            );
        }

        // Reopen
        {
            let store = CozoStore::open(tmp.path()).await.unwrap();
            assert_eq!(
                store.get_schema_version().await.unwrap(),
                CURRENT_SCHEMA_VERSION
            );
        }
    }

    #[tokio::test]
    async fn test_store_and_get_learning() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let learning = Learning::new(
            Scope::User("test".into()),
            LearningCategory::Preference,
            LearningContent {
                description: "Test learning".into(),
                pattern: None,
                insight: "Test insight".into(),
            },
            LearningSource::UserCreated,
        );

        let id = store.store(&learning).await.unwrap();
        assert_eq!(id, learning.id);

        let retrieved = store.get(id).await.unwrap().unwrap();
        assert_eq!(retrieved.id, learning.id);
        assert_eq!(retrieved.content.description, learning.content.description);
    }

    #[tokio::test]
    async fn test_find_by_scope() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let user_scope = Scope::User("alice".into());
        let other_scope = Scope::User("bob".into());

        for i in 0..3 {
            let learning = Learning::new(
                user_scope.clone(),
                LearningCategory::Preference,
                LearningContent {
                    description: format!("Alice learning {i}"),
                    pattern: None,
                    insight: "insight".into(),
                },
                LearningSource::UserCreated,
            );
            store.store(&learning).await.unwrap();
        }

        let bob_learning = Learning::new(
            other_scope.clone(),
            LearningCategory::Preference,
            LearningContent {
                description: "Bob learning".into(),
                pattern: None,
                insight: "insight".into(),
            },
            LearningSource::UserCreated,
        );
        store.store(&bob_learning).await.unwrap();

        let alice_learnings = store.find_by_scope(&user_scope).await.unwrap();
        assert_eq!(alice_learnings.len(), 3);

        let bob_learnings = store.find_by_scope(&other_scope).await.unwrap();
        assert_eq!(bob_learnings.len(), 1);
    }

    #[tokio::test]
    async fn test_delete_learning() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let learning = Learning::new(
            Scope::Global,
            LearningCategory::Solution,
            LearningContent {
                description: "To be deleted".into(),
                pattern: None,
                insight: "insight".into(),
            },
            LearningSource::UserCreated,
        );

        let id = store.store(&learning).await.unwrap();
        assert!(store.get(id).await.unwrap().is_some());

        let deleted = store.delete(id).await.unwrap();
        assert!(deleted);

        assert!(store.get(id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_count_learnings() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        assert_eq!(store.count().await.unwrap(), 0);

        for i in 0..5 {
            let learning = Learning::new(
                Scope::Global,
                LearningCategory::CodePattern,
                LearningContent {
                    description: format!("Learning {i}"),
                    pattern: None,
                    insight: "insight".into(),
                },
                LearningSource::UserCreated,
            );
            store.store(&learning).await.unwrap();
        }

        assert_eq!(store.count().await.unwrap(), 5);
    }

    #[tokio::test]
    async fn test_find_by_category() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        // Create learnings with different categories
        for i in 0..2 {
            let learning = Learning::new(
                Scope::Global,
                LearningCategory::CodePattern,
                LearningContent {
                    description: format!("Code pattern {i}"),
                    pattern: None,
                    insight: "insight".into(),
                },
                LearningSource::UserCreated,
            );
            store.store(&learning).await.unwrap();
        }

        let preference = Learning::new(
            Scope::Global,
            LearningCategory::Preference,
            LearningContent {
                description: "A preference".into(),
                pattern: None,
                insight: "insight".into(),
            },
            LearningSource::UserCreated,
        );
        store.store(&preference).await.unwrap();

        let code_patterns = store
            .find_by_category(&LearningCategory::CodePattern)
            .await
            .unwrap();
        assert_eq!(code_patterns.len(), 2);

        let preferences = store
            .find_by_category(&LearningCategory::Preference)
            .await
            .unwrap();
        assert_eq!(preferences.len(), 1);
    }

    #[tokio::test]
    async fn test_get_default_usage_stats() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let learning = Learning::new(
            Scope::Global,
            LearningCategory::Preference,
            LearningContent {
                description: "Test".into(),
                pattern: None,
                insight: "insight".into(),
            },
            LearningSource::UserCreated,
        );
        store.store(&learning).await.unwrap();

        let stats = store.get_usage(learning.id).await.unwrap().unwrap();
        assert_eq!(stats.times_injected, 0);
        assert_eq!(stats.confidence_alpha, 1.0);
        assert_eq!(stats.confidence_beta, 1.0);
    }

    #[tokio::test]
    async fn test_update_usage_stats() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let learning = Learning::new(
            Scope::Global,
            LearningCategory::Preference,
            LearningContent {
                description: "Test".into(),
                pattern: None,
                insight: "insight".into(),
            },
            LearningSource::UserCreated,
        );
        store.store(&learning).await.unwrap();

        let mut stats = store.get_usage(learning.id).await.unwrap().unwrap();
        stats.record_outcome(Outcome::Helpful);
        stats.record_outcome(Outcome::Helpful);

        store.update_usage(learning.id, &stats).await.unwrap();

        let updated = store.get_usage(learning.id).await.unwrap().unwrap();
        assert_eq!(updated.times_injected, 2);
        assert_eq!(updated.times_helpful, 2);
        assert_eq!(updated.confidence_alpha, 3.0); // 1.0 + 2*1.0
    }

    #[tokio::test]
    async fn test_usage_stats_not_found() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let fake_id = uuid::Uuid::now_v7();
        let stats = store.get_usage(fake_id).await.unwrap();
        assert!(stats.is_none());
    }

    #[tokio::test]
    async fn test_store_relation() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let learning1 = Learning::new(
            Scope::Global,
            LearningCategory::CodePattern,
            LearningContent {
                description: "Original pattern".into(),
                pattern: None,
                insight: "insight".into(),
            },
            LearningSource::UserCreated,
        );
        let learning2 = Learning::new(
            Scope::Global,
            LearningCategory::CodePattern,
            LearningContent {
                description: "Improved pattern".into(),
                pattern: None,
                insight: "insight".into(),
            },
            LearningSource::UserCreated,
        );

        store.store(&learning1).await.unwrap();
        store.store(&learning2).await.unwrap();

        let relation = LearningRelation::new(learning2.id, RelationType::Supersedes, learning1.id);
        store.store_relation(&relation).await.unwrap();

        // Verify by finding related
        let related = store.find_related(learning2.id, None).await.unwrap();
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].id, learning1.id);
    }

    #[tokio::test]
    async fn test_find_related_with_filter() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let base = Learning::new(
            Scope::Global,
            LearningCategory::CodePattern,
            LearningContent {
                description: "Base".into(),
                pattern: None,
                insight: "insight".into(),
            },
            LearningSource::UserCreated,
        );
        let supersedes = Learning::new(
            Scope::Global,
            LearningCategory::CodePattern,
            LearningContent {
                description: "Supersedes".into(),
                pattern: None,
                insight: "insight".into(),
            },
            LearningSource::UserCreated,
        );
        let related = Learning::new(
            Scope::Global,
            LearningCategory::CodePattern,
            LearningContent {
                description: "Related".into(),
                pattern: None,
                insight: "insight".into(),
            },
            LearningSource::UserCreated,
        );

        store.store(&base).await.unwrap();
        store.store(&supersedes).await.unwrap();
        store.store(&related).await.unwrap();

        store
            .store_relation(&LearningRelation::new(
                base.id,
                RelationType::Supersedes,
                supersedes.id,
            ))
            .await
            .unwrap();
        store
            .store_relation(&LearningRelation::new(
                base.id,
                RelationType::RelatedTo,
                related.id,
            ))
            .await
            .unwrap();

        // All relations
        let all = store.find_related(base.id, None).await.unwrap();
        assert_eq!(all.len(), 2);

        // Filtered by type
        let only_supersedes = store
            .find_related(base.id, Some(&RelationType::Supersedes))
            .await
            .unwrap();
        assert_eq!(only_supersedes.len(), 1);
        assert_eq!(only_supersedes[0].id, supersedes.id);
    }

    #[tokio::test]
    async fn test_find_related_empty() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let learning = Learning::new(
            Scope::Global,
            LearningCategory::CodePattern,
            LearningContent {
                description: "Isolated".into(),
                pattern: None,
                insight: "insight".into(),
            },
            LearningSource::UserCreated,
        );
        store.store(&learning).await.unwrap();

        let related = store.find_related(learning.id, None).await.unwrap();
        assert!(related.is_empty());
    }

    // ===== ParamStore Tests =====

    #[tokio::test]
    async fn test_store_and_get_param() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let param = SystemParam::with_prior("injection_budget", 8.0, 2.0);

        store.store_param(&param).await.unwrap();

        let retrieved = store.get_param("injection_budget").await.unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, "injection_budget");
        assert!((retrieved.param.value - 0.8).abs() < 0.001);
        assert!((retrieved.param.prior_alpha - 8.0).abs() < 0.001);
        assert!((retrieved.param.prior_beta - 2.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_get_param_not_found() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let retrieved = store.get_param("nonexistent").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_all_params() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let param1 = SystemParam::new("injection_budget");
        let param2 = SystemParam::with_prior("context_relevance", 5.0, 5.0);
        let param3 = SystemParam::new("recency_weight");

        store.store_param(&param1).await.unwrap();
        store.store_param(&param2).await.unwrap();
        store.store_param(&param3).await.unwrap();

        let all = store.all_params().await.unwrap();
        assert_eq!(all.len(), 3);

        // Check all names are present
        let names: Vec<&str> = all.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"injection_budget"));
        assert!(names.contains(&"context_relevance"));
        assert!(names.contains(&"recency_weight"));
    }

    #[tokio::test]
    async fn test_store_param_upsert() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        // Store initial param
        let mut param = SystemParam::new("injection_budget");
        store.store_param(&param).await.unwrap();

        // Update it via Bayesian update
        param.param.update(1.0, 1.0);

        // Store again (upsert)
        store.store_param(&param).await.unwrap();

        // Retrieve and verify update was applied
        let retrieved = store.get_param("injection_budget").await.unwrap().unwrap();
        assert!(retrieved.param.value > 0.5); // Value should have increased after positive outcome
        assert_eq!(retrieved.param.observations, 1);
    }

    // ===== Embedding/Semantic Search Tests =====

    /// Create a deterministic 384-dim embedding for testing
    fn make_test_embedding(seed: u8) -> Vec<f32> {
        (0..384)
            .map(|i| ((i as u8).wrapping_add(seed) as f32) / 255.0)
            .collect()
    }

    #[tokio::test]
    async fn test_store_embedding() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        // Create a learning first
        let learning = Learning::new(
            Scope::Global,
            LearningCategory::CodePattern,
            LearningContent {
                description: "Test pattern".into(),
                pattern: None,
                insight: "Test insight".into(),
            },
            LearningSource::UserCreated,
        );
        store.store(&learning).await.unwrap();

        // Store embedding for the learning
        let embedding = make_test_embedding(42);
        store
            .store_embedding(learning.id, &embedding)
            .await
            .unwrap();

        // Verify by doing a semantic search that should find it
        let results = store.semantic_search(&embedding, 1).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.id, learning.id);
    }

    #[tokio::test]
    async fn test_store_embedding_invalid_dimension() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let learning = Learning::new(
            Scope::Global,
            LearningCategory::CodePattern,
            LearningContent {
                description: "Test".into(),
                pattern: None,
                insight: "insight".into(),
            },
            LearningSource::UserCreated,
        );
        store.store(&learning).await.unwrap();

        // Try to store embedding with wrong dimension (128 instead of 384)
        let bad_embedding: Vec<f32> = (0..128).map(|i| i as f32 / 128.0).collect();
        let result = store.store_embedding(learning.id, &bad_embedding).await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("384"),
            "Error should mention expected dimension 384"
        );
    }

    #[tokio::test]
    async fn test_semantic_search_basic() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        // Create multiple learnings with embeddings
        let mut learnings = Vec::new();
        for i in 0..5u8 {
            let learning = Learning::new(
                Scope::Global,
                LearningCategory::CodePattern,
                LearningContent {
                    description: format!("Pattern {}", i),
                    pattern: None,
                    insight: format!("Insight {}", i),
                },
                LearningSource::UserCreated,
            );
            store.store(&learning).await.unwrap();

            let embedding = make_test_embedding(i * 10);
            store
                .store_embedding(learning.id, &embedding)
                .await
                .unwrap();
            learnings.push(learning);
        }

        // Search with an embedding similar to seed 0
        let query = make_test_embedding(0);
        let results = store.semantic_search(&query, 3).await.unwrap();

        // Should find results
        assert!(!results.is_empty());
        assert!(results.len() <= 3);

        // First result should be the most similar (seed 0)
        assert_eq!(results[0].0.id, learnings[0].id);

        // Results should be ordered by distance (ascending)
        for window in results.windows(2) {
            assert!(
                window[0].1 <= window[1].1,
                "Results should be ordered by distance"
            );
        }
    }

    #[tokio::test]
    async fn test_semantic_search_empty() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        // Search when no embeddings are stored
        let query = make_test_embedding(42);
        let results = store.semantic_search(&query, 10).await.unwrap();

        assert!(results.is_empty(), "Should return empty when no embeddings");
    }

    #[tokio::test]
    async fn test_semantic_search_respects_limit() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        // Create 10 learnings with embeddings
        for i in 0..10u8 {
            let learning = Learning::new(
                Scope::Global,
                LearningCategory::CodePattern,
                LearningContent {
                    description: format!("Pattern {}", i),
                    pattern: None,
                    insight: format!("Insight {}", i),
                },
                LearningSource::UserCreated,
            );
            store.store(&learning).await.unwrap();

            let embedding = make_test_embedding(i);
            store
                .store_embedding(learning.id, &embedding)
                .await
                .unwrap();
        }

        // Search with limit of 3
        let query = make_test_embedding(5);
        let results = store.semantic_search(&query, 3).await.unwrap();

        assert_eq!(results.len(), 3, "Should return at most 'limit' results");
    }

    #[tokio::test]
    async fn test_semantic_search_invalid_dimension() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        // Try to search with wrong dimension
        let bad_query: Vec<f32> = (0..256).map(|i| i as f32 / 256.0).collect();
        let result = store.semantic_search(&bad_query, 10).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("384"),
            "Error should mention expected dimension 384"
        );
    }

    // ===== Export/Import Tests =====

    #[tokio::test]
    async fn test_export_empty_store() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let export = store.export().await.unwrap();

        assert_eq!(export.version, crate::EXPORT_VERSION);
        assert!(export.learnings.is_empty());
        assert!(export.params.is_empty());
        assert!(export.relations.is_empty());
    }

    #[tokio::test]
    async fn test_export_with_data() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        // Store a learning with usage stats
        let learning = Learning::new(
            Scope::Global,
            LearningCategory::CodePattern,
            LearningContent {
                description: "Test".into(),
                pattern: None,
                insight: "Test".into(),
            },
            LearningSource::UserCreated,
        );
        let id = store.store(&learning).await.unwrap();

        // Update usage stats
        let mut stats = store.get_usage(id).await.unwrap().unwrap();
        stats.record_outcome(Outcome::Helpful);
        store.update_usage(id, &stats).await.unwrap();

        // Store a param
        let param = SystemParam::new("test_param");
        store.store_param(&param).await.unwrap();

        let export = store.export().await.unwrap();

        assert_eq!(export.learnings.len(), 1);
        assert_eq!(export.learnings[0].id, id);
        assert_eq!(export.learnings[0].usage_stats.times_helpful, 1);
        assert_eq!(export.params.len(), 1);
    }

    #[tokio::test]
    async fn test_export_includes_relations() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        // Store two learnings
        let learning1 = Learning::new(
            Scope::Global,
            LearningCategory::CodePattern,
            LearningContent {
                description: "Original".into(),
                pattern: None,
                insight: "insight".into(),
            },
            LearningSource::UserCreated,
        );
        let learning2 = Learning::new(
            Scope::Global,
            LearningCategory::CodePattern,
            LearningContent {
                description: "Updated".into(),
                pattern: None,
                insight: "insight".into(),
            },
            LearningSource::UserCreated,
        );
        store.store(&learning1).await.unwrap();
        store.store(&learning2).await.unwrap();

        // Create a relation
        let relation = LearningRelation::new(learning2.id, RelationType::Supersedes, learning1.id);
        store.store_relation(&relation).await.unwrap();

        let export = store.export().await.unwrap();

        assert_eq!(export.learnings.len(), 2);
        assert_eq!(export.relations.len(), 1);
        assert_eq!(export.relations[0].from_id, learning2.id);
        assert_eq!(export.relations[0].to_id, learning1.id);
    }

    #[tokio::test]
    async fn test_import_into_empty_store() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let mut export = crate::GrooveExport::new();
        export.learnings.push(crate::LearningExport {
            id: uuid::Uuid::now_v7(),
            scope: Scope::Global,
            category: LearningCategory::CodePattern,
            content: LearningContent {
                description: "Test".into(),
                pattern: None,
                insight: "Test".into(),
            },
            confidence: 0.8,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            source: LearningSource::UserCreated,
            usage_stats: UsageStats::default(),
        });

        let stats = store.import(&export).await.unwrap();

        assert_eq!(stats.learnings_imported, 1);
        assert_eq!(stats.learnings_skipped, 0);

        // Verify it was imported
        let count = store.count().await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_import_skips_duplicates() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        // Store a learning
        let learning = Learning::new(
            Scope::Global,
            LearningCategory::CodePattern,
            LearningContent {
                description: "Test".into(),
                pattern: None,
                insight: "Test".into(),
            },
            LearningSource::UserCreated,
        );
        let id = store.store(&learning).await.unwrap();

        // Try to import with same ID
        let mut export = crate::GrooveExport::new();
        export.learnings.push(crate::LearningExport {
            id, // Same ID
            scope: Scope::Global,
            category: LearningCategory::Solution,
            content: LearningContent {
                description: "Different".into(),
                pattern: None,
                insight: "Different".into(),
            },
            confidence: 0.9,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            source: LearningSource::UserCreated,
            usage_stats: UsageStats::default(),
        });

        let stats = store.import(&export).await.unwrap();

        assert_eq!(stats.learnings_imported, 0);
        assert_eq!(stats.learnings_skipped, 1);

        // Verify original content is unchanged
        let retrieved = store.get(id).await.unwrap().unwrap();
        assert_eq!(retrieved.content.description, "Test");
    }

    #[tokio::test]
    async fn test_import_params() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let mut export = crate::GrooveExport::new();
        export
            .params
            .push(SystemParam::with_prior("test_param", 8.0, 2.0));

        let stats = store.import(&export).await.unwrap();

        assert_eq!(stats.params_imported, 1);

        // Verify param was imported
        let param = store.get_param("test_param").await.unwrap();
        assert!(param.is_some());
        assert!((param.unwrap().param.prior_alpha - 8.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_import_relations() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let id1 = uuid::Uuid::now_v7();
        let id2 = uuid::Uuid::now_v7();

        let mut export = crate::GrooveExport::new();
        // Add learnings first
        export.learnings.push(crate::LearningExport {
            id: id1,
            scope: Scope::Global,
            category: LearningCategory::CodePattern,
            content: LearningContent {
                description: "First".into(),
                pattern: None,
                insight: "insight".into(),
            },
            confidence: 0.5,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            source: LearningSource::UserCreated,
            usage_stats: UsageStats::default(),
        });
        export.learnings.push(crate::LearningExport {
            id: id2,
            scope: Scope::Global,
            category: LearningCategory::CodePattern,
            content: LearningContent {
                description: "Second".into(),
                pattern: None,
                insight: "insight".into(),
            },
            confidence: 0.5,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            source: LearningSource::UserCreated,
            usage_stats: UsageStats::default(),
        });
        // Add relation
        export
            .relations
            .push(LearningRelation::new(id2, RelationType::Supersedes, id1));

        let stats = store.import(&export).await.unwrap();

        assert_eq!(stats.learnings_imported, 2);
        assert_eq!(stats.relations_imported, 1);

        // Verify relation was imported
        let related = store.find_related(id2, None).await.unwrap();
        assert_eq!(related.len(), 1);
        assert_eq!(related[0].id, id1);
    }

    #[tokio::test]
    async fn test_roundtrip_export_import() {
        let tmp1 = TempDir::new().unwrap();
        let tmp2 = TempDir::new().unwrap();

        // Store data in first store
        let store1 = CozoStore::open(tmp1.path()).await.unwrap();
        let learning = Learning::new(
            Scope::Global,
            LearningCategory::CodePattern,
            LearningContent {
                description: "Roundtrip".into(),
                pattern: None,
                insight: "Test".into(),
            },
            LearningSource::UserCreated,
        );
        let id = store1.store(&learning).await.unwrap();

        // Update usage stats
        let mut stats = store1.get_usage(id).await.unwrap().unwrap();
        stats.record_outcome(Outcome::Helpful);
        stats.record_outcome(Outcome::Helpful);
        store1.update_usage(id, &stats).await.unwrap();

        // Export
        let export = store1.export().await.unwrap();

        // Import into second store
        let store2 = CozoStore::open(tmp2.path()).await.unwrap();
        let import_stats = store2.import(&export).await.unwrap();

        assert_eq!(import_stats.learnings_imported, 1);

        // Verify data matches
        let imported = store2.get(id).await.unwrap().unwrap();
        assert_eq!(imported.content.description, "Roundtrip");

        // Verify usage stats preserved
        let imported_stats = store2.get_usage(id).await.unwrap().unwrap();
        assert_eq!(imported_stats.times_helpful, 2);
    }

    #[tokio::test]
    async fn test_import_preserves_original_id() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let original_id = uuid::Uuid::now_v7();
        let mut export = crate::GrooveExport::new();
        export.learnings.push(crate::LearningExport {
            id: original_id,
            scope: Scope::User("alice".into()),
            category: LearningCategory::Preference,
            content: LearningContent {
                description: "Preserved ID".into(),
                pattern: None,
                insight: "insight".into(),
            },
            confidence: 0.75,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            source: LearningSource::UserCreated,
            usage_stats: UsageStats::default(),
        });

        store.import(&export).await.unwrap();

        // Verify the ID was preserved
        let retrieved = store.get(original_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, original_id);
    }

    // =============================================================================
    // Tests for new extraction-related methods
    // =============================================================================

    #[tokio::test]
    async fn test_update_learning() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        // Store a learning
        let mut learning = Learning::new(
            Scope::User("test".into()),
            LearningCategory::Preference,
            LearningContent {
                description: "Original description".into(),
                pattern: None,
                insight: "Original insight".into(),
            },
            LearningSource::UserCreated,
        );
        store.store(&learning).await.unwrap();

        // Update it
        learning.content.description = "Updated description".into();
        learning.content.insight = "Updated insight".into();
        learning.confidence = 0.9;

        store.update(&learning).await.unwrap();

        // Verify the update
        let retrieved = store.get(learning.id).await.unwrap().unwrap();
        assert_eq!(retrieved.content.description, "Updated description");
        assert_eq!(retrieved.content.insight, "Updated insight");
        assert!((retrieved.confidence - 0.9).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_find_similar_with_threshold() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        // Create learnings with embeddings
        let learning1 = Learning::new(
            Scope::Global,
            LearningCategory::CodePattern,
            LearningContent {
                description: "Pattern A".into(),
                pattern: None,
                insight: "Insight A".into(),
            },
            LearningSource::UserCreated,
        );
        store.store(&learning1).await.unwrap();
        let embedding1 = make_test_embedding(0);
        store
            .store_embedding(learning1.id, &embedding1)
            .await
            .unwrap();

        let learning2 = Learning::new(
            Scope::Global,
            LearningCategory::CodePattern,
            LearningContent {
                description: "Pattern B".into(),
                pattern: None,
                insight: "Insight B".into(),
            },
            LearningSource::UserCreated,
        );
        store.store(&learning2).await.unwrap();
        let embedding2 = make_test_embedding(100); // Very different
        store
            .store_embedding(learning2.id, &embedding2)
            .await
            .unwrap();

        // Search with high similarity threshold - should find only closest
        let query = make_test_embedding(0);
        let results = store.find_similar(&query, 0.9, 10).await.unwrap();

        // Should find at least the very similar one
        assert!(!results.is_empty());
        // First result should be learning1 (same embedding)
        assert_eq!(results[0].0.id, learning1.id);
        // Similarity should be >= threshold
        assert!(results[0].1 >= 0.9);
    }

    #[tokio::test]
    async fn test_find_for_injection_with_embedding() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let scope = Scope::Project("/test".into());

        // Create learnings with embeddings
        let learning1 = Learning::new(
            scope.clone(),
            LearningCategory::Preference,
            LearningContent {
                description: "Project preference".into(),
                pattern: None,
                insight: "insight".into(),
            },
            LearningSource::UserCreated,
        );
        store.store(&learning1).await.unwrap();
        let embedding1 = make_test_embedding(0);
        store
            .store_embedding(learning1.id, &embedding1)
            .await
            .unwrap();

        // Create learning in different scope
        let learning2 = Learning::new(
            Scope::Global,
            LearningCategory::Preference,
            LearningContent {
                description: "Global preference".into(),
                pattern: None,
                insight: "insight".into(),
            },
            LearningSource::UserCreated,
        );
        store.store(&learning2).await.unwrap();
        store
            .store_embedding(learning2.id, &embedding1)
            .await
            .unwrap();

        // Find for injection with context
        let results = store
            .find_for_injection(&scope, Some(&embedding1), 10)
            .await
            .unwrap();

        // Should only return project-scoped learning
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, learning1.id);
    }

    #[tokio::test]
    async fn test_find_for_injection_without_embedding() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let scope = Scope::User("alice".into());

        // Create learnings with different confidence
        for (i, conf) in [0.9, 0.5, 0.7].iter().enumerate() {
            let mut learning = Learning::new(
                scope.clone(),
                LearningCategory::Preference,
                LearningContent {
                    description: format!("Preference {}", i),
                    pattern: None,
                    insight: format!("insight {}", i),
                },
                LearningSource::UserCreated,
            );
            learning.confidence = *conf;
            store.store(&learning).await.unwrap();
        }

        // Find for injection without context - should order by confidence
        let results = store.find_for_injection(&scope, None, 10).await.unwrap();

        assert_eq!(results.len(), 3);
        // Should be ordered by confidence descending
        assert!((results[0].confidence - 0.9).abs() < f64::EPSILON);
        assert!((results[1].confidence - 0.7).abs() < f64::EPSILON);
        assert!((results[2].confidence - 0.5).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_count_by_scope() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        let scope1 = Scope::User("alice".into());
        let scope2 = Scope::User("bob".into());

        // Create 3 learnings for alice, 2 for bob
        for _ in 0..3 {
            let learning = Learning::new(
                scope1.clone(),
                LearningCategory::Preference,
                LearningContent {
                    description: "Test".into(),
                    pattern: None,
                    insight: "insight".into(),
                },
                LearningSource::UserCreated,
            );
            store.store(&learning).await.unwrap();
        }
        for _ in 0..2 {
            let learning = Learning::new(
                scope2.clone(),
                LearningCategory::Preference,
                LearningContent {
                    description: "Test".into(),
                    pattern: None,
                    insight: "insight".into(),
                },
                LearningSource::UserCreated,
            );
            store.store(&learning).await.unwrap();
        }

        assert_eq!(store.count_by_scope(&scope1).await.unwrap(), 3);
        assert_eq!(store.count_by_scope(&scope2).await.unwrap(), 2);
        assert_eq!(store.count_by_scope(&Scope::Global).await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_count_by_category() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();

        // Create learnings with different categories
        for _ in 0..3 {
            let learning = Learning::new(
                Scope::Global,
                LearningCategory::Preference,
                LearningContent {
                    description: "Test".into(),
                    pattern: None,
                    insight: "insight".into(),
                },
                LearningSource::UserCreated,
            );
            store.store(&learning).await.unwrap();
        }
        for _ in 0..2 {
            let learning = Learning::new(
                Scope::Global,
                LearningCategory::Correction,
                LearningContent {
                    description: "Test".into(),
                    pattern: None,
                    insight: "insight".into(),
                },
                LearningSource::UserCreated,
            );
            store.store(&learning).await.unwrap();
        }

        assert_eq!(
            store
                .count_by_category(&LearningCategory::Preference)
                .await
                .unwrap(),
            3
        );
        assert_eq!(
            store
                .count_by_category(&LearningCategory::Correction)
                .await
                .unwrap(),
            2
        );
        assert_eq!(
            store
                .count_by_category(&LearningCategory::ErrorRecovery)
                .await
                .unwrap(),
            0
        );
    }
}
