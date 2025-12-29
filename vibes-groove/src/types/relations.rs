//! Relation types for knowledge graph edges between learnings

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::LearningId;

/// Relationship between two learnings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningRelation {
    pub from_id: LearningId,
    pub relation_type: RelationType,
    pub to_id: LearningId,
    pub weight: f64,
    pub created_at: DateTime<Utc>,
}

impl LearningRelation {
    pub fn new(from_id: LearningId, relation_type: RelationType, to_id: LearningId) -> Self {
        Self {
            from_id,
            relation_type,
            to_id,
            weight: 1.0,
            created_at: Utc::now(),
        }
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }
}

/// Types of relationships between learnings
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationType {
    /// from_id replaces/deprecates to_id
    Supersedes,
    /// from_id conflicts with to_id
    Contradicts,
    /// from_id was derived/generalized from to_id
    DerivedFrom,
    /// from_id is related to to_id (same topic)
    RelatedTo,
    /// from_id is a specific case of general to_id
    Specializes,
}

impl RelationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Supersedes => "supersedes",
            Self::Contradicts => "contradicts",
            Self::DerivedFrom => "derived_from",
            Self::RelatedTo => "related_to",
            Self::Specializes => "specializes",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "supersedes" => Some(Self::Supersedes),
            "contradicts" => Some(Self::Contradicts),
            "derived_from" => Some(Self::DerivedFrom),
            "related_to" => Some(Self::RelatedTo),
            "specializes" => Some(Self::Specializes),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relation_type_as_str() {
        assert_eq!(RelationType::Supersedes.as_str(), "supersedes");
        assert_eq!(RelationType::Contradicts.as_str(), "contradicts");
        assert_eq!(RelationType::DerivedFrom.as_str(), "derived_from");
        assert_eq!(RelationType::RelatedTo.as_str(), "related_to");
        assert_eq!(RelationType::Specializes.as_str(), "specializes");
    }

    #[test]
    fn test_relation_type_from_str() {
        assert_eq!(
            RelationType::from_str("supersedes"),
            Some(RelationType::Supersedes)
        );
        assert_eq!(RelationType::from_str("unknown"), None);
    }

    #[test]
    fn test_learning_relation_serialization() {
        let relation = LearningRelation {
            from_id: uuid::Uuid::now_v7(),
            relation_type: RelationType::Supersedes,
            to_id: uuid::Uuid::now_v7(),
            weight: 1.0,
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&relation).unwrap();
        let parsed: LearningRelation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.from_id, relation.from_id);
        assert_eq!(parsed.relation_type, relation.relation_type);
    }
}
