//! Security foundation for groove
//!
//! This module provides:
//! - Trust hierarchy and context
//! - Policy system for enterprise control
//! - Content scanning for injection protection
//! - Audit logging for compliance
//! - Quarantine management

mod audit;
mod error;
mod injector;
mod policy;
mod provenance;
mod quarantine;
mod scanning;
mod trust;

pub use error::{SecurityError, SecurityResult};
pub use provenance::{
    ContentHash, CreationEvent, CustodyEvent, CustodyEventType, Provenance,
};
pub use trust::{TrustContext, TrustLevel, TrustSource, Verification, VerifiedBy};

// TODO: Uncomment as modules are implemented
// pub use policy::*;
// pub use scanning::*;
// pub use audit::*;
// pub use quarantine::*;
// pub use injector::*;
