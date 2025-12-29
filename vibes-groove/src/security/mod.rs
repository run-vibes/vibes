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
    ScanFinding, ScanResult, Severity,
};
pub use trust::{TrustContext, TrustLevel, TrustSource, Verification, VerifiedBy};

// TODO: Uncomment as modules are implemented
// pub use policy::*;
// pub use audit::*;
// pub use injector::*;
