//! Dashboard data handler
//!
//! Provides data for dashboard topics by querying the various stores.

use std::sync::Arc;

use crate::strategy::StrategyStore;
use crate::{
    AttributionStore, LearningId, LearningStore,
    dashboard::{
        AttributionData, DashboardData, DashboardTopic, HealthData, LearningDetailData,
        LearningsData, LearningsFilter, OverviewData, Period, SessionTimelineData,
        StrategyDistributionsData, StrategyOverridesData,
    },
};

use crate::attribution::LearningStatus;
use chrono::Utc;

/// Handler for dashboard data queries
pub struct DashboardHandler {
    learning_store: Arc<dyn LearningStore>,
    attribution_store: Arc<dyn AttributionStore>,
    _strategy_store: Arc<dyn StrategyStore>,
}

impl DashboardHandler {
    /// Create a new dashboard handler
    pub fn new(
        learning_store: Arc<dyn LearningStore>,
        attribution_store: Arc<dyn AttributionStore>,
        strategy_store: Arc<dyn StrategyStore>,
    ) -> Self {
        Self {
            learning_store,
            attribution_store,
            _strategy_store: strategy_store,
        }
    }

    /// Get data for a topic
    pub async fn get_data(&self, topic: &DashboardTopic) -> Result<DashboardData, String> {
        match topic {
            DashboardTopic::Overview => self.get_overview_data().await,
            DashboardTopic::Learnings { filters } => self.get_learnings_data(filters).await,
            DashboardTopic::LearningDetail { id } => self.get_learning_detail(id).await,
            DashboardTopic::Attribution { period } => self.get_attribution_data(period).await,
            DashboardTopic::SessionTimeline { period } => {
                self.get_session_timeline_data(period).await
            }
            DashboardTopic::StrategyDistributions => self.get_strategy_distributions_data().await,
            DashboardTopic::StrategyOverrides => self.get_strategy_overrides_data().await,
            DashboardTopic::Health => self.get_health_data().await,
        }
    }

    async fn get_overview_data(&self) -> Result<DashboardData, String> {
        Ok(DashboardData::Overview(OverviewData::default()))
    }

    async fn get_learnings_data(
        &self,
        _filters: &LearningsFilter,
    ) -> Result<DashboardData, String> {
        Ok(DashboardData::Learnings(LearningsData::default()))
    }

    async fn get_learning_detail(&self, id: &LearningId) -> Result<DashboardData, String> {
        let learning = self
            .learning_store
            .get(*id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Learning not found: {}", id))?;

        Ok(DashboardData::LearningDetail(LearningDetailData {
            id: learning.id,
            content: learning.content.description,
            category: learning.category,
            scope: learning.scope,
            status: crate::attribution::LearningStatus::Active,
            estimated_value: 0.0,
            confidence: learning.confidence,
            times_injected: 0,
            activation_rate: 0.0,
            session_count: 0,
            created_at: learning.created_at,
            source_session: None,
            extraction_method: "Unknown".to_string(),
        }))
    }

    async fn get_attribution_data(&self, _period: &Period) -> Result<DashboardData, String> {
        Ok(DashboardData::Attribution(AttributionData::default()))
    }

    async fn get_session_timeline_data(&self, _period: &Period) -> Result<DashboardData, String> {
        Ok(DashboardData::SessionTimeline(
            SessionTimelineData::default(),
        ))
    }

    async fn get_strategy_distributions_data(&self) -> Result<DashboardData, String> {
        Ok(DashboardData::StrategyDistributions(
            StrategyDistributionsData::default(),
        ))
    }

    async fn get_strategy_overrides_data(&self) -> Result<DashboardData, String> {
        Ok(DashboardData::StrategyOverrides(
            StrategyOverridesData::default(),
        ))
    }

    async fn get_health_data(&self) -> Result<DashboardData, String> {
        Ok(DashboardData::Health(HealthData::default()))
    }

    // ============================================================
    // Learning Actions
    // ============================================================

    /// Disable a learning (prevents injection)
    pub async fn handle_disable_learning(&self, id: LearningId) -> Result<(), String> {
        // Get or create learning value
        let mut value = self
            .attribution_store
            .get_learning_value(id)
            .await
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| crate::attribution::LearningValue {
                learning_id: id,
                estimated_value: 0.0,
                confidence: 0.0,
                session_count: 0,
                activation_rate: 0.0,
                temporal_value: 0.0,
                temporal_confidence: 0.0,
                ablation_value: None,
                ablation_confidence: None,
                status: LearningStatus::Active,
                updated_at: Utc::now(),
            });

        value.status = LearningStatus::Disabled;
        value.updated_at = Utc::now();

        self.attribution_store
            .update_learning_value(&value)
            .await
            .map_err(|e| e.to_string())
    }

