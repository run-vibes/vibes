//! Attribution storage trait and CozoDB implementation

use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use cozo::{DataValue, DbInstance, ScriptMutability};

use crate::assessment::SessionId;
use crate::error::{GrooveError, Result};
use crate::types::LearningId;

use super::types::{
    AblationExperiment, AblationResult, AttributionRecord, LearningStatus, LearningValue,
};

/// CozoDB schema for attribution tables
pub const ATTRIBUTION_SCHEMA: &str = r#"
{
    :create attribution {
        learning_id: String,
        session_id: String =>
        timestamp: Int,
        was_activated: Bool,
        activation_confidence: Float,
        activation_signals_json: String,
        temporal_positive: Float,
        temporal_negative: Float,
        net_temporal: Float,
        was_withheld: Bool,
        session_outcome: Float,
        attributed_value: Float
    }
}
{
    :create learning_value {
        learning_id: String =>
        estimated_value: Float,
        confidence: Float,
        session_count: Int,
        activation_rate: Float,
        temporal_value: Float,
        temporal_confidence: Float,
        ablation_value: Float?,
        ablation_confidence: Float?,
        status: String,
        status_reason: String?,
        updated_at: Int
    }
}
{
    :create ablation_experiment {
        learning_id: String =>
        started_at: Int,
        sessions_with_json: String,
        sessions_without_json: String,
        marginal_value: Float?,
        confidence: Float?,
        is_significant: Bool?
    }
}
{
    ::index create attribution:by_learning { learning_id }
}
{
    ::index create attribution:by_session { session_id }
}
{
    ::index create learning_value:by_value { estimated_value }
}
{
    ::index create learning_value:by_status { status }
}
"#;

/// Storage interface for attribution data
#[async_trait]
pub trait AttributionStore: Send + Sync {
    // Attribution records
    async fn store_attribution(&self, record: &AttributionRecord) -> Result<()>;
    async fn get_attributions_for_learning(&self, id: LearningId)
    -> Result<Vec<AttributionRecord>>;
    async fn get_attributions_for_session(&self, id: &SessionId) -> Result<Vec<AttributionRecord>>;

    // Learning values
    async fn get_learning_value(&self, id: LearningId) -> Result<Option<LearningValue>>;
    async fn update_learning_value(&self, value: &LearningValue) -> Result<()>;
    async fn list_learning_values(&self, limit: usize) -> Result<Vec<LearningValue>>;

    // Ablation experiments
    async fn get_experiment(&self, id: LearningId) -> Result<Option<AblationExperiment>>;
    async fn update_experiment(&self, exp: &AblationExperiment) -> Result<()>;
}

/// CozoDB-backed attribution store
pub struct CozoAttributionStore {
    db: Arc<DbInstance>,
}

impl CozoAttributionStore {
    pub fn new(db: Arc<DbInstance>) -> Self {
        Self { db }
    }

    /// Initialize the attribution schema
    pub fn init_schema(db: &DbInstance) -> Result<()> {
        db.run_script(
            ATTRIBUTION_SCHEMA,
            Default::default(),
            ScriptMutability::Mutable,
        )
        .map_err(|e| GrooveError::Database(format!("Schema init failed: {e}")))?;
        Ok(())
    }

    /// Run a query and return results
    async fn run_query(
        &self,
        query: &str,
        params: BTreeMap<String, DataValue>,
    ) -> Result<cozo::NamedRows> {
        self.db
            .run_script(query, params, ScriptMutability::Immutable)
            .map_err(|e| GrooveError::Database(format!("Query failed: {e}")))
    }

    /// Run a mutation query
    async fn run_mutation(
        &self,
        query: &str,
        params: BTreeMap<String, DataValue>,
    ) -> Result<cozo::NamedRows> {
        self.db
            .run_script(query, params, ScriptMutability::Mutable)
            .map_err(|e| GrooveError::Database(format!("Mutation failed: {e}")))
    }
}

