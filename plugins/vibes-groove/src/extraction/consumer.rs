//! Extraction consumer for processing heavy assessment events
//!
//! This consumer subscribes to `groove.assessment.heavy` events, runs the
//! extraction pipeline (LLM candidates, pattern detectors, deduplication),
//! and persists learnings.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace};
use vibes_iggy::{EventConsumer, Offset, SeekPosition};

use crate::assessment::{EventId, HeavyEvent, SessionId};
use crate::capture::ParsedTranscript;
use crate::extraction::patterns::{CorrectionDetector, ErrorRecoveryDetector};
use crate::extraction::{DeduplicationStrategy, Embedder, LearningCandidate};
use crate::store::LearningStore;
use crate::types::{Learning, LearningCategory, LearningContent, LearningSource, Scope};
use crate::{GrooveError, Result};

/// Default poll timeout for extraction consumer.
const DEFAULT_POLL_TIMEOUT: Duration = Duration::from_secs(1);

/// Default batch size for extraction consumer.
const DEFAULT_BATCH_SIZE: usize = 10;

/// Configuration for the extraction consumer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionConfig {
    /// Whether extraction is enabled
    pub enabled: bool,
    /// Minimum confidence threshold for extracted learnings
    pub min_confidence: f64,
    /// Consumer group name
    pub group: String,
    /// Maximum events per poll
    pub batch_size: usize,
    /// Poll timeout in milliseconds
    pub poll_timeout_ms: u64,
}

impl ExtractionConfig {
    /// Get poll timeout as Duration
    pub fn poll_timeout(&self) -> Duration {
        Duration::from_millis(self.poll_timeout_ms)
    }
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_confidence: 0.6,
            group: "extraction".to_string(),
            batch_size: DEFAULT_BATCH_SIZE,
            poll_timeout_ms: DEFAULT_POLL_TIMEOUT.as_millis() as u64,
        }
    }
}

