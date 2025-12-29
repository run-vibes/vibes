//! Trust hierarchy for learnings
//!
//! Provides TrustLevel enum and TrustContext for policy evaluation.

use chrono::{DateTime, Utc};
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

impl TrustLevel {
    /// Check if this trust level allows injection
    pub fn allows_injection(&self) -> bool {
        *self > TrustLevel::Quarantined
    }

    /// Check if this trust level requires content scanning
    pub fn requires_scanning(&self) -> bool {
        *self <= TrustLevel::OrganizationUnverified
    }

    /// Get string representation for storage
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Quarantined => "quarantined",
            Self::PublicUnverified => "public_unverified",
            Self::PublicVerified => "public_verified",
            Self::OrganizationUnverified => "organization_unverified",
            Self::OrganizationVerified => "organization_verified",
            Self::PrivateCloud => "private_cloud",
            Self::Local => "local",
        }
    }
}

impl std::str::FromStr for TrustLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "quarantined" => Ok(Self::Quarantined),
            "public_unverified" => Ok(Self::PublicUnverified),
            "public_verified" => Ok(Self::PublicVerified),
            "organization_unverified" => Ok(Self::OrganizationUnverified),
            "organization_verified" => Ok(Self::OrganizationVerified),
            "private_cloud" => Ok(Self::PrivateCloud),
            "local" => Ok(Self::Local),
            _ => Err(format!("unknown trust level: {}", s)),
        }
    }
}

/// Source of trust - where the learning came from
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustSource {
    /// Created locally by this user
    Local { user_id: String },

    /// From enterprise tier
    Enterprise {
        org_id: String,
        author_id: String,
        verified_by: Option<String>,
    },

    /// Imported from external source
    Imported {
        source: String,
        imported_at: DateTime<Utc>,
    },

    /// From public/community source
    Public {
        author_id: Option<String>,
        source_url: Option<String>,
    },
}

impl TrustSource {
    /// Get the organization ID if this is an enterprise source
    pub fn org_id(&self) -> Option<&str> {
        match self {
            Self::Enterprise { org_id, .. } => Some(org_id),
            _ => None,
        }
    }

    /// Get the author ID if available
    pub fn author_id(&self) -> Option<&str> {
        match self {
            Self::Local { user_id } => Some(user_id),
            Self::Enterprise { author_id, .. } => Some(author_id),
            Self::Public { author_id, .. } => author_id.as_deref(),
            Self::Imported { .. } => None,
        }
    }
}

/// Verification status for trust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    pub verified_at: DateTime<Utc>,
    pub verified_by: VerifiedBy,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Who verified the learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerifiedBy {
    Curator {
        curator_id: String,
    },
    CommunityVote {
        votes_required: u32,
        votes_received: u32,
    },
    Automated {
        checker: String,
    },
}

/// Full trust context combining level, source, and verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustContext {
    pub level: TrustLevel,
    pub source: TrustSource,
    pub verification: Option<Verification>,
}

impl TrustContext {
    /// Create a new local trust context
    pub fn local(user_id: impl Into<String>) -> Self {
        Self {
            level: TrustLevel::Local,
            source: TrustSource::Local {
                user_id: user_id.into(),
            },
            verification: None,
        }
    }

    /// Create a trust context for imported content
    pub fn imported(source: impl Into<String>) -> Self {
        Self {
            level: TrustLevel::PublicUnverified,
            source: TrustSource::Imported {
                source: source.into(),
                imported_at: Utc::now(),
            },
            verification: None,
        }
    }

    /// Create an enterprise trust context
    pub fn enterprise(
        org_id: impl Into<String>,
        author_id: impl Into<String>,
        verified: bool,
    ) -> Self {
        let level = if verified {
            TrustLevel::OrganizationVerified
        } else {
            TrustLevel::OrganizationUnverified
        };

        Self {
            level,
            source: TrustSource::Enterprise {
                org_id: org_id.into(),
                author_id: author_id.into(),
                verified_by: None,
            },
            verification: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_level_ordering() {
        assert!(TrustLevel::Local > TrustLevel::PrivateCloud);
        assert!(TrustLevel::PrivateCloud > TrustLevel::OrganizationVerified);
        assert!(TrustLevel::OrganizationVerified > TrustLevel::OrganizationUnverified);
        assert!(TrustLevel::OrganizationUnverified > TrustLevel::PublicVerified);
        assert!(TrustLevel::PublicVerified > TrustLevel::PublicUnverified);
        assert!(TrustLevel::PublicUnverified > TrustLevel::Quarantined);
    }

    #[test]
    fn test_trust_level_default() {
        assert_eq!(TrustLevel::default(), TrustLevel::Local);
    }

    #[test]
    fn test_trust_level_serialization() {
        let level = TrustLevel::OrganizationVerified;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"OrganizationVerified\"");

        let parsed: TrustLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, level);
    }

    #[test]
    fn test_trust_level_allows_injection() {
        assert!(TrustLevel::Local.allows_injection());
        assert!(TrustLevel::PublicUnverified.allows_injection());
        assert!(!TrustLevel::Quarantined.allows_injection());
    }

    #[test]
    fn test_trust_level_requires_scanning() {
        assert!(!TrustLevel::Local.requires_scanning());
        assert!(!TrustLevel::OrganizationVerified.requires_scanning());
        assert!(TrustLevel::OrganizationUnverified.requires_scanning());
        assert!(TrustLevel::PublicUnverified.requires_scanning());
    }

    #[test]
    fn test_trust_level_str_roundtrip() {
        for level in [
            TrustLevel::Quarantined,
            TrustLevel::PublicUnverified,
            TrustLevel::PublicVerified,
            TrustLevel::OrganizationUnverified,
            TrustLevel::OrganizationVerified,
            TrustLevel::PrivateCloud,
            TrustLevel::Local,
        ] {
            let s = level.as_str();
            let parsed: TrustLevel = s.parse().unwrap();
            assert_eq!(parsed, level);
        }
    }

    #[test]
    fn test_trust_context_local() {
        let ctx = TrustContext::local("alice");
        assert_eq!(ctx.level, TrustLevel::Local);
        assert!(matches!(ctx.source, TrustSource::Local { .. }));
    }

    #[test]
    fn test_trust_context_imported() {
        let ctx = TrustContext::imported("rust-patterns.json");
        assert_eq!(ctx.level, TrustLevel::PublicUnverified);
    }

    #[test]
    fn test_trust_context_enterprise() {
        let ctx = TrustContext::enterprise("acme-corp", "alice", true);
        assert_eq!(ctx.level, TrustLevel::OrganizationVerified);
        assert_eq!(ctx.source.org_id(), Some("acme-corp"));
        assert_eq!(ctx.source.author_id(), Some("alice"));

        let ctx_unverified = TrustContext::enterprise("acme-corp", "bob", false);
        assert_eq!(ctx_unverified.level, TrustLevel::OrganizationUnverified);
    }
}
