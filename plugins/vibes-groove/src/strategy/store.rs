//! Strategy storage trait and CozoDB implementation

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::TimeZone;
use cozo::{DataValue, DbInstance, ScriptMutability};

use crate::assessment::SessionId;
use crate::error::{GrooveError, Result};
use crate::types::LearningId;

use super::types::{
    ContextType, InjectionStrategy, LearningCategory, LearningStrategyOverride,
    StrategyDistribution, StrategyEvent,
};

/// CozoDB schema for strategy tables
pub const STRATEGY_SCHEMA: &str = r#"
{
    :create strategy_distribution {
        category: String,
        context_type: String =>
        strategy_weights_json: String,
        strategy_params_json: String,
        session_count: Int,
        updated_at: Int
    }
}
{
    :create learning_strategy_override {
        learning_id: String =>
        base_category: String,
        specialized_weights_json: String?,
        specialization_threshold: Int,
        session_count: Int,
        updated_at: Int
    }
}
{
    :create strategy_event {
        event_id: String =>
        learning_id: String,
        session_id: String,
        strategy_variant: String,
        strategy_params_json: String,
        outcome_value: Float,
        outcome_confidence: Float,
        outcome_source: String,
        timestamp: Int
    }
}
{
    :create strategy_session_cache {
        session_id: String,
        learning_id: String =>
        strategy_json: String,
        selected_at: Int
    }
}
{
    ::index create strategy_distribution:by_category { category }
}
{
    ::index create learning_strategy_override:by_category { base_category }
}
{
    ::index create strategy_event:by_learning { learning_id }
}
{
    ::index create strategy_event:by_session { session_id }
}
{
    ::index create strategy_event:by_time { timestamp }
}
"#;

/// Storage interface for strategy data
#[async_trait]
pub trait StrategyStore: Send + Sync {
    // Distributions
    async fn load_distributions(
        &self,
    ) -> Result<HashMap<(LearningCategory, ContextType), StrategyDistribution>>;
    async fn save_distributions(
        &self,
        distributions: &HashMap<(LearningCategory, ContextType), StrategyDistribution>,
    ) -> Result<()>;

    // Learning overrides
    async fn load_overrides(&self) -> Result<HashMap<LearningId, LearningStrategyOverride>>;
    async fn save_overrides(
        &self,
        overrides: &HashMap<LearningId, LearningStrategyOverride>,
    ) -> Result<()>;

    // Strategy events
    async fn store_strategy_event(&self, event: &StrategyEvent) -> Result<()>;
    async fn get_strategy_history(
        &self,
        learning_id: LearningId,
        limit: usize,
    ) -> Result<Vec<StrategyEvent>>;

    // Session cache
    async fn cache_strategy(
        &self,
        session_id: SessionId,
        learning_id: LearningId,
        strategy: &InjectionStrategy,
    ) -> Result<()>;
    async fn get_cached_strategy(
        &self,
        session_id: SessionId,
        learning_id: LearningId,
    ) -> Result<Option<InjectionStrategy>>;
    async fn clear_session_cache(&self, session_id: SessionId) -> Result<()>;
}

/// CozoDB-backed strategy store
pub struct CozoStrategyStore {
    db: Arc<DbInstance>,
}

impl CozoStrategyStore {
    pub fn new(db: Arc<DbInstance>) -> Self {
        Self { db }
    }

    /// Initialize the strategy schema
    pub fn init_schema(db: &DbInstance) -> Result<()> {
        db.run_script(
            STRATEGY_SCHEMA,
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
        params: std::collections::BTreeMap<String, DataValue>,
    ) -> Result<cozo::NamedRows> {
        self.db
            .run_script(query, params, ScriptMutability::Immutable)
            .map_err(|e| GrooveError::Database(format!("Query failed: {e}")))
    }

    /// Run a mutation query
    async fn run_mutation(
        &self,
        query: &str,
        params: std::collections::BTreeMap<String, DataValue>,
    ) -> Result<cozo::NamedRows> {
        self.db
            .run_script(query, params, ScriptMutability::Mutable)
            .map_err(|e| GrooveError::Database(format!("Mutation failed: {e}")))
    }
}

#[async_trait]
impl StrategyStore for CozoStrategyStore {
    async fn load_distributions(
        &self,
    ) -> Result<HashMap<(LearningCategory, ContextType), StrategyDistribution>> {
        let query = r#"
            ?[category, context_type, strategy_weights_json, strategy_params_json,
              session_count, updated_at] :=
            *strategy_distribution{category, context_type, strategy_weights_json,
                                  strategy_params_json, session_count, updated_at}
        "#;

        let rows = self.run_query(query, Default::default()).await?;
        let mut distributions = HashMap::new();

        for row in &rows.rows {
            let dist = parse_distribution_row(row)?;
            let key = (dist.category.clone(), dist.context_type);
            distributions.insert(key, dist);
        }

        Ok(distributions)
    }