#[async_trait]
impl AttributionStore for CozoAttributionStore {
    async fn store_attribution(&self, record: &AttributionRecord) -> Result<()> {
        let signals_json = serde_json::to_string(&record.activation_signals)
            .map_err(|e| GrooveError::Serialization(e.to_string()))?
            .replace('\'', "''");

        let query = format!(
            r#"?[learning_id, session_id, timestamp, was_activated, activation_confidence,
               activation_signals_json, temporal_positive, temporal_negative, net_temporal,
               was_withheld, session_outcome, attributed_value] <- [[
                '{}', '{}', {}, {}, {},
                '{}', {}, {}, {},
                {}, {}, {}
            ]]
            :put attribution {{
                learning_id, session_id =>
                timestamp, was_activated, activation_confidence, activation_signals_json,
                temporal_positive, temporal_negative, net_temporal, was_withheld,
                session_outcome, attributed_value
            }}"#,
            record.learning_id,
            record.session_id,
            record.timestamp.timestamp(),
            record.was_activated,
            record.activation_confidence,
            signals_json,
            record.temporal_positive,
            record.temporal_negative,
            record.net_temporal,
            record.was_withheld,
            record.session_outcome,
            record.attributed_value,
        );

        self.run_mutation(&query, Default::default()).await?;
        Ok(())
    }

    async fn get_attributions_for_learning(
        &self,
        id: LearningId,
    ) -> Result<Vec<AttributionRecord>> {
        let query = format!(
            r#"?[learning_id, session_id, timestamp, was_activated, activation_confidence,
               activation_signals_json, temporal_positive, temporal_negative, net_temporal,
               was_withheld, session_outcome, attributed_value] :=
               *attribution{{learning_id, session_id, timestamp, was_activated, activation_confidence,
                           activation_signals_json, temporal_positive, temporal_negative, net_temporal,
                           was_withheld, session_outcome, attributed_value}},
               learning_id = '{}'"#,
            id
        );

        let rows = self.run_query(&query, Default::default()).await?;
        rows.rows
            .iter()
            .map(|row| parse_attribution_row(row))
            .collect()
    }

    async fn get_attributions_for_session(&self, id: &SessionId) -> Result<Vec<AttributionRecord>> {
        let query = format!(
            r#"?[learning_id, session_id, timestamp, was_activated, activation_confidence,
               activation_signals_json, temporal_positive, temporal_negative, net_temporal,
               was_withheld, session_outcome, attributed_value] :=
               *attribution{{learning_id, session_id, timestamp, was_activated, activation_confidence,
                           activation_signals_json, temporal_positive, temporal_negative, net_temporal,
                           was_withheld, session_outcome, attributed_value}},
               session_id = '{}'"#,
            id
        );

        let rows = self.run_query(&query, Default::default()).await?;
        rows.rows
            .iter()
            .map(|row| parse_attribution_row(row))
            .collect()
    }

    async fn get_learning_value(&self, id: LearningId) -> Result<Option<LearningValue>> {
        let query = format!(
            r#"?[learning_id, estimated_value, confidence, session_count, activation_rate,
               temporal_value, temporal_confidence, ablation_value, ablation_confidence,
               status, status_reason, updated_at] :=
               *learning_value{{learning_id, estimated_value, confidence, session_count, activation_rate,
                              temporal_value, temporal_confidence, ablation_value, ablation_confidence,
                              status, status_reason, updated_at}},
               learning_id = '{}'"#,
            id
        );

        let rows = self.run_query(&query, Default::default()).await?;
        if rows.rows.is_empty() {
            return Ok(None);
        }

        Ok(Some(parse_learning_value_row(&rows.rows[0])?))
    }

    async fn update_learning_value(&self, value: &LearningValue) -> Result<()> {
        let (status_str, status_reason) = match &value.status {
            LearningStatus::Active => ("active", None),
            LearningStatus::Deprecated { reason } => ("deprecated", Some(reason.clone())),
            LearningStatus::Experimental => ("experimental", None),
        };

        let ablation_value_str = value
            .ablation_value
            .map(|v| v.to_string())
            .unwrap_or_else(|| "null".to_string());
        let ablation_confidence_str = value
            .ablation_confidence
            .map(|v| v.to_string())
            .unwrap_or_else(|| "null".to_string());
        let status_reason_str = status_reason
            .map(|r| format!("'{}'", r.replace('\'', "''")))
            .unwrap_or_else(|| "null".to_string());

        let query = format!(
            r#"?[learning_id, estimated_value, confidence, session_count, activation_rate,
               temporal_value, temporal_confidence, ablation_value, ablation_confidence,
               status, status_reason, updated_at] <- [[
                '{}', {}, {}, {}, {},
                {}, {}, {}, {},
                '{}', {}, {}
            ]]
            :put learning_value {{
                learning_id =>
                estimated_value, confidence, session_count, activation_rate,
                temporal_value, temporal_confidence, ablation_value, ablation_confidence,
                status, status_reason, updated_at
            }}"#,
            value.learning_id,
            value.estimated_value,
            value.confidence,
            value.session_count,
            value.activation_rate,
            value.temporal_value,
            value.temporal_confidence,
            ablation_value_str,
            ablation_confidence_str,
            status_str,
            status_reason_str,
            value.updated_at.timestamp(),
        );

        self.run_mutation(&query, Default::default()).await?;
        Ok(())
    }

    async fn list_learning_values(&self, limit: usize) -> Result<Vec<LearningValue>> {
        let query = format!(
            r#"?[learning_id, estimated_value, confidence, session_count, activation_rate,
               temporal_value, temporal_confidence, ablation_value, ablation_confidence,
               status, status_reason, updated_at] :=
               *learning_value{{learning_id, estimated_value, confidence, session_count, activation_rate,
                              temporal_value, temporal_confidence, ablation_value, ablation_confidence,
                              status, status_reason, updated_at}}
               :limit {}"#,
            limit
        );

        let rows = self.run_query(&query, Default::default()).await?;
        rows.rows
            .iter()
            .map(|row| parse_learning_value_row(row))
            .collect()
    }

    async fn get_experiment(&self, id: LearningId) -> Result<Option<AblationExperiment>> {
        let query = format!(
            r#"?[learning_id, started_at, sessions_with_json, sessions_without_json,
               marginal_value, confidence, is_significant] :=
               *ablation_experiment{{learning_id, started_at, sessions_with_json, sessions_without_json,
                                   marginal_value, confidence, is_significant}},
               learning_id = '{}'"#,
            id
        );

        let rows = self.run_query(&query, Default::default()).await?;
        if rows.rows.is_empty() {
            return Ok(None);
        }

        Ok(Some(parse_experiment_row(&rows.rows[0])?))
    }

    async fn update_experiment(&self, exp: &AblationExperiment) -> Result<()> {
        let sessions_with_json = serde_json::to_string(&exp.sessions_with)
            .map_err(|e| GrooveError::Serialization(e.to_string()))?
            .replace('\'', "''");
        let sessions_without_json = serde_json::to_string(&exp.sessions_without)
            .map_err(|e| GrooveError::Serialization(e.to_string()))?
            .replace('\'', "''");

        let (marginal_value, confidence, is_significant) = match &exp.result {
            Some(result) => (
                result.marginal_value.to_string(),
                result.confidence.to_string(),
                result.is_significant.to_string(),
            ),
            None => ("null".to_string(), "null".to_string(), "null".to_string()),
        };

        let query = format!(
            r#"?[learning_id, started_at, sessions_with_json, sessions_without_json,
               marginal_value, confidence, is_significant] <- [[
                '{}', {}, '{}', '{}',
                {}, {}, {}
            ]]
            :put ablation_experiment {{
                learning_id =>
                started_at, sessions_with_json, sessions_without_json,
                marginal_value, confidence, is_significant
            }}"#,
            exp.learning_id,
            exp.started_at.timestamp(),
            sessions_with_json,
            sessions_without_json,
            marginal_value,
            confidence,
            is_significant,
        );

        self.run_mutation(&query, Default::default()).await?;
        Ok(())
    }
}

