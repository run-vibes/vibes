//! Export/Import types for backup and portability of learnings between systems

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    Learning, LearningCategory, LearningContent, LearningId, LearningRelation, LearningSource,
    Scope, SystemParam, UsageStats,
};

/// Current version of the export format
pub const EXPORT_VERSION: u32 = 1;

/// Complete export of learnings, parameters, and relations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrooveExport {
    pub version: u32,
    pub exported_at: DateTime<Utc>,
    pub learnings: Vec<LearningExport>,
    pub params: Vec<SystemParam>,
    pub relations: Vec<LearningRelation>,
}

impl GrooveExport {
    pub fn new() -> Self {
        Self {
            version: EXPORT_VERSION,
            exported_at: Utc::now(),
            learnings: vec![],
            params: vec![],
            relations: vec![],
        }
    }
}

impl Default for GrooveExport {
    fn default() -> Self {
        Self::new()
    }
}

/// Learning data for export (without embedding vectors)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningExport {
    pub id: LearningId,
    pub scope: Scope,
    pub category: LearningCategory,
    pub content: LearningContent,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source: LearningSource,
    pub usage_stats: UsageStats,
}

impl From<Learning> for LearningExport {
    fn from(learning: Learning) -> Self {
        Self {
            id: learning.id,
            scope: learning.scope,
            category: learning.category,
            content: learning.content,
            confidence: learning.confidence,
            created_at: learning.created_at,
            updated_at: learning.updated_at,
            source: learning.source,
            usage_stats: UsageStats::default(),
        }
    }
}

/// Statistics from an import operation
#[derive(Debug, Clone, Default)]
pub struct ImportStats {
    pub learnings_imported: u32,
    pub learnings_skipped: u32,
    pub params_imported: u32,
    pub relations_imported: u32,
    pub embeddings_queued: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_version() {
        assert_eq!(EXPORT_VERSION, 1);
    }

    #[test]
    fn test_import_stats_default() {
        let stats = ImportStats::default();
        assert_eq!(stats.learnings_imported, 0);
        assert_eq!(stats.embeddings_queued, 0);
    }

    #[test]
    fn test_groove_export_serialization() {
        let export = GrooveExport {
            version: EXPORT_VERSION,
            exported_at: chrono::Utc::now(),
            learnings: vec![],
            params: vec![],
            relations: vec![],
        };
        let json = serde_json::to_string(&export).unwrap();
        let parsed: GrooveExport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, EXPORT_VERSION);
    }

    #[test]
    fn test_groove_export_new() {
        let export = GrooveExport::new();
        assert_eq!(export.version, EXPORT_VERSION);
        assert!(export.learnings.is_empty());
        assert!(export.params.is_empty());
        assert!(export.relations.is_empty());
    }

    #[test]
    fn test_groove_export_default() {
        let export = GrooveExport::default();
        assert_eq!(export.version, EXPORT_VERSION);
    }

    #[test]
    fn test_learning_export_from_learning() {
        let learning = Learning::new(
            Scope::User("test".into()),
            LearningCategory::Preference,
            LearningContent {
                description: "Test description".into(),
                pattern: None,
                insight: "Test insight".into(),
            },
            LearningSource::UserCreated,
        );
        let learning_id = learning.id;
        let export = LearningExport::from(learning);
        assert_eq!(export.id, learning_id);
        assert_eq!(export.category, LearningCategory::Preference);
        assert_eq!(export.usage_stats.times_injected, 0);
    }

    #[test]
    fn test_learning_export_serialization() {
        let learning = Learning::new(
            Scope::Global,
            LearningCategory::CodePattern,
            LearningContent {
                description: "Use Result for errors".into(),
                pattern: Some(serde_json::json!({"language": "rust"})),
                insight: "Prefer Result over panic".into(),
            },
            LearningSource::UserCreated,
        );
        let export = LearningExport::from(learning);
        let json = serde_json::to_string(&export).unwrap();
        let parsed: LearningExport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, export.id);
        assert_eq!(parsed.scope, Scope::Global);
    }
}
