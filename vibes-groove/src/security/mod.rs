//! Security foundation for groove
//!
//! This module provides:
//! - Trust hierarchy and context
//! - Policy system for enterprise control
//! - Content scanning for injection protection
//! - Audit logging for compliance
//! - Quarantine management
//! - Role-based access control

mod audit;
mod error;
mod injector;
mod policy;
mod provenance;
mod quarantine;
mod rbac;
mod scanning;
mod trust;

pub use error::{SecurityError, SecurityResult};
pub use provenance::{ContentHash, CreationEvent, CustodyEvent, CustodyEventType, Provenance};
pub use quarantine::{QuarantineReason, QuarantineStatus, ReviewOutcome};
pub use rbac::{Operation, OrgRole, Permissions};
pub use scanning::{
    ContentScanner, DlpScanner, InjectionDetector, NoOpDlpScanner, NoOpInjectionDetector,
    RegexScanner, ScanFinding, ScanResult, Severity,
};
pub use trust::{TrustContext, TrustLevel, TrustSource, Verification, VerifiedBy};

pub use audit::{
    ActionOutcome, ActorId, AuditAction, AuditContext, AuditFilter, AuditLog, AuditLogEntry,
    JsonlAuditLog, ResourceRef,
};
pub use policy::{
    AuditPolicy, CapturePolicy, FilePolicyProvider, IdentityPolicy, ImportExportPolicy,
    InjectionPolicy, MemoryPolicyProvider, Policy, PolicyChangeAction, PolicyProvider,
    PresentationPolicy, QuarantineAction, QuarantinePolicy, ScanPatterns, ScanningPolicy,
    TiersPolicy, WrapperConfig, WrapperType, load_policy_from_file, load_policy_or_default,
    parse_policy, validate_policy,
};

pub use injector::{InjectableContent, InjectionResult, InjectorConfig, SecureInjector};
