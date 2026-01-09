//! Attribution consumer for processing heavy assessment events
//!
//! This consumer subscribes to `groove.assessment.heavy` events and runs the
//! 4-layer attribution pipeline for each learning that was active during the session.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use vibes_iggy::{EventConsumer, SeekPosition};

use crate::assessment::{HeavyEvent, LightweightEvent, Outcome, SessionId};
use crate::capture::ParsedTranscript;
use crate::extraction::Embedder;
use crate::types::LearningId;
use crate::{GrooveError, Result};

use super::activation::ActivationDetector;
use super::aggregation::ValueAggregator;
use super::store::AttributionStore;
use super::temporal::TemporalCorrelator;
use super::types::{ActivationSignal, AttributionRecord, LearningStatus, LearningValue};
use super::{AblationConfig, AblationManager, AblationStrategy, ActivationConfig, TemporalConfig};

/// Default poll timeout for attribution consumer.
const DEFAULT_POLL_TIMEOUT: Duration = Duration::from_secs(1);

/// Default batch size for attribution consumer.
const DEFAULT_BATCH_SIZE: usize = 10;

/// Configuration for the attribution consumer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionConfig {
    /// Whether attribution is enabled
    pub enabled: bool,
    /// Activation detection settings
    #[serde(default)]
    pub activation: ActivationConfig,
    /// Temporal correlation settings
    #[serde(default)]
    pub temporal: TemporalConfig,
    /// Ablation testing settings
    #[serde(default)]
    pub ablation: AblationConfig,
    /// Consumer group name
    pub group: String,
    /// Maximum events per poll
    pub batch_size: usize,
    /// Poll timeout in milliseconds
    pub poll_timeout_ms: u64,
}

impl Default for AttributionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            activation: ActivationConfig::default(),
            temporal: TemporalConfig::default(),
            ablation: AblationConfig::default(),
            group: "attribution".to_string(),
            batch_size: DEFAULT_BATCH_SIZE,
            poll_timeout_ms: DEFAULT_POLL_TIMEOUT.as_millis() as u64,
        }
    }
}

impl AttributionConfig {
    /// Get poll timeout as Duration
    pub fn poll_timeout(&self) -> Duration {
        Duration::from_millis(self.poll_timeout_ms)
    }
}

/// Trait for fetching transcripts by session ID
#[async_trait]
pub trait TranscriptFetcher: Send + Sync {
    /// Fetch the parsed transcript for a session
    async fn fetch(&self, session_id: &SessionId) -> Result<Option<ParsedTranscript>>;
}

/// Trait for fetching lightweight events by session ID
#[async_trait]
pub trait LightweightEventFetcher: Send + Sync {
    /// Fetch lightweight events for a session
    async fn fetch(&self, session_id: &SessionId) -> Result<Vec<LightweightEvent>>;
}

/// Trait for loading learnings by ID
///
/// This is a simplified trait used by the attribution consumer.
/// It only requires the ability to load learnings by ID.
#[async_trait]
pub trait LearningLoader: Send + Sync {
    /// Load a learning by ID
    async fn load(&self, id: LearningId) -> Result<Option<crate::types::Learning>>;
}

/// Attribution consumer that processes heavy assessment events
pub struct AttributionConsumer<A, T, S, L, E, TF, LF>
where
    A: ActivationDetector,
    T: TemporalCorrelator,
    S: AblationStrategy,
    L: LearningLoader,
    E: Embedder,
    TF: TranscriptFetcher,
    LF: LightweightEventFetcher,
{
    /// Activation detector (Layer 1)
    activation_detector: Arc<A>,
    /// Temporal correlator (Layer 2)
    temporal_correlator: Arc<T>,
    /// Ablation manager (Layer 3) - full integration deferred to ablation experiment story
    #[allow(dead_code)]
    ablation_manager: AblationManager<S>,
    /// Value aggregator (Layer 4)
    value_aggregator: ValueAggregator,
    /// Attribution store for persisting records
    attribution_store: Arc<dyn AttributionStore>,
    /// Learning loader for loading learnings
    learning_loader: Arc<L>,
    /// Embedder for activation detection
    embedder: Arc<E>,
    /// Transcript fetcher
    transcript_fetcher: Arc<TF>,
    /// Lightweight event fetcher
    lightweight_fetcher: Arc<LF>,
    /// Configuration
    config: AttributionConfig,
}