impl ExtractionConfig {
    /// Create a new configuration with defaults
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set minimum confidence threshold
    #[must_use]
    pub fn with_min_confidence(mut self, confidence: f64) -> Self {
        self.min_confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set the poll timeout
    #[must_use]
    pub fn with_poll_timeout(mut self, timeout: Duration) -> Self {
        self.poll_timeout_ms = timeout.as_millis() as u64;
        self
    }

    /// Set the batch size
    #[must_use]
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Enable or disable extraction
    #[must_use]
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Trait for fetching transcripts by session ID
#[async_trait]
pub trait TranscriptFetcher: Send + Sync {
    /// Fetch the parsed transcript for a session
    async fn fetch(&self, session_id: &SessionId) -> Result<Option<ParsedTranscript>>;
}

/// Extraction consumer that processes heavy assessment events
pub struct ExtractionConsumer<S, E, D, T>
where
    S: LearningStore,
    E: Embedder,
    D: DeduplicationStrategy,
    T: TranscriptFetcher,
{
    /// Learning store for persisting extracted learnings
    store: Arc<S>,
    /// Embedder for generating semantic embeddings
    embedder: Arc<E>,
    /// Deduplication strategy
    dedup: Arc<D>,
    /// Transcript fetcher
    transcript_fetcher: Arc<T>,
    /// Correction pattern detector
    correction_detector: CorrectionDetector,
    /// Error recovery pattern detector
    error_recovery_detector: ErrorRecoveryDetector,
    /// Configuration
    config: ExtractionConfig,
}

impl<S, E, D, T> ExtractionConsumer<S, E, D, T>
where
    S: LearningStore,
    E: Embedder,
    D: DeduplicationStrategy,
    T: TranscriptFetcher,
{
    /// Create a new extraction consumer
    pub fn new(
        store: Arc<S>,
        embedder: Arc<E>,
        dedup: Arc<D>,
        transcript_fetcher: Arc<T>,
        config: ExtractionConfig,
    ) -> Self {
        Self {
            store,
            embedder,
            dedup,
            transcript_fetcher,
            correction_detector: CorrectionDetector::new(),
            error_recovery_detector: ErrorRecoveryDetector::new(),
            config,
        }
    }

    /// Check if extraction is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the minimum confidence threshold
    pub fn min_confidence(&self) -> f64 {
        self.config.min_confidence
    }

    /// Process a heavy assessment event
    ///
    /// This runs the full extraction pipeline:
    /// 1. Collect LLM-extracted candidates from event
    /// 2. Fetch transcript and run pattern detectors
    /// 3. Filter by minimum confidence
    /// 4. Embed, deduplicate, and store learnings
    pub async fn process_heavy_event(&self, event: &HeavyEvent) -> Result<ExtractionResult> {
        if !self.config.enabled {
            return Ok(ExtractionResult::disabled());
        }

        let mut result = ExtractionResult::new(event.context.event_id);
        let mut candidates = Vec::new();

        // Collect LLM-extracted candidates from the event
        for extraction_candidate in &event.extraction_candidates {
            let source = crate::extraction::ExtractionSource::new(
                event.context.session_id.clone(),
                event.context.event_id,
                crate::extraction::ExtractionMethod::Llm,
            )
            .with_message_range(
                extraction_candidate.message_range.0,
                extraction_candidate.message_range.1,
            );

            candidates.push(LearningCandidate::new(
                extraction_candidate.description.clone(),
                extraction_candidate.description.clone(),
                extraction_candidate.confidence,
                source,
            ));
        }

        // Fetch transcript and run pattern detectors
        if let Some(transcript) = self
            .transcript_fetcher
            .fetch(&event.context.session_id)
            .await?
        {
            // Run correction detector
            let correction_candidates = self.correction_detector.detect(&transcript)?;
            candidates.extend(correction_candidates);

            // Run error recovery detector
            let error_recovery_candidates = self.error_recovery_detector.detect(&transcript)?;
            candidates.extend(error_recovery_candidates);
        }

        result.candidates_processed = candidates.len() as u32;

        // Process each candidate
        for candidate in candidates {
            self.process_candidate(&candidate, &mut result).await?;
        }

        Ok(result)
    }

    /// Process a single learning candidate
    async fn process_candidate(
        &self,
        candidate: &LearningCandidate,
        result: &mut ExtractionResult,
    ) -> Result<()> {
        // Filter by minimum confidence
        if candidate.confidence < self.config.min_confidence {
            result.rejected += 1;
            debug!(
                confidence = candidate.confidence,
                min = self.config.min_confidence,
                "Rejecting candidate below confidence threshold"
            );
            return Ok(());
        }

        // Generate embedding
        let embedding = self
            .embedder
            .embed(&candidate.description)
            .await
            .map_err(|e| GrooveError::Embedding(e.to_string()))?;

        // Create a learning from the candidate
        let learning = self.candidate_to_learning(candidate, embedding.clone())?;

        // Check for duplicates
        if let Some(existing) = self
            .dedup
            .find_duplicate(&learning, self.store.as_ref())
            .await?
        {
            // Merge with existing learning
            let merged = self.dedup.merge(&existing, &learning).await?;
            self.store.update(&merged).await?;
            result.merged.push((learning.id, existing.id));
            debug!(
                new_id = %learning.id,
                existing_id = %existing.id,
                "Merged duplicate learning"
            );
        } else {
            // Store as new learning
            let id = self.store.store(&learning).await?;
            result.created.push(id);
            debug!(id = %id, "Created new learning");
        }

        Ok(())
    }

    /// Convert a candidate to a Learning
    fn candidate_to_learning(
        &self,
        candidate: &LearningCandidate,
        _embedding: Vec<f32>,
    ) -> Result<Learning> {
        // Determine scope from session context
        // For now, default to user scope
        let scope = Scope::User("default".to_string());

        // Determine category from extraction method
        let category = match &candidate.source.extraction_method {
            crate::extraction::ExtractionMethod::Pattern(pattern_type) => match pattern_type {
                crate::extraction::PatternType::Correction => LearningCategory::Preference,
                crate::extraction::PatternType::ErrorRecovery => LearningCategory::ErrorRecovery,
            },
            crate::extraction::ExtractionMethod::Llm => LearningCategory::CodePattern,
        };

        let content = LearningContent {
            description: candidate.description.clone(),
            pattern: candidate.pattern.clone(),
            insight: candidate.insight.clone(),
        };

        // Use Transcript source with the session info
        let message_index = candidate
            .source
            .message_range
            .map(|(start, _)| start as usize)
            .unwrap_or(0);

        let source = LearningSource::Transcript {
            session_id: candidate.source.session_id.as_str().to_string(),
            message_index,
        };

        let mut learning = Learning::new(scope, category, content, source);
        learning.confidence = candidate.confidence;
        // Note: embedding is stored separately in CozoStore, not on the Learning struct

        Ok(learning)
    }
}

/// Result of processing a heavy event
#[derive(Debug, Clone, Default)]
pub struct ExtractionResult {
    /// Event ID of the source heavy event
    pub source_event_id: Option<EventId>,
    /// Number of candidates processed
    pub candidates_processed: u32,
    /// IDs of newly created learnings
    pub created: Vec<uuid::Uuid>,
    /// Pairs of (new_id, merged_into_id) for merged learnings
    pub merged: Vec<(uuid::Uuid, uuid::Uuid)>,
    /// Number of candidates rejected (below confidence threshold)
    pub rejected: u32,
    /// Whether extraction was disabled
    pub disabled: bool,
}

impl ExtractionResult {
    /// Create a new result for a specific event
    pub fn new(event_id: EventId) -> Self {
        Self {
            source_event_id: Some(event_id),
            ..Default::default()
        }
    }

    /// Create a result indicating extraction is disabled
    pub fn disabled() -> Self {
        Self {
            disabled: true,
            ..Default::default()
        }
    }

    /// Check if any learnings were processed
    pub fn has_activity(&self) -> bool {
        !self.created.is_empty() || !self.merged.is_empty() || self.rejected > 0
    }

    /// Total learnings created or updated
    pub fn total_stored(&self) -> usize {
        self.created.len() + self.merged.len()
    }
}

/// Events emitted by the extraction consumer to the `groove.extraction` topic.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExtractionEvent {
    /// A new learning was created from extraction
    LearningCreated {
        /// ID of the newly created learning
        learning_id: uuid::Uuid,
        /// Category of the learning
        category: LearningCategory,
        /// Confidence score
        confidence: f64,
        /// Source event ID that triggered this extraction
        source_event_id: Option<EventId>,
    },
    /// A learning was merged with an existing one
    LearningMerged {
        /// ID of the learning that was merged into
        learning_id: uuid::Uuid,
        /// ID of the duplicate that was merged
        merged_from: uuid::Uuid,
        /// Source event ID that triggered this extraction
        source_event_id: Option<EventId>,
    },
    /// Extraction failed for a candidate
    ExtractionFailed {
        /// Reason for the failure
        reason: String,
        /// Source event ID
        source_event_id: Option<EventId>,
        /// Optional error details
        error: Option<String>,
    },
}

impl ExtractionEvent {
    /// Create a LearningCreated event
    pub fn learning_created(
        learning_id: uuid::Uuid,
        category: LearningCategory,
        confidence: f64,
        source_event_id: Option<EventId>,
    ) -> Self {
        Self::LearningCreated {
            learning_id,
            category,
            confidence,
            source_event_id,
        }
    }

