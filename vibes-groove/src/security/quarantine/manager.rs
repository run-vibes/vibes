//! Quarantine manager
//!
//! Orchestrates quarantine workflow with storage, audit, and RBAC integration.

use std::sync::Arc;

use super::{QuarantineReason, QuarantineStatus, ReviewOutcome};
use crate::security::{
    ActionOutcome, ActorId, AuditAction, AuditLog, AuditLogEntry, Operation, Permissions,
    ResourceRef, ScanFinding, ScanResult, SecureLearning, SecureLearningStore, SecurityError,
    SecurityResult, TrustLevel,
};
use crate::LearningId;

/// Result of a quarantine review
#[derive(Debug, Clone)]
pub struct ReviewResult {
    /// The action taken
    pub outcome: ReviewOutcome,
    /// Whether the learning was restored
    pub restored: bool,
    /// Whether the learning was deleted
    pub deleted: bool,
}

/// Quarantine manager configuration
pub struct QuarantineManagerConfig<S, A>
where
    S: SecureLearningStore,
    A: AuditLog,
{
    /// The store to manage
    pub store: Arc<S>,
    /// Audit log for recording actions
    pub audit_log: Arc<A>,
    /// Whether to require admin permission for reviews
    pub require_admin_for_review: bool,
}

/// Quarantine workflow manager
///
/// Orchestrates quarantine operations with proper audit logging and permission checks.
pub struct QuarantineManager<S, A>
where
    S: SecureLearningStore,
    A: AuditLog,
{
    store: Arc<S>,
    audit_log: Arc<A>,
    require_admin_for_review: bool,
}

impl<S, A> QuarantineManager<S, A>
where
    S: SecureLearningStore,
    A: AuditLog,
{
    /// Create a new quarantine manager
    pub fn new(config: QuarantineManagerConfig<S, A>) -> Self {
        Self {
            store: config.store,
            audit_log: config.audit_log,
            require_admin_for_review: config.require_admin_for_review,
        }
    }

    /// Quarantine a learning
    ///
    /// Records the quarantine in storage and audit log.
    pub async fn quarantine(
        &self,
        id: LearningId,
        reason: QuarantineReason,
        findings: Vec<ScanFinding>,
        actor_id: &str,
    ) -> SecurityResult<()> {
        let quarantine = QuarantineStatus::new(reason.clone(), findings.clone());

        // Update the store
        self.store.update_quarantine(id, Some(quarantine)).await?;

        // Log the action
        let entry = AuditLogEntry::new(
            ActorId::User(actor_id.to_string()),
            AuditAction::QuarantinedLearning {
                learning_id: id.to_string(),
                reason: format!("{:?}", reason),
            },
            ResourceRef::Learning(id.into()),
            ActionOutcome::Success,
        );
        self.audit_log.log(entry).await?;

        Ok(())
    }

    /// Quarantine a learning based on scan result
    ///
    /// Convenience method for scan-triggered quarantine.
    pub async fn quarantine_from_scan(
        &self,
        id: LearningId,
        scan_result: &ScanResult,
        actor_id: &str,
    ) -> SecurityResult<()> {
        // Record the scan first
        self.store.record_scan(id, scan_result.clone()).await?;

        // If scan failed, quarantine
        if !scan_result.passed {
            let reason = QuarantineReason::ImportScanFailed;
            self.quarantine(id, reason, scan_result.findings.clone(), actor_id)
                .await?;
        }

        Ok(())
    }

    /// Review a quarantined learning
    ///
    /// Checks permissions, processes the review, and takes appropriate action.
    pub async fn review(
        &self,
        id: LearningId,
        reviewer_id: &str,
        permissions: &Permissions,
        outcome: ReviewOutcome,
    ) -> SecurityResult<ReviewResult> {
        // Check permissions - require Review or Delete permission
        if self.require_admin_for_review && !permissions.allows(Operation::Review) {
            return Err(SecurityError::PermissionDenied(
                "quarantine review requires review permissions".to_string(),
            ));
        }

        // Get current quarantine status
        let learning = self
            .store
            .get_secure(id)
            .await?
            .ok_or_else(|| SecurityError::PolicyViolation(format!("learning {} not found", id)))?;

        let mut quarantine = learning.quarantine.ok_or_else(|| {
            SecurityError::PolicyViolation(format!("learning {} is not quarantined", id))
        })?;

        // Record the review
        quarantine.review(reviewer_id, outcome);

        let mut result = ReviewResult {
            outcome,
            restored: false,
            deleted: false,
        };

        // Take action based on outcome
        match outcome {
            ReviewOutcome::Approved => {
                // Remove quarantine, restore trust level
                self.store.update_quarantine(id, None).await?;

                // Update trust level back to normal (imported for now)
                let mut trust = learning.trust;
                if trust.level == TrustLevel::Quarantined {
                    trust.level = TrustLevel::PublicUnverified;
                }
                self.store.update_trust(id, trust).await?;

                result.restored = true;
            }
            ReviewOutcome::Rejected => {
                // Delete the learning
                self.store.delete_secure(id).await?;
                result.deleted = true;
            }
            ReviewOutcome::Escalated => {
                // Just update the quarantine status, keep quarantined
                self.store.update_quarantine(id, Some(quarantine)).await?;
            }
        }

        // Log the review
        let entry = AuditLogEntry::new(
            ActorId::User(reviewer_id.to_string()),
            AuditAction::ReviewedQuarantine {
                learning_id: id.to_string(),
                outcome: format!("{:?}", outcome),
            },
            ResourceRef::Learning(id.into()),
            ActionOutcome::Success,
        );
        self.audit_log.log(entry).await?;

        Ok(result)
    }

    /// List all learnings pending quarantine review
    pub async fn list_pending(&self) -> SecurityResult<Vec<SecureLearning>> {
        let quarantined = self.store.find_quarantined().await?;

        // Filter to only pending reviews
        Ok(quarantined
            .into_iter()
            .filter(|l| {
                l.quarantine
                    .as_ref()
                    .map(|q| q.is_pending_review())
                    .unwrap_or(false)
            })
            .collect())
    }

    /// Get quarantine statistics
    pub async fn stats(&self) -> SecurityResult<QuarantineStats> {
        let quarantined = self.store.find_quarantined().await?;

        let mut stats = QuarantineStats::default();

        for learning in &quarantined {
            if let Some(ref q) = learning.quarantine {
                stats.total += 1;

                if q.is_pending_review() {
                    stats.pending_review += 1;
                }

                match q.review_outcome {
                    Some(ReviewOutcome::Approved) => stats.approved += 1,
                    Some(ReviewOutcome::Rejected) => stats.rejected += 1,
                    Some(ReviewOutcome::Escalated) => stats.escalated += 1,
                    None => {}
                }
            }
        }

        Ok(stats)
    }
}