impl<A, T, S, L, E, TF, LF> AttributionConsumer<A, T, S, L, E, TF, LF>
where
    A: ActivationDetector,
    T: TemporalCorrelator,
    S: AblationStrategy,
    L: LearningLoader,
    E: Embedder,
    TF: TranscriptFetcher,
    LF: LightweightEventFetcher,
{
    /// Create a new attribution consumer
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        activation_detector: Arc<A>,
        temporal_correlator: Arc<T>,
        ablation_manager: AblationManager<S>,
        value_aggregator: ValueAggregator,
        attribution_store: Arc<dyn AttributionStore>,
        learning_loader: Arc<L>,
        embedder: Arc<E>,
        transcript_fetcher: Arc<TF>,
        lightweight_fetcher: Arc<LF>,
        config: AttributionConfig,
    ) -> Self {
        Self {
            activation_detector,
            temporal_correlator,
            ablation_manager,
            value_aggregator,
            attribution_store,
            learning_loader,
            embedder,
            transcript_fetcher,
            lightweight_fetcher,
            config,
        }
    }

    /// Check if attribution is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Process a heavy assessment event
    ///
    /// Runs the full 4-layer attribution pipeline for each active learning.
    pub async fn process_heavy_event(&self, event: &HeavyEvent) -> Result<AttributionResult> {
        if !self.config.enabled {
            return Ok(AttributionResult::disabled());
        }

        let mut result = AttributionResult::new(&event.context.session_id);
        let session_id = &event.context.session_id;

        // Fetch transcript for activation detection
        let transcript = match self.transcript_fetcher.fetch(session_id).await? {
            Some(t) => t,
            None => {
                warn!(session_id = %session_id, "No transcript found for session");
                return Ok(result);
            }
        };

        // Fetch lightweight events for temporal correlation
        let lightweight_events = self.lightweight_fetcher.fetch(session_id).await?;

        // Convert lightweight events to (message_idx, signal) pairs
        let signals: Vec<(u32, crate::assessment::LightweightSignal)> = lightweight_events
            .iter()
            .flat_map(|e| e.signals.iter().map(move |s| (e.message_idx, s.clone())))
            .collect();

        // Convert outcome to numeric value
        let outcome_value = outcome_to_value(&event.outcome);

        // Process each active learning
        for learning_id in &event.context.active_learnings {
            match self
                .process_learning(
                    *learning_id,
                    event,
                    &transcript,
                    &signals,
                    outcome_value,
                    &mut result,
                )
                .await
            {
                Ok(()) => result.learnings_processed += 1,
                Err(e) => {
                    warn!(
                        learning_id = %learning_id,
                        error = %e,
                        "Failed to process learning attribution"
                    );
                    result.errors.push((*learning_id, e.to_string()));
                }
            }
        }

        Ok(result)
    }

    /// Process attribution for a single learning
    async fn process_learning(
        &self,
        learning_id: LearningId,
        event: &HeavyEvent,
        transcript: &ParsedTranscript,
        signals: &[(u32, crate::assessment::LightweightSignal)],
        outcome_value: f64,
        result: &mut AttributionResult,
    ) -> Result<()> {
        let session_id = &event.context.session_id;

        // 1. Load learning
        let learning = self
            .learning_loader
            .load(learning_id)
            .await?
            .ok_or(GrooveError::NotFound(learning_id))?;

        // 2. Run activation detection (Layer 1)
        let activation = self
            .activation_detector
            .detect(&learning, transcript, self.embedder.as_ref())
            .await?;

        if !activation.was_activated {
            debug!(
                learning_id = %learning_id,
                session_id = %session_id,
                "Learning was not activated in session"
            );
            // Still record the attribution with zero value
            let record = AttributionRecord {
                learning_id,
                session_id: session_id.clone(),
                timestamp: Utc::now(),
                was_activated: false,
                activation_confidence: activation.confidence,
                activation_signals: vec![],
                temporal_positive: 0.0,
                temporal_negative: 0.0,
                net_temporal: 0.0,
                was_withheld: false,
                session_outcome: outcome_value,
                attributed_value: 0.0,
            };
            self.attribution_store.store_attribution(&record).await?;
            return Ok(());
        }

        // 3. Run temporal correlation (Layer 2)
        let activation_points: Vec<u32> = activation
            .signals
            .iter()
            .map(|s| match s {
                ActivationSignal::EmbeddingSimilarity { message_idx, .. } => *message_idx,
                ActivationSignal::ExplicitReference { message_idx, .. } => *message_idx,
            })
            .collect();

        let temporal = self
            .temporal_correlator
            .correlate(&activation_points, signals);

        // 4. Check ablation status (Layer 3)
        // Note: The decision to withhold was made at injection time, stored in event
        // Here we just record the observation
        let was_withheld = false; // TODO: Track this in HeavyEvent if needed

        // Get or create learning value
        let current_value = self
            .attribution_store
            .get_learning_value(learning_id)
            .await?
            .unwrap_or_else(|| create_initial_learning_value(learning_id));

        // 5. Compute attributed value (Layer 4)
        // Use the complete update method that handles temporal, aggregation, and deprecation
        let activation_rate = if activation.was_activated { 1.0 } else { 0.0 };
        let learning_value =
            self.value_aggregator
                .update_learning_value(current_value, &temporal, activation_rate);

        // Check if learning was deprecated
        if matches!(learning_value.status, LearningStatus::Deprecated { .. }) {
            result.deprecated.push(learning_id);
        }

        // 6. Store attribution record
        let record = AttributionRecord {
            learning_id,
            session_id: session_id.clone(),
            timestamp: Utc::now(),
            was_activated: activation.was_activated,
            activation_confidence: activation.confidence,
            activation_signals: activation.signals.clone(),
            temporal_positive: temporal.positive_score,
            temporal_negative: temporal.negative_score,
            net_temporal: temporal.net_score,
            was_withheld,
            session_outcome: outcome_value,
            attributed_value: learning_value.estimated_value,
        };
        self.attribution_store.store_attribution(&record).await?;

        // 7. Update learning value
        self.attribution_store
            .update_learning_value(&learning_value)
            .await?;

        debug!(
            learning_id = %learning_id,
            session_id = %session_id,
            activated = activation.was_activated,
            temporal_net = temporal.net_score,
            value = learning_value.estimated_value,
            "Processed attribution"
        );

        Ok(())
    }
}