    /// Create a LearningMerged event
    pub fn learning_merged(
        learning_id: uuid::Uuid,
        merged_from: uuid::Uuid,
        source_event_id: Option<EventId>,
    ) -> Self {
        Self::LearningMerged {
            learning_id,
            merged_from,
            source_event_id,
        }
    }

    /// Create an ExtractionFailed event
    pub fn extraction_failed(
        reason: impl Into<String>,
        source_event_id: Option<EventId>,
        error: Option<String>,
    ) -> Self {
        Self::ExtractionFailed {
            reason: reason.into(),
            source_event_id,
            error,
        }
    }
}

/// Result of running the extraction consumer loop.
#[derive(Debug)]
pub enum ConsumerResult {
    /// Consumer stopped due to shutdown signal.
    Shutdown,
    /// Consumer stopped due to an error.
    Error(String),
}

/// Run the extraction consumer loop.
///
/// This function processes HeavyEvents from the assessment log and runs the
/// extraction pipeline on each event. It runs until the shutdown token is
/// cancelled or an error occurs.
///
/// # Arguments
///
/// * `consumer` - The event consumer to poll from.
/// * `processor` - The extraction consumer processor.
/// * `event_producer` - Optional event log to emit ExtractionEvents to.
/// * `shutdown` - Cancellation token to signal shutdown.
///
/// # Returns
///
/// Returns `ConsumerResult::Shutdown` on graceful shutdown, or
/// `ConsumerResult::Error` if an unrecoverable error occurred.
pub async fn extraction_consumer_loop<S, E, D, T, P>(
    mut consumer: Box<dyn EventConsumer<HeavyEvent>>,
    processor: Arc<ExtractionConsumer<S, E, D, T>>,
    event_producer: Option<Arc<P>>,
    shutdown: CancellationToken,
) -> ConsumerResult
where
    S: LearningStore + 'static,
    E: Embedder + 'static,
    D: DeduplicationStrategy + 'static,
    T: TranscriptFetcher + 'static,
    P: vibes_iggy::EventLog<ExtractionEvent> + 'static,
{
    let config = &processor.config;
    info!(group = %config.group, "Extraction consumer starting");

    // Seek to beginning to process all events (can resume from committed offset)
    if let Err(e) = consumer.seek(SeekPosition::Beginning).await {
        error!(error = %e, "Failed to seek to start position");
        return ConsumerResult::Error(format!("Seek failed: {e}"));
    }

    loop {
        tokio::select! {
            biased;

            _ = shutdown.cancelled() => {
                info!(group = %config.group, "Extraction consumer received shutdown signal");
                return ConsumerResult::Shutdown;
            }

            result = consumer.poll(config.batch_size, config.poll_timeout()) => {
                match result {
                    Ok(batch) => {
                        if batch.is_empty() {
                            trace!(group = %config.group, "Empty batch, waiting before retry");
                            tokio::time::sleep(config.poll_timeout()).await;
                            continue;
                        }

                        debug!(group = %config.group, count = batch.len(), "Processing batch");

                        let mut last_offset: Option<Offset> = None;
                        for (offset, event) in batch {
                            let source_event_id = Some(event.context.event_id);

                            // Process the heavy event through extraction pipeline
                            match processor.process_heavy_event(&event).await {
                                Ok(result) => {
                                    // Emit events for created learnings
                                    if let Some(ref producer) = event_producer {
                                        for learning_id in &result.created {
                                            // Default to CodePattern for now - the actual category
                                            // would need to be tracked in ExtractionResult
                                            let event = ExtractionEvent::learning_created(
                                                *learning_id,
                                                LearningCategory::CodePattern,
                                                processor.min_confidence(),
                                                source_event_id,
                                            );
                                            if let Err(e) = producer.append(event).await {
                                                error!(error = %e, "Failed to emit LearningCreated event");
                                            }
                                        }

                                        // Emit events for merged learnings
                                        for (merged_from, learning_id) in &result.merged {
                                            let event = ExtractionEvent::learning_merged(
                                                *learning_id,
                                                *merged_from,
                                                source_event_id,
                                            );
                                            if let Err(e) = producer.append(event).await {
                                                error!(error = %e, "Failed to emit LearningMerged event");
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!(
                                        event_id = ?event.context.event_id,
                                        error = %e,
                                        "Failed to process heavy event"
                                    );

                                    // Emit failure event
                                    if let Some(ref producer) = event_producer {
                                        let event = ExtractionEvent::extraction_failed(
                                            "Processing failed",
                                            source_event_id,
                                            Some(e.to_string()),
                                        );
                                        if let Err(emit_err) = producer.append(event).await {
                                            error!(error = %emit_err, "Failed to emit ExtractionFailed event");
                                        }
                                    }
                                }
                            }
                            last_offset = Some(offset);
                        }

                        // Commit after processing batch
                        if let Some(offset) = last_offset
                            && let Err(e) = consumer.commit(offset).await
                        {
                            error!(group = %config.group, error = %e, "Failed to commit offset");
                            // Continue processing - commit failure is not fatal
                        }
                    }
                    Err(e) => {
                        error!(group = %config.group, error = %e, "Poll failed");
                        // Back off on error
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }
    }
}

/// Start the extraction consumer.
///
/// This is the main entry point for starting extraction processing.
/// It creates the consumer and spawns a background task that runs until
/// the shutdown token is cancelled.
///
/// # Arguments
///
/// * `event_log` - The event log to consume HeavyEvents from
/// * `processor` - The extraction processor
/// * `event_producer` - Optional event log to emit ExtractionEvents to
/// * `shutdown` - Cancellation token for graceful shutdown
///
/// # Returns
///
/// Returns a `JoinHandle` that can be awaited to wait for the consumer to stop.
pub async fn start_extraction_consumer<S, E, D, T, L, P>(
    event_log: Arc<L>,
    processor: Arc<ExtractionConsumer<S, E, D, T>>,
    event_producer: Option<Arc<P>>,
    shutdown: CancellationToken,
) -> std::result::Result<JoinHandle<ConsumerResult>, StartConsumerError>
where
    S: LearningStore + 'static,
    E: Embedder + 'static,
    D: DeduplicationStrategy + 'static,
    T: TranscriptFetcher + 'static,
    L: vibes_iggy::EventLog<HeavyEvent> + 'static,
    P: vibes_iggy::EventLog<ExtractionEvent> + 'static,
{
    let group = &processor.config.group;

    // Create consumer from event log
    let consumer = event_log
        .consumer(group)
        .await
        .map_err(|e| StartConsumerError::ConsumerCreation(e.to_string()))?;

    info!(group = %group, "Starting extraction consumer");

    // Spawn the consumer loop
    let handle = tokio::spawn(async move {
        extraction_consumer_loop(consumer, processor, event_producer, shutdown).await
    });

    Ok(handle)
}

/// Error type for starting the extraction consumer.
#[derive(Debug, thiserror::Error)]
pub enum StartConsumerError {
    /// Failed to create a consumer from the event log.
    #[error("Failed to create consumer: {0}")]
    ConsumerCreation(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assessment::{AssessmentContext, ExtractionCandidate, Outcome};
    use crate::capture::{TranscriptMessage, TranscriptMetadata};
    use crate::extraction::EmbedderResult;
    use crate::types::LearningId;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use vibes_iggy::{EventLog as _, InMemoryEventLog};

    // --- Mock implementations ---

    struct MockStore {
        learnings: Mutex<HashMap<LearningId, Learning>>,
        similar_results: Mutex<Vec<(Learning, f64)>>,
    }

    impl MockStore {
        fn new() -> Self {
            Self {
                learnings: Mutex::new(HashMap::new()),
                similar_results: Mutex::new(Vec::new()),
            }
        }

        #[allow(dead_code)]
        fn with_similar(self, learning: Learning, similarity: f64) -> Self {
            self.similar_results
                .lock()
                .unwrap()
                .push((learning, similarity));
            self
        }

        fn stored_count(&self) -> usize {
            self.learnings.lock().unwrap().len()
        }
    }

    #[async_trait]
    impl LearningStore for MockStore {
        async fn store(&self, learning: &Learning) -> std::result::Result<LearningId, GrooveError> {
            let id = learning.id;
            self.learnings.lock().unwrap().insert(id, learning.clone());
            Ok(id)
        }

        async fn get(&self, id: LearningId) -> std::result::Result<Option<Learning>, GrooveError> {
            Ok(self.learnings.lock().unwrap().get(&id).cloned())
        }

        async fn find_by_scope(
            &self,
            _scope: &Scope,
        ) -> std::result::Result<Vec<Learning>, GrooveError> {
            Ok(Vec::new())
        }

        async fn find_by_category(
            &self,
            _category: &LearningCategory,
        ) -> std::result::Result<Vec<Learning>, GrooveError> {
            Ok(Vec::new())
        }

        async fn semantic_search(
            &self,
            _embedding: &[f32],
            _limit: usize,
        ) -> std::result::Result<Vec<(Learning, f64)>, GrooveError> {
            Ok(Vec::new())
        }

        async fn update_usage(
            &self,
            _id: LearningId,
            _stats: &crate::UsageStats,
        ) -> std::result::Result<(), GrooveError> {
            Ok(())
        }

        async fn find_related(
            &self,
            _id: LearningId,
            _relation_type: Option<&crate::RelationType>,
        ) -> std::result::Result<Vec<Learning>, GrooveError> {
            Ok(Vec::new())
        }

        async fn store_relation(
            &self,
            _relation: &crate::LearningRelation,
        ) -> std::result::Result<(), GrooveError> {
            Ok(())
        }

        async fn delete(&self, _id: LearningId) -> std::result::Result<bool, GrooveError> {
            Ok(true)
        }

        async fn count(&self) -> std::result::Result<u64, GrooveError> {
            Ok(self.learnings.lock().unwrap().len() as u64)
        }

        async fn update(&self, learning: &Learning) -> std::result::Result<(), GrooveError> {
            self.learnings
                .lock()
                .unwrap()
                .insert(learning.id, learning.clone());
            Ok(())
        }

        async fn find_similar(
            &self,
            _embedding: &[f32],
            _threshold: f64,
            _limit: usize,
        ) -> std::result::Result<Vec<(Learning, f64)>, GrooveError> {
            Ok(self.similar_results.lock().unwrap().clone())
        }

        async fn find_for_injection(
            &self,
            _scope: &Scope,
            _context_embedding: Option<&[f32]>,
            _limit: usize,
        ) -> std::result::Result<Vec<Learning>, GrooveError> {
            Ok(Vec::new())
        }

        async fn count_by_scope(&self, _scope: &Scope) -> std::result::Result<u64, GrooveError> {
            Ok(0)
        }

        async fn count_by_category(
            &self,
            _category: &LearningCategory,
        ) -> std::result::Result<u64, GrooveError> {
            Ok(0)
        }
    }

    struct MockEmbedder {
        embedding: Vec<f32>,
    }

    impl MockEmbedder {
        fn new() -> Self {
            Self {
                embedding: vec![0.1, 0.2, 0.3],
            }
        }
    }

    #[async_trait]
    impl Embedder for MockEmbedder {
        async fn embed(&self, _text: &str) -> EmbedderResult<Vec<f32>> {
            Ok(self.embedding.clone())
        }

        fn dimensions(&self) -> usize {
            3
        }
    }

    struct MockDedup;

    #[async_trait]
    impl DeduplicationStrategy for MockDedup {
        async fn find_duplicate(
            &self,
            _candidate: &Learning,
            _store: &dyn LearningStore,
        ) -> std::result::Result<Option<Learning>, GrooveError> {
            Ok(None)
        }

        async fn merge(
            &self,
            existing: &Learning,
            _duplicate: &Learning,
        ) -> std::result::Result<Learning, GrooveError> {
            Ok(existing.clone())
        }
    }

    struct MockTranscriptFetcher {
        transcript: Mutex<Option<ParsedTranscript>>,
    }

    impl MockTranscriptFetcher {
        fn new() -> Self {
            Self {
                transcript: Mutex::new(None),
            }
        }

        fn with_transcript(self, transcript: ParsedTranscript) -> Self {
            *self.transcript.lock().unwrap() = Some(transcript);
            self
        }
    }

    #[async_trait]
    impl TranscriptFetcher for MockTranscriptFetcher {
        async fn fetch(&self, _session_id: &SessionId) -> Result<Option<ParsedTranscript>> {
            Ok(self.transcript.lock().unwrap().clone())
        }
    }

    fn make_heavy_event(candidates: Vec<ExtractionCandidate>) -> HeavyEvent {
        HeavyEvent::new(AssessmentContext::new("test-session"), Outcome::Success)
            .with_extraction_candidates(candidates)
    }

    fn make_transcript(messages: Vec<(&str, &str)>) -> ParsedTranscript {
        ParsedTranscript {
            session_id: "test-session".to_string(),
            messages: messages
                .into_iter()
                .map(|(role, content)| TranscriptMessage {
                    role: role.to_string(),
                    content: content.to_string(),
                    timestamp: None,
                })
                .collect(),
            tool_uses: Vec::new(),
            metadata: TranscriptMetadata::default(),
        }
    }

    // --- Configuration tests ---

    #[test]
    fn test_config_defaults() {
        let config = ExtractionConfig::default();
        assert!(config.enabled);
        assert!((config.min_confidence - 0.6).abs() < f64::EPSILON);
        assert_eq!(config.group, "extraction");
        assert_eq!(config.batch_size, DEFAULT_BATCH_SIZE);
    }

    #[test]
    fn test_config_builder() {
        let config = ExtractionConfig::new()
            .with_min_confidence(0.8)
            .with_enabled(false)
            .with_batch_size(20)
            .with_poll_timeout(Duration::from_secs(5));

        assert!(!config.enabled);
        assert!((config.min_confidence - 0.8).abs() < f64::EPSILON);
        assert_eq!(config.batch_size, 20);
        assert_eq!(config.poll_timeout(), Duration::from_secs(5));
    }

    #[test]
    fn test_config_clamps_confidence() {
        let high = ExtractionConfig::new().with_min_confidence(1.5);
        assert!((high.min_confidence - 1.0).abs() < f64::EPSILON);

        let low = ExtractionConfig::new().with_min_confidence(-0.5);
        assert!(low.min_confidence.abs() < f64::EPSILON);
    }

    // --- ExtractionResult tests ---

    #[test]
    fn test_extraction_result_defaults() {
        let result = ExtractionResult::default();
        assert!(result.source_event_id.is_none());
        assert_eq!(result.candidates_processed, 0);
        assert!(result.created.is_empty());
        assert!(result.merged.is_empty());
        assert_eq!(result.rejected, 0);
        assert!(!result.disabled);
    }

    #[test]
    fn test_extraction_result_disabled() {
        let result = ExtractionResult::disabled();
        assert!(result.disabled);
        assert!(!result.has_activity());
    }

    #[test]
    fn test_extraction_result_has_activity() {
        let mut result = ExtractionResult::default();
        assert!(!result.has_activity());

        result.created.push(uuid::Uuid::now_v7());
        assert!(result.has_activity());

        result.created.clear();
        result.rejected = 1;
        assert!(result.has_activity());
    }

    // --- Consumer creation tests ---

    #[tokio::test]
    async fn test_consumer_creation() {
        let store = Arc::new(MockStore::new());
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = Arc::new(MockDedup);
        let fetcher = Arc::new(MockTranscriptFetcher::new());

        let consumer =
            ExtractionConsumer::new(store, embedder, dedup, fetcher, ExtractionConfig::default());

        assert!(consumer.is_enabled());
        assert!((consumer.min_confidence() - 0.6).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_disabled_consumer_skips_processing() {
        let store = Arc::new(MockStore::new());
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = Arc::new(MockDedup);
        let fetcher = Arc::new(MockTranscriptFetcher::new());
        let config = ExtractionConfig::new().with_enabled(false);

        let consumer = ExtractionConsumer::new(store, embedder, dedup, fetcher, config);

        let event = make_heavy_event(vec![ExtractionCandidate::new((0, 5), "Test learning", 0.9)]);

        let result = consumer.process_heavy_event(&event).await.unwrap();
        assert!(result.disabled);
        assert_eq!(result.candidates_processed, 0);
    }

    // --- Processing tests ---

    #[tokio::test]
    async fn test_processes_llm_candidates() {
        let store = Arc::new(MockStore::new());
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = Arc::new(MockDedup);
        let fetcher = Arc::new(MockTranscriptFetcher::new());

        let consumer = ExtractionConsumer::new(
            store.clone(),
            embedder,
            dedup,
            fetcher,
            ExtractionConfig::default(),
        );

        let event = make_heavy_event(vec![
            ExtractionCandidate::new((0, 5), "Use snake_case for variables", 0.9),
            ExtractionCandidate::new((5, 10), "Prefer const over let", 0.8),
        ]);

        let result = consumer.process_heavy_event(&event).await.unwrap();

        assert_eq!(result.candidates_processed, 2);
        assert_eq!(result.created.len(), 2);
        assert_eq!(store.stored_count(), 2);
    }

    #[tokio::test]
    async fn test_runs_pattern_detectors_on_transcript() {
        let store = Arc::new(MockStore::new());
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = Arc::new(MockDedup);
        let transcript = make_transcript(vec![
            ("user", "No, use tabs instead of spaces"),
            ("assistant", "I'll use tabs from now on."),
        ]);
        let fetcher = Arc::new(MockTranscriptFetcher::new().with_transcript(transcript));

        let consumer = ExtractionConsumer::new(
            store.clone(),
            embedder,
            dedup,
            fetcher,
            ExtractionConfig::default(),
        );

        let event = make_heavy_event(vec![]);

        let result = consumer.process_heavy_event(&event).await.unwrap();

        // Should detect the correction pattern
        assert!(result.candidates_processed > 0);
        assert!(!result.created.is_empty());
    }

    #[tokio::test]
    async fn test_filters_by_confidence() {
        let store = Arc::new(MockStore::new());
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = Arc::new(MockDedup);
        let fetcher = Arc::new(MockTranscriptFetcher::new());
        let config = ExtractionConfig::new().with_min_confidence(0.85);

        let consumer = ExtractionConsumer::new(store.clone(), embedder, dedup, fetcher, config);

        let event = make_heavy_event(vec![
            ExtractionCandidate::new((0, 5), "High confidence", 0.9),
            ExtractionCandidate::new((5, 10), "Low confidence", 0.5),
        ]);

        let result = consumer.process_heavy_event(&event).await.unwrap();

        assert_eq!(result.candidates_processed, 2);
        assert_eq!(result.created.len(), 1); // Only high confidence
        assert_eq!(result.rejected, 1); // Low confidence rejected
    }

    #[tokio::test]
    async fn test_combines_llm_and_pattern_candidates() {
        let store = Arc::new(MockStore::new());
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = Arc::new(MockDedup);
        let transcript = make_transcript(vec![
            ("user", "Actually, use async/await instead"),
            ("assistant", "I'll switch to async/await."),
        ]);
        let fetcher = Arc::new(MockTranscriptFetcher::new().with_transcript(transcript));

        let consumer = ExtractionConsumer::new(
            store.clone(),
            embedder,
            dedup,
            fetcher,
            ExtractionConfig::default(),
        );

        let event = make_heavy_event(vec![ExtractionCandidate::new(
            (0, 5),
            "LLM extracted pattern",
            0.8,
        )]);

        let result = consumer.process_heavy_event(&event).await.unwrap();

        // Should have LLM candidate + correction pattern
        assert!(result.candidates_processed >= 2);
    }

    // --- Consumer loop tests ---

    #[tokio::test]
    async fn test_extraction_consumer_respects_shutdown() {
        // Setup: EventLog with HeavyEvent
        let log = Arc::new(InMemoryEventLog::<HeavyEvent>::new());

        // Append one event
        log.append(make_heavy_event(vec![ExtractionCandidate::new(
            (0, 5),
            "test learning",
            0.9,
        )]))
        .await
        .unwrap();

        // Create consumer
        let iggy_consumer = log.consumer("extraction-shutdown-test").await.unwrap();

        // Create processor
        let store = Arc::new(MockStore::new());
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = Arc::new(MockDedup);
        let fetcher = Arc::new(MockTranscriptFetcher::new());
        let config = ExtractionConfig::new().with_poll_timeout(Duration::from_millis(50));
        let processor = Arc::new(ExtractionConsumer::new(
            store, embedder, dedup, fetcher, config,
        ));

        // Create shutdown token
        let shutdown = CancellationToken::new();
        let shutdown_clone = shutdown.clone();

        // Spawn consumer loop (no event producer for this test)
        let handle = tokio::spawn(async move {
            extraction_consumer_loop::<_, _, _, _, InMemoryEventLog<ExtractionEvent>>(
                iggy_consumer,
                processor,
                None,
                shutdown_clone,
            )
            .await
        });

        // Give it time to start processing
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Signal shutdown
        shutdown.cancel();

        // Wait for consumer to stop
        let result = tokio::time::timeout(Duration::from_secs(1), handle)
            .await
            .expect("should complete within timeout")
            .expect("task should not panic");

        // Should have shut down gracefully
        assert!(matches!(result, ConsumerResult::Shutdown));
    }

    #[tokio::test]
    async fn test_extraction_consumer_processes_events() {
        let log = Arc::new(InMemoryEventLog::<HeavyEvent>::new());

        // Append 3 events
        for i in 0..3 {
            log.append(make_heavy_event(vec![ExtractionCandidate::new(
                (0, 5),
                format!("learning {i}"),
                0.9,
            )]))
            .await
            .unwrap();
        }

        // Create processor that tracks calls
        let store = Arc::new(MockStore::new());
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = Arc::new(MockDedup);
        let fetcher = Arc::new(MockTranscriptFetcher::new());
        let config = ExtractionConfig::new()
            .with_poll_timeout(Duration::from_millis(50))
            .with_batch_size(10);

        let processor = Arc::new(ExtractionConsumer::new(
            store.clone(),
            embedder,
            dedup,
            fetcher,
            config,
        ));

        let shutdown = CancellationToken::new();
        let shutdown_clone = shutdown.clone();
        let store_clone = store.clone();

        // Spawn consumer loop (no event producer for this test)
        let handle = tokio::spawn(async move {
            let consumer = log.consumer("process-test").await.unwrap();
            extraction_consumer_loop::<_, _, _, _, InMemoryEventLog<ExtractionEvent>>(
                consumer,
                processor,
                None,
                shutdown_clone,
            )
            .await
        });

        // Give it time to process
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Signal shutdown
        shutdown.cancel();

        // Wait for completion
        tokio::time::timeout(Duration::from_secs(1), handle)
            .await
            .expect("should complete within timeout")
            .expect("task should not panic");

        // Should have stored 3 learnings (one per event)
        assert_eq!(store_clone.stored_count(), 3);
    }

    #[tokio::test]
    async fn test_extraction_consumer_commits_after_batch() {
        let log = Arc::new(InMemoryEventLog::<HeavyEvent>::new());

        // Append events
        for i in 0..3 {
            log.append(make_heavy_event(vec![ExtractionCandidate::new(
                (0, 5),
                format!("commit test {i}"),
                0.9,
            )]))
            .await
            .unwrap();
        }

        // Create consumer and process events
        let mut consumer = log.consumer("commit-test-group").await.unwrap();
        consumer.seek(SeekPosition::Beginning).await.unwrap();

        // Initial committed offset should be 0
        assert_eq!(consumer.committed_offset(), 0);

        // Poll and process
        let batch = consumer
            .poll(100, Duration::from_millis(100))
            .await
            .unwrap();
        assert_eq!(batch.len(), 3);

        // Commit the last offset
        let last_offset = batch.last_offset().unwrap();
        consumer.commit(last_offset).await.unwrap();

        // Committed offset should now be updated
        assert_eq!(consumer.committed_offset(), last_offset);

        // Create a new consumer in the same group - should resume from committed offset
        let consumer2 = log.consumer("commit-test-group").await.unwrap();
        assert_eq!(consumer2.committed_offset(), last_offset);
    }

    #[tokio::test]
    async fn test_start_extraction_consumer() {
        let log = Arc::new(InMemoryEventLog::<HeavyEvent>::new());

        // Append one event
        log.append(make_heavy_event(vec![ExtractionCandidate::new(
            (0, 5),
            "start test",
            0.9,
        )]))
        .await
        .unwrap();

        let store = Arc::new(MockStore::new());
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = Arc::new(MockDedup);
        let fetcher = Arc::new(MockTranscriptFetcher::new());
        let config = ExtractionConfig::new().with_poll_timeout(Duration::from_millis(50));

        let processor = Arc::new(ExtractionConsumer::new(
            store.clone(),
            embedder,
            dedup,
            fetcher,
            config,
        ));

        let shutdown = CancellationToken::new();

        // Start consumer (no event producer for this test)
        let handle = start_extraction_consumer::<_, _, _, _, _, InMemoryEventLog<ExtractionEvent>>(
            log,
            processor,
            None,
            shutdown.clone(),
        )
        .await
        .expect("should start successfully");

        // Give it time to process
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Signal shutdown
        shutdown.cancel();

        // Wait for completion
        let result = tokio::time::timeout(Duration::from_secs(1), handle)
            .await
            .expect("should complete within timeout")
            .expect("task should not panic");

        assert!(matches!(result, ConsumerResult::Shutdown));
        assert_eq!(store.stored_count(), 1);
    }

    #[tokio::test]
    async fn test_extraction_consumer_emits_events() {
        // Setup event logs for HeavyEvents and ExtractionEvents
        let heavy_log = Arc::new(InMemoryEventLog::<HeavyEvent>::new());
        let extraction_log = Arc::new(InMemoryEventLog::<ExtractionEvent>::new());

        // Append 2 events
        for i in 0..2 {
            heavy_log
                .append(make_heavy_event(vec![ExtractionCandidate::new(
                    (0, 5),
                    format!("event {i} learning"),
                    0.9,
                )]))
                .await
                .unwrap();
        }

        // Create processor
        let store = Arc::new(MockStore::new());
        let embedder = Arc::new(MockEmbedder::new());
        let dedup = Arc::new(MockDedup);
        let fetcher = Arc::new(MockTranscriptFetcher::new());
        let config = ExtractionConfig::new()
            .with_poll_timeout(Duration::from_millis(50))
            .with_batch_size(10);

        let processor = Arc::new(ExtractionConsumer::new(
            store.clone(),
            embedder,
            dedup,
            fetcher,
            config,
        ));

        let shutdown = CancellationToken::new();
        let shutdown_clone = shutdown.clone();
        let extraction_log_clone = extraction_log.clone();

        // Spawn consumer loop with event producer
        let handle = tokio::spawn(async move {
            let consumer = heavy_log.consumer("emit-test").await.unwrap();
            extraction_consumer_loop(
                consumer,
                processor,
                Some(extraction_log_clone),
                shutdown_clone,
            )
            .await
        });

        // Give it time to process
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Signal shutdown
        shutdown.cancel();

        // Wait for completion
        tokio::time::timeout(Duration::from_secs(1), handle)
            .await
            .expect("should complete within timeout")
            .expect("task should not panic");

        // Should have emitted 2 LearningCreated events
        assert_eq!(extraction_log.len().await, 2);

        // Verify event contents
        let mut consumer = extraction_log.consumer("verify").await.unwrap();
        let batch = consumer.poll(10, Duration::from_millis(100)).await.unwrap();

        assert_eq!(batch.len(), 2);
        for (_, event) in batch {
            match event {
                ExtractionEvent::LearningCreated {
                    category,
                    confidence,
                    ..
                } => {
                    assert_eq!(category, LearningCategory::CodePattern);
                    assert!((confidence - 0.6).abs() < f64::EPSILON);
                }
                _ => panic!("Expected LearningCreated event"),
            }
        }
    }
}
