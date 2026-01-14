//! Turso/libSQL implementation of evaluation storage.
//!
//! This module provides persistent storage using Turso (libSQL).
//! It can connect to:
//! - Remote Turso database (cloud)
//! - Local embedded SQLite file

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use libsql::{Builder, Connection, Database};
use tracing::{debug, instrument};

use super::{Error, EvalProjection, EvalStorage, Result};
use crate::events::{EvalEvent, StoredEvalEvent};
use crate::metrics::LongitudinalMetrics;
use crate::study::{Checkpoint, PeriodType, Study, StudyConfig, StudyStatus};
use crate::types::{CheckpointId, StudyId};

/// SQL schema for the studies table.
const SCHEMA_STUDIES: &str = r#"
CREATE TABLE IF NOT EXISTS studies (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    status TEXT NOT NULL,
    period_type TEXT NOT NULL,
    period_value INTEGER,
    config TEXT NOT NULL,
    created_at TEXT NOT NULL,
    started_at TEXT,
    stopped_at TEXT
)
"#;

/// SQL schema for the checkpoints table.
const SCHEMA_CHECKPOINTS: &str = r#"
CREATE TABLE IF NOT EXISTS checkpoints (
    id TEXT PRIMARY KEY,
    study_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    metrics TEXT NOT NULL,
    events_analyzed INTEGER NOT NULL,
    sessions_included TEXT NOT NULL
)
"#;

/// SQL index for efficient checkpoint queries.
const INDEX_CHECKPOINTS: &str = r#"
CREATE INDEX IF NOT EXISTS idx_checkpoints_study_time
ON checkpoints(study_id, timestamp)
"#;

/// Turso-backed evaluation storage.
///
/// Provides read-only queries against the projection.
#[derive(Clone)]
pub struct TursoEvalStorage {
    db: Arc<Database>,
}

impl TursoEvalStorage {
    /// Create a new storage instance with a local embedded database.
    pub async fn new_local(path: &Path) -> Result<Self> {
        let db = Builder::new_local(path).build().await?;
        let storage = Self { db: Arc::new(db) };
        storage.ensure_schema().await?;
        Ok(storage)
    }

    /// Create a new storage instance connected to a remote Turso database.
    pub async fn new_remote(url: &str, token: &str) -> Result<Self> {
        let db = Builder::new_remote(url.to_string(), token.to_string())
            .build()
            .await?;
        let storage = Self { db: Arc::new(db) };
        storage.ensure_schema().await?;
        Ok(storage)
    }

    /// Create a new in-memory storage instance (for testing).
    pub async fn new_memory() -> Result<Self> {
        let db = Builder::new_local(":memory:").build().await?;
        let storage = Self { db: Arc::new(db) };
        storage.ensure_schema().await?;
        Ok(storage)
    }

    /// Get a database connection.
    async fn conn(&self) -> Result<Connection> {
        Ok(self.db.connect()?)
    }

    /// Ensure the database schema exists.
    async fn ensure_schema(&self) -> Result<()> {
        let conn = self.conn().await?;
        conn.execute(SCHEMA_STUDIES, ()).await?;
        conn.execute(SCHEMA_CHECKPOINTS, ()).await?;
        conn.execute(INDEX_CHECKPOINTS, ()).await?;
        Ok(())
    }

    /// Parse a study from a database row.
    fn parse_study(row: &libsql::Row) -> Result<Study> {
        let id_str: String = row.get(0)?;
        let name: String = row.get(1)?;
        let status_str: String = row.get(2)?;
        let period_type_str: String = row.get(3)?;
        let period_value: Option<i64> = row.get(4)?;
        let config_json: String = row.get(5)?;
        let created_at_str: String = row.get(6)?;
        let started_at_str: Option<String> = row.get(7)?;
        let stopped_at_str: Option<String> = row.get(8)?;

        let id = StudyId(
            id_str
                .parse()
                .map_err(|_| Error::InvalidData(format!("invalid study id: {}", id_str)))?,
        );
        let status = StudyStatus::parse(&status_str)
            .ok_or_else(|| Error::InvalidData(format!("invalid status: {}", status_str)))?;
        let period_type = PeriodType::parse(&period_type_str).ok_or_else(|| {
            Error::InvalidData(format!("invalid period type: {}", period_type_str))
        })?;
        let config: StudyConfig = serde_json::from_str(&config_json)?;
        let created_at = parse_datetime(&created_at_str)?;
        let started_at = started_at_str
            .as_ref()
            .map(|s| parse_datetime(s))
            .transpose()?;
        let stopped_at = stopped_at_str
            .as_ref()
            .map(|s| parse_datetime(s))
            .transpose()?;

        Ok(Study {
            id,
            name,
            status,
            period_type,
            period_value: period_value.map(|v| v as u32),
            config,
            created_at,
            started_at,
            stopped_at,
        })
    }

