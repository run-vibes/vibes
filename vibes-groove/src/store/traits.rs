//! Storage traits for the continual learning system

use async_trait::async_trait;

use crate::{
    GrooveError, Learning, LearningCategory, LearningId, LearningRelation, RelationType, Scope,
    SystemParam, UsageStats,
};

/// Storage operations for a single learning database
#[async_trait]
pub trait LearningStore: Send + Sync {
    /// Store a new learning, returns its ID
    async fn store(&self, learning: &Learning) -> Result<LearningId, GrooveError>;

    /// Retrieve a learning by ID
    async fn get(&self, id: LearningId) -> Result<Option<Learning>, GrooveError>;

    /// Find all learnings in a scope
    async fn find_by_scope(&self, scope: &Scope) -> Result<Vec<Learning>, GrooveError>;

    /// Find learnings by category
    async fn find_by_category(
        &self,
        category: &LearningCategory,
    ) -> Result<Vec<Learning>, GrooveError>;

    /// Semantic search using embedding vector
    async fn semantic_search(
        &self,
        embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(Learning, f64)>, GrooveError>;

    /// Update usage statistics for a learning
    async fn update_usage(&self, id: LearningId, stats: &UsageStats) -> Result<(), GrooveError>;

    /// Find related learnings by relation type
    async fn find_related(
        &self,
        id: LearningId,
        relation_type: Option<&RelationType>,
    ) -> Result<Vec<Learning>, GrooveError>;

    /// Store a relation between learnings
    async fn store_relation(&self, relation: &LearningRelation) -> Result<(), GrooveError>;

    /// Delete a learning
    async fn delete(&self, id: LearningId) -> Result<bool, GrooveError>;

    /// Count learnings (for stats)
    async fn count(&self) -> Result<u64, GrooveError>;
}

/// System parameter storage
#[async_trait]
pub trait ParamStore: Send + Sync {
    /// Get a system parameter by name
    async fn get_param(&self, name: &str) -> Result<Option<SystemParam>, GrooveError>;

    /// Store/update a system parameter
    async fn store_param(&self, param: &SystemParam) -> Result<(), GrooveError>;

    /// Get all system parameters
    async fn all_params(&self) -> Result<Vec<SystemParam>, GrooveError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Verify traits are object-safe
    #[test]
    fn test_learning_store_is_object_safe() {
        fn _takes_boxed(_: Box<dyn LearningStore>) {}
    }

    #[test]
    fn test_param_store_is_object_safe() {
        fn _takes_boxed(_: Box<dyn ParamStore>) {}
    }
}