/// Result of processing a heavy event
#[derive(Debug, Clone, Default)]
pub struct AttributionResult {
    /// Session ID that was processed
    pub session_id: Option<SessionId>,
    /// Number of learnings processed
    pub learnings_processed: u32,
    /// Learnings that were deprecated due to negative value
    pub deprecated: Vec<LearningId>,
    /// Errors encountered (learning_id, error message)
    pub errors: Vec<(LearningId, String)>,
    /// Whether attribution was disabled
    pub disabled: bool,
}

impl AttributionResult {
    /// Create a new result for a session
    pub fn new(session_id: &SessionId) -> Self {
        Self {
            session_id: Some(session_id.clone()),
            ..Default::default()
        }
    }

    /// Create a disabled result
    pub fn disabled() -> Self {
        Self {
            disabled: true,
            ..Default::default()
        }
    }

    /// Check if processing was successful (no errors)
    pub fn is_success(&self) -> bool {
        self.errors.is_empty() && !self.disabled
    }
}

/// Convert session outcome to numeric value
fn outcome_to_value(outcome: &Outcome) -> f64 {
    match outcome {
        Outcome::Success => 1.0,
        Outcome::Partial => 0.5,
        Outcome::Failure => -0.5,
        Outcome::Abandoned => -1.0,
    }
}

/// Create initial learning value for a new learning
fn create_initial_learning_value(learning_id: LearningId) -> LearningValue {
    LearningValue {
        learning_id,
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
    }
}

// =============================================================================
// Consumer loop and startup
// =============================================================================

/// Result of running the attribution consumer loop.
#[derive(Debug)]
pub enum ConsumerLoopResult {
    /// Consumer stopped due to shutdown signal.
    Shutdown,
    /// Consumer stopped due to an error.
    Error(String),
}

/// Error type for starting the attribution consumer.
#[derive(Debug, thiserror::Error)]
pub enum StartConsumerError {
    /// Failed to create a consumer from the event log.
    #[error("Failed to create consumer: {0}")]
    ConsumerCreation(String),
}

/// Start the attribution consumer.
///
/// This function creates a consumer for heavy assessment events and spawns
/// a task that processes them through the 4-layer attribution pipeline.
///
/// # Arguments
///
/// * `event_log` - The event log to consume HeavyEvents from
/// * `processor` - The attribution consumer processor
/// * `shutdown` - Cancellation token for graceful shutdown
///
/// # Returns
///
/// Returns a `JoinHandle` that can be awaited to wait for the consumer to stop.
pub async fn start_attribution_consumer<A, T, S, L, E, TF, LF, EL>(
    event_log: Arc<EL>,
    processor: Arc<AttributionConsumer<A, T, S, L, E, TF, LF>>,
    shutdown: CancellationToken,
) -> std::result::Result<JoinHandle<ConsumerLoopResult>, StartConsumerError>
where
    A: ActivationDetector + 'static,
    T: TemporalCorrelator + 'static,
    S: AblationStrategy + 'static,
    L: LearningLoader + 'static,
    E: Embedder + 'static,
    TF: TranscriptFetcher + 'static,
    LF: LightweightEventFetcher + 'static,
    EL: vibes_iggy::EventLog<HeavyEvent> + 'static,
{
    let group = &processor.config.group;

    // Create consumer from event log
    let consumer = event_log
        .consumer(group)
        .await
        .map_err(|e| StartConsumerError::ConsumerCreation(e.to_string()))?;

    info!(group = %group, "Starting attribution consumer");

    // Spawn the consumer loop
    let handle =
        tokio::spawn(async move { attribution_consumer_loop(consumer, processor, shutdown).await });

    Ok(handle)
}

