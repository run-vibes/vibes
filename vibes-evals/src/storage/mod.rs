//! Storage traits and implementations for evaluation data.
//!
//! This module follows the CQRS pattern:
//! - [`EvalStorage`] - Read-only queries against the projection
//! - [`EvalProjection`] - Applies events to update the projection
//!
//! The Turso implementations store data in libSQL for fast queries.

mod error;
mod turso;

pub use error::{Error, Result};
pub use turso::{TursoEvalProjection, TursoEvalStorage};

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::events::StoredEvalEvent;
use crate::study::{Checkpoint, Study, StudyStatus};
use crate::types::StudyId;

/// Read-only queries against the evaluation projection.
///
/// This trait provides fast queries against the Turso database.
/// All data is derived from events - the projection can be rebuilt
/// by replaying events from Iggy.
#[async_trait]
pub trait EvalStorage: Send + Sync {
    /// Get a study by ID.
    async fn get_study(&self, id: StudyId) -> Result<Option<Study>>;

    /// List all studies.
    async fn list_studies(&self) -> Result<Vec<Study>>;

    /// List studies filtered by status.
    async fn list_studies_by_status(&self, status: StudyStatus) -> Result<Vec<Study>>;

    /// Get all checkpoints for a study.
    async fn get_checkpoints(&self, study_id: StudyId) -> Result<Vec<Checkpoint>>;

    /// Get the latest checkpoint for a study.
    async fn get_latest_checkpoint(&self, study_id: StudyId) -> Result<Option<Checkpoint>>;

    /// Get checkpoints within a time range.
    async fn get_checkpoints_in_range(
        &self,
        study_id: StudyId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Checkpoint>>;
}

/// Applies events to update the projection.
///
/// This is the write side of CQRS. Events from Iggy are processed
/// and applied to the Turso database to keep the projection current.
#[async_trait]
pub trait EvalProjection: Send + Sync {
    /// Apply an event to update the projection.
    async fn apply(&self, event: &StoredEvalEvent) -> Result<()>;

    /// Clear all data (for rebuild).
    async fn clear(&self) -> Result<()>;
}
