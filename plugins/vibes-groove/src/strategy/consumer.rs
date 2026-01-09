//! Strategy consumer for orchestrating the strategy learning pipeline
//!
//! Subscribes to attribution events and updates strategy distributions based
//! on outcomes, integrating the outcome router and distribution updater.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use vibes_iggy::{EventConsumer, SeekPosition};

use crate::Result;
use crate::assessment::SessionId;
use crate::assessment::types::LightweightEvent;
use crate::attribution::AttributionRecord;
use crate::types::Learning;

use super::learner::SessionContext;
use super::router::{OutcomeRouter, OutcomeRouterConfig};
use super::store::StrategyStore;
use super::types::{InjectionStrategy, StrategyEvent, StrategyOutcome};
use super::updater::{DistributionUpdater, UpdaterConfig};

/// Default poll timeout for strategy consumer.
const DEFAULT_POLL_TIMEOUT: Duration = Duration::from_secs(1);

/// Default batch size for strategy consumer.
const DEFAULT_BATCH_SIZE: usize = 10;

/// Configuration for the strategy consumer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConsumerConfig {
    /// Whether the consumer is enabled
    pub enabled: bool,
    /// Outcome router configuration
    #[serde(default)]
    pub outcome: OutcomeRouterConfig,
    /// Distribution updater configuration
    #[serde(default)]
    pub updater: UpdaterConfig,
    /// Consumer group name for Iggy
    #[serde(default = "default_group")]
    pub group: String,
    /// Maximum events per poll
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    /// Poll timeout in milliseconds
    #[serde(default = "default_poll_timeout_ms")]
    pub poll_timeout_ms: u64,
}

fn default_group() -> String {
    "strategy".to_string()
}

fn default_batch_size() -> usize {
    DEFAULT_BATCH_SIZE
}

fn default_poll_timeout_ms() -> u64 {
    DEFAULT_POLL_TIMEOUT.as_millis() as u64
}

impl Default for StrategyConsumerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            outcome: OutcomeRouterConfig::default(),
            updater: UpdaterConfig::default(),
            group: default_group(),
            batch_size: default_batch_size(),
            poll_timeout_ms: default_poll_timeout_ms(),
        }
    }
}

impl StrategyConsumerConfig {
    /// Get poll timeout as Duration
    pub fn poll_timeout(&self) -> Duration {
        Duration::from_millis(self.poll_timeout_ms)
    }
}

/// Hook for novelty detection (future extension point)
#[async_trait]
pub trait NoveltyHook: Send + Sync {
    /// Called when a strategy outcome is computed
    async fn on_strategy_outcome(
        &self,
        learning: &Learning,
        context: &SessionContext,
        strategy: &InjectionStrategy,
        outcome: &StrategyOutcome,
    ) -> Result<()>;
}

/// Input event for strategy processing
///
/// Contains attribution records and lightweight events for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyInput {
    /// Attribution records from the attribution pipeline
    pub attribution_records: Vec<AttributionRecord>,
    /// Lightweight events from direct signals
    pub lightweight_events: Vec<LightweightEvent>,
}

/// Trait for loading learnings by ID
#[async_trait]
pub trait LearningLoader: Send + Sync {
    /// Load a learning by ID
    async fn load(&self, id: uuid::Uuid) -> Result<Option<Learning>>;
}

/// Trait for getting session context
#[async_trait]
pub trait SessionContextProvider: Send + Sync {
    /// Get session context by session ID
    async fn get(&self, session_id: &SessionId) -> Result<Option<SessionContext>>;
}

/// Trait for getting the strategy used for a learning in a session
#[async_trait]
pub trait UsedStrategyProvider: Send + Sync {
    /// Get the strategy that was used for a learning in a session
    async fn get(
        &self,
        session_id: &SessionId,
        learning_id: uuid::Uuid,
    ) -> Result<Option<InjectionStrategy>>;
}

/// Result of processing a strategy input
#[derive(Debug)]
pub struct StrategyConsumerResult {
    /// Number of attribution records processed
    pub records_processed: usize,
    /// Number of outcomes computed (non-None)
    pub outcomes_computed: usize,
    /// Number of distributions updated
    pub distributions_updated: usize,
}