    async fn save_distributions(
        &self,
        distributions: &HashMap<(LearningCategory, ContextType), StrategyDistribution>,
    ) -> Result<()> {
        for dist in distributions.values() {
            let weights_json = serde_json::to_string(&dist.strategy_weights)
                .map_err(|e| GrooveError::Serialization(e.to_string()))?
                .replace('\'', "''");
            let params_json = serde_json::to_string(&dist.strategy_params)
                .map_err(|e| GrooveError::Serialization(e.to_string()))?
                .replace('\'', "''");

            let query = format!(
                r#"?[category, context_type, strategy_weights_json, strategy_params_json,
                   session_count, updated_at] <- [[
                    '{}', '{}', '{}', '{}', {}, {}
                ]]
                :put strategy_distribution {{
                    category, context_type =>
                    strategy_weights_json, strategy_params_json, session_count, updated_at
                }}"#,
                dist.category.as_str(),
                dist.context_type.as_str(),
                weights_json,
                params_json,
                dist.session_count,
                dist.updated_at.timestamp(),
            );

            self.run_mutation(&query, Default::default()).await?;
        }
        Ok(())
    }

    async fn load_overrides(&self) -> Result<HashMap<LearningId, LearningStrategyOverride>> {
        let query = r#"
            ?[learning_id, base_category, specialized_weights_json,
              specialization_threshold, session_count, updated_at] :=
            *learning_strategy_override{learning_id, base_category, specialized_weights_json,
                                       specialization_threshold, session_count, updated_at}
        "#;

        let rows = self.run_query(query, Default::default()).await?;
        let mut overrides = HashMap::new();

        for row in &rows.rows {
            let override_ = parse_override_row(row)?;
            overrides.insert(override_.learning_id, override_);
        }

        Ok(overrides)
    }