// Helper functions for parsing CozoDB rows

fn parse_attribution_row(row: &[DataValue]) -> Result<AttributionRecord> {
    let learning_id = row[0]
        .get_str()
        .ok_or_else(|| GrooveError::Database("Invalid learning_id".into()))?;
    let session_id = row[1]
        .get_str()
        .ok_or_else(|| GrooveError::Database("Invalid session_id".into()))?;
    let timestamp = row[2]
        .get_int()
        .ok_or_else(|| GrooveError::Database("Invalid timestamp".into()))?;
    let signals_json = row[5]
        .get_str()
        .ok_or_else(|| GrooveError::Database("Invalid signals json".into()))?;

    Ok(AttributionRecord {
        learning_id: learning_id
            .parse()
            .map_err(|e| GrooveError::Database(format!("Invalid UUID: {e}")))?,
        session_id: SessionId::from(session_id),
        timestamp: Utc
            .timestamp_opt(timestamp, 0)
            .single()
            .ok_or_else(|| GrooveError::Database("Invalid timestamp".into()))?,
        was_activated: row[3].get_bool().unwrap_or(false),
        activation_confidence: row[4].get_float().unwrap_or(0.0),
        activation_signals: serde_json::from_str(signals_json)
            .map_err(|e| GrooveError::Serialization(e.to_string()))?,
        temporal_positive: row[6].get_float().unwrap_or(0.0),
        temporal_negative: row[7].get_float().unwrap_or(0.0),
        net_temporal: row[8].get_float().unwrap_or(0.0),
        was_withheld: row[9].get_bool().unwrap_or(false),
        session_outcome: row[10].get_float().unwrap_or(0.0),
        attributed_value: row[11].get_float().unwrap_or(0.0),
    })
}