    /// Parse a checkpoint from a database row.
    fn parse_checkpoint(row: &libsql::Row) -> Result<Checkpoint> {
        let id_str: String = row.get(0)?;
        let study_id_str: String = row.get(1)?;
        let timestamp_str: String = row.get(2)?;
        let metrics_json: String = row.get(3)?;
        let events_analyzed: i64 = row.get(4)?;
        let sessions_json: String = row.get(5)?;

        let id = CheckpointId(
            id_str
                .parse()
                .map_err(|_| Error::InvalidData(format!("invalid checkpoint id: {}", id_str)))?,
        );
        let study_id = StudyId(
            study_id_str
                .parse()
                .map_err(|_| Error::InvalidData(format!("invalid study id: {}", study_id_str)))?,
        );
        let timestamp = parse_datetime(&timestamp_str)?;
        let metrics: LongitudinalMetrics = serde_json::from_str(&metrics_json)?;
        let sessions_included: Vec<String> = serde_json::from_str(&sessions_json)?;

        Ok(Checkpoint {
            id,
            study_id,
            timestamp,
            metrics,
            events_analyzed: events_analyzed as u64,
            sessions_included,
        })
    }
}

#[async_trait]
impl EvalStorage for TursoEvalStorage {
    #[instrument(skip(self), level = "debug")]
    async fn get_study(&self, id: StudyId) -> Result<Option<Study>> {
        let conn = self.conn().await?;
        let mut rows = conn
            .query(
                "SELECT id, name, status, period_type, period_value, config, created_at, started_at, stopped_at FROM studies WHERE id = ?",
                [id.0.to_string()],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(Self::parse_study(&row)?))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip(self), level = "debug")]
    async fn list_studies(&self) -> Result<Vec<Study>> {
        let conn = self.conn().await?;
        let mut rows = conn
            .query(
                "SELECT id, name, status, period_type, period_value, config, created_at, started_at, stopped_at FROM studies ORDER BY created_at DESC",
                (),
            )
            .await?;

        let mut studies = Vec::new();
        while let Some(row) = rows.next().await? {
            studies.push(Self::parse_study(&row)?);
        }
        Ok(studies)
    }

    #[instrument(skip(self), level = "debug")]
    async fn list_studies_by_status(&self, status: StudyStatus) -> Result<Vec<Study>> {
        let conn = self.conn().await?;
        let mut rows = conn
            .query(
                "SELECT id, name, status, period_type, period_value, config, created_at, started_at, stopped_at FROM studies WHERE status = ? ORDER BY created_at DESC",
                [status.as_str()],
            )
            .await?;

        let mut studies = Vec::new();
        while let Some(row) = rows.next().await? {
            studies.push(Self::parse_study(&row)?);
        }
        Ok(studies)
    }

    #[instrument(skip(self), level = "debug")]
    async fn get_checkpoints(&self, study_id: StudyId) -> Result<Vec<Checkpoint>> {
        let conn = self.conn().await?;
        let mut rows = conn
            .query(
                "SELECT id, study_id, timestamp, metrics, events_analyzed, sessions_included FROM checkpoints WHERE study_id = ? ORDER BY timestamp ASC",
                [study_id.0.to_string()],
            )
            .await?;

        let mut checkpoints = Vec::new();
        while let Some(row) = rows.next().await? {
            checkpoints.push(Self::parse_checkpoint(&row)?);
        }
        Ok(checkpoints)
    }

    #[instrument(skip(self), level = "debug")]
    async fn get_latest_checkpoint(&self, study_id: StudyId) -> Result<Option<Checkpoint>> {
        let conn = self.conn().await?;
        let mut rows = conn
            .query(
                "SELECT id, study_id, timestamp, metrics, events_analyzed, sessions_included FROM checkpoints WHERE study_id = ? ORDER BY timestamp DESC LIMIT 1",
                [study_id.0.to_string()],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(Self::parse_checkpoint(&row)?))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip(self), level = "debug")]
    async fn get_checkpoints_in_range(
        &self,
        study_id: StudyId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Checkpoint>> {
        let conn = self.conn().await?;
        let mut rows = conn
            .query(
                "SELECT id, study_id, timestamp, metrics, events_analyzed, sessions_included FROM checkpoints WHERE study_id = ? AND timestamp >= ? AND timestamp < ? ORDER BY timestamp ASC",
                libsql::params![study_id.0.to_string(), format_datetime(start), format_datetime(end)],
            )
            .await?;

        let mut checkpoints = Vec::new();
        while let Some(row) = rows.next().await? {
            checkpoints.push(Self::parse_checkpoint(&row)?);
        }
        Ok(checkpoints)
    }
}

