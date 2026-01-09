//! Extraction pipeline types
//!
//! Types specific to the learning extraction pipeline, including events for Iggy
//! audit trail and extraction methods.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::assessment::{EventId, SessionId};
use crate::types::LearningId;

/// Method used to extract a learning
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtractionMethod {
    /// Extracted via pattern detector
    Pattern(PatternType),
    /// Extracted via LLM analysis
    Llm,
}

/// Type of pattern detector that found the learning
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    /// User correction pattern ("No, use X instead of Y")
    Correction,
    /// Error recovery pattern (failure → fix → success)
    ErrorRecovery,
}

/// Source information for extracted learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionSource {
    /// Session where extraction occurred
    pub session_id: SessionId,
    /// Heavy assessment event that triggered extraction
    pub event_id: EventId,
    /// Message range in transcript (start, end)
    pub message_range: Option<(u32, u32)>,
    /// How this learning was extracted
    pub extraction_method: ExtractionMethod,
}

impl ExtractionSource {
    /// Create a new extraction source
    pub fn new(
        session_id: SessionId,
        event_id: EventId,
        extraction_method: ExtractionMethod,
    ) -> Self {
        Self {
            session_id,
            event_id,
            message_range: None,
            extraction_method,
        }
    }

    /// Set message range
    pub fn with_message_range(mut self, start: u32, end: u32) -> Self {
        self.message_range = Some((start, end));
        self
    }
}

/// Raw extraction event written to Iggy for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionEvent {
    /// Unique event ID
    pub event_id: EventId,
    /// When extraction occurred
    pub timestamp: DateTime<Utc>,
    /// Which heavy assessment event triggered this
    pub source_heavy_event: EventId,
    /// Number of candidates processed
    pub candidates_processed: u32,
    /// Learnings successfully created
    pub learnings_created: Vec<LearningId>,
    /// Learnings merged (new_id, merged_into_id)
    pub learnings_merged: Vec<(LearningId, LearningId)>,
    /// Learnings rejected (below confidence threshold)
    pub learnings_rejected: u32,
}

impl ExtractionEvent {
    /// Create a new extraction event
    pub fn new(source_heavy_event: EventId) -> Self {
        Self {
            event_id: EventId::new(),
            timestamp: Utc::now(),
            source_heavy_event,
            candidates_processed: 0,
            learnings_created: Vec::new(),
            learnings_merged: Vec::new(),
            learnings_rejected: 0,
        }
    }

    /// Record a learning creation
    pub fn record_created(&mut self, learning_id: LearningId) {
        self.learnings_created.push(learning_id);
    }

    /// Record a learning merge
    pub fn record_merged(&mut self, new_id: LearningId, merged_into: LearningId) {
        self.learnings_merged.push((new_id, merged_into));
    }

    /// Record a rejected candidate
    pub fn record_rejected(&mut self) {
        self.learnings_rejected += 1;
    }

    /// Set total candidates processed
    pub fn set_candidates_processed(&mut self, count: u32) {
        self.candidates_processed = count;
    }
}

/// A candidate learning before validation and deduplication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningCandidate {
    /// Human-readable description
    pub description: String,
    /// What triggers this learning
    pub pattern: Option<serde_json::Value>,
    /// Actionable insight
    pub insight: String,
    /// Extraction confidence (0.0-1.0)
    pub confidence: f64,
    /// Extraction source info
    pub source: ExtractionSource,
    /// Pre-computed embedding (optional, computed if missing)
    pub embedding: Option<Vec<f32>>,
}

impl LearningCandidate {
    /// Create a new learning candidate
    pub fn new(
        description: impl Into<String>,
        insight: impl Into<String>,
        confidence: f64,
        source: ExtractionSource,
    ) -> Self {
        Self {
            description: description.into(),
            pattern: None,
            insight: insight.into(),
            confidence,
            source,
            embedding: None,
        }
    }

    /// Set pattern data
    pub fn with_pattern(mut self, pattern: serde_json::Value) -> Self {
        self.pattern = Some(pattern);
        self
    }

    /// Set pre-computed embedding
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_extraction_method_pattern() {
        let method = ExtractionMethod::Pattern(PatternType::Correction);
        assert_eq!(method, ExtractionMethod::Pattern(PatternType::Correction));
    }

    #[test]
    fn test_extraction_method_llm() {
        let method = ExtractionMethod::Llm;
        assert_eq!(method, ExtractionMethod::Llm);
    }