fn parse_learning_value_row(row: &[DataValue]) -> Result<LearningValue> {
    let learning_id = row[0]
        .get_str()
        .ok_or_else(|| GrooveError::Database("Invalid learning_id".into()))?;
    let status_str = row[9]
        .get_str()
        .ok_or_else(|| GrooveError::Database("Invalid status".into()))?;
    let status_reason = row[10].get_str().map(|s| s.to_string());
    let timestamp = row[11]
        .get_int()
        .ok_or_else(|| GrooveError::Database("Invalid updated_at".into()))?;

    Ok(LearningValue {
        learning_id: learning_id
            .parse()
            .map_err(|e| GrooveError::Database(format!("Invalid UUID: {e}")))?,
        estimated_value: row[1].get_float().unwrap_or(0.0),
        confidence: row[2].get_float().unwrap_or(0.0),
        session_count: row[3].get_int().unwrap_or(0) as u32,
        activation_rate: row[4].get_float().unwrap_or(0.0),
        temporal_value: row[5].get_float().unwrap_or(0.0),
        temporal_confidence: row[6].get_float().unwrap_or(0.0),
        ablation_value: row[7].get_float(),
        ablation_confidence: row[8].get_float(),
        status: LearningStatus::from_str_with_reason(status_str, status_reason),
        updated_at: Utc
            .timestamp_opt(timestamp, 0)
            .single()
            .ok_or_else(|| GrooveError::Database("Invalid timestamp".into()))?,
    })
}