/// Strategy consumer that orchestrates the strategy learning pipeline
pub struct StrategyConsumer<L, C, U>
where
    L: LearningLoader,
    C: SessionContextProvider,
    U: UsedStrategyProvider,
{
    /// Outcome router for combining signals
    outcome_router: OutcomeRouter,
    /// Distribution updater for updating Beta posteriors
    distribution_updater: DistributionUpdater,
    /// Strategy store for persistence
    store: Arc<dyn StrategyStore>,
    /// Learning loader
    learning_loader: Arc<L>,
    /// Session context provider
    context_provider: Arc<C>,
    /// Used strategy provider
    strategy_provider: Arc<U>,
    /// Optional novelty hook
    novelty_hook: Option<Arc<dyn NoveltyHook>>,
    /// Configuration
    config: StrategyConsumerConfig,
}

impl<L, C, U> StrategyConsumer<L, C, U>
where
    L: LearningLoader,
    C: SessionContextProvider,
    U: UsedStrategyProvider,
{
    /// Create a new strategy consumer
    pub fn new(
        store: Arc<dyn StrategyStore>,
        learning_loader: Arc<L>,
        context_provider: Arc<C>,
        strategy_provider: Arc<U>,
        config: StrategyConsumerConfig,
    ) -> Self {
        Self {
            outcome_router: OutcomeRouter::new(config.outcome.clone()),
            distribution_updater: DistributionUpdater::new(config.updater.clone()),
            store,
            learning_loader,
            context_provider,
            strategy_provider,
            novelty_hook: None,
            config,
        }
    }

    /// Set the novelty hook
    pub fn with_novelty_hook(mut self, hook: Arc<dyn NoveltyHook>) -> Self {
        self.novelty_hook = Some(hook);
        self
    }

    /// Process a strategy input (batch of attribution records + lightweight events)
    pub async fn process(&self, input: &StrategyInput) -> Result<StrategyConsumerResult> {
        let mut records_processed = 0;
        let mut outcomes_computed = 0;
        let mut distributions_updated = 0;

        // Load current distributions and overrides
        let mut distributions = self.store.load_distributions().await?;
        let mut overrides = self.store.load_overrides().await?;

        for record in &input.attribution_records {
            records_processed += 1;

            // Load learning
            let learning = match self.learning_loader.load(record.learning_id).await? {
                Some(l) => l,
                None => {
                    warn!(
                        learning_id = %record.learning_id,
                        "Learning not found, skipping"
                    );
                    continue;
                }
            };

            // Get session context
            let context = match self.context_provider.get(&record.session_id).await? {
                Some(c) => c,
                None => {
                    warn!(
                        session_id = %record.session_id,
                        "Session context not found, skipping"
                    );
                    continue;
                }
            };

            // Get the strategy that was used
            let strategy = match self
                .strategy_provider
                .get(&record.session_id, record.learning_id)
                .await?
            {
                Some(s) => s,
                None => {
                    debug!(
                        session_id = %record.session_id,
                        learning_id = %record.learning_id,
                        "No strategy found for session/learning, skipping"
                    );
                    continue;
                }
            };

            // Compute outcome from attribution + direct signals
            let outcome = self.outcome_router.compute_outcome(
                Some(record),
                &input.lightweight_events,
                &strategy,
            );

            if let Some(ref outcome) = outcome {
                outcomes_computed += 1;

                // Update distributions
                self.distribution_updater.update(
                    &mut distributions,
                    &mut overrides,
                    &learning,
                    &context,
                    &strategy,
                    outcome,
                );
                distributions_updated += 1;

                // Store strategy event
                let event = StrategyEvent::new(
                    learning.id,
                    record.session_id.clone(),
                    strategy.clone(),
                    outcome.clone(),
                );
                self.store.store_strategy_event(&event).await?;

                // Invoke novelty hook if present
                if let Some(ref hook) = self.novelty_hook
                    && let Err(e) = hook
                        .on_strategy_outcome(&learning, &context, &strategy, outcome)
                        .await
                {
                    warn!(error = %e, "Novelty hook failed");
                }
            }
        }

        // Persist updated distributions
        self.store.save_distributions(&distributions).await?;
        self.store.save_overrides(&overrides).await?;

        Ok(StrategyConsumerResult {
            records_processed,
            outcomes_computed,
            distributions_updated,
        })
    }

    /// Get the configuration
    pub fn config(&self) -> &StrategyConsumerConfig {
        &self.config
    }

    /// Check if the consumer is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

// =============================================================================
// Consumer loop and startup
// =============================================================================

/// Result of running the strategy consumer loop.
#[derive(Debug)]
pub enum ConsumerLoopResult {
    /// Consumer stopped due to shutdown signal.
    Shutdown,
    /// Consumer stopped due to an error.
    Error(String),
}

/// Error type for starting the strategy consumer.
#[derive(Debug, thiserror::Error)]
pub enum StartConsumerError {
    /// Failed to create a consumer from the event log.
    #[error("Failed to create consumer: {0}")]
    ConsumerCreation(String),
}

/// Start the strategy consumer.
///
/// This function creates a consumer for attribution events and spawns
/// a task that processes them through the strategy learning pipeline.
///
/// # Arguments
///
/// * `event_log` - The event log to consume StrategyInput events from
/// * `processor` - The strategy consumer processor
/// * `shutdown` - Cancellation token for graceful shutdown
///
/// # Returns
///
/// Returns a `JoinHandle` that can be awaited to wait for the consumer to stop.
pub async fn start_strategy_consumer<L, C, U, EL>(
    event_log: Arc<EL>,
    processor: Arc<StrategyConsumer<L, C, U>>,
    shutdown: CancellationToken,
) -> std::result::Result<JoinHandle<ConsumerLoopResult>, StartConsumerError>
where
    L: LearningLoader + 'static,
    C: SessionContextProvider + 'static,
    U: UsedStrategyProvider + 'static,
    EL: vibes_iggy::EventLog<StrategyInput> + 'static,
{
    let group = &processor.config.group;

    // Create consumer from event log
    let consumer = event_log
        .consumer(group)
        .await
        .map_err(|e| StartConsumerError::ConsumerCreation(e.to_string()))?;

    info!(group = %group, "Starting strategy consumer");

    // Spawn the consumer loop
    let handle =
        tokio::spawn(async move { strategy_consumer_loop(consumer, processor, shutdown).await });

    Ok(handle)
}

/// Run the strategy consumer loop.
///
/// This function processes StrategyInput events from the event log and runs the
/// strategy learning pipeline on each event. It runs until the shutdown token is
/// cancelled or an error occurs.
///
/// # Arguments
///
/// * `consumer` - The event consumer to poll from.
/// * `processor` - The strategy consumer processor.
/// * `shutdown` - Cancellation token to signal shutdown.
///
/// # Returns
///
/// Returns `ConsumerLoopResult::Shutdown` on graceful shutdown, or
/// `ConsumerLoopResult::Error` if an unrecoverable error occurred.
pub async fn strategy_consumer_loop<L, C, U>(
    mut consumer: Box<dyn EventConsumer<StrategyInput>>,
    processor: Arc<StrategyConsumer<L, C, U>>,
    shutdown: CancellationToken,
) -> ConsumerLoopResult
where
    L: LearningLoader + 'static,
    C: SessionContextProvider + 'static,
    U: UsedStrategyProvider + 'static,
{
    let config = &processor.config;
    info!(group = %config.group, "Strategy consumer loop starting");

    // Seek to beginning to process all events (can resume from committed offset)
    if let Err(e) = consumer.seek(SeekPosition::Beginning).await {
        error!(error = %e, "Failed to seek to start position");
        return ConsumerLoopResult::Error(format!("Seek failed: {e}"));
    }

    loop {
        tokio::select! {
            biased;

            _ = shutdown.cancelled() => {
                info!(group = %config.group, "Strategy consumer received shutdown signal");
                return ConsumerLoopResult::Shutdown;
            }

            poll_result = consumer.poll(config.batch_size, config.poll_timeout()) => {
                match poll_result {
                    Ok(batch) if batch.is_empty() => {
                        // No events, continue polling
                        continue;
                    }
                    Ok(batch) => {
                        debug!(
                            group = %config.group,
                            count = batch.len(),
                            "Processing strategy batch"
                        );

                        let mut last_offset = None;
                        for (offset, input) in batch {
                            match processor.process(&input).await {
                                Ok(result) => {
                                    debug!(
                                        records = result.records_processed,
                                        outcomes = result.outcomes_computed,
                                        distributions = result.distributions_updated,
                                        "Processed strategy input"
                                    );
                                }
                                Err(e) => {
                                    warn!(
                                        offset = offset,
                                        error = %e,
                                        "Failed to process strategy input"
                                    );
                                    // Continue processing other events
                                }
                            }
                            last_offset = Some(offset);
                        }

                        // Commit after processing batch
                        if let Some(offset) = last_offset
                            && let Err(e) = consumer.commit(offset).await
                        {
                            warn!(error = %e, "Failed to commit offset");
                        }
                    }
                    Err(e) => {
                        error!(error = %e, "Poll error");
                        return ConsumerLoopResult::Error(format!("Poll failed: {e}"));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assessment::types::{AssessmentContext, LightweightSignal};
    use crate::assessment::{EventId, HarnessType, InjectionMethod};
    use crate::strategy::types::{ContextPosition, ContextType, InjectionFormat, StrategyVariant};
    use crate::types::{LearningCategory, LearningContent, LearningSource, Scope};
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uuid::Uuid;

    // Mock implementations for testing

    struct MockLearningLoader {
        learnings: Mutex<HashMap<Uuid, Learning>>,
    }

    impl MockLearningLoader {
        fn new() -> Self {
            Self {
                learnings: Mutex::new(HashMap::new()),
            }
        }

        fn add(&self, learning: Learning) {
            self.learnings.lock().unwrap().insert(learning.id, learning);
        }
    }

    #[async_trait]
    impl LearningLoader for MockLearningLoader {
        async fn load(&self, id: Uuid) -> Result<Option<Learning>> {
            Ok(self.learnings.lock().unwrap().get(&id).cloned())
        }
    }

    struct MockContextProvider {
        contexts: Mutex<HashMap<String, SessionContext>>,
    }

    impl MockContextProvider {
        fn new() -> Self {
            Self {
                contexts: Mutex::new(HashMap::new()),
            }
        }

        fn add(&self, session_id: &str, context: SessionContext) {
            self.contexts
                .lock()
                .unwrap()
                .insert(session_id.to_string(), context);
        }
    }

    #[async_trait]
    impl SessionContextProvider for MockContextProvider {
        async fn get(&self, session_id: &SessionId) -> Result<Option<SessionContext>> {
            Ok(self
                .contexts
                .lock()
                .unwrap()
                .get(&session_id.to_string())
                .cloned())
        }
    }

    struct MockStrategyProvider {
        strategies: Mutex<HashMap<(String, Uuid), InjectionStrategy>>,
    }

    impl MockStrategyProvider {
        fn new() -> Self {
            Self {
                strategies: Mutex::new(HashMap::new()),
            }
        }

        fn add(&self, session_id: &str, learning_id: Uuid, strategy: InjectionStrategy) {
            self.strategies
                .lock()
                .unwrap()
                .insert((session_id.to_string(), learning_id), strategy);
        }
    }

    #[async_trait]
    impl UsedStrategyProvider for MockStrategyProvider {
        async fn get(
            &self,
            session_id: &SessionId,
            learning_id: Uuid,
        ) -> Result<Option<InjectionStrategy>> {
            Ok(self
                .strategies
                .lock()
                .unwrap()
                .get(&(session_id.to_string(), learning_id))
                .cloned())
        }
    }

    struct MockStrategyStore {
        distributions: Mutex<
            HashMap<(LearningCategory, ContextType), crate::strategy::types::StrategyDistribution>,
        >,
        overrides: Mutex<HashMap<Uuid, crate::strategy::types::LearningStrategyOverride>>,
        events: Mutex<Vec<StrategyEvent>>,
    }

    impl MockStrategyStore {
        fn new() -> Self {
            Self {
                distributions: Mutex::new(HashMap::new()),
                overrides: Mutex::new(HashMap::new()),
                events: Mutex::new(Vec::new()),
            }
        }

        fn with_distribution(self, category: LearningCategory, context_type: ContextType) -> Self {
            self.distributions.lock().unwrap().insert(
                (category.clone(), context_type),
                crate::strategy::types::StrategyDistribution::new(category, context_type),
            );
            self
        }

        fn events(&self) -> Vec<StrategyEvent> {
            self.events.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl StrategyStore for MockStrategyStore {
        async fn load_distributions(
            &self,
        ) -> Result<
            HashMap<(LearningCategory, ContextType), crate::strategy::types::StrategyDistribution>,
        > {
            Ok(self.distributions.lock().unwrap().clone())
        }

        async fn save_distributions(
            &self,
            distributions: &HashMap<
                (LearningCategory, ContextType),
                crate::strategy::types::StrategyDistribution,
            >,
        ) -> Result<()> {
            *self.distributions.lock().unwrap() = distributions.clone();
            Ok(())
        }

        async fn load_overrides(
            &self,
        ) -> Result<HashMap<Uuid, crate::strategy::types::LearningStrategyOverride>> {
            Ok(self.overrides.lock().unwrap().clone())
        }

        async fn save_overrides(
            &self,
            overrides: &HashMap<Uuid, crate::strategy::types::LearningStrategyOverride>,
        ) -> Result<()> {
            *self.overrides.lock().unwrap() = overrides.clone();
            Ok(())
        }

        async fn store_strategy_event(&self, event: &StrategyEvent) -> Result<()> {
            self.events.lock().unwrap().push(event.clone());
            Ok(())
        }

        async fn get_strategy_history(
            &self,
            learning_id: Uuid,
            limit: usize,
        ) -> Result<Vec<StrategyEvent>> {
            Ok(self
                .events
                .lock()
                .unwrap()
                .iter()
                .filter(|e| e.learning_id == learning_id)
                .take(limit)
                .cloned()
                .collect())
        }

        async fn cache_strategy(
            &self,
            _session_id: SessionId,
            _learning_id: Uuid,
            _strategy: &InjectionStrategy,
        ) -> Result<()> {
            Ok(())
        }

        async fn get_cached_strategy(
            &self,
            _session_id: SessionId,
            _learning_id: Uuid,
        ) -> Result<Option<InjectionStrategy>> {
            Ok(None)
        }

        async fn clear_session_cache(&self, _session_id: SessionId) -> Result<()> {
            Ok(())
        }
    }

    struct MockNoveltyHook {
        calls: Mutex<Vec<(Uuid, StrategyVariant)>>,
    }

    impl MockNoveltyHook {
        fn new() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
            }
        }

        fn call_count(&self) -> usize {
            self.calls.lock().unwrap().len()
        }
    }

    #[async_trait]
    impl NoveltyHook for MockNoveltyHook {
        async fn on_strategy_outcome(
            &self,
            learning: &Learning,
            _context: &SessionContext,
            strategy: &InjectionStrategy,
            _outcome: &StrategyOutcome,
        ) -> Result<()> {
            self.calls
                .lock()
                .unwrap()
                .push((learning.id, strategy.variant()));
            Ok(())
        }
    }

    fn test_learning(id: Uuid) -> Learning {
        Learning {
            id,
            scope: Scope::Project("test-project".into()),
            category: LearningCategory::CodePattern,
            content: LearningContent {
                description: "Test learning".into(),
                pattern: None,
                insight: "Test insight".into(),
            },
            confidence: 0.8,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            source: LearningSource::UserCreated,
        }
    }

    fn test_attribution(learning_id: Uuid, session_id: &str) -> AttributionRecord {
        AttributionRecord {
            learning_id,
            session_id: SessionId::from(session_id),
            timestamp: Utc::now(),
            was_activated: true,
            activation_confidence: 0.9,
            activation_signals: vec![],
            temporal_positive: 0.5,
            temporal_negative: 0.1,
            net_temporal: 0.4,
            was_withheld: false,
            session_outcome: 0.7,
            attributed_value: 0.7,
        }
    }

    fn test_context(session_id: &str) -> SessionContext {
        SessionContext::new(SessionId::from(session_id), ContextType::Interactive)
    }

    fn test_assessment_context(session_id: &str) -> AssessmentContext {
        AssessmentContext {
            session_id: SessionId::from(session_id),
            event_id: EventId::new(),
            timestamp: Utc::now(),
            active_learnings: vec![],
            injection_method: InjectionMethod::ClaudeMd,
            injection_scope: None,
            harness_type: HarnessType::ClaudeCode,
            harness_version: None,
            project_id: None,
            user_id: None,
        }
    }

    fn test_strategy() -> InjectionStrategy {
        InjectionStrategy::MainContext {
            position: ContextPosition::Prefix,
            format: InjectionFormat::Plain,
        }
    }

    fn test_lightweight_event(session_id: &str) -> LightweightEvent {
        LightweightEvent {
            context: test_assessment_context(session_id),
            message_idx: 0,
            signals: vec![LightweightSignal::Positive {
                pattern: "thanks".into(),
                confidence: 0.8,
            }],
            frustration_ema: 0.1,
            success_ema: 0.9,
            triggering_event_id: Uuid::now_v7(),
        }
    }

    #[tokio::test]
    async fn test_process_attribution_event_updates_distributions() {
        let learning_id = Uuid::now_v7();
        let session_id = "test-session";

        let learning_loader = Arc::new(MockLearningLoader::new());
        learning_loader.add(test_learning(learning_id));

        let context_provider = Arc::new(MockContextProvider::new());
        context_provider.add(session_id, test_context(session_id));

        let strategy_provider = Arc::new(MockStrategyProvider::new());
        strategy_provider.add(session_id, learning_id, test_strategy());

        let store = Arc::new(
            MockStrategyStore::new()
                .with_distribution(LearningCategory::CodePattern, ContextType::Interactive),
        );

        let consumer = StrategyConsumer::new(
            store.clone(),
            learning_loader,
            context_provider,
            strategy_provider,
            StrategyConsumerConfig::default(),
        );

        let input = StrategyInput {
            attribution_records: vec![test_attribution(learning_id, session_id)],
            lightweight_events: vec![test_lightweight_event(session_id)],
        };

        let result = consumer.process(&input).await.unwrap();

        assert_eq!(result.records_processed, 1);
        assert_eq!(result.outcomes_computed, 1);
        assert_eq!(result.distributions_updated, 1);
    }

    #[tokio::test]
    async fn test_process_stores_strategy_event() {
        let learning_id = Uuid::now_v7();
        let session_id = "test-session";

        let learning_loader = Arc::new(MockLearningLoader::new());
        learning_loader.add(test_learning(learning_id));

        let context_provider = Arc::new(MockContextProvider::new());
        context_provider.add(session_id, test_context(session_id));

        let strategy_provider = Arc::new(MockStrategyProvider::new());
        strategy_provider.add(session_id, learning_id, test_strategy());

        let store = Arc::new(
            MockStrategyStore::new()
                .with_distribution(LearningCategory::CodePattern, ContextType::Interactive),
        );

        let consumer = StrategyConsumer::new(
            store.clone(),
            learning_loader,
            context_provider,
            strategy_provider,
            StrategyConsumerConfig::default(),
        );

        let input = StrategyInput {
            attribution_records: vec![test_attribution(learning_id, session_id)],
            lightweight_events: vec![],
        };

        consumer.process(&input).await.unwrap();

        let events = store.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].learning_id, learning_id);
        assert_eq!(events[0].strategy.variant(), StrategyVariant::MainContext);
    }

    #[tokio::test]
    async fn test_process_invokes_novelty_hook() {
        let learning_id = Uuid::now_v7();
        let session_id = "test-session";

        let learning_loader = Arc::new(MockLearningLoader::new());
        learning_loader.add(test_learning(learning_id));

        let context_provider = Arc::new(MockContextProvider::new());
        context_provider.add(session_id, test_context(session_id));

        let strategy_provider = Arc::new(MockStrategyProvider::new());
        strategy_provider.add(session_id, learning_id, test_strategy());

        let store = Arc::new(
            MockStrategyStore::new()
                .with_distribution(LearningCategory::CodePattern, ContextType::Interactive),
        );

        let hook = Arc::new(MockNoveltyHook::new());

        let consumer = StrategyConsumer::new(
            store,
            learning_loader,
            context_provider,
            strategy_provider,
            StrategyConsumerConfig::default(),
        )
        .with_novelty_hook(hook.clone());

        let input = StrategyInput {
            attribution_records: vec![test_attribution(learning_id, session_id)],
            lightweight_events: vec![],
        };

        consumer.process(&input).await.unwrap();

        assert_eq!(hook.call_count(), 1);
    }

    #[tokio::test]
    async fn test_process_skips_unknown_learning() {
        let learning_id = Uuid::now_v7();
        let session_id = "test-session";

        let learning_loader = Arc::new(MockLearningLoader::new());
        // NOT adding the learning

        let context_provider = Arc::new(MockContextProvider::new());
        context_provider.add(session_id, test_context(session_id));

        let strategy_provider = Arc::new(MockStrategyProvider::new());
        strategy_provider.add(session_id, learning_id, test_strategy());

        let store = Arc::new(MockStrategyStore::new());

        let consumer = StrategyConsumer::new(
            store.clone(),
            learning_loader,
            context_provider,
            strategy_provider,
            StrategyConsumerConfig::default(),
        );

        let input = StrategyInput {
            attribution_records: vec![test_attribution(learning_id, session_id)],
            lightweight_events: vec![],
        };

        let result = consumer.process(&input).await.unwrap();

        assert_eq!(result.records_processed, 1);
        assert_eq!(result.outcomes_computed, 0);
    }

    #[tokio::test]
    async fn test_process_skips_unknown_context() {
        let learning_id = Uuid::now_v7();
        let session_id = "test-session";

        let learning_loader = Arc::new(MockLearningLoader::new());
        learning_loader.add(test_learning(learning_id));

        let context_provider = Arc::new(MockContextProvider::new());
        // NOT adding the context

        let strategy_provider = Arc::new(MockStrategyProvider::new());
        strategy_provider.add(session_id, learning_id, test_strategy());

        let store = Arc::new(MockStrategyStore::new());

        let consumer = StrategyConsumer::new(
            store.clone(),
            learning_loader,
            context_provider,
            strategy_provider,
            StrategyConsumerConfig::default(),
        );

        let input = StrategyInput {
            attribution_records: vec![test_attribution(learning_id, session_id)],
            lightweight_events: vec![],
        };

        let result = consumer.process(&input).await.unwrap();

        assert_eq!(result.records_processed, 1);
        assert_eq!(result.outcomes_computed, 0);
    }

    #[tokio::test]
    async fn test_process_skips_missing_strategy() {
        let learning_id = Uuid::now_v7();
        let session_id = "test-session";

        let learning_loader = Arc::new(MockLearningLoader::new());
        learning_loader.add(test_learning(learning_id));

        let context_provider = Arc::new(MockContextProvider::new());
        context_provider.add(session_id, test_context(session_id));

        let strategy_provider = Arc::new(MockStrategyProvider::new());
        // NOT adding the strategy

        let store = Arc::new(MockStrategyStore::new());

        let consumer = StrategyConsumer::new(
            store.clone(),
            learning_loader,
            context_provider,
            strategy_provider,
            StrategyConsumerConfig::default(),
        );

        let input = StrategyInput {
            attribution_records: vec![test_attribution(learning_id, session_id)],
            lightweight_events: vec![],
        };

        let result = consumer.process(&input).await.unwrap();

        assert_eq!(result.records_processed, 1);
        assert_eq!(result.outcomes_computed, 0);
    }

    #[tokio::test]
    async fn test_process_multiple_records() {
        let learning_id1 = Uuid::now_v7();
        let learning_id2 = Uuid::now_v7();
        let session_id = "test-session";

        let learning_loader = Arc::new(MockLearningLoader::new());
        learning_loader.add(test_learning(learning_id1));
        learning_loader.add(test_learning(learning_id2));

        let context_provider = Arc::new(MockContextProvider::new());
        context_provider.add(session_id, test_context(session_id));

        let strategy_provider = Arc::new(MockStrategyProvider::new());
        strategy_provider.add(session_id, learning_id1, test_strategy());
        strategy_provider.add(session_id, learning_id2, test_strategy());

        let store = Arc::new(
            MockStrategyStore::new()
                .with_distribution(LearningCategory::CodePattern, ContextType::Interactive),
        );

        let consumer = StrategyConsumer::new(
            store.clone(),
            learning_loader,
            context_provider,
            strategy_provider,
            StrategyConsumerConfig::default(),
        );

        let input = StrategyInput {
            attribution_records: vec![
                test_attribution(learning_id1, session_id),
                test_attribution(learning_id2, session_id),
            ],
            lightweight_events: vec![],
        };

        let result = consumer.process(&input).await.unwrap();

        assert_eq!(result.records_processed, 2);
        assert_eq!(result.outcomes_computed, 2);
        assert_eq!(store.events().len(), 2);
    }

    #[tokio::test]
    async fn test_config_defaults() {
        let config = StrategyConsumerConfig::default();
        assert!(config.enabled);
        assert_eq!(config.group, "strategy");
        assert_eq!(config.batch_size, 10);
        assert_eq!(config.poll_timeout_ms, 1000);
        assert_eq!(config.poll_timeout(), std::time::Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_consumer_disabled() {
        let learning_loader = Arc::new(MockLearningLoader::new());
        let context_provider = Arc::new(MockContextProvider::new());
        let strategy_provider = Arc::new(MockStrategyProvider::new());
        let store = Arc::new(MockStrategyStore::new());

        let config = StrategyConsumerConfig {
            enabled: false,
            ..Default::default()
        };

        let consumer = StrategyConsumer::new(
            store,
            learning_loader,
            context_provider,
            strategy_provider,
            config,
        );

        assert!(!consumer.is_enabled());
    }
}
