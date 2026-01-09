//! Integration tests for the extraction pipeline.
//!
//! These tests validate the end-to-end extraction pipeline including:
//! - Full pipeline with heavy events
//! - Duplicate detection and merging
//! - Pattern detection on transcripts
//! - Event emission to groove.extraction topic

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use std::sync::Mutex;
use tokio_util::sync::CancellationToken;
use vibes_iggy::{EventLog as _, InMemoryEventLog};

use vibes_groove::assessment::{AssessmentContext, ExtractionCandidate, HeavyEvent, Outcome};
use vibes_groove::capture::{ParsedTranscript, TranscriptMessage, TranscriptMetadata};
use vibes_groove::extraction::{
    DeduplicationStrategy, Embedder, EmbedderResult, ExtractionConfig, ExtractionConsumer,
    ExtractionEvent, extraction_consumer_loop,
};
use vibes_groove::store::LearningStore;
use vibes_groove::{
    GrooveError, Learning, LearningCategory, LearningContent, LearningId, LearningRelation,
    LearningSource, RelationType, Result, Scope, SessionId, TranscriptFetcher, UsageStats,
};

// --- Mock implementations ---

struct MockStore {
    learnings: Mutex<HashMap<LearningId, Learning>>,
}

impl MockStore {
    fn new() -> Self {
        Self {
            learnings: Mutex::new(HashMap::new()),
        }
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
        _stats: &UsageStats,
    ) -> std::result::Result<(), GrooveError> {
        Ok(())
    }

    async fn find_related(
        &self,
        _id: LearningId,
        _relation_type: Option<&RelationType>,
    ) -> std::result::Result<Vec<Learning>, GrooveError> {
        Ok(Vec::new())
    }

