//! Correction pattern detector
//!
//! Detects user corrections in transcripts to extract preference learnings.

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::Result;
use crate::assessment::{EventId, SessionId};
use crate::capture::ParsedTranscript;
use crate::extraction::{ExtractionMethod, ExtractionSource, LearningCandidate, PatternType};

/// Default correction patterns
const DEFAULT_PATTERNS: &[&str] = &[
    r"^[Nn]o,?\s+",           // "No, ..."
    r"^[Aa]ctually,?\s+",     // "Actually, ..."
    r"[Ii] meant\s+",         // "I meant ..."
    r"[Uu]se\s+.+\s+instead", // "use X instead"
    r"[Pp]lease don't",       // "Please don't..."
];

/// Acknowledgment patterns that indicate Claude understood the correction
const ACKNOWLEDGMENT_PATTERNS: &[&str] = &[
    r"(?i)i'll use",
    r"(?i)got it",
    r"(?i)understood",
    r"(?i)you're right",
    r"(?i)i'll .+ instead",
    r"(?i)switching to",
    r"(?i)from now on",
];

/// Configuration for the correction detector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionConfig {
    /// Whether correction detection is enabled
    pub enabled: bool,
    /// Custom patterns to match (in addition to defaults)
    pub patterns: Vec<String>,
    /// Minimum confidence threshold for extracted corrections
    pub min_confidence: f64,
}

impl Default for CorrectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            patterns: Vec::new(),
            min_confidence: 0.5,
        }
    }
}

/// Detects correction patterns in transcripts
pub struct CorrectionDetector {
    patterns: Vec<Regex>,
    acknowledgment_patterns: Vec<Regex>,
    min_confidence: f64,
}