/// Quarantine statistics
#[derive(Debug, Clone, Default)]
pub struct QuarantineStats {
    /// Total quarantined learnings
    pub total: usize,
    /// Pending review
    pub pending_review: usize,
    /// Approved (restored)
    pub approved: usize,
    /// Rejected (deleted)
    pub rejected: usize,
    /// Escalated
    pub escalated: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::audit::InMemoryAuditLog;
    use crate::security::store::MemorySecureLearningStore;
    use crate::security::{OrgRole, Severity};
    use crate::{Learning, LearningCategory, LearningContent, LearningSource, Scope};

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

    async fn setup_manager() -> (
        QuarantineManager<MemorySecureLearningStore, InMemoryAuditLog>,
        Arc<MemorySecureLearningStore>,
        Arc<InMemoryAuditLog>,
    ) {
        let store = Arc::new(MemorySecureLearningStore::new());
        let audit_log = Arc::new(InMemoryAuditLog::new());

        let manager = QuarantineManager::new(QuarantineManagerConfig {
            store: Arc::clone(&store),
            audit_log: Arc::clone(&audit_log),
            require_admin_for_review: true,
        });

        (manager, store, audit_log)
    }

    #[tokio::test]
    async fn test_quarantine_learning() {
        let (manager, store, audit_log) = setup_manager().await;

        // Store a learning
        let learning = make_learning();
        let secure = SecureLearning::new_local(learning, "alice");
        let id = secure.id();
        store.store_secure(&secure).await.unwrap();

        // Quarantine it
        manager
            .quarantine(id, QuarantineReason::ImportScanFailed, vec![], "admin")
            .await
            .unwrap();

        // Verify it's quarantined
        let retrieved = store.get_secure(id).await.unwrap().unwrap();
        assert!(retrieved.is_quarantined());

        // Verify audit entry
        let entries = audit_log.entries().await;
        assert_eq!(entries.len(), 1);
        assert!(matches!(
            entries[0].action,
            AuditAction::QuarantinedLearning { .. }
        ));
    }

