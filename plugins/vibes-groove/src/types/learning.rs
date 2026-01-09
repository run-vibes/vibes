//! Learning types for the continual learning system

use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Scope;

/// UUIDv7 provides time-ordered unique identifiers
pub type LearningId = Uuid;

/// A captured piece of knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Learning {
    pub id: LearningId,
    pub scope: Scope,
    pub category: LearningCategory,
    pub content: LearningContent,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source: LearningSource,
}

impl Learning {
    /// Create a new learning with generated ID and timestamps
    pub fn new(
        scope: Scope,
        category: LearningCategory,
        content: LearningContent,
        source: LearningSource,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            scope,
            category,
            content,
            confidence: 0.5, // Neutral starting confidence
            created_at: now,
            updated_at: now,
            source,
        }
    }
}

/// Category of learning for filtering and organization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LearningCategory {
    /// User corrected Claude's behavior
    Correction,
    /// Detected code patterns worth remembering
    CodePattern,
    /// User preferences
    Preference,
    /// Solutions to specific problems
    Solution,
    /// Successful error recovery strategies
    ErrorRecovery,
    /// Tool usage patterns
    ToolUsage,
    /// Knowledge about the harness (Claude Code, etc.)
    HarnessKnowledge,
}

impl LearningCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Correction => "correction",
            Self::CodePattern => "code_pattern",
            Self::Preference => "preference",
            Self::Solution => "solution",
            Self::ErrorRecovery => "error_recovery",
            Self::ToolUsage => "tool_usage",
            Self::HarnessKnowledge => "harness_knowledge",
        }
    }
}

/// Error type for parsing LearningCategory from string
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseLearningCategoryError(String);

impl std::fmt::Display for ParseLearningCategoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown learning category: {}", self.0)
    }
}

impl std::error::Error for ParseLearningCategoryError {}

impl FromStr for LearningCategory {
    type Err = ParseLearningCategoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "correction" => Ok(Self::Correction),
            "code_pattern" => Ok(Self::CodePattern),
            "preference" => Ok(Self::Preference),
            "solution" => Ok(Self::Solution),
            "error_recovery" => Ok(Self::ErrorRecovery),
            "tool_usage" => Ok(Self::ToolUsage),
            "harness_knowledge" => Ok(Self::HarnessKnowledge),
            _ => Err(ParseLearningCategoryError(s.to_string())),
        }
    }
}

/// The actual content of a learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningContent {
    /// Human-readable description of what was learned
    pub description: String,

    /// Structured pattern data (flexible JSON for different pattern types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<serde_json::Value>,

    /// Actionable insight for injection into sessions
    pub insight: String,
}

/// Where this learning originated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningSource {
    Transcript {
        session_id: String,
        message_index: usize,
    },
    UserCreated,
    Promoted {
        from_scope: Scope,
        original_id: LearningId,
    },
    Imported {
        source_file: String,
        imported_at: DateTime<Utc>,
    },
    EnterpriseCurated {
        curator: String,
    },
}

impl LearningSource {
    pub fn source_type(&self) -> &'static str {
        match self {
            Self::Transcript { .. } => "transcript",
            Self::UserCreated => "user_created",
            Self::Promoted { .. } => "promoted",
            Self::Imported { .. } => "imported",
            Self::EnterpriseCurated { .. } => "enterprise_curated",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learning_category_roundtrip() {
        for category in [
            LearningCategory::Correction,
            LearningCategory::CodePattern,
            LearningCategory::Preference,
            LearningCategory::Solution,
            LearningCategory::ErrorRecovery,
            LearningCategory::ToolUsage,
            LearningCategory::HarnessKnowledge,
        ] {
            let s = category.as_str();
            let parsed = LearningCategory::from_str(s).unwrap();
            assert_eq!(parsed, category);
        }
    }

    #[test]
    fn test_learning_source_type() {
        let source = LearningSource::UserCreated;
        assert_eq!(source.source_type(), "user_created");

        let source = LearningSource::Transcript {
            session_id: "sess-1".into(),
            message_index: 5,
        };
        assert_eq!(source.source_type(), "transcript");
    }

    #[test]
    fn test_learning_content_serialization() {
        let content = LearningContent {
            description: "Use Result for errors".into(),
            pattern: Some(serde_json::json!({"language": "rust"})),
            insight: "Prefer Result over panic".into(),
        };
        let json = serde_json::to_string(&content).unwrap();
        let parsed: LearningContent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.description, content.description);
    }

    #[test]
    fn test_learning_id_is_uuid_v7() {
        let id = LearningId::now_v7();
        // UUIDv7 starts with version nibble 7
        assert_eq!(id.get_version_num(), 7);
    }
}
