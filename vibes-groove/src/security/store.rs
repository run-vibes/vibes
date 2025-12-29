//! Security-aware storage integration
//!
//! Wraps learning storage with security metadata and policy enforcement.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{
    Provenance, QuarantineReason, QuarantineStatus, ScanResult, SecurityError, SecurityResult,
    TrustContext, TrustLevel,
};
use crate::{Learning, LearningId, Scope};

/// A learning with full security metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureLearning {
    /// The underlying learning
    pub learning: Learning,
    /// Trust context for this learning
    pub trust: TrustContext,
    /// Content provenance
    pub provenance: Provenance,
    /// Quarantine status (None if not quarantined)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quarantine: Option<QuarantineStatus>,
    /// Last scan result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_scan: Option<ScanResult>,
}

impl SecureLearning {
    /// Create a new secure learning (local/trusted)
    pub fn new_local(learning: Learning, author: impl Into<String>) -> Self {
        let author = author.into();
        let provenance = Provenance::new(&learning.content.insight, &author);

        Self {
            trust: TrustContext::local(&author),
            provenance,
            quarantine: None,
            last_scan: None,
            learning,
        }
    }

    /// Create from imported content
    pub fn from_import(learning: Learning, source: impl Into<String>) -> Self {
        let source = source.into();
        let provenance = Provenance::new(&learning.content.insight, "import");

        Self {
            trust: TrustContext::imported(&source),
            provenance,
            quarantine: None,
            last_scan: None,
            learning,
        }
    }

    /// Create from enterprise content
    pub fn from_enterprise(
        learning: Learning,
        org_id: impl Into<String>,
        author_id: impl Into<String>,
        verified: bool,
    ) -> Self {
        let org_id = org_id.into();
        let author_id = author_id.into();
        let provenance = Provenance::new(&learning.content.insight, &author_id);

        Self {
            trust: TrustContext::enterprise(&org_id, &author_id, verified),
            provenance,
            quarantine: None,
            last_scan: None,
            learning,
        }
    }

    /// Get the learning ID
    pub fn id(&self) -> LearningId {
        self.learning.id
    }

    /// Check if this learning is quarantined
    pub fn is_quarantined(&self) -> bool {
        self.trust.level == TrustLevel::Quarantined
            || self.quarantine.as_ref().map(|q| q.is_pending_review()).unwrap_or(false)
    }

    /// Check if injection is allowed based on trust level
    pub fn allows_injection(&self) -> bool {
        !self.is_quarantined() && self.trust.level.allows_injection()
    }

    /// Quarantine this learning
    pub fn quarantine(&mut self, reason: QuarantineReason, findings: Vec<super::ScanFinding>) {
        self.quarantine = Some(QuarantineStatus::new(reason, findings));
        self.trust.level = TrustLevel::Quarantined;
    }

    /// Record a scan result
    pub fn record_scan(&mut self, result: ScanResult) {
        self.last_scan = Some(result);
    }
}

/// Query filter for secure learnings
#[derive(Debug, Clone, Default)]
pub struct SecureLearningFilter {
    /// Filter by scope
    pub scope: Option<Scope>,
    /// Filter by minimum trust level
    pub min_trust_level: Option<TrustLevel>,
    /// Include quarantined learnings
    pub include_quarantined: bool,
    /// Filter by trust source type
    pub source_type: Option<String>,
    /// Maximum results
    pub limit: Option<usize>,
}

/// Security-aware learning store operations
#[async_trait]
pub trait SecureLearningStore: Send + Sync {
    /// Store a learning with security metadata
    async fn store_secure(&self, learning: &SecureLearning) -> SecurityResult<LearningId>;

    /// Get a secure learning by ID
    async fn get_secure(&self, id: LearningId) -> SecurityResult<Option<SecureLearning>>;

    /// Find learnings with security filtering
    async fn find_secure(&self, filter: &SecureLearningFilter) -> SecurityResult<Vec<SecureLearning>>;

    /// Get all quarantined learnings
    async fn find_quarantined(&self) -> SecurityResult<Vec<SecureLearning>>;

    /// Update quarantine status
    async fn update_quarantine(
        &self,
        id: LearningId,
        quarantine: Option<QuarantineStatus>,
    ) -> SecurityResult<()>;

