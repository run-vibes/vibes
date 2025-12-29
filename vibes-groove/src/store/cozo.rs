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
use cozo::{DataValue, DbInstance, NamedRows};

use super::schema::MIGRATIONS;
use crate::{
    GrooveError, Learning, LearningCategory, LearningContent, LearningId, LearningSource, Result,
    Scope, UsageStats,
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
    #[allow(dead_code)]
    async fn run_mutation(
        &self,
        query: &str,
        params: BTreeMap<String, DataValue>,
    ) -> Result<NamedRows> {
        self.db
            .run_script(query, params, cozo::ScriptMutability::Mutable)
            .map_err(|e| GrooveError::Database(format!("Mutation failed: {e}")))
    }

    /// Get a reference to the underlying database
    pub fn db(&self) -> &DbInstance {
        &self.db
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
}

#[cfg(test)]
mod tests {
    use super::super::schema::CURRENT_SCHEMA_VERSION;
    use super::*;
    use crate::{Learning, LearningCategory, LearningContent, LearningSource, Outcome, Scope};
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
}