    #[tokio::test]
    async fn test_quarantine_from_scan() {
        let (manager, store, _) = setup_manager().await;

        // Store a learning
        let learning = make_learning();
        let secure = SecureLearning::new_local(learning, "alice");
        let id = secure.id();
        store.store_secure(&secure).await.unwrap();

        // Create a failing scan result
        let scan_result = ScanResult::failed(vec![ScanFinding {
            severity: Severity::Critical,
            category: "prompt_injection".into(),
            pattern_matched: "ignore previous".into(),
            location: Some("line 1".into()),
        }]);

        // Quarantine from scan
        manager
            .quarantine_from_scan(id, &scan_result, "scanner")
            .await
            .unwrap();

        // Verify it's quarantined
        let retrieved = store.get_secure(id).await.unwrap().unwrap();
        assert!(retrieved.is_quarantined());
        assert!(retrieved.last_scan.is_some());
    }

    #[tokio::test]
    async fn test_review_approve() {
        let (manager, store, _) = setup_manager().await;

        // Store and quarantine a learning
        let learning = make_learning();
        let secure = SecureLearning::new_local(learning, "alice");
        let id = secure.id();
        store.store_secure(&secure).await.unwrap();

        manager
            .quarantine(id, QuarantineReason::ImportScanFailed, vec![], "admin")
            .await
            .unwrap();

        // Review with admin permissions
        let permissions = OrgRole::Admin.permissions();
        let result = manager
            .review(id, "reviewer", &permissions, ReviewOutcome::Approved)
            .await
            .unwrap();

        assert_eq!(result.outcome, ReviewOutcome::Approved);
        assert!(result.restored);
        assert!(!result.deleted);

        // Verify not quarantined anymore
        let retrieved = store.get_secure(id).await.unwrap().unwrap();
        assert!(!retrieved.is_quarantined());
    }

    #[tokio::test]
    async fn test_review_reject() {
        let (manager, store, _) = setup_manager().await;

        // Store and quarantine a learning
        let learning = make_learning();
        let secure = SecureLearning::new_local(learning, "alice");
        let id = secure.id();
        store.store_secure(&secure).await.unwrap();

        manager
            .quarantine(id, QuarantineReason::ImportScanFailed, vec![], "admin")
            .await
            .unwrap();

        // Review with admin permissions
        let permissions = OrgRole::Admin.permissions();
        let result = manager
            .review(id, "reviewer", &permissions, ReviewOutcome::Rejected)
            .await
            .unwrap();

        assert_eq!(result.outcome, ReviewOutcome::Rejected);
        assert!(!result.restored);
        assert!(result.deleted);

        // Verify deleted
        let retrieved = store.get_secure(id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_review_requires_permission() {
        let (manager, store, _) = setup_manager().await;

        // Store and quarantine a learning
        let learning = make_learning();
        let secure = SecureLearning::new_local(learning, "alice");
        let id = secure.id();
        store.store_secure(&secure).await.unwrap();

        manager
            .quarantine(id, QuarantineReason::ImportScanFailed, vec![], "admin")
            .await
            .unwrap();

        // Try to review with viewer permissions (should fail)
        let permissions = OrgRole::Viewer.permissions();
        let result = manager
            .review(id, "viewer", &permissions, ReviewOutcome::Approved)
            .await;

        assert!(matches!(result, Err(SecurityError::PermissionDenied(_))));
    }

    #[tokio::test]
    async fn test_list_pending() {
        let (manager, store, _) = setup_manager().await;

        // Store multiple learnings
        for i in 0..3 {
            let learning = make_learning();
            let secure = SecureLearning::new_local(learning, format!("user{}", i));
            let id = secure.id();
            store.store_secure(&secure).await.unwrap();

            // Quarantine the first two
            if i < 2 {
                manager
                    .quarantine(id, QuarantineReason::ImportScanFailed, vec![], "admin")
                    .await
                    .unwrap();
            }
        }

        // List pending
        let pending = manager.list_pending().await.unwrap();
        assert_eq!(pending.len(), 2);
    }

    #[tokio::test]
    async fn test_quarantine_stats() {
        let (manager, store, _) = setup_manager().await;

        // Store and quarantine multiple learnings
        let mut ids = Vec::new();
        for i in 0..3 {
            let learning = make_learning();
            let secure = SecureLearning::new_local(learning, format!("user{}", i));
            let id = secure.id();
            ids.push(id);
            store.store_secure(&secure).await.unwrap();

            manager
                .quarantine(id, QuarantineReason::ImportScanFailed, vec![], "admin")
                .await
                .unwrap();
        }

        // Review one as approved
        let permissions = OrgRole::Admin.permissions();
        manager
            .review(ids[0], "reviewer", &permissions, ReviewOutcome::Approved)
            .await
            .unwrap();

        // Get stats
        let stats = manager.stats().await.unwrap();
        assert_eq!(stats.total, 2); // One was restored, so total is 2
        assert_eq!(stats.pending_review, 2);
    }
}
