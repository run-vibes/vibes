//! Audit logging for compliance
//!
//! Provides append-only JSONL audit logs.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::SecurityResult;

/// Actor who performed an action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActorId {
    User(String),
    System,
    Policy,
    Scanner,
}

/// Type of audited action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditAction {
    // Learning lifecycle
    LearningCreated,
    LearningInjected,
    LearningPromoted,
    LearningDeleted,
    LearningModified,

    // Security events
    PolicyViolation,
    InjectionAttemptBlocked,
    ScanFindingDetected,
    QuarantineTriggered,
    QuarantineReviewed,

    // Import/Export
    ImportAttempted,
    ImportBlocked,
    ExportAttempted,
    ExportBlocked,

    // Policy
    PolicyLoaded,
    PolicyChanged,
    RescanTriggered,
}

/// Reference to a resource
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResourceRef {
    Learning(Uuid),
    Policy(String),
    Session(String),
    Import(String),
}

/// Context for audit entry
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Outcome of an action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionOutcome {
    Success,
    Blocked { reason: String },
    Failed { error: String },
}

/// A single audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub actor: ActorId,
    pub action: AuditAction,
    pub resource: ResourceRef,
    pub context: AuditContext,
    pub outcome: ActionOutcome,
}

impl AuditLogEntry {
    /// Create a new audit entry
    pub fn new(
        actor: ActorId,
        action: AuditAction,
        resource: ResourceRef,
        outcome: ActionOutcome,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            timestamp: Utc::now(),
            actor,
            action,
            resource,
            context: AuditContext::default(),
            outcome,
        }
    }

    /// Add context to the entry
    pub fn with_context(mut self, context: AuditContext) -> Self {
        self.context = context;
        self
    }
}

/// Filter for querying audit logs
#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    pub actor: Option<ActorId>,
    pub action: Option<AuditAction>,
    pub resource: Option<ResourceRef>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

/// Audit log trait
#[async_trait]
pub trait AuditLog: Send + Sync {
    /// Write an entry to the audit log
    async fn log(&self, entry: AuditLogEntry) -> SecurityResult<()>;

    /// Query audit entries
    async fn query(&self, filter: AuditFilter) -> SecurityResult<Vec<AuditLogEntry>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry_serialization() {
        let entry = AuditLogEntry {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            actor: ActorId::User("alice".to_string()),
            action: AuditAction::LearningCreated,
            resource: ResourceRef::Learning(uuid::Uuid::new_v4()),
            context: AuditContext::default(),
            outcome: ActionOutcome::Success,
        };

        let json = serde_json::to_string(&entry).unwrap();
        let parsed: AuditLogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, entry.id);
        assert_eq!(parsed.actor, entry.actor);
        assert_eq!(parsed.action, entry.action);
    }

    #[test]
    fn test_audit_entry_new() {
        let entry = AuditLogEntry::new(
            ActorId::System,
            AuditAction::PolicyLoaded,
            ResourceRef::Policy("default".into()),
            ActionOutcome::Success,
        );

        assert_eq!(entry.actor, ActorId::System);
        assert_eq!(entry.action, AuditAction::PolicyLoaded);
    }

    #[test]
    fn test_audit_entry_with_context() {
        let entry = AuditLogEntry::new(
            ActorId::User("bob".into()),
            AuditAction::ImportAttempted,
            ResourceRef::Import("file.json".into()),
            ActionOutcome::Blocked {
                reason: "policy violation".into(),
            },
        )
        .with_context(AuditContext {
            ip_address: Some("192.168.1.1".into()),
            session_id: Some("sess-123".into()),
            device_id: None,
            details: Some(serde_json::json!({"file_size": 1024})),
        });

        assert_eq!(entry.context.ip_address, Some("192.168.1.1".into()));
        assert!(entry.context.details.is_some());
    }

    #[test]
    fn test_action_outcome_variants() {
        let success = ActionOutcome::Success;
        let blocked = ActionOutcome::Blocked {
            reason: "test".into(),
        };
        let failed = ActionOutcome::Failed {
            error: "test error".into(),
        };

        // Verify they serialize correctly
        let json = serde_json::to_string(&success).unwrap();
        assert!(json.contains("Success"));

        let json = serde_json::to_string(&blocked).unwrap();
        assert!(json.contains("Blocked"));

        let json = serde_json::to_string(&failed).unwrap();
        assert!(json.contains("Failed"));
    }

    #[test]
    fn test_audit_filter_default() {
        let filter = AuditFilter::default();
        assert!(filter.actor.is_none());
        assert!(filter.action.is_none());
        assert!(filter.limit.is_none());
    }
}
