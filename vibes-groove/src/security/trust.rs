//! Trust hierarchy for learnings
//!
//! Provides TrustLevel enum and TrustContext for policy evaluation.

use serde::{Deserialize, Serialize};

/// Trust hierarchy for learnings
///
/// Higher values indicate more trust. Local (100) is most trusted,
/// Quarantined (0) is least trusted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Quarantined - blocked from injection
    Quarantined = 0,
    /// Public unverified - community content, no verification
    PublicUnverified = 10,
    /// Public verified - community content, verified by community
    PublicVerified = 30,
    /// Organization unverified - enterprise content, not yet approved
    OrganizationUnverified = 50,
    /// Organization verified - enterprise content, curator approved
    OrganizationVerified = 70,
    /// Private cloud - synced from user's own cloud
    PrivateCloud = 90,
    /// Local - created locally by this user
    Local = 100,
}

impl Default for TrustLevel {
    fn default() -> Self {
        Self::Local
    }
}

// TODO: Task 2 will add TrustContext and full implementation
