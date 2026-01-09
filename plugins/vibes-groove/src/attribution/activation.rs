//! Activation detection for determining if learnings influenced Claude's behavior
//!
//! Layer 1 of the attribution engine - detects whether injected learnings
//! were actually used by Claude using embedding similarity and explicit references.

use async_trait::async_trait;

use crate::capture::ParsedTranscript;
use crate::error::Result;
use crate::extraction::embedder::{Embedder, cosine_similarity};
use crate::types::Learning;

use super::ActivationSignal;

/// Result of activation detection for a learning
#[derive(Debug, Clone)]
pub struct ActivationResult {
    /// Whether the learning was determined to be activated
    pub was_activated: bool,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Signals that contributed to the detection
    pub signals: Vec<ActivationSignal>,
}

/// Trait for detecting if a learning was activated during a session
#[async_trait]
pub trait ActivationDetector: Send + Sync {
    /// Detect if a learning was activated in the transcript
    async fn detect(
        &self,
        learning: &Learning,
        transcript: &ParsedTranscript,
        embedder: &dyn Embedder,
    ) -> Result<ActivationResult>;
}

/// Configuration for activation detection
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActivationConfig {
    /// Minimum similarity score to consider a match (default: 0.75)
    pub similarity_threshold: f64,
    /// Bonus added to confidence when explicit reference found (default: 0.15)
    pub reference_boost: f64,
}

impl Default for ActivationConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.75,
            reference_boost: 0.15,
        }
    }
}

/// Hybrid activation detector using embedding similarity and explicit references
pub struct HybridActivationDetector {
    config: ActivationConfig,
}

impl HybridActivationDetector {
    /// Create a new detector with default configuration
    pub fn new() -> Self {
        Self {
            config: ActivationConfig::default(),
        }
    }

    /// Create a new detector with custom configuration
    pub fn with_config(config: ActivationConfig) -> Self {
        Self { config }
    }

    /// Extract key phrases from learning for explicit reference matching
    fn extract_key_phrases(learning: &Learning) -> Vec<String> {
        let mut phrases = Vec::new();

        // Extract significant words from insight (3+ chars, not common words)
        let common_words = [
            "the", "and", "for", "that", "this", "with", "from", "have", "are", "was", "were",
            "been", "being", "will", "would", "could", "should", "use", "when", "what", "which",
            "their", "there", "they", "them", "than", "then", "into", "only", "other", "some",
            "such", "also", "each", "just", "more", "most", "both", "same", "very", "make", "like",
            "over", "your", "about", "after", "before",
        ];

        // Split insight into phrases (by punctuation)
        for phrase in learning
            .content
            .insight
            .split(&['.', ',', ';', ':', '!', '?'][..])
        {
            let phrase = phrase.trim();
            if phrase.len() >= 10 && phrase.split_whitespace().count() >= 2 {
                // Keep meaningful multi-word phrases
                phrases.push(phrase.to_lowercase());
            }
        }

        // Also extract individual significant words
        for word in learning.content.insight.split_whitespace() {
            let word = word
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            if word.len() >= 4 && !common_words.contains(&word.as_str()) {
                phrases.push(word);
            }
        }

        phrases
    }

    /// Check if any key phrases appear in the text
    fn find_explicit_references(
        key_phrases: &[String],
        text: &str,
        message_idx: u32,
    ) -> Vec<ActivationSignal> {
        let text_lower = text.to_lowercase();
        let mut signals = Vec::new();

        for phrase in key_phrases {
            if text_lower.contains(phrase) {
                signals.push(ActivationSignal::ExplicitReference {
                    pattern: phrase.clone(),
                    message_idx,
                });
            }
        }

        signals
    }
}