    #[test]
    fn test_pattern_type_variants() {
        let correction = PatternType::Correction;
        let error_recovery = PatternType::ErrorRecovery;
        assert_ne!(correction, error_recovery);
    }

    #[test]
    fn test_extraction_source_new() {
        let source = ExtractionSource::new(
            SessionId::from("session-123"),
            EventId::new(),
            ExtractionMethod::Llm,
        );
        assert_eq!(source.session_id.as_str(), "session-123");
        assert!(source.message_range.is_none());
    }

    #[test]
    fn test_extraction_source_with_message_range() {
        let source = ExtractionSource::new(
            SessionId::from("session-123"),
            EventId::new(),
            ExtractionMethod::Llm,
        )
        .with_message_range(5, 10);

        assert_eq!(source.message_range, Some((5, 10)));
    }

    #[test]
    fn test_extraction_event_new() {
        let source_event = EventId::new();
        let event = ExtractionEvent::new(source_event);

        assert_eq!(event.source_heavy_event, source_event);
        assert_eq!(event.candidates_processed, 0);
        assert!(event.learnings_created.is_empty());
        assert!(event.learnings_merged.is_empty());
        assert_eq!(event.learnings_rejected, 0);
    }

    #[test]
    fn test_extraction_event_record_created() {
        let mut event = ExtractionEvent::new(EventId::new());
        let learning_id = Uuid::now_v7();

        event.record_created(learning_id);

        assert_eq!(event.learnings_created.len(), 1);
        assert_eq!(event.learnings_created[0], learning_id);
    }

    #[test]
    fn test_extraction_event_record_merged() {
        let mut event = ExtractionEvent::new(EventId::new());
        let new_id = Uuid::now_v7();
        let merged_into = Uuid::now_v7();

        event.record_merged(new_id, merged_into);

        assert_eq!(event.learnings_merged.len(), 1);
        assert_eq!(event.learnings_merged[0], (new_id, merged_into));
    }

    #[test]
    fn test_extraction_event_record_rejected() {
        let mut event = ExtractionEvent::new(EventId::new());

        event.record_rejected();
        event.record_rejected();

        assert_eq!(event.learnings_rejected, 2);
    }

    #[test]
    fn test_learning_candidate_new() {
        let source = ExtractionSource::new(
            SessionId::from("session-123"),
            EventId::new(),
            ExtractionMethod::Pattern(PatternType::Correction),
        );
        let candidate = LearningCandidate::new(
            "Use snake_case for variables",
            "Prefer snake_case over camelCase in Rust",
            0.85,
            source,
        );

        assert_eq!(candidate.description, "Use snake_case for variables");
        assert_eq!(candidate.confidence, 0.85);
        assert!(candidate.pattern.is_none());
        assert!(candidate.embedding.is_none());
    }

    #[test]
    fn test_learning_candidate_with_pattern() {
        let source = ExtractionSource::new(
            SessionId::from("session-123"),
            EventId::new(),
            ExtractionMethod::Llm,
        );
        let pattern = serde_json::json!({
            "trigger": "variable naming",
            "language": "rust"
        });
        let candidate = LearningCandidate::new("Use snake_case", "Prefer snake_case", 0.9, source)
            .with_pattern(pattern.clone());

        assert_eq!(candidate.pattern, Some(pattern));
    }

    #[test]
    fn test_learning_candidate_with_embedding() {
        let source = ExtractionSource::new(
            SessionId::from("session-123"),
            EventId::new(),
            ExtractionMethod::Llm,
        );
        let embedding = vec![0.1, 0.2, 0.3];
        let candidate = LearningCandidate::new("Test", "Test insight", 0.5, source)
            .with_embedding(embedding.clone());

        assert_eq!(candidate.embedding, Some(embedding));
    }

    #[test]
    fn test_extraction_event_serialization() {
        let event = ExtractionEvent::new(EventId::new());
        let json = serde_json::to_string(&event).unwrap();
        let parsed: ExtractionEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.event_id, event.event_id);
        assert_eq!(parsed.source_heavy_event, event.source_heavy_event);
    }

    #[test]
    fn test_learning_candidate_serialization() {
        let source = ExtractionSource::new(
            SessionId::from("session-123"),
            EventId::new(),
            ExtractionMethod::Llm,
        );
        let candidate = LearningCandidate::new("Test description", "Test insight", 0.75, source);

        let json = serde_json::to_string(&candidate).unwrap();
        let parsed: LearningCandidate = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.description, candidate.description);
        assert_eq!(parsed.confidence, candidate.confidence);
    }
}