    async fn store_relation(
        &self,
        _relation: &LearningRelation,
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
        Ok(Vec::new())
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

struct MockDedup {
    duplicates: Mutex<HashMap<String, Learning>>,
}

impl MockDedup {
    fn new() -> Self {
        Self {
            duplicates: Mutex::new(HashMap::new()),
        }
    }

    fn with_duplicate(self, key: impl Into<String>, learning: Learning) -> Self {
        self.duplicates.lock().unwrap().insert(key.into(), learning);
        self
    }
}

#[async_trait]
impl DeduplicationStrategy for MockDedup {
    async fn find_duplicate(
        &self,
        candidate: &Learning,
        _store: &dyn LearningStore,
    ) -> std::result::Result<Option<Learning>, GrooveError> {
        let duplicates = self.duplicates.lock().unwrap();
        // Simple check: if description matches any key, return that learning
        for (key, learning) in duplicates.iter() {
            if candidate.content.description.contains(key) {
                return Ok(Some(learning.clone()));
            }
        }
        Ok(None)
    }

    async fn merge(
        &self,
        existing: &Learning,
        duplicate: &Learning,
    ) -> std::result::Result<Learning, GrooveError> {
        // Simple merge: return existing with higher confidence
        let mut merged = existing.clone();
        merged.confidence = merged.confidence.max(duplicate.confidence);
        Ok(merged)
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

// --- Helper functions ---

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

fn make_learning(description: &str) -> Learning {
    Learning::new(
        Scope::User("test".to_string()),
        LearningCategory::CodePattern,
        LearningContent {
            description: description.to_string(),
            pattern: Some(serde_json::json!({"pattern": "test pattern"})),
            insight: "Test insight".to_string(),
        },
        LearningSource::Transcript {
            session_id: "test".to_string(),
            message_index: 0,
        },
    )
}

// --- Integration tests ---

#[tokio::test]
async fn extraction_pipeline_end_to_end() {
    // Setup event logs
    let heavy_log = Arc::new(InMemoryEventLog::<HeavyEvent>::new());
    let extraction_log = Arc::new(InMemoryEventLog::<ExtractionEvent>::new());

    // Append heavy events
    for i in 0..3 {
        heavy_log
            .append(make_heavy_event(vec![ExtractionCandidate::new(
                (0, 5),
                format!("Learning from session {i}"),
                0.9,
            )]))
            .await
            .unwrap();
    }

    // Create processor
    let store = Arc::new(MockStore::new());
    let embedder = Arc::new(MockEmbedder::new());
    let dedup = Arc::new(MockDedup::new());
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

    // Spawn consumer loop
    let handle = tokio::spawn(async move {
        let consumer = heavy_log.consumer("e2e-test").await.unwrap();
        extraction_consumer_loop(
            consumer,
            processor,
            Some(extraction_log_clone),
            shutdown_clone,
        )
        .await
    });

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Signal shutdown
    shutdown.cancel();

    // Wait for completion
    tokio::time::timeout(Duration::from_secs(1), handle)
        .await
        .expect("should complete within timeout")
        .expect("task should not panic");

    // Verify learnings were stored
    assert_eq!(store.stored_count(), 3, "Expected 3 learnings");

    // Verify events were emitted
    assert_eq!(
        extraction_log.len().await,
        3,
        "Expected 3 extraction events"
    );
}

#[tokio::test]
async fn extraction_detects_duplicates() {
    // Setup
    let heavy_log = Arc::new(InMemoryEventLog::<HeavyEvent>::new());

    // Append event with a candidate that will be marked as duplicate
    heavy_log
        .append(make_heavy_event(vec![ExtractionCandidate::new(
            (0, 5),
            "Use snake_case for variables", // Contains "snake_case" which will match
            0.9,
        )]))
        .await
        .unwrap();

    // Create processor with dedup that will find a match
    let store = Arc::new(MockStore::new());
    let embedder = Arc::new(MockEmbedder::new());
    let existing_learning = make_learning("Existing snake_case learning");
    let dedup = Arc::new(MockDedup::new().with_duplicate("snake_case", existing_learning.clone()));

    // Pre-store the existing learning
    store.store(&existing_learning).await.unwrap();

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

    // Spawn consumer loop
    let handle = tokio::spawn(async move {
        let consumer = heavy_log.consumer("dedup-test").await.unwrap();
        extraction_consumer_loop::<_, _, _, _, InMemoryEventLog<ExtractionEvent>>(
            consumer,
            processor,
            None,
            shutdown_clone,
        )
        .await
    });

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Signal shutdown
    shutdown.cancel();

    // Wait for completion
    tokio::time::timeout(Duration::from_secs(1), handle)
        .await
        .expect("should complete")
        .expect("no panic");

    // Verify only 1 learning exists (merged, not new)
    assert_eq!(store.stored_count(), 1, "Should have 1 learning (merged)");
}

#[tokio::test]
async fn extraction_runs_pattern_detectors() {
    // Setup
    let heavy_log = Arc::new(InMemoryEventLog::<HeavyEvent>::new());

    // Append event without LLM candidates (to test pattern detection only)
    heavy_log.append(make_heavy_event(vec![])).await.unwrap();

    // Create processor with transcript fetcher that returns a correction pattern
    let store = Arc::new(MockStore::new());
    let embedder = Arc::new(MockEmbedder::new());
    let dedup = Arc::new(MockDedup::new());
    let transcript = make_transcript(vec![
        ("user", "No, use tabs instead of spaces"),
        ("assistant", "I'll switch to tabs from now on."),
    ]);
    let fetcher = Arc::new(MockTranscriptFetcher::new().with_transcript(transcript));
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

    // Spawn consumer loop
    let handle = tokio::spawn(async move {
        let consumer = heavy_log.consumer("pattern-test").await.unwrap();
        extraction_consumer_loop::<_, _, _, _, InMemoryEventLog<ExtractionEvent>>(
            consumer,
            processor,
            None,
            shutdown_clone,
        )
        .await
    });

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Signal shutdown
    shutdown.cancel();

    // Wait for completion
    tokio::time::timeout(Duration::from_secs(1), handle)
        .await
        .expect("should complete")
        .expect("no panic");

    // Verify pattern detector found something
    assert!(
        store.stored_count() > 0,
        "Pattern detector should have found at least one learning"
    );
}

#[tokio::test]
async fn extraction_events_have_correct_structure() {
    // Setup
    let heavy_log = Arc::new(InMemoryEventLog::<HeavyEvent>::new());
    let extraction_log = Arc::new(InMemoryEventLog::<ExtractionEvent>::new());

    // Append event
    heavy_log
        .append(make_heavy_event(vec![ExtractionCandidate::new(
            (0, 5),
            "Test learning",
            0.9,
        )]))
        .await
        .unwrap();

    // Create processor
    let store = Arc::new(MockStore::new());
    let embedder = Arc::new(MockEmbedder::new());
    let dedup = Arc::new(MockDedup::new());
    let fetcher = Arc::new(MockTranscriptFetcher::new());
    let config = ExtractionConfig::new()
        .with_poll_timeout(Duration::from_millis(50))
        .with_min_confidence(0.5);

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

    // Spawn consumer loop
    let handle = tokio::spawn(async move {
        let consumer = heavy_log.consumer("event-test").await.unwrap();
        extraction_consumer_loop(
            consumer,
            processor,
            Some(extraction_log_clone),
            shutdown_clone,
        )
        .await
    });

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Signal shutdown
    shutdown.cancel();

    // Wait for completion
    tokio::time::timeout(Duration::from_secs(1), handle)
        .await
        .expect("should complete")
        .expect("no panic");

    // Verify event structure
    let mut consumer = extraction_log.consumer("verify").await.unwrap();
    let batch = consumer.poll(10, Duration::from_millis(100)).await.unwrap();

    assert!(!batch.is_empty(), "Should have emitted events");

    for (_, event) in batch {
        match event {
            ExtractionEvent::LearningCreated {
                learning_id,
                category,
                confidence,
                source_event_id,
            } => {
                // Verify all fields are populated correctly
                assert!(!learning_id.is_nil(), "Learning ID should not be nil");
                assert_eq!(category, LearningCategory::CodePattern);
                assert!((0.0..=1.0).contains(&confidence));
                assert!(source_event_id.is_some(), "Source event ID should be set");
            }
            ExtractionEvent::LearningMerged { .. } => {
                // This test doesn't set up duplicates, so we shouldn't see this
            }
            ExtractionEvent::ExtractionFailed { .. } => {
                panic!("Should not have extraction failures in this test");
            }
        }
    }
}

#[tokio::test]
async fn extraction_respects_confidence_threshold() {
    // Setup
    let heavy_log = Arc::new(InMemoryEventLog::<HeavyEvent>::new());

    // Append events with different confidence levels
    heavy_log
        .append(make_heavy_event(vec![
            ExtractionCandidate::new((0, 5), "High confidence", 0.9),
            ExtractionCandidate::new((5, 10), "Low confidence", 0.3),
        ]))
        .await
        .unwrap();

    // Create processor with 0.5 confidence threshold
    let store = Arc::new(MockStore::new());
    let embedder = Arc::new(MockEmbedder::new());
    let dedup = Arc::new(MockDedup::new());
    let fetcher = Arc::new(MockTranscriptFetcher::new());
    let config = ExtractionConfig::new()
        .with_poll_timeout(Duration::from_millis(50))
        .with_min_confidence(0.5);

    let processor = Arc::new(ExtractionConsumer::new(
        store.clone(),
        embedder,
        dedup,
        fetcher,
        config,
    ));

    let shutdown = CancellationToken::new();
    let shutdown_clone = shutdown.clone();

    // Spawn consumer loop
    let handle = tokio::spawn(async move {
        let consumer = heavy_log.consumer("confidence-test").await.unwrap();
        extraction_consumer_loop::<_, _, _, _, InMemoryEventLog<ExtractionEvent>>(
            consumer,
            processor,
            None,
            shutdown_clone,
        )
        .await
    });

    // Wait for processing
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Signal shutdown
    shutdown.cancel();

    // Wait for completion
    tokio::time::timeout(Duration::from_secs(1), handle)
        .await
        .expect("should complete")
        .expect("no panic");

    // Only high confidence learning should be stored
    assert_eq!(
        store.stored_count(),
        1,
        "Only high confidence learning should be stored"
    );
}