    /// Enable a learning (allows injection)
    pub async fn handle_enable_learning(&self, id: LearningId) -> Result<(), String> {
        // Get or create learning value
        let mut value = self
            .attribution_store
            .get_learning_value(id)
            .await
            .map_err(|e| e.to_string())?
            .unwrap_or_else(|| crate::attribution::LearningValue {
                learning_id: id,
                estimated_value: 0.0,
                confidence: 0.0,
                session_count: 0,
                activation_rate: 0.0,
                temporal_value: 0.0,
                temporal_confidence: 0.0,
                ablation_value: None,
                ablation_confidence: None,
                status: LearningStatus::Disabled,
                updated_at: Utc::now(),
            });

        value.status = LearningStatus::Active;
        value.updated_at = Utc::now();

        self.attribution_store
            .update_learning_value(&value)
            .await
            .map_err(|e| e.to_string())
    }

    /// Delete a learning permanently
    pub async fn handle_delete_learning(&self, id: LearningId) -> Result<(), String> {
        self.learning_store
            .delete(id)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    use async_trait::async_trait;
    use tokio::sync::RwLock;

    use crate::{
        GrooveError, Learning, LearningCategory, LearningRelation, RelationType, Scope, UsageStats,
        assessment::SessionId,
        attribution::{AblationExperiment, AttributionRecord, LearningValue},
        strategy::{
            ContextType, InjectionStrategy, LearningStrategyOverride, StrategyDistribution,
            StrategyEvent,
        },
    };

    // ============================================================
    // Mock Stores for Testing
    // ============================================================

    struct MockLearningStore {
        learnings: RwLock<Vec<Learning>>,
    }

    impl MockLearningStore {
        fn new() -> Self {
            Self {
                learnings: RwLock::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl LearningStore for MockLearningStore {
        async fn store(&self, learning: &Learning) -> Result<crate::LearningId, GrooveError> {
            self.learnings.write().await.push(learning.clone());
            Ok(learning.id)
        }
        async fn get(&self, id: crate::LearningId) -> Result<Option<Learning>, GrooveError> {
            Ok(self
                .learnings
                .read()
                .await
                .iter()
                .find(|l| l.id == id)
                .cloned())
        }
        async fn find_by_scope(&self, scope: &Scope) -> Result<Vec<Learning>, GrooveError> {
            Ok(self
                .learnings
                .read()
                .await
                .iter()
                .filter(|l| &l.scope == scope)
                .cloned()
                .collect())
        }
        async fn find_by_category(
            &self,
            category: &LearningCategory,
        ) -> Result<Vec<Learning>, GrooveError> {
            Ok(self
                .learnings
                .read()
                .await
                .iter()
                .filter(|l| &l.category == category)
                .cloned()
                .collect())
        }
        async fn semantic_search(
            &self,
            _: &[f32],
            _: usize,
        ) -> Result<Vec<(Learning, f64)>, GrooveError> {
            Ok(Vec::new())
        }
        async fn update_usage(
            &self,
            _: crate::LearningId,
            _: &UsageStats,
        ) -> Result<(), GrooveError> {
            Ok(())
        }
        async fn find_related(
            &self,
            _: crate::LearningId,
            _: Option<&RelationType>,
        ) -> Result<Vec<Learning>, GrooveError> {
            Ok(Vec::new())
        }
        async fn store_relation(&self, _: &LearningRelation) -> Result<(), GrooveError> {
            Ok(())
        }
        async fn delete(&self, _: crate::LearningId) -> Result<bool, GrooveError> {
            Ok(true)
        }
        async fn count(&self) -> Result<u64, GrooveError> {
            Ok(self.learnings.read().await.len() as u64)
        }
        async fn update(&self, _: &Learning) -> Result<(), GrooveError> {
            Ok(())
        }
        async fn find_similar(
            &self,
            _: &[f32],
            _: f64,
            _: usize,
        ) -> Result<Vec<(Learning, f64)>, GrooveError> {
            Ok(Vec::new())
        }
        async fn find_for_injection(
            &self,
            _: &Scope,
            _: Option<&[f32]>,
            _: usize,
        ) -> Result<Vec<Learning>, GrooveError> {
            Ok(Vec::new())
        }
        async fn count_by_scope(&self, scope: &Scope) -> Result<u64, GrooveError> {
            Ok(self
                .learnings
                .read()
                .await
                .iter()
                .filter(|l| &l.scope == scope)
                .count() as u64)
        }
        async fn count_by_category(&self, category: &LearningCategory) -> Result<u64, GrooveError> {
            Ok(self
                .learnings
                .read()
                .await
                .iter()
                .filter(|l| &l.category == category)
                .count() as u64)
        }
    }

    struct MockAttributionStore {
        values: RwLock<Vec<LearningValue>>,
    }

    impl MockAttributionStore {
        fn new() -> Self {
            Self {
                values: RwLock::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl AttributionStore for MockAttributionStore {
        async fn store_attribution(&self, _: &AttributionRecord) -> crate::Result<()> {
            Ok(())
        }
        async fn get_attributions_for_learning(
            &self,
            _: crate::LearningId,
        ) -> crate::Result<Vec<AttributionRecord>> {
            Ok(Vec::new())
        }
        async fn get_attributions_for_session(
            &self,
            _: &SessionId,
        ) -> crate::Result<Vec<AttributionRecord>> {
            Ok(Vec::new())
        }
        async fn get_learning_value(
            &self,
            id: crate::LearningId,
        ) -> crate::Result<Option<LearningValue>> {
            Ok(self
                .values
                .read()
                .await
                .iter()
                .find(|v| v.learning_id == id)
                .cloned())
        }
        async fn update_learning_value(&self, value: &LearningValue) -> crate::Result<()> {
            self.values.write().await.push(value.clone());
            Ok(())
        }
        async fn list_learning_values(&self, limit: usize) -> crate::Result<Vec<LearningValue>> {
            Ok(self
                .values
                .read()
                .await
                .iter()
                .take(limit)
                .cloned()
                .collect())
        }
        async fn get_experiment(
            &self,
            _: crate::LearningId,
        ) -> crate::Result<Option<AblationExperiment>> {
            Ok(None)
        }
        async fn update_experiment(&self, _: &AblationExperiment) -> crate::Result<()> {
            Ok(())
        }
    }

    struct MockStrategyStore {
        distributions: RwLock<HashMap<(LearningCategory, ContextType), StrategyDistribution>>,
    }

    impl MockStrategyStore {
        fn new() -> Self {
            Self {
                distributions: RwLock::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl StrategyStore for MockStrategyStore {
        async fn load_distributions(
            &self,
        ) -> crate::Result<HashMap<(LearningCategory, ContextType), StrategyDistribution>> {
            Ok(self.distributions.read().await.clone())
        }
        async fn save_distributions(
            &self,
            distributions: &HashMap<(LearningCategory, ContextType), StrategyDistribution>,
        ) -> crate::Result<()> {
            *self.distributions.write().await = distributions.clone();
            Ok(())
        }
        async fn load_overrides(
            &self,
        ) -> crate::Result<HashMap<crate::LearningId, LearningStrategyOverride>> {
            Ok(HashMap::new())
        }
        async fn save_overrides(
            &self,
            _: &HashMap<crate::LearningId, LearningStrategyOverride>,
        ) -> crate::Result<()> {
            Ok(())
        }
        async fn store_strategy_event(&self, _: &StrategyEvent) -> crate::Result<()> {
            Ok(())
        }
        async fn get_strategy_history(
            &self,
            _: crate::LearningId,
            _: usize,
        ) -> crate::Result<Vec<StrategyEvent>> {
            Ok(Vec::new())
        }
        async fn cache_strategy(
            &self,
            _: SessionId,
            _: crate::LearningId,
            _: &InjectionStrategy,
        ) -> crate::Result<()> {
            Ok(())
        }
        async fn get_cached_strategy(
            &self,
            _: SessionId,
            _: crate::LearningId,
        ) -> crate::Result<Option<InjectionStrategy>> {
            Ok(None)
        }
        async fn clear_session_cache(&self, _: SessionId) -> crate::Result<()> {
            Ok(())
        }
    }

    fn create_test_handler() -> DashboardHandler {
        DashboardHandler::new(
            Arc::new(MockLearningStore::new()),
            Arc::new(MockAttributionStore::new()),
            Arc::new(MockStrategyStore::new()),
        )
    }

    // ============================================================
    // RED: Write failing tests first
    // ============================================================

    use crate::dashboard::{DashboardData, LearningsFilter};

    #[tokio::test]
    async fn get_overview_data_returns_overview() {
        let handler = create_test_handler();
        let result = handler.get_data(&DashboardTopic::Overview).await;

        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        match result.unwrap() {
            DashboardData::Overview(_) => {}
            other => panic!("Expected Overview data, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn get_learnings_data_returns_learnings() {
        let handler = create_test_handler();
        let result = handler
            .get_data(&DashboardTopic::Learnings {
                filters: LearningsFilter::default(),
            })
            .await;

        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        match result.unwrap() {
            DashboardData::Learnings(_) => {}
            other => panic!("Expected Learnings data, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn get_health_data_returns_health() {
        let handler = create_test_handler();
        let result = handler.get_data(&DashboardTopic::Health).await;

        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        match result.unwrap() {
            DashboardData::Health(_) => {}
            other => panic!("Expected Health data, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn get_strategy_distributions_returns_distributions() {
        let handler = create_test_handler();
        let result = handler
            .get_data(&DashboardTopic::StrategyDistributions)
            .await;

        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        match result.unwrap() {
            DashboardData::StrategyDistributions(_) => {}
            other => panic!("Expected StrategyDistributions data, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn get_attribution_data_returns_attribution() {
        use crate::dashboard::Period;
        let handler = create_test_handler();
        let result = handler
            .get_data(&DashboardTopic::Attribution {
                period: Period::default(),
            })
            .await;

        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        match result.unwrap() {
            DashboardData::Attribution(_) => {}
            other => panic!("Expected Attribution data, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn get_session_timeline_returns_timeline() {
        use crate::dashboard::Period;
        let handler = create_test_handler();
        let result = handler
            .get_data(&DashboardTopic::SessionTimeline {
                period: Period::default(),
            })
            .await;

        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        match result.unwrap() {
            DashboardData::SessionTimeline(_) => {}
            other => panic!("Expected SessionTimeline data, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn get_strategy_overrides_returns_overrides() {
        let handler = create_test_handler();
        let result = handler.get_data(&DashboardTopic::StrategyOverrides).await;

        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        match result.unwrap() {
            DashboardData::StrategyOverrides(_) => {}
            other => panic!("Expected StrategyOverrides data, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn get_learning_detail_returns_detail() {
        use uuid::Uuid;
        let handler = create_test_handler();
        let id = Uuid::now_v7();
        let result = handler
            .get_data(&DashboardTopic::LearningDetail { id })
            .await;

        // Should return error for non-existent learning
        assert!(
            result.is_err(),
            "Expected Err for non-existent learning, got {:?}",
            result
        );
    }

    // ============================================================
    // Learning Action Tests
    // ============================================================

    #[tokio::test]
    async fn handle_disable_learning_creates_value_if_not_exists() {
        use uuid::Uuid;
        let handler = create_test_handler();
        let id = Uuid::now_v7();

        let result = handler.handle_disable_learning(id).await;
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    }

    #[tokio::test]
    async fn handle_enable_learning_creates_value_if_not_exists() {
        use uuid::Uuid;
        let handler = create_test_handler();
        let id = Uuid::now_v7();

        let result = handler.handle_enable_learning(id).await;
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    }

    #[tokio::test]
    async fn handle_delete_learning_succeeds() {
        use uuid::Uuid;
        let handler = create_test_handler();
        let id = Uuid::now_v7();

        let result = handler.handle_delete_learning(id).await;
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    }
}