    async fn save_overrides(
        &self,
        overrides: &HashMap<LearningId, LearningStrategyOverride>,
    ) -> Result<()> {
        for override_ in overrides.values() {
            let specialized_json = match &override_.specialized_weights {
                Some(weights) => {
                    let json = serde_json::to_string(weights)
                        .map_err(|e| GrooveError::Serialization(e.to_string()))?
                        .replace('\'', "''");
                    format!("'{json}'")
                }
                None => "null".to_string(),
            };

            let query = format!(
                r#"?[learning_id, base_category, specialized_weights_json,
                   specialization_threshold, session_count, updated_at] <- [[
                    '{}', '{}', {}, {}, {}, {}
                ]]
                :put learning_strategy_override {{
                    learning_id =>
                    base_category, specialized_weights_json, specialization_threshold,
                    session_count, updated_at
                }}"#,
                override_.learning_id,
                override_.base_category.as_str(),
                specialized_json,
                override_.specialization_threshold,
                override_.session_count,
                override_.updated_at.timestamp(),
            );

            self.run_mutation(&query, Default::default()).await?;
        }
        Ok(())
    }

    async fn store_strategy_event(&self, event: &StrategyEvent) -> Result<()> {
        let params_json = serde_json::to_string(&event.strategy)
            .map_err(|e| GrooveError::Serialization(e.to_string()))?
            .replace('\'', "''");

        let query = format!(
            r#"?[event_id, learning_id, session_id, strategy_variant, strategy_params_json,
               outcome_value, outcome_confidence, outcome_source, timestamp] <- [[
                '{}', '{}', '{}', '{}', '{}',
                {}, {}, '{}', {}
            ]]
            :put strategy_event {{
                event_id =>
                learning_id, session_id, strategy_variant, strategy_params_json,
                outcome_value, outcome_confidence, outcome_source, timestamp
            }}"#,
            event.event_id,
            event.learning_id,
            event.session_id,
            event.strategy.variant().as_str(),
            params_json,
            event.outcome.value,
            event.outcome.confidence,
            event.outcome.source.as_str(),
            event.timestamp.timestamp(),
        );

        self.run_mutation(&query, Default::default()).await?;
        Ok(())
    }

    async fn get_strategy_history(
        &self,
        learning_id: LearningId,
        limit: usize,
    ) -> Result<Vec<StrategyEvent>> {
        let query = format!(
            r#"?[event_id, learning_id, session_id, strategy_variant, strategy_params_json,
               outcome_value, outcome_confidence, outcome_source, timestamp] :=
            *strategy_event{{event_id, learning_id, session_id, strategy_variant,
                           strategy_params_json, outcome_value, outcome_confidence,
                           outcome_source, timestamp}},
            learning_id = '{}'
            :order -timestamp
            :limit {}"#,
            learning_id, limit
        );

        let rows = self.run_query(&query, Default::default()).await?;
        rows.rows
            .iter()
            .map(|row| parse_event_row(row.as_slice()))
            .collect()
    }

    async fn cache_strategy(
        &self,
        session_id: SessionId,
        learning_id: LearningId,
        strategy: &InjectionStrategy,
    ) -> Result<()> {
        let strategy_json = serde_json::to_string(strategy)
            .map_err(|e| GrooveError::Serialization(e.to_string()))?
            .replace('\'', "''");

        let query = format!(
            r#"?[session_id, learning_id, strategy_json, selected_at] <- [[
                '{}', '{}', '{}', {}
            ]]
            :put strategy_session_cache {{
                session_id, learning_id =>
                strategy_json, selected_at
            }}"#,
            session_id,
            learning_id,
            strategy_json,
            chrono::Utc::now().timestamp(),
        );

        self.run_mutation(&query, Default::default()).await?;
        Ok(())
    }

    async fn get_cached_strategy(
        &self,
        session_id: SessionId,
        learning_id: LearningId,
    ) -> Result<Option<InjectionStrategy>> {
        let query = format!(
            r#"?[session_id, learning_id, strategy_json, selected_at] :=
            *strategy_session_cache{{session_id, learning_id, strategy_json, selected_at}},
            session_id = '{}',
            learning_id = '{}'"#,
            session_id, learning_id
        );

        let rows = self.run_query(&query, Default::default()).await?;
        if rows.rows.is_empty() {
            return Ok(None);
        }

        let strategy_json = rows.rows[0][2]
            .get_str()
            .ok_or_else(|| GrooveError::Database("Invalid strategy_json".into()))?;

        let strategy: InjectionStrategy = serde_json::from_str(strategy_json)
            .map_err(|e| GrooveError::Serialization(e.to_string()))?;

        Ok(Some(strategy))
    }

    async fn clear_session_cache(&self, session_id: SessionId) -> Result<()> {
        let query = format!(
            r#"?[session_id, learning_id] :=
            *strategy_session_cache{{session_id, learning_id}},
            session_id = '{}'
            :rm strategy_session_cache {{session_id, learning_id}}"#,
            session_id
        );

        self.run_mutation(&query, Default::default()).await?;
        Ok(())
    }
}

// Helper functions for parsing CozoDB rows

fn parse_distribution_row(row: &[DataValue]) -> Result<StrategyDistribution> {
    use std::str::FromStr;

    let category_str = row[0]
        .get_str()
        .ok_or_else(|| GrooveError::Database("Invalid category".into()))?;
    let context_type_str = row[1]
        .get_str()
        .ok_or_else(|| GrooveError::Database("Invalid context_type".into()))?;
    let weights_json = row[2]
        .get_str()
        .ok_or_else(|| GrooveError::Database("Invalid strategy_weights_json".into()))?;
    let params_json = row[3]
        .get_str()
        .ok_or_else(|| GrooveError::Database("Invalid strategy_params_json".into()))?;
    let session_count = row[4]
        .get_int()
        .ok_or_else(|| GrooveError::Database("Invalid session_count".into()))?;
    let updated_at = row[5]
        .get_int()
        .ok_or_else(|| GrooveError::Database("Invalid updated_at".into()))?;

    Ok(StrategyDistribution {
        category: LearningCategory::from_str(category_str)
            .map_err(|e| GrooveError::Database(format!("Invalid category: {e}")))?,
        context_type: ContextType::from_str(context_type_str)
            .map_err(|e| GrooveError::Database(format!("Invalid context_type: {e}")))?,
        strategy_weights: serde_json::from_str(weights_json)
            .map_err(|e| GrooveError::Serialization(e.to_string()))?,
        strategy_params: serde_json::from_str(params_json)
            .map_err(|e| GrooveError::Serialization(e.to_string()))?,
        session_count: session_count as u32,
        updated_at: chrono::Utc
            .timestamp_opt(updated_at, 0)
            .single()
            .ok_or_else(|| GrooveError::Database("Invalid timestamp".into()))?,
    })
}