fn parse_experiment_row(row: &[DataValue]) -> Result<AblationExperiment> {
    let learning_id = row[0]
        .get_str()
        .ok_or_else(|| GrooveError::Database("Invalid learning_id".into()))?;
    let started_at = row[1]
        .get_int()
        .ok_or_else(|| GrooveError::Database("Invalid started_at".into()))?;
    let sessions_with_json = row[2]
        .get_str()
        .ok_or_else(|| GrooveError::Database("Invalid sessions_with_json".into()))?;
    let sessions_without_json = row[3]
        .get_str()
        .ok_or_else(|| GrooveError::Database("Invalid sessions_without_json".into()))?;

    let result = match (row[4].get_float(), row[5].get_float(), row[6].get_bool()) {
        (Some(mv), Some(c), Some(sig)) => Some(AblationResult {
            marginal_value: mv,
            confidence: c,
            is_significant: sig,
        }),
        _ => None,
    };

    Ok(AblationExperiment {
        learning_id: learning_id
            .parse()
            .map_err(|e| GrooveError::Database(format!("Invalid UUID: {e}")))?,
        started_at: Utc
            .timestamp_opt(started_at, 0)
            .single()
            .ok_or_else(|| GrooveError::Database("Invalid timestamp".into()))?,
        sessions_with: serde_json::from_str(sessions_with_json)
            .map_err(|e| GrooveError::Serialization(e.to_string()))?,
        sessions_without: serde_json::from_str(sessions_without_json)
            .map_err(|e| GrooveError::Serialization(e.to_string()))?,
        result,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    use crate::attribution::{AblationResult, ActivationSignal, LearningStatus, SessionOutcome};

    fn create_test_db() -> Arc<DbInstance> {
        let db = DbInstance::new("mem", "", Default::default()).unwrap();
        CozoAttributionStore::init_schema(&db).unwrap();
        Arc::new(db)
    }

    // ============================================================
    // Attribution Record Tests
    // ============================================================

    #[tokio::test]
    async fn test_store_and_retrieve_attribution_by_learning() {
        let db = create_test_db();
        let store = CozoAttributionStore::new(db);

        let learning_id = Uuid::now_v7();
        let record = AttributionRecord {
            learning_id,
            session_id: SessionId::from("session-1"),
            timestamp: Utc::now(),
            was_activated: true,
            activation_confidence: 0.85,
            activation_signals: vec![ActivationSignal::EmbeddingSimilarity {
                score: 0.9,
                message_idx: 3,
            }],
            temporal_positive: 0.7,
            temporal_negative: 0.1,
            net_temporal: 0.6,
            was_withheld: false,
            session_outcome: 0.8,
            attributed_value: 0.72,
        };

        store.store_attribution(&record).await.unwrap();

        let retrieved = store
            .get_attributions_for_learning(learning_id)
            .await
            .unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].learning_id, learning_id);
        assert!(retrieved[0].was_activated);
        assert_eq!(retrieved[0].activation_confidence, 0.85);
    }

    #[tokio::test]
    async fn test_retrieve_attribution_by_session() {
        let db = create_test_db();
        let store = CozoAttributionStore::new(db);

        let session_id = SessionId::from("session-abc");
        let learning1 = Uuid::now_v7();
        let learning2 = Uuid::now_v7();

        let record1 = AttributionRecord {
            learning_id: learning1,
            session_id: session_id.clone(),
            timestamp: Utc::now(),
            was_activated: true,
            activation_confidence: 0.9,
            activation_signals: vec![],
            temporal_positive: 0.5,
            temporal_negative: 0.0,
            net_temporal: 0.5,
            was_withheld: false,
            session_outcome: 0.7,
            attributed_value: 0.5,
        };

        let record2 = AttributionRecord {
            learning_id: learning2,
            session_id: session_id.clone(),
            timestamp: Utc::now(),
            was_activated: false,
            activation_confidence: 0.3,
            activation_signals: vec![],
            temporal_positive: 0.0,
            temporal_negative: 0.0,
            net_temporal: 0.0,
            was_withheld: true,
            session_outcome: 0.7,
            attributed_value: 0.0,
        };

        store.store_attribution(&record1).await.unwrap();
        store.store_attribution(&record2).await.unwrap();

        let retrieved = store
            .get_attributions_for_session(&session_id)
            .await
            .unwrap();
        assert_eq!(retrieved.len(), 2);
    }

    // ============================================================
    // Learning Value Tests
    // ============================================================

    #[tokio::test]
    async fn test_learning_value_crud() {
        let db = create_test_db();
        let store = CozoAttributionStore::new(db);

        let learning_id = Uuid::now_v7();

        // Initially none
        let value = store.get_learning_value(learning_id).await.unwrap();
        assert!(value.is_none());

        // Create
        let new_value = LearningValue {
            learning_id,
            estimated_value: 0.65,
            confidence: 0.8,
            session_count: 10,
            activation_rate: 0.4,
            temporal_value: 0.6,
            temporal_confidence: 0.75,
            ablation_value: None,
            ablation_confidence: None,
            status: LearningStatus::Active,
            updated_at: Utc::now(),
        };
        store.update_learning_value(&new_value).await.unwrap();

        // Read back
        let retrieved = store
            .get_learning_value(learning_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.estimated_value, 0.65);
        assert_eq!(retrieved.session_count, 10);
        assert_eq!(retrieved.status, LearningStatus::Active);

        // Update
        let updated_value = LearningValue {
            session_count: 15,
            estimated_value: 0.7,
            ablation_value: Some(0.68),
            ablation_confidence: Some(0.9),
            ..retrieved
        };
        store.update_learning_value(&updated_value).await.unwrap();

        let retrieved = store
            .get_learning_value(learning_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.session_count, 15);
        assert_eq!(retrieved.estimated_value, 0.7);
        assert_eq!(retrieved.ablation_value, Some(0.68));
    }

    #[tokio::test]
    async fn test_list_learning_values_with_limit() {
        let db = create_test_db();
        let store = CozoAttributionStore::new(db);

        // Create 5 learning values
        for i in 0..5 {
            let value = LearningValue {
                learning_id: Uuid::now_v7(),
                estimated_value: 0.1 * (i as f64),
                confidence: 0.8,
                session_count: i as u32,
                activation_rate: 0.5,
                temporal_value: 0.5,
                temporal_confidence: 0.7,
                ablation_value: None,
                ablation_confidence: None,
                status: LearningStatus::Active,
                updated_at: Utc::now(),
            };
            store.update_learning_value(&value).await.unwrap();
        }

        // List with limit
        let values = store.list_learning_values(3).await.unwrap();
        assert_eq!(values.len(), 3);

        // List all
        let values = store.list_learning_values(100).await.unwrap();
        assert_eq!(values.len(), 5);
    }

    // ============================================================
    // Ablation Experiment Tests
    // ============================================================

    #[tokio::test]
    async fn test_ablation_experiment_tracking() {
        let db = create_test_db();
        let store = CozoAttributionStore::new(db);

        let learning_id = Uuid::now_v7();

        // Initially none
        let exp = store.get_experiment(learning_id).await.unwrap();
        assert!(exp.is_none());

        // Create experiment
        let experiment = AblationExperiment {
            learning_id,
            started_at: Utc::now(),
            sessions_with: vec![SessionOutcome {
                session_id: SessionId::from("s1"),
                outcome: 0.8,
                timestamp: Utc::now(),
            }],
            sessions_without: vec![SessionOutcome {
                session_id: SessionId::from("s2"),
                outcome: 0.6,
                timestamp: Utc::now(),
            }],
            result: None,
        };
        store.update_experiment(&experiment).await.unwrap();

        // Read back
        let retrieved = store.get_experiment(learning_id).await.unwrap().unwrap();
        assert_eq!(retrieved.sessions_with.len(), 1);
        assert_eq!(retrieved.sessions_without.len(), 1);
        assert!(retrieved.result.is_none());

        // Update with result
        let completed = AblationExperiment {
            sessions_with: vec![
                SessionOutcome {
                    session_id: SessionId::from("s1"),
                    outcome: 0.8,
                    timestamp: Utc::now(),
                },
                SessionOutcome {
                    session_id: SessionId::from("s3"),
                    outcome: 0.75,
                    timestamp: Utc::now(),
                },
            ],
            result: Some(AblationResult {
                marginal_value: 0.15,
                confidence: 0.85,
                is_significant: true,
            }),
            ..retrieved
        };
        store.update_experiment(&completed).await.unwrap();

        let retrieved = store.get_experiment(learning_id).await.unwrap().unwrap();
        assert_eq!(retrieved.sessions_with.len(), 2);
        assert!(retrieved.result.is_some());
        assert!(retrieved.result.unwrap().is_significant);
    }
}
