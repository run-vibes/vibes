//! CozoDB-backed learning store implementation
//!
//! This module provides the main `CozoStore` struct that wraps CozoDB with
//! RocksDB backend. It handles database creation, schema initialization,
//! and migrations.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use chrono::Utc;
use cozo::{DataValue, DbInstance, NamedRows};

use super::schema::MIGRATIONS;
use crate::{GrooveError, Result};

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
}

#[cfg(test)]
mod tests {
    use super::super::schema::CURRENT_SCHEMA_VERSION;
    use super::*;
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
}