fn parse_override_row(row: &[DataValue]) -> Result<LearningStrategyOverride> {
    use std::str::FromStr;

    let learning_id = row[0]
        .get_str()
        .ok_or_else(|| GrooveError::Database("Invalid learning_id".into()))?;
    let base_category_str = row[1]
        .get_str()
        .ok_or_else(|| GrooveError::Database("Invalid base_category".into()))?;
    let specialized_json = row[2].get_str();
    let specialization_threshold = row[3]
        .get_int()
        .ok_or_else(|| GrooveError::Database("Invalid specialization_threshold".into()))?;
    let session_count = row[4]
        .get_int()
        .ok_or_else(|| GrooveError::Database("Invalid session_count".into()))?;
    let updated_at = row[5]
        .get_int()
        .ok_or_else(|| GrooveError::Database("Invalid updated_at".into()))?;

    let specialized_weights = match specialized_json {
        Some(json) if !json.is_empty() => Some(
            serde_json::from_str(json).map_err(|e| GrooveError::Serialization(e.to_string()))?,
        ),
        _ => None,
    };

    Ok(LearningStrategyOverride {
        learning_id: learning_id
            .parse()
            .map_err(|e| GrooveError::Database(format!("Invalid UUID: {e}")))?,
        base_category: LearningCategory::from_str(base_category_str)
            .map_err(|e| GrooveError::Database(format!("Invalid category: {e}")))?,
        specialized_weights,
        specialization_threshold: specialization_threshold as u32,
        session_count: session_count as u32,
        updated_at: chrono::Utc
            .timestamp_opt(updated_at, 0)
            .single()
            .ok_or_else(|| GrooveError::Database("Invalid timestamp".into()))?,
    })
}