/// Turso-backed evaluation projection.
///
/// Applies events to update the database.
pub struct TursoEvalProjection {
    storage: TursoEvalStorage,
}

impl TursoEvalProjection {
    /// Create a new projection instance.
    pub fn new(storage: TursoEvalStorage) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl EvalProjection for TursoEvalProjection {
    #[instrument(skip(self, event), level = "debug")]
    async fn apply(&self, event: &StoredEvalEvent) -> Result<()> {
        let conn = self.storage.conn().await?;

        match &event.event {
            EvalEvent::StudyCreated {
                id,
                name,
                period_type,
                period_value,
                config,
            } => {
                debug!(?id, name, "applying StudyCreated event");
                let config_json = serde_json::to_string(config)?;
                conn.execute(
                    "INSERT INTO studies (id, name, status, period_type, period_value, config, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
                    libsql::params![
                        id.0.to_string(),
                        name.clone(),
                        StudyStatus::Pending.as_str(),
                        period_type.as_str(),
                        period_value.map(|v| v as i64),
                        config_json,
                        format_datetime(Utc::now())
                    ],
                )
                .await?;
            }

            EvalEvent::StudyStarted { id } => {
                debug!(?id, "applying StudyStarted event");
                conn.execute(
                    "UPDATE studies SET status = ?, started_at = ? WHERE id = ?",
                    libsql::params![
                        StudyStatus::Running.as_str(),
                        format_datetime(Utc::now()),
                        id.0.to_string()
                    ],
                )
                .await?;
            }

            EvalEvent::StudyPaused { id } => {
                debug!(?id, "applying StudyPaused event");
                conn.execute(
                    "UPDATE studies SET status = ? WHERE id = ?",
                    libsql::params![StudyStatus::Paused.as_str(), id.0.to_string()],
                )
                .await?;
            }

            EvalEvent::StudyResumed { id } => {
                debug!(?id, "applying StudyResumed event");
                conn.execute(
                    "UPDATE studies SET status = ? WHERE id = ?",
                    libsql::params![StudyStatus::Running.as_str(), id.0.to_string()],
                )
                .await?;
            }

            EvalEvent::StudyStopped { id } => {
                debug!(?id, "applying StudyStopped event");
                conn.execute(
                    "UPDATE studies SET status = ?, stopped_at = ? WHERE id = ?",
                    libsql::params![
                        StudyStatus::Stopped.as_str(),
                        format_datetime(Utc::now()),
                        id.0.to_string()
                    ],
                )
                .await?;
            }

            EvalEvent::CheckpointRecorded {
                id,
                study_id,
                timestamp,
                metrics,
                events_analyzed,
                sessions_included,
            } => {
                debug!(?id, ?study_id, "applying CheckpointRecorded event");
                let metrics_json = serde_json::to_string(metrics)?;
                let sessions_json = serde_json::to_string(sessions_included)?;
                conn.execute(
                    "INSERT INTO checkpoints (id, study_id, timestamp, metrics, events_analyzed, sessions_included) VALUES (?, ?, ?, ?, ?, ?)",
                    libsql::params![
                        id.0.to_string(),
                        study_id.0.to_string(),
                        format_datetime(*timestamp),
                        metrics_json,
                        *events_analyzed as i64,
                        sessions_json
                    ],
                )
                .await?;
            }
        }

        Ok(())
    }

    #[instrument(skip(self), level = "debug")]
    async fn clear(&self) -> Result<()> {
        let conn = self.storage.conn().await?;
        conn.execute("DELETE FROM checkpoints", ()).await?;
        conn.execute("DELETE FROM studies", ()).await?;
        Ok(())
    }
}

/// Format a datetime for storage.
fn format_datetime(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

/// Parse a datetime from storage.
fn parse_datetime(s: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| Error::InvalidData(format!("invalid datetime: {}", s)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    async fn create_test_storage() -> TursoEvalStorage {
        TursoEvalStorage::new_memory().await.unwrap()
    }

    fn sample_study_id() -> StudyId {
        StudyId(Uuid::new_v4())
    }

    fn sample_checkpoint_id() -> CheckpointId {
        CheckpointId(Uuid::new_v4())
    }

    #[tokio::test]
    async fn storage_returns_none_for_nonexistent_study() {
        let storage = create_test_storage().await;

        let result = storage.get_study(sample_study_id()).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn storage_returns_empty_list_when_no_studies() {
        let storage = create_test_storage().await;

        let result = storage.list_studies().await.unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn projection_creates_study_from_event() {
        let storage = create_test_storage().await;
        let projection = TursoEvalProjection::new(storage.clone());
        let study_id = sample_study_id();

        let event = StoredEvalEvent::new(EvalEvent::StudyCreated {
            id: study_id,
            name: "test-study".to_string(),
            period_type: PeriodType::Weekly,
            period_value: Some(2),
            config: StudyConfig::default(),
        });

        projection.apply(&event).await.unwrap();

        let study = storage.get_study(study_id).await.unwrap().unwrap();
        assert_eq!(study.name, "test-study");
        assert_eq!(study.status, StudyStatus::Pending);
        assert_eq!(study.period_type, PeriodType::Weekly);
        assert_eq!(study.period_value, Some(2));
    }

    #[tokio::test]
    async fn projection_starts_study() {
        let storage = create_test_storage().await;
        let projection = TursoEvalProjection::new(storage.clone());
        let study_id = sample_study_id();

        // Create study
        projection
            .apply(&StoredEvalEvent::new(EvalEvent::StudyCreated {
                id: study_id,
                name: "test".to_string(),
                period_type: PeriodType::Daily,
                period_value: None,
                config: StudyConfig::default(),
            }))
            .await
            .unwrap();

        // Start study
        projection
            .apply(&StoredEvalEvent::new(EvalEvent::StudyStarted {
                id: study_id,
            }))
            .await
            .unwrap();

        let study = storage.get_study(study_id).await.unwrap().unwrap();
        assert_eq!(study.status, StudyStatus::Running);
        assert!(study.started_at.is_some());
    }

    #[tokio::test]
    async fn projection_pauses_and_resumes_study() {
        let storage = create_test_storage().await;
        let projection = TursoEvalProjection::new(storage.clone());
        let study_id = sample_study_id();

        // Create and start study
        projection
            .apply(&StoredEvalEvent::new(EvalEvent::StudyCreated {
                id: study_id,
                name: "test".to_string(),
                period_type: PeriodType::Daily,
                period_value: None,
                config: StudyConfig::default(),
            }))
            .await
            .unwrap();
        projection
            .apply(&StoredEvalEvent::new(EvalEvent::StudyStarted {
                id: study_id,
            }))
            .await
            .unwrap();

        // Pause
        projection
            .apply(&StoredEvalEvent::new(EvalEvent::StudyPaused {
                id: study_id,
            }))
            .await
            .unwrap();
        let study = storage.get_study(study_id).await.unwrap().unwrap();
        assert_eq!(study.status, StudyStatus::Paused);

        // Resume
        projection
            .apply(&StoredEvalEvent::new(EvalEvent::StudyResumed {
                id: study_id,
            }))
            .await
            .unwrap();
        let study = storage.get_study(study_id).await.unwrap().unwrap();
        assert_eq!(study.status, StudyStatus::Running);
    }

    #[tokio::test]
    async fn projection_stops_study() {
        let storage = create_test_storage().await;
        let projection = TursoEvalProjection::new(storage.clone());
        let study_id = sample_study_id();

        // Create and start study
        projection
            .apply(&StoredEvalEvent::new(EvalEvent::StudyCreated {
                id: study_id,
                name: "test".to_string(),
                period_type: PeriodType::Daily,
                period_value: None,
                config: StudyConfig::default(),
            }))
            .await
            .unwrap();
        projection
            .apply(&StoredEvalEvent::new(EvalEvent::StudyStarted {
                id: study_id,
            }))
            .await
            .unwrap();

        // Stop
        projection
            .apply(&StoredEvalEvent::new(EvalEvent::StudyStopped {
                id: study_id,
            }))
            .await
            .unwrap();

        let study = storage.get_study(study_id).await.unwrap().unwrap();
        assert_eq!(study.status, StudyStatus::Stopped);
        assert!(study.stopped_at.is_some());
    }

    #[tokio::test]
    async fn projection_records_checkpoint() {
        let storage = create_test_storage().await;
        let projection = TursoEvalProjection::new(storage.clone());
        let study_id = sample_study_id();
        let checkpoint_id = sample_checkpoint_id();

        // Create study first
        projection
            .apply(&StoredEvalEvent::new(EvalEvent::StudyCreated {
                id: study_id,
                name: "test".to_string(),
                period_type: PeriodType::Daily,
                period_value: None,
                config: StudyConfig::default(),
            }))
            .await
            .unwrap();

        // Record checkpoint
        let timestamp = Utc::now();
        let metrics = LongitudinalMetrics::default();
        projection
            .apply(&StoredEvalEvent::new(EvalEvent::CheckpointRecorded {
                id: checkpoint_id,
                study_id,
                timestamp,
                metrics: metrics.clone(),
                events_analyzed: 100,
                sessions_included: vec!["sess-1".to_string()],
            }))
            .await
            .unwrap();

        let checkpoints = storage.get_checkpoints(study_id).await.unwrap();
        assert_eq!(checkpoints.len(), 1);
        assert_eq!(checkpoints[0].id, checkpoint_id);
        assert_eq!(checkpoints[0].events_analyzed, 100);
    }

    #[tokio::test]
    async fn storage_lists_studies_by_status() {
        let storage = create_test_storage().await;
        let projection = TursoEvalProjection::new(storage.clone());

        // Create two studies
        let study1 = sample_study_id();
        let study2 = sample_study_id();

        projection
            .apply(&StoredEvalEvent::new(EvalEvent::StudyCreated {
                id: study1,
                name: "study-1".to_string(),
                period_type: PeriodType::Daily,
                period_value: None,
                config: StudyConfig::default(),
            }))
            .await
            .unwrap();
        projection
            .apply(&StoredEvalEvent::new(EvalEvent::StudyCreated {
                id: study2,
                name: "study-2".to_string(),
                period_type: PeriodType::Daily,
                period_value: None,
                config: StudyConfig::default(),
            }))
            .await
            .unwrap();

        // Start only the first
        projection
            .apply(&StoredEvalEvent::new(EvalEvent::StudyStarted {
                id: study1,
            }))
            .await
            .unwrap();

        // Check filtering
        let pending = storage
            .list_studies_by_status(StudyStatus::Pending)
            .await
            .unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].name, "study-2");

        let running = storage
            .list_studies_by_status(StudyStatus::Running)
            .await
            .unwrap();
        assert_eq!(running.len(), 1);
        assert_eq!(running[0].name, "study-1");
    }

    #[tokio::test]
    async fn storage_gets_latest_checkpoint() {
        let storage = create_test_storage().await;
        let projection = TursoEvalProjection::new(storage.clone());
        let study_id = sample_study_id();

        // Create study
        projection
            .apply(&StoredEvalEvent::new(EvalEvent::StudyCreated {
                id: study_id,
                name: "test".to_string(),
                period_type: PeriodType::Daily,
                period_value: None,
                config: StudyConfig::default(),
            }))
            .await
            .unwrap();

        // Record two checkpoints
        let cp1 = sample_checkpoint_id();
        let cp2 = sample_checkpoint_id();
        let t1 = Utc::now() - chrono::Duration::hours(1);
        let t2 = Utc::now();

        projection
            .apply(&StoredEvalEvent::new(EvalEvent::CheckpointRecorded {
                id: cp1,
                study_id,
                timestamp: t1,
                metrics: LongitudinalMetrics::default(),
                events_analyzed: 50,
                sessions_included: vec![],
            }))
            .await
            .unwrap();
        projection
            .apply(&StoredEvalEvent::new(EvalEvent::CheckpointRecorded {
                id: cp2,
                study_id,
                timestamp: t2,
                metrics: LongitudinalMetrics::default(),
                events_analyzed: 100,
                sessions_included: vec![],
            }))
            .await
            .unwrap();

        let latest = storage.get_latest_checkpoint(study_id).await.unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().id, cp2);
    }

    #[tokio::test]
    async fn projection_clear_removes_all_data() {
        let storage = create_test_storage().await;
        let projection = TursoEvalProjection::new(storage.clone());

        // Create study with checkpoint
        let study_id = sample_study_id();
        projection
            .apply(&StoredEvalEvent::new(EvalEvent::StudyCreated {
                id: study_id,
                name: "test".to_string(),
                period_type: PeriodType::Daily,
                period_value: None,
                config: StudyConfig::default(),
            }))
            .await
            .unwrap();
        projection
            .apply(&StoredEvalEvent::new(EvalEvent::CheckpointRecorded {
                id: sample_checkpoint_id(),
                study_id,
                timestamp: Utc::now(),
                metrics: LongitudinalMetrics::default(),
                events_analyzed: 0,
                sessions_included: vec![],
            }))
            .await
            .unwrap();

        // Clear
        projection.clear().await.unwrap();

        assert!(storage.list_studies().await.unwrap().is_empty());
        assert!(storage.get_checkpoints(study_id).await.unwrap().is_empty());
    }
}