/// Run the attribution consumer loop.
///
/// This function processes HeavyEvents from the assessment log and runs the
/// attribution pipeline on each event. It runs until the shutdown token is
/// cancelled or an error occurs.
///
/// # Arguments
///
/// * `consumer` - The event consumer to poll from.
/// * `processor` - The attribution consumer processor.
/// * `shutdown` - Cancellation token to signal shutdown.
///
/// # Returns
///
/// Returns `ConsumerLoopResult::Shutdown` on graceful shutdown, or
/// `ConsumerLoopResult::Error` if an unrecoverable error occurred.
pub async fn attribution_consumer_loop<A, T, S, L, E, TF, LF>(
    mut consumer: Box<dyn EventConsumer<HeavyEvent>>,
    processor: Arc<AttributionConsumer<A, T, S, L, E, TF, LF>>,
    shutdown: CancellationToken,
) -> ConsumerLoopResult
where
    A: ActivationDetector + 'static,
    T: TemporalCorrelator + 'static,
    S: AblationStrategy + 'static,
    L: LearningLoader + 'static,
    E: Embedder + 'static,
    TF: TranscriptFetcher + 'static,
    LF: LightweightEventFetcher + 'static,
{
    let config = &processor.config;
    info!(group = %config.group, "Attribution consumer loop starting");

    // Seek to beginning to process all events (can resume from committed offset)
    if let Err(e) = consumer.seek(SeekPosition::Beginning).await {
        error!(error = %e, "Failed to seek to start position");
        return ConsumerLoopResult::Error(format!("Seek failed: {e}"));
    }

    loop {
        tokio::select! {
            biased;

            _ = shutdown.cancelled() => {
                info!(group = %config.group, "Attribution consumer received shutdown signal");
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
                            "Processing attribution batch"
                        );

                        let mut last_offset = None;
                        for (offset, event) in batch {
                            match processor.process_heavy_event(&event).await {
                                Ok(result) => {
                                    debug!(
                                        session_id = ?result.session_id,
                                        learnings = result.learnings_processed,
                                        deprecated = result.deprecated.len(),
                                        errors = result.errors.len(),
                                        "Processed attribution event"
                                    );
                                }
                                Err(e) => {
                                    warn!(
                                        offset = offset,
                                        error = %e,
                                        "Failed to process attribution event"
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
    use crate::ActivationResult;
    use crate::assessment::{AssessmentContext, LightweightSignal};
    use crate::attribution::temporal::ExponentialDecayCorrelator;
    use crate::attribution::{AblationExperiment, ConservativeAblation};
    use crate::capture::{TranscriptMessage, TranscriptMetadata};
    use crate::extraction::embedder::EmbedderResult;
    use crate::types::{Learning, LearningCategory, LearningContent, LearningSource, Scope};
    use std::collections::HashMap;
    use std::sync::Mutex;
    use uuid::Uuid;

    // ==========================================================================
    // Mock implementations for testing
    // ==========================================================================

    struct MockActivationDetector {
        /// Map of learning_id -> activation result
        results: Mutex<HashMap<LearningId, ActivationResult>>,
    }

    impl MockActivationDetector {
        fn new() -> Self {
            Self {
                results: Mutex::new(HashMap::new()),
            }
        }

        fn set_result(&self, learning_id: LearningId, result: ActivationResult) {
            self.results.lock().unwrap().insert(learning_id, result);
        }
    }

    #[async_trait]
    impl ActivationDetector for MockActivationDetector {
        async fn detect(
            &self,
            learning: &Learning,
            _transcript: &ParsedTranscript,
            _embedder: &dyn Embedder,
        ) -> Result<ActivationResult> {
            let results = self.results.lock().unwrap();
            Ok(results
                .get(&learning.id)
                .cloned()
                .unwrap_or(ActivationResult {
                    was_activated: false,
                    confidence: 0.0,
                    signals: vec![],
                }))
        }
    }

    struct MockEmbedder;

    #[async_trait]
    impl Embedder for MockEmbedder {
        async fn embed(&self, _text: &str) -> EmbedderResult<Vec<f32>> {
            Ok(vec![0.0; 384])
        }

        fn dimensions(&self) -> usize {
            384
        }
    }

    struct MockLearningLoader {
        learnings: Mutex<HashMap<LearningId, Learning>>,
    }

    impl MockLearningLoader {
        fn new() -> Self {
            Self {
                learnings: Mutex::new(HashMap::new()),
            }
        }

        fn add_learning(&self, learning: Learning) {
            self.learnings.lock().unwrap().insert(learning.id, learning);
        }
    }

    #[async_trait]
    impl LearningLoader for MockLearningLoader {
        async fn load(&self, id: LearningId) -> Result<Option<Learning>> {
            Ok(self.learnings.lock().unwrap().get(&id).cloned())
        }
    }

    struct MockAttributionStore {
        attributions: Mutex<Vec<AttributionRecord>>,
        values: Mutex<HashMap<LearningId, LearningValue>>,
        experiments: Mutex<HashMap<LearningId, AblationExperiment>>,
    }

    impl MockAttributionStore {
        fn new() -> Self {
            Self {
                attributions: Mutex::new(Vec::new()),
                values: Mutex::new(HashMap::new()),
                experiments: Mutex::new(HashMap::new()),
            }
        }

        fn get_attributions(&self) -> Vec<AttributionRecord> {
            self.attributions.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl AttributionStore for MockAttributionStore {
        async fn store_attribution(&self, record: &AttributionRecord) -> Result<()> {
            self.attributions.lock().unwrap().push(record.clone());
            Ok(())
        }

        async fn get_attributions_for_learning(
            &self,
            id: LearningId,
        ) -> Result<Vec<AttributionRecord>> {
            Ok(self
                .attributions
                .lock()
                .unwrap()
                .iter()
                .filter(|r| r.learning_id == id)
                .cloned()
                .collect())
        }

        async fn get_attributions_for_session(
            &self,
            id: &SessionId,
        ) -> Result<Vec<AttributionRecord>> {
            Ok(self
                .attributions
                .lock()
                .unwrap()
                .iter()
                .filter(|r| &r.session_id == id)
                .cloned()
                .collect())
        }

        async fn get_learning_value(&self, id: LearningId) -> Result<Option<LearningValue>> {
            Ok(self.values.lock().unwrap().get(&id).cloned())
        }

        async fn update_learning_value(&self, value: &LearningValue) -> Result<()> {
            self.values
                .lock()
                .unwrap()
                .insert(value.learning_id, value.clone());
            Ok(())
        }

        async fn list_learning_values(&self, limit: usize) -> Result<Vec<LearningValue>> {
            Ok(self
                .values
                .lock()
                .unwrap()
                .values()
                .take(limit)
                .cloned()
                .collect())
        }

        async fn get_experiment(&self, id: LearningId) -> Result<Option<AblationExperiment>> {
            Ok(self.experiments.lock().unwrap().get(&id).cloned())
        }

        async fn update_experiment(&self, exp: &AblationExperiment) -> Result<()> {
            self.experiments
                .lock()
                .unwrap()
                .insert(exp.learning_id, exp.clone());
            Ok(())
        }
    }

    struct MockTranscriptFetcher {
        transcripts: Mutex<HashMap<String, ParsedTranscript>>,
    }

    impl MockTranscriptFetcher {
        fn new() -> Self {
            Self {
                transcripts: Mutex::new(HashMap::new()),
            }
        }

        fn add_transcript(&self, session_id: &str, transcript: ParsedTranscript) {
            self.transcripts
                .lock()
                .unwrap()
                .insert(session_id.to_string(), transcript);
        }
    }

    #[async_trait]
    impl TranscriptFetcher for MockTranscriptFetcher {
        async fn fetch(&self, session_id: &SessionId) -> Result<Option<ParsedTranscript>> {
            Ok(self
                .transcripts
                .lock()
                .unwrap()
                .get(session_id.as_str())
                .cloned())
        }
    }

    struct MockLightweightFetcher {
        events: Mutex<HashMap<String, Vec<LightweightEvent>>>,
    }

    impl MockLightweightFetcher {
        fn new() -> Self {
            Self {
                events: Mutex::new(HashMap::new()),
            }
        }

        fn add_events(&self, session_id: &str, events: Vec<LightweightEvent>) {
            self.events
                .lock()
                .unwrap()
                .insert(session_id.to_string(), events);
        }
    }

    #[async_trait]
    impl LightweightEventFetcher for MockLightweightFetcher {
        async fn fetch(&self, session_id: &SessionId) -> Result<Vec<LightweightEvent>> {
            Ok(self
                .events
                .lock()
                .unwrap()
                .get(session_id.as_str())
                .cloned()
                .unwrap_or_default())
        }
    }

    // ==========================================================================
    // Helper functions
    // ==========================================================================

    fn create_test_learning(id: LearningId) -> Learning {
        Learning {
            id,
            scope: Scope::User("test-user".into()),
            category: LearningCategory::Preference,
            content: LearningContent {
                description: "Test learning".into(),
                pattern: None,
                insight: "Use Result for error handling".into(),
            },
            confidence: 0.8,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            source: LearningSource::UserCreated,
        }
    }

    fn create_test_transcript() -> ParsedTranscript {
        ParsedTranscript {
            session_id: "test-session".into(),
            messages: vec![
                TranscriptMessage {
                    role: "user".into(),
                    content: "How do I handle errors?".into(),
                    timestamp: None,
                },
                TranscriptMessage {
                    role: "assistant".into(),
                    content: "Use Result types for error handling.".into(),
                    timestamp: None,
                },
            ],
            tool_uses: vec![],
            metadata: TranscriptMetadata::default(),
        }
    }

    #[allow(clippy::type_complexity)]
    fn create_test_consumer() -> (
        AttributionConsumer<
            MockActivationDetector,
            ExponentialDecayCorrelator,
            ConservativeAblation,
            MockLearningLoader,
            MockEmbedder,
            MockTranscriptFetcher,
            MockLightweightFetcher,
        >,
        Arc<MockActivationDetector>,
        Arc<MockLearningLoader>,
        Arc<MockAttributionStore>,
        Arc<MockTranscriptFetcher>,
        Arc<MockLightweightFetcher>,
    ) {
        let activation_detector = Arc::new(MockActivationDetector::new());
        let temporal_correlator = Arc::new(ExponentialDecayCorrelator::new());
        let attribution_store = Arc::new(MockAttributionStore::new());
        let learning_loader = Arc::new(MockLearningLoader::new());
        let embedder = Arc::new(MockEmbedder);
        let transcript_fetcher = Arc::new(MockTranscriptFetcher::new());
        let lightweight_fetcher = Arc::new(MockLightweightFetcher::new());

        let ablation_manager = AblationManager::new(
            ConservativeAblation::new(),
            attribution_store.clone() as Arc<dyn AttributionStore>,
        );

        let consumer = AttributionConsumer::new(
            activation_detector.clone(),
            temporal_correlator,
            ablation_manager,
            ValueAggregator::new(),
            attribution_store.clone() as Arc<dyn AttributionStore>,
            learning_loader.clone(),
            embedder,
            transcript_fetcher.clone(),
            lightweight_fetcher.clone(),
            AttributionConfig::default(),
        );

        (
            consumer,
            activation_detector,
            learning_loader,
            attribution_store,
            transcript_fetcher,
            lightweight_fetcher,
        )
    }

    // ==========================================================================
    // Tests
    // ==========================================================================

    #[test]
    fn test_attribution_config_defaults() {
        let config = AttributionConfig::default();
        assert!(config.enabled);
        assert_eq!(config.group, "attribution");
        assert_eq!(config.batch_size, 10);
    }

    #[test]
    fn test_outcome_to_value() {
        assert_eq!(outcome_to_value(&Outcome::Success), 1.0);
        assert_eq!(outcome_to_value(&Outcome::Partial), 0.5);
        assert_eq!(outcome_to_value(&Outcome::Failure), -0.5);
        assert_eq!(outcome_to_value(&Outcome::Abandoned), -1.0);
    }

    #[test]
    fn test_attribution_result_new() {
        let session_id = SessionId::from("test-session");
        let result = AttributionResult::new(&session_id);
        assert_eq!(result.session_id, Some(session_id));
        assert_eq!(result.learnings_processed, 0);
        assert!(result.errors.is_empty());
        assert!(!result.disabled);
    }

    #[test]
    fn test_attribution_result_disabled() {
        let result = AttributionResult::disabled();
        assert!(result.disabled);
        assert!(result.session_id.is_none());
    }

    #[test]
    fn test_attribution_result_is_success() {
        let mut result = AttributionResult::default();
        assert!(result.is_success());

        result.errors.push((Uuid::now_v7(), "test error".into()));
        assert!(!result.is_success());

        let disabled = AttributionResult::disabled();
        assert!(!disabled.is_success());
    }

    #[tokio::test]
    async fn test_process_heavy_event_when_disabled() {
        let (_consumer, _, _, _, _, _) = create_test_consumer();

        // Create a consumer with disabled config
        let config = AttributionConfig {
            enabled: false,
            ..Default::default()
        };

        let activation_detector = Arc::new(MockActivationDetector::new());
        let temporal_correlator = Arc::new(ExponentialDecayCorrelator::new());
        let attribution_store = Arc::new(MockAttributionStore::new());
        let learning_loader = Arc::new(MockLearningLoader::new());
        let embedder = Arc::new(MockEmbedder);
        let transcript_fetcher = Arc::new(MockTranscriptFetcher::new());
        let lightweight_fetcher = Arc::new(MockLightweightFetcher::new());

        let ablation_manager = AblationManager::new(
            ConservativeAblation::new(),
            attribution_store.clone() as Arc<dyn AttributionStore>,
        );

        let disabled_consumer = AttributionConsumer::new(
            activation_detector,
            temporal_correlator,
            ablation_manager,
            ValueAggregator::new(),
            attribution_store as Arc<dyn AttributionStore>,
            learning_loader,
            embedder,
            transcript_fetcher,
            lightweight_fetcher,
            config,
        );

        let event = HeavyEvent::new(AssessmentContext::new("test"), Outcome::Success);
        let result = disabled_consumer.process_heavy_event(&event).await.unwrap();
        assert!(result.disabled);
    }

    #[tokio::test]
    async fn test_process_heavy_event_no_transcript() {
        let (consumer, _, _, attribution_store, _, _) = create_test_consumer();

        let ctx = AssessmentContext::new("missing-transcript").with_learnings(vec![Uuid::now_v7()]);
        let event = HeavyEvent::new(ctx, Outcome::Success);

        let result = consumer.process_heavy_event(&event).await.unwrap();

        // Should complete without errors but process nothing
        assert_eq!(result.learnings_processed, 0);
        assert!(result.errors.is_empty());

        // No attributions stored
        assert!(attribution_store.get_attributions().is_empty());
    }

    #[tokio::test]
    async fn test_process_heavy_event_with_activated_learning() {
        let (
            consumer,
            activation_detector,
            learning_loader,
            attribution_store,
            transcript_fetcher,
            lightweight_fetcher,
        ) = create_test_consumer();

        let learning_id = Uuid::now_v7();
        let session_id = "test-session";

        // Set up test data
        learning_loader.add_learning(create_test_learning(learning_id));
        transcript_fetcher.add_transcript(session_id, create_test_transcript());
        lightweight_fetcher.add_events(
            session_id,
            vec![
                LightweightEvent::new(AssessmentContext::new(session_id), 1, Uuid::now_v7())
                    .with_signal(LightweightSignal::Positive {
                        pattern: "thanks".into(),
                        confidence: 0.9,
                    }),
            ],
        );

        // Configure activation detector to return activated
        activation_detector.set_result(
            learning_id,
            ActivationResult {
                was_activated: true,
                confidence: 0.85,
                signals: vec![ActivationSignal::EmbeddingSimilarity {
                    score: 0.85,
                    message_idx: 1,
                }],
            },
        );

        let ctx = AssessmentContext::new(session_id).with_learnings(vec![learning_id]);
        let event = HeavyEvent::new(ctx, Outcome::Success);

        let result = consumer.process_heavy_event(&event).await.unwrap();

        assert_eq!(result.learnings_processed, 1);
        assert!(result.errors.is_empty());

        // Check attribution was stored
        let attributions = attribution_store.get_attributions();
        assert_eq!(attributions.len(), 1);
        assert_eq!(attributions[0].learning_id, learning_id);
        assert!(attributions[0].was_activated);
        assert_eq!(attributions[0].session_outcome, 1.0); // Success
    }

    #[tokio::test]
    async fn test_process_heavy_event_not_activated() {
        let (
            consumer,
            _activation_detector,
            learning_loader,
            attribution_store,
            transcript_fetcher,
            _lightweight_fetcher,
        ) = create_test_consumer();

        let learning_id = Uuid::now_v7();
        let session_id = "test-session";

        learning_loader.add_learning(create_test_learning(learning_id));
        transcript_fetcher.add_transcript(session_id, create_test_transcript());

        // Activation detector returns not activated by default

        let ctx = AssessmentContext::new(session_id).with_learnings(vec![learning_id]);
        let event = HeavyEvent::new(ctx, Outcome::Success);

        let result = consumer.process_heavy_event(&event).await.unwrap();

        assert_eq!(result.learnings_processed, 1);

        // Attribution stored but with zero value
        let attributions = attribution_store.get_attributions();
        assert_eq!(attributions.len(), 1);
        assert!(!attributions[0].was_activated);
        assert_eq!(attributions[0].attributed_value, 0.0);
    }

    #[tokio::test]
    async fn test_process_heavy_event_missing_learning() {
        let (consumer, _, _, _, transcript_fetcher, _) = create_test_consumer();

        let learning_id = Uuid::now_v7();
        let session_id = "test-session";

        // Don't add the learning to the store
        transcript_fetcher.add_transcript(session_id, create_test_transcript());

        let ctx = AssessmentContext::new(session_id).with_learnings(vec![learning_id]);
        let event = HeavyEvent::new(ctx, Outcome::Success);

        let result = consumer.process_heavy_event(&event).await.unwrap();

        // Should record an error for the missing learning
        assert_eq!(result.learnings_processed, 0);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].0, learning_id);
    }

    #[tokio::test]
    async fn test_process_multiple_learnings() {
        let (
            consumer,
            activation_detector,
            learning_loader,
            attribution_store,
            transcript_fetcher,
            _,
        ) = create_test_consumer();

        let learning1 = Uuid::now_v7();
        let learning2 = Uuid::now_v7();
        let session_id = "test-session";

        learning_loader.add_learning(create_test_learning(learning1));
        learning_loader.add_learning(create_test_learning(learning2));
        transcript_fetcher.add_transcript(session_id, create_test_transcript());

        // First learning activated, second not
        activation_detector.set_result(
            learning1,
            ActivationResult {
                was_activated: true,
                confidence: 0.9,
                signals: vec![ActivationSignal::EmbeddingSimilarity {
                    score: 0.9,
                    message_idx: 1,
                }],
            },
        );

        let ctx = AssessmentContext::new(session_id).with_learnings(vec![learning1, learning2]);
        let event = HeavyEvent::new(ctx, Outcome::Partial);

        let result = consumer.process_heavy_event(&event).await.unwrap();

        assert_eq!(result.learnings_processed, 2);
        assert!(result.errors.is_empty());

        let attributions = attribution_store.get_attributions();
        assert_eq!(attributions.len(), 2);

        // Check session outcome is partial (0.5) for both
        for attr in &attributions {
            assert_eq!(attr.session_outcome, 0.5);
        }
    }

    #[tokio::test]
    async fn test_temporal_correlation_signals() {
        let (
            consumer,
            activation_detector,
            learning_loader,
            attribution_store,
            transcript_fetcher,
            lightweight_fetcher,
        ) = create_test_consumer();

        let learning_id = Uuid::now_v7();
        let session_id = "test-session";

        learning_loader.add_learning(create_test_learning(learning_id));
        transcript_fetcher.add_transcript(session_id, create_test_transcript());

        // Add mixed signals
        lightweight_fetcher.add_events(
            session_id,
            vec![
                LightweightEvent::new(AssessmentContext::new(session_id), 0, Uuid::now_v7())
                    .with_signal(LightweightSignal::Negative {
                        pattern: "frustrated".into(),
                        confidence: 0.7,
                    }),
                LightweightEvent::new(AssessmentContext::new(session_id), 1, Uuid::now_v7())
                    .with_signal(LightweightSignal::Positive {
                        pattern: "thanks".into(),
                        confidence: 0.9,
                    }),
            ],
        );

        activation_detector.set_result(
            learning_id,
            ActivationResult {
                was_activated: true,
                confidence: 0.85,
                signals: vec![ActivationSignal::EmbeddingSimilarity {
                    score: 0.85,
                    message_idx: 1,
                }],
            },
        );

        let ctx = AssessmentContext::new(session_id).with_learnings(vec![learning_id]);
        let event = HeavyEvent::new(ctx, Outcome::Success);

        let result = consumer.process_heavy_event(&event).await.unwrap();
        assert!(result.is_success());

        let attributions = attribution_store.get_attributions();
        assert_eq!(attributions.len(), 1);

        // Should have temporal scores from the mixed signals
        let attr = &attributions[0];
        assert!(
            attr.temporal_positive > 0.0,
            "Should have positive temporal score"
        );
        assert!(
            attr.temporal_negative > 0.0,
            "Should have negative temporal score"
        );
    }
}