impl CorrectionDetector {
    /// Create a new CorrectionDetector with default patterns
    pub fn new() -> Self {
        Self::with_config(&CorrectionConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: &CorrectionConfig) -> Self {
        let mut patterns: Vec<Regex> = DEFAULT_PATTERNS
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();

        // Add custom patterns
        for custom in &config.patterns {
            if let Ok(re) = Regex::new(custom) {
                patterns.push(re);
            }
        }

        let acknowledgment_patterns: Vec<Regex> = ACKNOWLEDGMENT_PATTERNS
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();

        Self {
            patterns,
            acknowledgment_patterns,
            min_confidence: config.min_confidence,
        }
    }

    /// Detect corrections in a transcript
    pub fn detect(&self, transcript: &ParsedTranscript) -> Result<Vec<LearningCandidate>> {
        let mut candidates = Vec::new();

        let messages = &transcript.messages;
        for (i, msg) in messages.iter().enumerate() {
            // Only check user messages
            if msg.role != "user" {
                continue;
            }

            // Check if message matches any correction pattern
            let matched_pattern = self.patterns.iter().any(|p| p.is_match(&msg.content));
            if !matched_pattern {
                continue;
            }

            // Look for assistant acknowledgment in next message
            let has_acknowledgment = if i + 1 < messages.len() {
                let next_msg = &messages[i + 1];
                next_msg.role == "assistant"
                    && self
                        .acknowledgment_patterns
                        .iter()
                        .any(|p| p.is_match(&next_msg.content))
            } else {
                false
            };

            // Calculate confidence based on acknowledgment
            let base_confidence = 0.6;
            let ack_bonus = if has_acknowledgment { 0.25 } else { 0.0 };
            let confidence = base_confidence + ack_bonus;

            // Skip if below minimum confidence
            if confidence < self.min_confidence {
                continue;
            }

            // Extract the correction description
            let description = self.extract_description(&msg.content);

            // Create extraction source
            let end_idx = if has_acknowledgment { i + 1 } else { i };
            let source = ExtractionSource::new(
                SessionId::from(transcript.session_id.as_str()),
                EventId::new(),
                ExtractionMethod::Pattern(PatternType::Correction),
            )
            .with_message_range(i as u32, end_idx as u32);

            // Create learning candidate
            let candidate = LearningCandidate::new(
                description.clone(),
                format!("User prefers: {}", description),
                confidence,
                source,
            );

            candidates.push(candidate);
        }

        Ok(candidates)
    }

    /// Extract a meaningful description from the correction message
    fn extract_description(&self, content: &str) -> String {
        // Remove common correction prefixes
        let cleaned = content
            .trim_start_matches(|c: char| c.is_whitespace())
            .trim_start_matches("No,")
            .trim_start_matches("no,")
            .trim_start_matches("Actually,")
            .trim_start_matches("actually,")
            .trim_start_matches("I meant")
            .trim_start_matches("i meant")
            .trim();

        // Take first sentence or up to 100 chars
        let desc = if let Some(period_idx) = cleaned.find('.') {
            &cleaned[..period_idx]
        } else if cleaned.len() > 100 {
            &cleaned[..100]
        } else {
            cleaned
        };

        desc.trim().to_string()
    }
}

impl Default for CorrectionDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::TranscriptMessage;

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
            metadata: Default::default(),
        }
    }

    // --- Pattern matching tests ---

    #[test]
    fn test_detects_no_correction_pattern() {
        let detector = CorrectionDetector::new();
        let transcript = make_transcript(vec![
            ("user", "No, use tabs not spaces"),
            ("assistant", "I'll use tabs instead of spaces."),
        ]);

        let candidates = detector.detect(&transcript).unwrap();

        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].description.contains("tabs"));
    }

    #[test]
    fn test_detects_actually_correction_pattern() {
        let detector = CorrectionDetector::new();
        let transcript = make_transcript(vec![
            ("user", "Actually, I want TypeScript not JavaScript"),
            ("assistant", "I'll use TypeScript for this project."),
        ]);

        let candidates = detector.detect(&transcript).unwrap();

        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].description.contains("TypeScript"));
    }

    #[test]
    fn test_detects_i_meant_pattern() {
        let detector = CorrectionDetector::new();
        let transcript = make_transcript(vec![
            ("user", "I meant the other file"),
            ("assistant", "Got it, I'll work on the other file."),
        ]);

        let candidates = detector.detect(&transcript).unwrap();

        assert_eq!(candidates.len(), 1);
    }

    #[test]
    fn test_detects_use_instead_pattern() {
        let detector = CorrectionDetector::new();
        let transcript = make_transcript(vec![
            ("user", "Use async/await instead of callbacks"),
            ("assistant", "I'll refactor to use async/await."),
        ]);

        let candidates = detector.detect(&transcript).unwrap();

        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].description.contains("async/await"));
    }

    #[test]
    fn test_no_detection_without_correction() {
        let detector = CorrectionDetector::new();
        let transcript = make_transcript(vec![
            ("user", "Can you help me write a function?"),
            ("assistant", "Sure, I'll help you write a function."),
        ]);

        let candidates = detector.detect(&transcript).unwrap();

        assert!(candidates.is_empty());
    }

    // --- Confidence tests ---

    #[test]
    fn test_higher_confidence_with_acknowledgment() {
        let detector = CorrectionDetector::new();

        // With clear acknowledgment
        let with_ack = make_transcript(vec![
            ("user", "No, use snake_case"),
            (
                "assistant",
                "You're right, I'll use snake_case from now on.",
            ),
        ]);

        // Without acknowledgment
        let without_ack = make_transcript(vec![
            ("user", "No, use snake_case"),
            ("assistant", "Here's the code."),
        ]);

        let candidates_with = detector.detect(&with_ack).unwrap();
        let candidates_without = detector.detect(&without_ack).unwrap();

        assert!(!candidates_with.is_empty());
        assert!(!candidates_without.is_empty());
        assert!(candidates_with[0].confidence > candidates_without[0].confidence);
    }

    // --- Configuration tests ---

    #[test]
    fn test_custom_pattern() {
        let config = CorrectionConfig {
            enabled: true,
            patterns: vec![r"^[Pp]lease don't".to_string()],
            min_confidence: 0.5,
        };
        let detector = CorrectionDetector::with_config(&config);

        let transcript = make_transcript(vec![
            ("user", "Please don't use var, use const"),
            ("assistant", "I'll use const instead."),
        ]);

        let candidates = detector.detect(&transcript).unwrap();

        assert_eq!(candidates.len(), 1);
    }

    #[test]
    fn test_min_confidence_filter() {
        let config = CorrectionConfig {
            enabled: true,
            patterns: Vec::new(),
            min_confidence: 0.9, // Very high threshold
        };
        let detector = CorrectionDetector::with_config(&config);

        // Weak correction without acknowledgment
        let transcript = make_transcript(vec![
            ("user", "No, the other one"),
            ("assistant", "Here it is."),
        ]);

        let candidates = detector.detect(&transcript).unwrap();

        // Should filter out low-confidence detections
        assert!(candidates.is_empty());
    }

    // --- Extraction source tests ---

    #[test]
    fn test_extraction_source_includes_message_range() {
        let detector = CorrectionDetector::new();
        let transcript = make_transcript(vec![
            ("user", "Hello"),
            ("assistant", "Hi there!"),
            ("user", "No, use Python instead"),
            ("assistant", "I'll use Python."),
        ]);

        let candidates = detector.detect(&transcript).unwrap();

        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].source.message_range.is_some());
        let (start, end) = candidates[0].source.message_range.unwrap();
        assert_eq!(start, 2); // User correction message index
        assert_eq!(end, 3); // Assistant acknowledgment index
    }

    // --- Multiple corrections tests ---

    #[test]
    fn test_detects_multiple_corrections() {
        let detector = CorrectionDetector::new();
        let transcript = make_transcript(vec![
            ("user", "No, use tabs"),
            ("assistant", "I'll use tabs."),
            ("user", "Actually, prefer const over let"),
            ("assistant", "Got it, I'll use const."),
        ]);

        let candidates = detector.detect(&transcript).unwrap();

        assert_eq!(candidates.len(), 2);
    }
}
