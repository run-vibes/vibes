//! Audit logging for compliance
//!
//! Provides append-only JSONL audit logs.

mod jsonl;
mod types;

pub use jsonl::JsonlAuditLog;
pub use types::{
    ActionOutcome, ActorId, AuditAction, AuditContext, AuditFilter, AuditLog, AuditLogEntry,
    InMemoryAuditLog, ResourceRef,
};
