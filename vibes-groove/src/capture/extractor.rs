//! Learning extraction from parsed transcripts
//!
//! This module provides a stub extractor that identifies learnable events
//! from parsed transcripts. AI-based analysis will be added in Milestone 4.5.

use serde::{Deserialize, Serialize};

use super::parser::ParsedTranscript;

/// Categories of learnings that can be extracted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearningCategory {
    /// Code patterns and idioms used
    Pattern,
    /// Development techniques applied
    Technique,
    /// User preferences discovered
    Preference,
    /// Contextual information about the project
    Context,
}

/// A learning extracted from a transcript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedLearning {
    /// The learning content text
    pub content: String,
    /// Category of this learning
    pub category: LearningCategory,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Tool that generated this learning, if applicable
    pub source_tool: Option<String>,
}

impl ExtractedLearning {
    /// Create a new extracted learning
    pub fn new(
        content: String,
        category: LearningCategory,
        confidence: f32,
        source_tool: Option<String>,
    ) -> Self {
        Self {
            content,
            category,
            confidence: confidence.clamp(0.0, 1.0),
            source_tool,
        }
    }
}

/// Extracts learnings from parsed transcripts
///
/// This is currently a stub implementation. AI-based extraction
/// will be added in Milestone 4.5.
pub struct LearningExtractor {
    /// Minimum confidence threshold for learnings
    min_confidence: f32,
}

impl Default for LearningExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl LearningExtractor {
    /// Create a new learning extractor
    pub fn new() -> Self {
        Self {
            min_confidence: 0.5,
        }
    }

    /// Create an extractor with a custom minimum confidence threshold
    pub fn with_min_confidence(min_confidence: f32) -> Self {
        Self {
            min_confidence: min_confidence.clamp(0.0, 1.0),
        }
    }

    /// Get the minimum confidence threshold
    pub fn min_confidence(&self) -> f32 {
        self.min_confidence
    }

    /// Extract learnings from a parsed transcript
    ///
    /// Currently returns an empty vec as a stub. AI-based extraction
    /// will be implemented in Milestone 4.5.
    pub fn extract(&self, _transcript: &ParsedTranscript) -> Vec<ExtractedLearning> {
        // Stub: Return empty vec for now
        // In Milestone 4.5, this will use AI to analyze the transcript
        // and extract meaningful learnings
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::parser::TranscriptParser;

    #[test]
    fn test_extractor_returns_empty_for_stub() {
        let content = r#"{"role": "user", "content": "Hello"}
{"role": "assistant", "content": "Hi there!"}"#;

        let parser = TranscriptParser::new();
        let transcript = parser.parse(content, "test-session").unwrap();

        let extractor = LearningExtractor::new();
        let learnings = extractor.extract(&transcript);

        assert!(learnings.is_empty(), "Stub should return empty vec");
    }

    #[test]
    fn test_learning_category_variants() {
        // Verify all category variants exist and can be matched
        let categories = [
            LearningCategory::Pattern,
            LearningCategory::Technique,
            LearningCategory::Preference,
            LearningCategory::Context,
        ];

        for category in categories {
            match category {
                LearningCategory::Pattern => assert_eq!(category, LearningCategory::Pattern),
                LearningCategory::Technique => assert_eq!(category, LearningCategory::Technique),
                LearningCategory::Preference => assert_eq!(category, LearningCategory::Preference),
                LearningCategory::Context => assert_eq!(category, LearningCategory::Context),
            }
        }
    }

    #[test]
    fn test_extracted_learning_construction() {
        let learning = ExtractedLearning::new(
            "Use TDD for components".to_string(),
            LearningCategory::Technique,
            0.85,
            Some("Bash".to_string()),
        );

        assert_eq!(learning.content, "Use TDD for components");
        assert_eq!(learning.category, LearningCategory::Technique);
        assert!((learning.confidence - 0.85).abs() < f32::EPSILON);
        assert_eq!(learning.source_tool, Some("Bash".to_string()));
    }

    #[test]
    fn test_confidence_is_clamped() {
        let too_high =
            ExtractedLearning::new("Test".to_string(), LearningCategory::Pattern, 1.5, None);
        assert!((too_high.confidence - 1.0).abs() < f32::EPSILON);

        let too_low =
            ExtractedLearning::new("Test".to_string(), LearningCategory::Pattern, -0.5, None);
        assert!(too_low.confidence.abs() < f32::EPSILON);
    }

    #[test]
    fn test_extractor_with_custom_min_confidence() {
        let extractor = LearningExtractor::with_min_confidence(0.8);
        assert!((extractor.min_confidence() - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_default_min_confidence() {
        let extractor = LearningExtractor::new();
        assert!((extractor.min_confidence() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_learning_serialization() {
        let learning = ExtractedLearning::new(
            "Prefer explicit types".to_string(),
            LearningCategory::Preference,
            0.9,
            None,
        );

        let json = serde_json::to_string(&learning).unwrap();
        let deserialized: ExtractedLearning = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.content, learning.content);
        assert_eq!(deserialized.category, learning.category);
    }
}
