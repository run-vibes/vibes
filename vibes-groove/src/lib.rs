//! vibes-groove - Continual learning storage
//!
//! This crate provides CozoDB-based storage for the groove continual learning system.
//! It implements three-tier storage (user/project/enterprise) with learning storage,
//! adaptive parameters, and semantic search capabilities.

// Module declarations - commented out until implemented
pub mod assessment;
pub mod capture;
pub mod config;
pub mod error;
pub mod export;
pub mod inject;
pub mod paths;
pub mod plugin;
pub mod security;
pub mod storage;
pub mod store;
pub mod types;

// Re-exports - commented out until modules are implemented
pub use config::{EnterpriseConfig, GrooveConfig, ProjectContext};
pub use error::{GrooveError, Result};
pub use export::{EXPORT_VERSION, GrooveExport, ImportStats, LearningExport};
pub use paths::GroovePaths;
pub use storage::GrooveStorage;
pub use store::{
    CURRENT_SCHEMA_VERSION, CozoStore, INITIAL_SCHEMA, LearningStore, MIGRATIONS, Migration,
    ParamStore,
};
pub use types::*;

// Assessment re-exports
pub use assessment::{
    AssessmentContext, AssessmentProcessor, EventId, HarnessType, InjectionMethod, ProjectId,
    SessionId, UserId,
};

// Security re-exports
pub use security::{
    ContentHash, ContentScanner, CreationEvent, CustodyEvent, CustodyEventType, DlpScanner,
    InjectionDetector, NoOpDlpScanner, NoOpInjectionDetector, Operation, OrgRole, Permissions,
    Provenance, QuarantineReason, QuarantineStatus, ReviewOutcome, ScanFinding, ScanResult,
    SecurityError, SecurityResult, Severity, TrustContext, TrustLevel, TrustSource, Verification,
    VerifiedBy,
};

// Plugin re-export
pub use plugin::GroovePlugin;