    /// Update trust level
    async fn update_trust(&self, id: LearningId, trust: TrustContext) -> SecurityResult<()>;

    /// Record a scan result
    async fn record_scan(&self, id: LearningId, result: ScanResult) -> SecurityResult<()>;

    /// Delete a learning (checks permissions)
    async fn delete_secure(&self, id: LearningId) -> SecurityResult<bool>;
}

/// In-memory implementation of SecureLearningStore for testing
pub struct MemorySecureLearningStore {
    learnings: tokio::sync::RwLock<std::collections::HashMap<LearningId, SecureLearning>>,
}

impl MemorySecureLearningStore {
    pub fn new() -> Self {
        Self {
            learnings: tokio::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

impl Default for MemorySecureLearningStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecureLearningStore for MemorySecureLearningStore {
    async fn store_secure(&self, learning: &SecureLearning) -> SecurityResult<LearningId> {
        let id = learning.id();
        let mut learnings = self.learnings.write().await;
        learnings.insert(id, learning.clone());
        Ok(id)
    }

    async fn get_secure(&self, id: LearningId) -> SecurityResult<Option<SecureLearning>> {
        let learnings = self.learnings.read().await;
        Ok(learnings.get(&id).cloned())
    }

    async fn find_secure(&self, filter: &SecureLearningFilter) -> SecurityResult<Vec<SecureLearning>> {
        let learnings = self.learnings.read().await;
        let mut results: Vec<_> = learnings
            .values()
            .filter(|l| {
                // Scope filter
                if let Some(ref scope) = filter.scope {
                    if &l.learning.scope != scope {
                        return false;
                    }
                }

                // Trust level filter
                if let Some(min_level) = filter.min_trust_level {
                    if l.trust.level < min_level {
                        return false;
                    }
                }

                // Quarantine filter
                if !filter.include_quarantined && l.is_quarantined() {
                    return false;
                }

                true
            })
            .cloned()
            .collect();

        // Apply limit
        if let Some(limit) = filter.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    async fn find_quarantined(&self) -> SecurityResult<Vec<SecureLearning>> {
        let learnings = self.learnings.read().await;
        Ok(learnings.values().filter(|l| l.is_quarantined()).cloned().collect())
    }

    async fn update_quarantine(
        &self,
        id: LearningId,
        quarantine: Option<QuarantineStatus>,
    ) -> SecurityResult<()> {
        let mut learnings = self.learnings.write().await;
        if let Some(learning) = learnings.get_mut(&id) {
            learning.quarantine = quarantine.clone();
            // Update trust level based on quarantine status
            if quarantine.as_ref().map(|q| q.is_pending_review()).unwrap_or(false) {
                learning.trust.level = TrustLevel::Quarantined;
            }
            Ok(())
        } else {
            Err(SecurityError::PolicyViolation(format!(
                "learning {} not found",
                id
            )))
        }
    }

    async fn update_trust(&self, id: LearningId, trust: TrustContext) -> SecurityResult<()> {
        let mut learnings = self.learnings.write().await;
        if let Some(learning) = learnings.get_mut(&id) {
            learning.trust = trust;
            Ok(())
        } else {
            Err(SecurityError::PolicyViolation(format!(
                "learning {} not found",
                id
            )))
        }
    }

    async fn record_scan(&self, id: LearningId, result: ScanResult) -> SecurityResult<()> {
        let mut learnings = self.learnings.write().await;
        if let Some(learning) = learnings.get_mut(&id) {
            learning.last_scan = Some(result);
            Ok(())
        } else {
            Err(SecurityError::PolicyViolation(format!(
                "learning {} not found",
                id
            )))
        }
    }

    async fn delete_secure(&self, id: LearningId) -> SecurityResult<bool> {
        let mut learnings = self.learnings.write().await;
        Ok(learnings.remove(&id).is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LearningCategory, LearningContent, LearningSource};

    fn make_learning() -> Learning {
        Learning::new(
            Scope::User("test-user".into()),
            LearningCategory::CodePattern,
            LearningContent {
                description: "Test pattern".into(),
                pattern: None,
                insight: "Test insight".into(),
            },
            LearningSource::UserCreated,
        )
    }

    #[test]
    fn test_secure_learning_new_local() {
        let learning = make_learning();
        let secure = SecureLearning::new_local(learning, "alice");

        assert_eq!(secure.trust.level, TrustLevel::Local);
        assert!(!secure.is_quarantined());
        assert!(secure.allows_injection());
    }

    #[test]
    fn test_secure_learning_from_import() {
        let learning = make_learning();
        let secure = SecureLearning::from_import(learning, "patterns.json");

        assert_eq!(secure.trust.level, TrustLevel::PublicUnverified);
        assert!(!secure.is_quarantined());
    }

    #[test]
    fn test_secure_learning_from_enterprise() {
        let learning = make_learning();
        let secure = SecureLearning::from_enterprise(learning, "acme", "bob", true);

        assert_eq!(secure.trust.level, TrustLevel::OrganizationVerified);
        assert!(secure.allows_injection());
    }

    #[test]
    fn test_secure_learning_quarantine() {
        let learning = make_learning();
        let mut secure = SecureLearning::new_local(learning, "alice");

        secure.quarantine(QuarantineReason::ImportScanFailed, vec![]);

        assert!(secure.is_quarantined());
        assert!(!secure.allows_injection());
        assert_eq!(secure.trust.level, TrustLevel::Quarantined);
    }

    #[tokio::test]
    async fn test_memory_store_crud() {
        let store = MemorySecureLearningStore::new();

        let learning = make_learning();
        let secure = SecureLearning::new_local(learning, "alice");
        let id = secure.id();

        // Store
        store.store_secure(&secure).await.unwrap();

        // Get
        let retrieved = store.get_secure(id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id(), id);

        // Delete
        let deleted = store.delete_secure(id).await.unwrap();
        assert!(deleted);

        // Verify deleted
        let retrieved = store.get_secure(id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_memory_store_filter() {
        let store = MemorySecureLearningStore::new();

        // Store some learnings
        for i in 0..5 {
            let learning = make_learning();
            let secure = SecureLearning::new_local(learning, format!("user{}", i));
            store.store_secure(&secure).await.unwrap();
        }

        // Store a quarantined one
        let learning = make_learning();
        let mut secure = SecureLearning::new_local(learning, "bad-user");
        secure.quarantine(QuarantineReason::ImportScanFailed, vec![]);
        store.store_secure(&secure).await.unwrap();

        // Find without quarantined
        let filter = SecureLearningFilter::default();
        let results = store.find_secure(&filter).await.unwrap();
        assert_eq!(results.len(), 5);

        // Find with quarantined
        let filter = SecureLearningFilter {
            include_quarantined: true,
            ..Default::default()
        };
        let results = store.find_secure(&filter).await.unwrap();
        assert_eq!(results.len(), 6);

        // Find only quarantined
        let quarantined = store.find_quarantined().await.unwrap();
        assert_eq!(quarantined.len(), 1);
    }

    #[tokio::test]
    async fn test_memory_store_update_quarantine() {
        let store = MemorySecureLearningStore::new();

        let learning = make_learning();
        let secure = SecureLearning::new_local(learning, "alice");
        let id = secure.id();
        store.store_secure(&secure).await.unwrap();

        // Quarantine
        let quarantine = QuarantineStatus::new(QuarantineReason::PolicyRescanFailed, vec![]);
        store.update_quarantine(id, Some(quarantine)).await.unwrap();

        let retrieved = store.get_secure(id).await.unwrap().unwrap();
        assert!(retrieved.is_quarantined());

        // Remove quarantine
        store.update_quarantine(id, None).await.unwrap();
        let retrieved = store.get_secure(id).await.unwrap().unwrap();
        assert!(retrieved.quarantine.is_none());
    }

    #[tokio::test]
    async fn test_memory_store_filter_by_trust_level() {
        let store = MemorySecureLearningStore::new();

        // Store local (trust 100)
        let learning = make_learning();
        let secure = SecureLearning::new_local(learning, "alice");
        store.store_secure(&secure).await.unwrap();

        // Store imported (trust 10)
        let learning = make_learning();
        let secure = SecureLearning::from_import(learning, "import.json");
        store.store_secure(&secure).await.unwrap();

        // Filter by min trust level
        let filter = SecureLearningFilter {
            min_trust_level: Some(TrustLevel::OrganizationVerified),
            ..Default::default()
        };
        let results = store.find_secure(&filter).await.unwrap();
        assert_eq!(results.len(), 1); // Only local meets the threshold
    }
}