fn parse_event_row(row: &[DataValue]) -> Result<StrategyEvent> {
    use super::types::{OutcomeSource, StrategyOutcome};
    use crate::assessment::EventId;
    use std::str::FromStr;
    use uuid::Uuid;

    let event_id_str = row[0]
        .get_str()
        .ok_or_else(|| GrooveError::Database("Invalid event_id".into()))?;
    let learning_id = row[1]
        .get_str()
        .ok_or_else(|| GrooveError::Database("Invalid learning_id".into()))?;
    let session_id = row[2]
        .get_str()
        .ok_or_else(|| GrooveError::Database("Invalid session_id".into()))?;
    // strategy_variant at [3] - not used, we deserialize the full strategy
    let strategy_json = row[4]
        .get_str()
        .ok_or_else(|| GrooveError::Database("Invalid strategy_params_json".into()))?;
    let outcome_value = row[5]
        .get_float()
        .ok_or_else(|| GrooveError::Database("Invalid outcome_value".into()))?;
    let outcome_confidence = row[6]
        .get_float()
        .ok_or_else(|| GrooveError::Database("Invalid outcome_confidence".into()))?;
    let outcome_source_str = row[7]
        .get_str()
        .ok_or_else(|| GrooveError::Database("Invalid outcome_source".into()))?;
    let timestamp = row[8]
        .get_int()
        .ok_or_else(|| GrooveError::Database("Invalid timestamp".into()))?;

    // Parse event_id as UUID, then wrap in EventId
    let event_uuid: Uuid = event_id_str
        .parse()
        .map_err(|e| GrooveError::Database(format!("Invalid event UUID: {e}")))?;

    Ok(StrategyEvent {
        event_id: EventId::from(event_uuid),
        learning_id: learning_id
            .parse()
            .map_err(|e| GrooveError::Database(format!("Invalid learning UUID: {e}")))?,
        session_id: SessionId::from(session_id),
        strategy: serde_json::from_str(strategy_json)
            .map_err(|e| GrooveError::Serialization(e.to_string()))?,
        outcome: StrategyOutcome {
            value: outcome_value,
            confidence: outcome_confidence,
            source: OutcomeSource::from_str(outcome_source_str)
                .map_err(|e| GrooveError::Database(format!("Invalid outcome_source: {e}")))?,
        },
        timestamp: chrono::Utc
            .timestamp_opt(timestamp, 0)
            .single()
            .ok_or_else(|| GrooveError::Database("Invalid timestamp".into()))?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    use crate::strategy::types::{
        ContextPosition, InjectionFormat, OutcomeSource, StrategyOutcome, StrategyVariant,
    };

    fn create_test_db() -> Arc<DbInstance> {
        let db = DbInstance::new("mem", "", Default::default()).unwrap();
        CozoStrategyStore::init_schema(&db).unwrap();
        Arc::new(db)
    }

    // ============================================================
    // Distribution Tests
    // ============================================================

    #[tokio::test]
    async fn test_save_and_load_distributions() {
        let db = create_test_db();
        let store = CozoStrategyStore::new(db);

        // Create a distribution
        let mut distributions = HashMap::new();
        let category = LearningCategory::CodePattern;
        let context_type = ContextType::Interactive;
        let key = (category.clone(), context_type);
        distributions.insert(
            key.clone(),
            StrategyDistribution::new(category, context_type),
        );

        // Save
        store.save_distributions(&distributions).await.unwrap();

        // Load back
        let loaded = store.load_distributions().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert!(loaded.contains_key(&key));

        let dist = &loaded[&key];
        assert_eq!(dist.category, LearningCategory::CodePattern);
        assert_eq!(dist.context_type, ContextType::Interactive);
        assert_eq!(dist.strategy_weights.len(), 4); // All 4 variants
    }

    #[tokio::test]
    async fn test_distribution_roundtrip_preserves_weights() {
        let db = create_test_db();
        let store = CozoStrategyStore::new(db);

        let mut distributions = HashMap::new();
        let category = LearningCategory::Preference;
        let context_type = ContextType::CodeReview;
        let mut dist = StrategyDistribution::new(category.clone(), context_type);

        // Modify weights
        dist.strategy_weights
            .get_mut(&StrategyVariant::MainContext)
            .unwrap()
            .update(1.0, 0.8);
        dist.session_count = 5;

        let key = (category.clone(), context_type);
        distributions.insert(key.clone(), dist);
        store.save_distributions(&distributions).await.unwrap();

        let loaded = store.load_distributions().await.unwrap();
        let loaded_dist = &loaded[&key];
        assert_eq!(loaded_dist.session_count, 5);

        // Weights should be preserved
        let main_ctx_weight = loaded_dist
            .strategy_weights
            .get(&StrategyVariant::MainContext)
            .unwrap();
        assert!(main_ctx_weight.value > 0.3); // Should have increased from default
    }

    // ============================================================
    // Override Tests
    // ============================================================

    #[tokio::test]
    async fn test_save_and_load_overrides() {
        let db = create_test_db();
        let store = CozoStrategyStore::new(db);

        let learning_id = Uuid::now_v7();
        let mut overrides = HashMap::new();
        overrides.insert(
            learning_id,
            LearningStrategyOverride::new(learning_id, LearningCategory::Solution),
        );

        store.save_overrides(&overrides).await.unwrap();

        let loaded = store.load_overrides().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert!(loaded.contains_key(&learning_id));

        let override_ = &loaded[&learning_id];
        assert_eq!(override_.base_category, LearningCategory::Solution);
        assert!(override_.specialized_weights.is_none());
    }

    #[tokio::test]
    async fn test_override_with_specialization() {
        let db = create_test_db();
        let store = CozoStrategyStore::new(db);

        let learning_id = Uuid::now_v7();
        let mut override_ =
            LearningStrategyOverride::new(learning_id, LearningCategory::Correction);

        // Create specialization
        let dist =
            StrategyDistribution::new(LearningCategory::Correction, ContextType::Interactive);
        override_.specialize_from(&dist);
        override_.session_count = 25;

        let mut overrides = HashMap::new();
        overrides.insert(learning_id, override_);

        store.save_overrides(&overrides).await.unwrap();

        let loaded = store.load_overrides().await.unwrap();
        let loaded_override = &loaded[&learning_id];
        assert!(loaded_override.specialized_weights.is_some());
        assert_eq!(loaded_override.session_count, 25);
    }

    // ============================================================
    // Strategy Event Tests
    // ============================================================

    #[tokio::test]
    async fn test_store_and_retrieve_strategy_event() {
        let db = create_test_db();
        let store = CozoStrategyStore::new(db);

        let learning_id = Uuid::now_v7();
        let event = StrategyEvent::new(
            learning_id,
            SessionId::from("session-1"),
            InjectionStrategy::MainContext {
                position: ContextPosition::Prefix,
                format: InjectionFormat::Tagged,
            },
            StrategyOutcome::new(0.7, 0.9, OutcomeSource::Attribution),
        );

        store.store_strategy_event(&event).await.unwrap();

        let history = store.get_strategy_history(learning_id, 10).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].learning_id, learning_id);
        assert_eq!(history[0].strategy.variant(), StrategyVariant::MainContext);
    }

    #[tokio::test]
    async fn test_strategy_history_respects_limit() {
        let db = create_test_db();
        let store = CozoStrategyStore::new(db);

        let learning_id = Uuid::now_v7();

        // Store 5 events
        for i in 0..5 {
            let event = StrategyEvent::new(
                learning_id,
                SessionId::from(format!("session-{i}")),
                InjectionStrategy::Deferred {
                    trigger: crate::strategy::types::DeferralTrigger::Explicit,
                    max_wait_ms: None,
                },
                StrategyOutcome::new(0.5, 0.8, OutcomeSource::Direct),
            );
            store.store_strategy_event(&event).await.unwrap();
        }

        // Request only 3
        let history = store.get_strategy_history(learning_id, 3).await.unwrap();
        assert_eq!(history.len(), 3);
    }

    // ============================================================
    // Session Cache Tests
    // ============================================================

    #[tokio::test]
    async fn test_cache_and_retrieve_strategy() {
        let db = create_test_db();
        let store = CozoStrategyStore::new(db);

        let session_id = SessionId::from("cache-session");
        let learning_id = Uuid::now_v7();
        let strategy = InjectionStrategy::Subagent {
            agent_type: crate::strategy::types::SubagentType::Explorer,
            blocking: false,
            prompt_template: Some("Check {{topic}}".into()),
        };

        store
            .cache_strategy(session_id.clone(), learning_id, &strategy)
            .await
            .unwrap();

        let cached = store
            .get_cached_strategy(session_id.clone(), learning_id)
            .await
            .unwrap();
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().variant(), StrategyVariant::Subagent);
    }

    #[tokio::test]
    async fn test_cache_returns_none_for_unknown() {
        let db = create_test_db();
        let store = CozoStrategyStore::new(db);

        let cached = store
            .get_cached_strategy(SessionId::from("nonexistent"), Uuid::now_v7())
            .await
            .unwrap();
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_clear_session_cache() {
        let db = create_test_db();
        let store = CozoStrategyStore::new(db);

        let session_id = SessionId::from("clear-test");
        let learning1 = Uuid::now_v7();
        let learning2 = Uuid::now_v7();

        let strategy = InjectionStrategy::MainContext {
            position: ContextPosition::Suffix,
            format: InjectionFormat::Plain,
        };

        // Cache strategies for both learnings
        store
            .cache_strategy(session_id.clone(), learning1, &strategy)
            .await
            .unwrap();
        store
            .cache_strategy(session_id.clone(), learning2, &strategy)
            .await
            .unwrap();

        // Clear cache for this session
        store.clear_session_cache(session_id.clone()).await.unwrap();

        // Both should be gone
        assert!(
            store
                .get_cached_strategy(session_id.clone(), learning1)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .get_cached_strategy(session_id.clone(), learning2)
                .await
                .unwrap()
                .is_none()
        );
    }
}