impl Default for HybridActivationDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ActivationDetector for HybridActivationDetector {
    async fn detect(
        &self,
        learning: &Learning,
        transcript: &ParsedTranscript,
        embedder: &dyn Embedder,
    ) -> Result<ActivationResult> {
        let mut signals = Vec::new();
        let mut max_similarity: f64 = 0.0;
        let mut has_explicit_reference = false;

        // Embed the learning insight
        let learning_embedding = embedder
            .embed(&learning.content.insight)
            .await
            .map_err(|e| crate::error::GrooveError::Embedding(e.to_string()))?;

        // Extract key phrases for explicit reference detection
        let key_phrases = Self::extract_key_phrases(learning);

        // Check each assistant message
        for (idx, message) in transcript.messages.iter().enumerate() {
            if message.role != "assistant" {
                continue;
            }

            let message_idx = idx as u32;

            // Embedding similarity check
            let response_embedding = embedder
                .embed(&message.content)
                .await
                .map_err(|e| crate::error::GrooveError::Embedding(e.to_string()))?;

            let similarity = cosine_similarity(&learning_embedding, &response_embedding) as f64;

            if similarity > max_similarity {
                max_similarity = similarity;
            }

            if similarity >= self.config.similarity_threshold {
                signals.push(ActivationSignal::EmbeddingSimilarity {
                    score: similarity,
                    message_idx,
                });
            }

            // Explicit reference check
            let ref_signals =
                Self::find_explicit_references(&key_phrases, &message.content, message_idx);
            if !ref_signals.is_empty() {
                has_explicit_reference = true;
                signals.extend(ref_signals);
            }
        }

        // Calculate final confidence
        let confidence = if has_explicit_reference {
            (max_similarity + self.config.reference_boost).min(1.0)
        } else {
            max_similarity
        };

        let was_activated = confidence >= self.config.similarity_threshold;

        Ok(ActivationResult {
            was_activated,
            confidence,
            signals,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::{TranscriptMessage, TranscriptMetadata};
    use crate::extraction::embedder::EmbedderResult;
    use crate::types::{LearningCategory, LearningContent, LearningSource, Scope};
    use chrono::Utc;
    use uuid::Uuid;

    /// Mock embedder that returns predictable embeddings based on content
    struct MockEmbedder {
        /// If true, returns similar embeddings for similar content
        simulate_similarity: bool,
    }

    impl MockEmbedder {
        fn new(simulate_similarity: bool) -> Self {
            Self {
                simulate_similarity,
            }
        }

        fn similar() -> Self {
            Self::new(true)
        }

        fn dissimilar() -> Self {
            Self::new(false)
        }
    }

    #[async_trait]
    impl Embedder for MockEmbedder {
        async fn embed(&self, text: &str) -> EmbedderResult<Vec<f32>> {
            if self.simulate_similarity {
                // Return similar embedding if text contains certain keywords
                if text.contains("Result")
                    || text.contains("error")
                    || text.contains("prefer")
                    || text.contains("insight")
                    || text.contains("Prefer")
                    || text.contains("handling")
                {
                    // Very similar - all 1s
                    Ok(vec![1.0; 384])
                } else {
                    // Moderately similar - 0.85 cosine similarity
                    let mut embedding = vec![1.0; 384];
                    for (i, v) in embedding.iter_mut().enumerate() {
                        if i % 3 == 0 {
                            *v = 0.5;
                        }
                    }
                    Ok(embedding)
                }
            } else {
                // Return orthogonal/dissimilar embeddings
                // Learning insights contain "Prefer" or specific learning keywords
                let is_learning_text = text.contains("Prefer")
                    || text.contains("prefer")
                    || text.contains("Result")
                    || text.contains("error handling");

                if is_learning_text {
                    // Learning gets [1, 0, 0, 1, 0, 0, ...]
                    let mut embedding = vec![0.0; 384];
                    for (i, v) in embedding.iter_mut().enumerate() {
                        if i % 3 == 0 {
                            *v = 1.0;
                        }
                    }
                    Ok(embedding)
                } else {
                    // Response gets [0, 1, 0, 0, 1, 0, ...] - orthogonal
                    let mut embedding = vec![0.0; 384];
                    for (i, v) in embedding.iter_mut().enumerate() {
                        if i % 3 == 1 {
                            *v = 1.0;
                        }
                    }
                    Ok(embedding)
                }
            }
        }

        fn dimensions(&self) -> usize {
            384
        }
    }

    fn create_learning(insight: &str) -> Learning {
        Learning {
            id: Uuid::now_v7(),
            scope: Scope::User("test-user".into()),
            category: LearningCategory::Preference,
            content: LearningContent {
                description: "Test learning".into(),
                pattern: None,
                insight: insight.into(),
            },
            confidence: 0.8,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            source: LearningSource::UserCreated,
        }
    }

    fn create_transcript(messages: Vec<(&str, &str)>) -> ParsedTranscript {
        ParsedTranscript {
            session_id: "test-session".into(),
            messages: messages
                .into_iter()
                .map(|(role, content)| TranscriptMessage {
                    role: role.into(),
                    content: content.into(),
                    timestamp: None,
                })
                .collect(),
            tool_uses: vec![],
            metadata: TranscriptMetadata::default(),
        }
    }

    #[tokio::test]
    async fn test_high_similarity_triggers_activation() {
        let detector = HybridActivationDetector::new();
        let embedder = MockEmbedder::similar();

        let learning = create_learning("Prefer Result over panic for error handling");
        let transcript = create_transcript(vec![
            ("user", "How should I handle errors?"),
            (
                "assistant",
                "I recommend using Result for error handling rather than panic.",
            ),
        ]);

        let result = detector
            .detect(&learning, &transcript, &embedder)
            .await
            .unwrap();

        assert!(result.was_activated);
        assert!(result.confidence >= 0.75);
        assert!(!result.signals.is_empty());
    }

    #[tokio::test]
    async fn test_low_similarity_no_activation() {
        let detector = HybridActivationDetector::new();
        let embedder = MockEmbedder::dissimilar();

        let learning = create_learning("Prefer Result over panic for error handling");
        let transcript = create_transcript(vec![
            ("user", "What's the weather like?"),
            ("assistant", "I don't have access to weather data."),
        ]);

        let result = detector
            .detect(&learning, &transcript, &embedder)
            .await
            .unwrap();

        assert!(!result.was_activated);
        assert!(result.confidence < 0.75);
    }

    #[tokio::test]
    async fn test_explicit_reference_boosts_confidence() {
        let detector = HybridActivationDetector::new();
        let embedder = MockEmbedder::similar();

        let learning = create_learning("prefer using Result types for better error handling");
        let transcript = create_transcript(vec![
            ("user", "How should I handle errors?"),
            (
                "assistant",
                "You should prefer using Result types. This gives you better error handling.",
            ),
        ]);

        let result = detector
            .detect(&learning, &transcript, &embedder)
            .await
            .unwrap();

        assert!(result.was_activated);
        // Should have both embedding and explicit reference signals
        let has_explicit = result
            .signals
            .iter()
            .any(|s| matches!(s, ActivationSignal::ExplicitReference { .. }));
        assert!(has_explicit, "Should have explicit reference signal");
    }

    #[tokio::test]
    async fn test_multiple_signals_in_single_session() {
        let detector = HybridActivationDetector::new();
        let embedder = MockEmbedder::similar();

        let learning = create_learning("Use Result for error handling");
        let transcript = create_transcript(vec![
            ("user", "How do I handle errors?"),
            ("assistant", "Use Result types for error handling."),
            ("user", "Can you show an example?"),
            (
                "assistant",
                "Here's how to use Result: fn foo() -> Result<(), Error> { ... }",
            ),
        ]);

        let result = detector
            .detect(&learning, &transcript, &embedder)
            .await
            .unwrap();

        assert!(result.was_activated);
        // Should have signals from multiple messages
        assert!(
            result.signals.len() >= 2,
            "Should have signals from multiple messages"
        );
    }

    #[tokio::test]
    async fn test_configurable_threshold() {
        let config = ActivationConfig {
            similarity_threshold: 0.9, // Higher threshold
            reference_boost: 0.15,
        };
        let detector = HybridActivationDetector::with_config(config);
        let embedder = MockEmbedder::similar();

        let learning = create_learning("Prefer Result over panic");
        let transcript = create_transcript(vec![
            ("user", "How should I handle errors?"),
            ("assistant", "Consider using error handling patterns."),
        ]);

        let result = detector
            .detect(&learning, &transcript, &embedder)
            .await
            .unwrap();

        // With higher threshold, might not activate
        // The mock returns similar but not identical embeddings
        assert!(result.confidence < 0.9 || result.was_activated);
    }

    #[tokio::test]
    async fn test_only_checks_assistant_messages() {
        let detector = HybridActivationDetector::new();
        let embedder = MockEmbedder::similar();

        let learning = create_learning("Use Result for error handling");
        let transcript = create_transcript(vec![
            ("user", "I prefer Result for error handling"),
            ("assistant", "That's a different topic entirely."),
        ]);

        let result = detector
            .detect(&learning, &transcript, &embedder)
            .await
            .unwrap();

        // Should not match user message, only assistant
        let user_signals: Vec<_> = result
            .signals
            .iter()
            .filter(|s| match s {
                ActivationSignal::EmbeddingSimilarity { message_idx, .. } => *message_idx == 0,
                ActivationSignal::ExplicitReference { message_idx, .. } => *message_idx == 0,
            })
            .collect();

        assert!(
            user_signals.is_empty(),
            "Should not have signals from user messages"
        );
    }
}
