//! Relation types for knowledge graph edges between learnings

use std::str::FromStr;

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
}

/// Error type for parsing RelationType from string
#[derive(Debug, Clone)]
pub struct ParseRelationTypeError(pub String);

impl std::fmt::Display for ParseRelationTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown relation type: {}", self.0)
    }
}

impl std::error::Error for ParseRelationTypeError {}

impl FromStr for RelationType {
    type Err = ParseRelationTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "supersedes" => Ok(Self::Supersedes),
            "contradicts" => Ok(Self::Contradicts),
            "derived_from" => Ok(Self::DerivedFrom),
            "related_to" => Ok(Self::RelatedTo),
            "specializes" => Ok(Self::Specializes),
            _ => Err(ParseRelationTypeError(s.to_string())),
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
            RelationType::from_str("supersedes").unwrap(),
            RelationType::Supersedes
        );
        assert!(RelationType::from_str("unknown").is_err());
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
