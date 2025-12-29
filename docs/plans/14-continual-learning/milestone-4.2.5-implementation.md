# Milestone 4.2.5: Security Foundation - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement enterprise-ready security foundation for groove including policy system, trust model, content scanning, audit logging, and quarantine management.

**Architecture:** See [milestone-4.2.5-decisions.md](milestone-4.2.5-decisions.md) for design decisions.

**Tech Stack:** Rust, CozoDB, SHA-256, TOML (policy), JSONL (audit), TypeScript/React (Web UI)

---

## Phase 1: Core Security Types

### Task 1: Create Security Module Structure

**Files:**
- Create: `vibes-groove/src/security/mod.rs`
- Create: `vibes-groove/src/security/error.rs`
- Modify: `vibes-groove/src/lib.rs`

**Step 1: Create security module directory**

```bash
mkdir -p vibes-groove/src/security
```

**Step 2: Create security error types**

```rust
// vibes-groove/src/security/error.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    #[error("Content scan failed: {0}")]
    ScanFailed(String),

    #[error("Quarantined learning: {0}")]
    Quarantined(uuid::Uuid),

    #[error("Trust level insufficient: requires {required:?}, has {actual:?}")]
    InsufficientTrust {
        required: super::TrustLevel,
        actual: super::TrustLevel,
    },

    #[error("Policy load error: {0}")]
    PolicyLoad(String),

    #[error("Audit log error: {0}")]
    AuditLog(String),

    #[error("Provenance verification failed: {0}")]
    ProvenanceFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type SecurityResult<T> = Result<T, SecurityError>;
```

**Step 3: Create security module entry point**

```rust
// vibes-groove/src/security/mod.rs

//! Security foundation for groove
//!
//! This module provides:
//! - Trust hierarchy and context
//! - Policy system for enterprise control
//! - Content scanning for injection protection
//! - Audit logging for compliance
//! - Quarantine management

mod error;
mod trust;
mod provenance;
mod policy;
mod scanning;
mod audit;
mod quarantine;
mod injector;

pub use error::{SecurityError, SecurityResult};
pub use trust::*;
pub use provenance::*;
pub use policy::*;
pub use scanning::*;
pub use audit::*;
pub use quarantine::*;
pub use injector::*;
```

**Step 4: Add security module to lib.rs**

Add to `vibes-groove/src/lib.rs`:
```rust
pub mod security;
pub use security::*;
```

**Step 5: Run check**

Run: `cargo check -p vibes-groove`
Expected: Compilation errors (missing submodules) - that's okay for now

**Step 6: Commit**

```bash
git add vibes-groove/src/security/
git commit -m "feat(groove): create security module structure"
```

---

### Task 2: Trust Level and Context Types

**Files:**
- Create: `vibes-groove/src/security/trust.rs`

**Step 1: Write test for TrustLevel ordering**

```rust
// vibes-groove/src/security/trust.rs

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
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-groove trust`
Expected: FAIL - TrustLevel not defined

**Step 3: Implement TrustLevel enum**

```rust
// vibes-groove/src/security/trust.rs

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
```

**Step 4: Add TrustSource and TrustContext**

```rust
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerifiedBy {
    Curator { curator_id: String },
    CommunityVote { votes_required: u32, votes_received: u32 },
    Automated { checker: String },
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
            source: TrustSource::Local { user_id: user_id.into() },
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
    pub fn enterprise(org_id: impl Into<String>, author_id: impl Into<String>, verified: bool) -> Self {
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
    // ... tests from step 1 ...

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
}
```

**Step 5: Run tests**

Run: `cargo test -p vibes-groove trust`
Expected: All tests pass

**Step 6: Commit**

```bash
git add vibes-groove/src/security/trust.rs
git commit -m "feat(groove): add trust level and context types"
```

---

### Task 3: Provenance Types

**Files:**
- Create: `vibes-groove/src/security/provenance.rs`

**Step 1: Write tests for ContentHash**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash_from_content() {
        let content = "Use thiserror for library errors";
        let hash = ContentHash::from_content(content);

        // Same content produces same hash
        let hash2 = ContentHash::from_content(content);
        assert_eq!(hash, hash2);

        // Different content produces different hash
        let hash3 = ContentHash::from_content("Different content");
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_content_hash_hex_roundtrip() {
        let content = "Test content";
        let hash = ContentHash::from_content(content);
        let hex = hash.to_hex();
        let parsed = ContentHash::from_hex(&hex).unwrap();
        assert_eq!(hash, parsed);
    }

    #[test]
    fn test_content_hash_verification() {
        let content = "Original content";
        let hash = ContentHash::from_content(content);

        assert!(hash.verify(content));
        assert!(!hash.verify("Modified content"));
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p vibes-groove provenance`
Expected: FAIL - ContentHash not defined

**Step 3: Implement ContentHash**

```rust
// vibes-groove/src/security/provenance.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// SHA-256 content hash for integrity verification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash(#[serde(with = "hex_bytes")] pub [u8; 32]);

impl ContentHash {
    /// Compute hash from content
    pub fn from_content(content: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        Self(result.into())
    }

    /// Create from hex string
    pub fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Verify content matches this hash
    pub fn verify(&self, content: &str) -> bool {
        let computed = Self::from_content(content);
        self == &computed
    }
}

// Custom serialization for hex bytes
mod hex_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("expected 32 bytes"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(arr)
    }
}
```

**Step 4: Add Provenance and custody chain types**

```rust
/// Full provenance record for a learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    /// Hash of the current content
    pub content_hash: ContentHash,
    /// When and by whom this was created
    pub created: CreationEvent,
    /// Chain of modifications
    pub custody_chain: Vec<CustodyEvent>,
}

impl Provenance {
    /// Create new provenance for content
    pub fn new(content: &str, creator_id: impl Into<String>) -> Self {
        Self {
            content_hash: ContentHash::from_content(content),
            created: CreationEvent {
                creator_id: creator_id.into(),
                created_at: Utc::now(),
                device_id: None,
                source_type: "user_created".to_string(),
            },
            custody_chain: Vec::new(),
        }
    }

    /// Record a modification in the custody chain
    pub fn record_modification(
        &mut self,
        new_content: &str,
        actor: impl Into<String>,
        event_type: CustodyEventType,
    ) {
        let previous_hash = self.content_hash.clone();
        let new_hash = ContentHash::from_content(new_content);

        self.custody_chain.push(CustodyEvent {
            event_type,
            timestamp: Utc::now(),
            actor: actor.into(),
            previous_hash,
            new_hash: new_hash.clone(),
        });

        self.content_hash = new_hash;
    }

    /// Verify content matches the current hash
    pub fn verify(&self, content: &str) -> bool {
        self.content_hash.verify(content)
    }
}

/// Record of initial creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreationEvent {
    /// ID of the creator
    pub creator_id: String,
    /// When created
    pub created_at: DateTime<Utc>,
    /// Device ID if available
    pub device_id: Option<String>,
    /// Type of source (user_created, transcript, imported, etc.)
    pub source_type: String,
}

/// Record of a custody chain event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyEvent {
    /// Type of event
    pub event_type: CustodyEventType,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// Who performed the action
    pub actor: String,
    /// Hash before the change
    pub previous_hash: ContentHash,
    /// Hash after the change
    pub new_hash: ContentHash,
}

/// Types of custody chain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustodyEventType {
    /// Content was modified
    Modified,
    /// Learning was promoted to different tier
    Promoted { from_tier: String, to_tier: String },
    /// Learning was imported from external source
    Imported { source: String },
    /// Learning was transferred between users/orgs
    Transferred { from: String, to: String },
    /// Learning was verified/reviewed
    Verified { verifier: String },
}

#[cfg(test)]
mod tests {
    // ... tests from step 1 ...

    #[test]
    fn test_provenance_new() {
        let content = "Test learning content";
        let prov = Provenance::new(content, "alice");

        assert!(prov.verify(content));
        assert_eq!(prov.created.creator_id, "alice");
        assert!(prov.custody_chain.is_empty());
    }

    #[test]
    fn test_provenance_modification_tracking() {
        let content = "Original content";
        let mut prov = Provenance::new(content, "alice");

        let new_content = "Modified content";
        prov.record_modification(new_content, "bob", CustodyEventType::Modified);

        assert!(!prov.verify(content)); // Old content no longer matches
        assert!(prov.verify(new_content)); // New content matches
        assert_eq!(prov.custody_chain.len(), 1);
        assert_eq!(prov.custody_chain[0].actor, "bob");
    }
}
```

**Step 5: Run tests**

Run: `cargo test -p vibes-groove provenance`
Expected: All tests pass

**Step 6: Commit**

```bash
git add vibes-groove/src/security/provenance.rs
git commit -m "feat(groove): add provenance types with SHA-256 hashing"
```

---

### Task 4: Scanning Types

**Files:**
- Create: `vibes-groove/src/security/scanning.rs`

**Step 1: Write tests for scan types**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }

    #[test]
    fn test_scan_result_passed() {
        let result = ScanResult::passed();
        assert!(result.passed);
        assert!(result.findings.is_empty());
    }

    #[test]
    fn test_scan_result_with_findings() {
        let mut result = ScanResult::passed();
        result.add_finding(ScanFinding {
            severity: Severity::High,
            category: "prompt_injection".to_string(),
            pattern_matched: "ignore previous instructions".to_string(),
            location: Some("line 5".to_string()),
        });

        assert!(!result.passed);
        assert_eq!(result.findings.len(), 1);
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p vibes-groove scanning`
Expected: FAIL

**Step 3: Implement scanning types**

```rust
// vibes-groove/src/security/scanning.rs

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{SecurityResult, TrustLevel};

/// Severity of a scan finding
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// Log only, don't block
    Low = 0,
    /// Warn user, allow with confirmation
    Medium = 1,
    /// Block and surface to user
    High = 2,
    /// Block immediately
    Critical = 3,
}

impl Severity {
    /// Check if this severity should block the action
    pub fn should_block(&self) -> bool {
        *self >= Severity::High
    }
}

/// A finding from content scanning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanFinding {
    /// Severity of the finding
    pub severity: Severity,
    /// Category (e.g., "prompt_injection", "data_exfiltration")
    pub category: String,
    /// The pattern that matched
    pub pattern_matched: String,
    /// Where in the content (if applicable)
    pub location: Option<String>,
}

/// Result of scanning content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Whether the content passed scanning
    pub passed: bool,
    /// All findings from the scan
    pub findings: Vec<ScanFinding>,
    /// When the scan was performed
    pub scanned_at: DateTime<Utc>,
}

impl ScanResult {
    /// Create a passed result with no findings
    pub fn passed() -> Self {
        Self {
            passed: true,
            findings: Vec::new(),
            scanned_at: Utc::now(),
        }
    }

    /// Create a failed result
    pub fn failed(findings: Vec<ScanFinding>) -> Self {
        Self {
            passed: false,
            findings,
            scanned_at: Utc::now(),
        }
    }

    /// Add a finding and update passed status
    pub fn add_finding(&mut self, finding: ScanFinding) {
        if finding.severity.should_block() {
            self.passed = false;
        }
        self.findings.push(finding);
    }

    /// Get the highest severity finding
    pub fn max_severity(&self) -> Option<Severity> {
        self.findings.iter().map(|f| f.severity).max()
    }
}

/// Content scanner trait for different scanning implementations
#[async_trait]
pub trait ContentScanner: Send + Sync {
    /// Scan content for security issues
    async fn scan(&self, content: &str, trust_level: TrustLevel) -> SecurityResult<ScanResult>;

    /// Get the name of this scanner
    fn name(&self) -> &'static str;
}

/// DLP scanner trait for data loss prevention
#[async_trait]
pub trait DlpScanner: Send + Sync {
    /// Scan for sensitive data
    async fn scan(&self, content: &str) -> SecurityResult<Vec<ScanFinding>>;
}

/// Injection detector trait for prompt injection detection
#[async_trait]
pub trait InjectionDetector: Send + Sync {
    /// Detect potential injection attempts
    async fn detect(&self, content: &str) -> SecurityResult<Vec<ScanFinding>>;
}

/// No-op DLP scanner (default implementation)
pub struct NoOpDlpScanner;

#[async_trait]
impl DlpScanner for NoOpDlpScanner {
    async fn scan(&self, _content: &str) -> SecurityResult<Vec<ScanFinding>> {
        Ok(Vec::new())
    }
}

/// No-op injection detector (default implementation)
pub struct NoOpInjectionDetector;

#[async_trait]
impl InjectionDetector for NoOpInjectionDetector {
    async fn detect(&self, _content: &str) -> SecurityResult<Vec<ScanFinding>> {
        Ok(Vec::new())
    }
}
```

**Step 4: Run tests**

Run: `cargo test -p vibes-groove scanning`
Expected: All tests pass

**Step 5: Commit**

```bash
git add vibes-groove/src/security/scanning.rs
git commit -m "feat(groove): add scanning types and traits"
```

---

### Task 5: Quarantine Types

**Files:**
- Create: `vibes-groove/src/security/quarantine.rs`

**Step 1: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quarantine_status_pending_review() {
        let status = QuarantineStatus::new(QuarantineReason::ImportScanFailed, vec![]);
        assert!(status.review_outcome.is_none());
        assert!(status.is_pending_review());
    }

    #[test]
    fn test_quarantine_review() {
        let mut status = QuarantineStatus::new(QuarantineReason::ImportScanFailed, vec![]);
        status.review("admin", ReviewOutcome::Approved);

        assert!(!status.is_pending_review());
        assert_eq!(status.reviewed_by, Some("admin".to_string()));
    }
}
```

**Step 2: Run tests to verify they fail**

**Step 3: Implement quarantine types**

```rust
// vibes-groove/src/security/quarantine.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ScanFinding;

/// Why a learning was quarantined
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuarantineReason {
    /// Failed scan at import time
    ImportScanFailed,
    /// Policy changed and learning no longer passes
    PolicyRescanFailed,
    /// Administrator manually quarantined
    ManualQuarantine { admin_id: String },
    /// User reported as problematic
    UserReported { reporter_id: String, reason: String },
}

/// Outcome of a quarantine review
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewOutcome {
    /// False positive, restore to normal
    Approved,
    /// Confirmed bad, should be deleted
    Rejected,
    /// Needs higher authority review
    Escalated,
}

/// Quarantine status for a learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineStatus {
    /// When quarantined
    pub quarantined_at: DateTime<Utc>,
    /// Why quarantined
    pub reason: QuarantineReason,
    /// Scan findings that triggered quarantine
    pub scan_findings: Vec<ScanFinding>,
    /// Who reviewed (if reviewed)
    pub reviewed_by: Option<String>,
    /// Review outcome (if reviewed)
    pub review_outcome: Option<ReviewOutcome>,
    /// When reviewed (if reviewed)
    pub reviewed_at: Option<DateTime<Utc>>,
}

impl QuarantineStatus {
    /// Create new quarantine status
    pub fn new(reason: QuarantineReason, scan_findings: Vec<ScanFinding>) -> Self {
        Self {
            quarantined_at: Utc::now(),
            reason,
            scan_findings,
            reviewed_by: None,
            review_outcome: None,
            reviewed_at: None,
        }
    }

    /// Check if pending review
    pub fn is_pending_review(&self) -> bool {
        self.review_outcome.is_none()
    }

    /// Record a review
    pub fn review(&mut self, reviewer: impl Into<String>, outcome: ReviewOutcome) {
        self.reviewed_by = Some(reviewer.into());
        self.review_outcome = Some(outcome);
        self.reviewed_at = Some(Utc::now());
    }
}
```

**Step 4: Run tests**

Run: `cargo test -p vibes-groove quarantine`
Expected: All tests pass

**Step 5: Commit**

```bash
git add vibes-groove/src/security/quarantine.rs
git commit -m "feat(groove): add quarantine types"
```

---

### Task 6: RBAC Types

**Files:**
- Create: `vibes-groove/src/security/rbac.rs`

**Step 1: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_has_all_permissions() {
        let perms = OrgRole::Admin.permissions();
        assert!(perms.can_create);
        assert!(perms.can_read);
        assert!(perms.can_modify);
        assert!(perms.can_delete);
        assert!(perms.can_publish);
        assert!(perms.can_review);
        assert!(perms.can_admin);
    }

    #[test]
    fn test_viewer_read_only() {
        let perms = OrgRole::Viewer.permissions();
        assert!(!perms.can_create);
        assert!(perms.can_read);
        assert!(!perms.can_modify);
        assert!(!perms.can_delete);
    }
}
```

**Step 2: Implement RBAC types**

```rust
// vibes-groove/src/security/rbac.rs

use serde::{Deserialize, Serialize};

/// Organization role for RBAC
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrgRole {
    /// Full control over everything
    Admin,
    /// Can create, review, and publish learnings
    Curator,
    /// Can create and share learnings
    Member,
    /// Read-only access
    Viewer,
}

impl OrgRole {
    /// Get permissions for this role
    pub fn permissions(&self) -> Permissions {
        match self {
            Self::Admin => Permissions {
                can_create: true,
                can_read: true,
                can_modify: true,
                can_delete: true,
                can_publish: true,
                can_review: true,
                can_admin: true,
            },
            Self::Curator => Permissions {
                can_create: true,
                can_read: true,
                can_modify: true,
                can_delete: false,
                can_publish: true,
                can_review: true,
                can_admin: false,
            },
            Self::Member => Permissions {
                can_create: true,
                can_read: true,
                can_modify: false,
                can_delete: false,
                can_publish: false,
                can_review: false,
                can_admin: false,
            },
            Self::Viewer => Permissions {
                can_create: false,
                can_read: true,
                can_modify: false,
                can_delete: false,
                can_publish: false,
                can_review: false,
                can_admin: false,
            },
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Some(Self::Admin),
            "curator" => Some(Self::Curator),
            "member" => Some(Self::Member),
            "viewer" => Some(Self::Viewer),
            _ => None,
        }
    }
}

/// Permission flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permissions {
    pub can_create: bool,
    pub can_read: bool,
    pub can_modify: bool,
    pub can_delete: bool,
    pub can_publish: bool,
    pub can_review: bool,
    pub can_admin: bool,
}

impl Default for Permissions {
    fn default() -> Self {
        // Default to viewer permissions
        OrgRole::Viewer.permissions()
    }
}
```

**Step 3: Run tests**

Run: `cargo test -p vibes-groove rbac`
Expected: All tests pass

**Step 4: Commit**

```bash
git add vibes-groove/src/security/rbac.rs
git commit -m "feat(groove): add RBAC types"
```

---

### Task 7: Audit Types

**Files:**
- Create: `vibes-groove/src/security/audit.rs`

**Step 1: Write tests**

```rust
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
    }
}
```

**Step 2: Implement audit types**

```rust
// vibes-groove/src/security/audit.rs

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::SecurityResult;

/// Actor who performed an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActorId {
    User(String),
    System,
    Policy,
    Scanner,
}

/// Type of audited action
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
```

**Step 3: Run tests**

Run: `cargo test -p vibes-groove audit`
Expected: All tests pass

**Step 4: Commit**

```bash
git add vibes-groove/src/security/audit.rs
git commit -m "feat(groove): add audit log types"
```

---

## Phase 2: Policy System

### Task 8: Policy Schema Types

**Files:**
- Create: `vibes-groove/src/security/policy/mod.rs`
- Create: `vibes-groove/src/security/policy/schema.rs`

**Step 1: Create policy module structure**

```rust
// vibes-groove/src/security/policy/mod.rs

mod schema;
mod loader;
mod provider;

pub use schema::*;
pub use loader::*;
pub use provider::*;
```

**Step 2: Implement policy schema types**

```rust
// vibes-groove/src/security/policy/schema.rs

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Complete policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    #[serde(default)]
    pub identity: IdentityPolicy,
    #[serde(default)]
    pub tiers: TiersPolicy,
    #[serde(default)]
    pub capture: CapturePolicy,
    #[serde(default)]
    pub injection: InjectionPolicy,
    #[serde(default)]
    pub import_export: ImportExportPolicy,
    #[serde(default)]
    pub scanning: ScanningPolicy,
    #[serde(default)]
    pub audit: AuditPolicy,
    #[serde(default)]
    pub quarantine: QuarantinePolicy,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            identity: IdentityPolicy::default(),
            tiers: TiersPolicy::default(),
            capture: CapturePolicy::default(),
            injection: InjectionPolicy::default(),
            import_export: ImportExportPolicy::default(),
            scanning: ScanningPolicy::default(),
            audit: AuditPolicy::default(),
            quarantine: QuarantinePolicy::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IdentityPolicy {
    pub enterprise_id: Option<String>,
    pub policy_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TiersPolicy {
    pub allow_personal_tier: bool,
    pub allow_project_tier: bool,
    pub enterprise_tier_required: bool,
}

impl Default for TiersPolicy {
    fn default() -> Self {
        Self {
            allow_personal_tier: true,
            allow_project_tier: true,
            enterprise_tier_required: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturePolicy {
    pub allow_capture_on_personal: bool,
    pub allow_capture_on_enterprise: bool,
    pub require_review_before_store: bool,
}

impl Default for CapturePolicy {
    fn default() -> Self {
        Self {
            allow_capture_on_personal: true,
            allow_capture_on_enterprise: false,
            require_review_before_store: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionPolicy {
    pub allow_personal_injection: bool,
    pub allow_unverified_injection: bool,
    pub enterprise_overrides_personal: bool,
    pub block_quarantined: bool,
    #[serde(default)]
    pub presentation: PresentationPolicy,
}

impl Default for InjectionPolicy {
    fn default() -> Self {
        Self {
            allow_personal_injection: true,
            allow_unverified_injection: false,
            enterprise_overrides_personal: true,
            block_quarantined: true,
            presentation: PresentationPolicy::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PresentationPolicy {
    #[serde(default)]
    pub personal: WrapperConfig,
    #[serde(default)]
    pub enterprise: WrapperConfig,
    #[serde(default)]
    pub imported: WrapperConfig,
    #[serde(default)]
    pub quarantined: WrapperConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrapperConfig {
    pub wrapper: WrapperType,
    #[serde(default)]
    pub show_author: bool,
    #[serde(default)]
    pub show_verification: bool,
    #[serde(default)]
    pub warning_text: Option<String>,
    #[serde(default)]
    pub sanitize: bool,
}

impl Default for WrapperConfig {
    fn default() -> Self {
        Self {
            wrapper: WrapperType::None,
            show_author: false,
            show_verification: false,
            warning_text: None,
            sanitize: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WrapperType {
    #[default]
    None,
    SourceTag,
    Warning,
    StrongWarning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportExportPolicy {
    pub allow_import_from_file: bool,
    pub allow_import_from_url: bool,
    #[serde(default)]
    pub allowed_import_sources: Vec<String>,
    pub allow_export_personal: bool,
    pub allow_export_enterprise: bool,
}

impl Default for ImportExportPolicy {
    fn default() -> Self {
        Self {
            allow_import_from_file: true,
            allow_import_from_url: false,
            allowed_import_sources: Vec::new(),
            allow_export_personal: true,
            allow_export_enterprise: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanningPolicy {
    pub require_scan_on_import: bool,
    pub require_scan_on_inject: bool,
    pub block_on_scan_failure: bool,
    #[serde(default)]
    pub patterns: ScanPatterns,
    #[serde(default)]
    pub on_policy_change: PolicyChangeAction,
}

impl Default for ScanningPolicy {
    fn default() -> Self {
        Self {
            require_scan_on_import: true,
            require_scan_on_inject: false,
            block_on_scan_failure: true,
            patterns: ScanPatterns::default(),
            on_policy_change: PolicyChangeAction::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanPatterns {
    #[serde(default)]
    pub prompt_injection: Vec<String>,
    #[serde(default)]
    pub data_exfiltration: Vec<String>,
    #[serde(default)]
    pub secrets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyChangeAction {
    pub action: QuarantineAction,
    pub notify_admin: bool,
}

impl Default for PolicyChangeAction {
    fn default() -> Self {
        Self {
            action: QuarantineAction::Quarantine,
            notify_admin: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuarantineAction {
    #[default]
    Quarantine,
    SoftDelete,
    HardDelete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditPolicy {
    pub enabled: bool,
    pub retention_days: u32,
}

impl Default for AuditPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantinePolicy {
    #[serde(default)]
    pub reviewers: Vec<String>,
    #[serde(default)]
    pub visible_to: Vec<String>,
    pub auto_delete_after_days: Option<u32>,
}

impl Default for QuarantinePolicy {
    fn default() -> Self {
        Self {
            reviewers: vec!["Admin".to_string(), "Curator".to_string()],
            visible_to: vec!["Admin".to_string(), "Curator".to_string()],
            auto_delete_after_days: Some(90),
        }
    }
}
```

**Step 3: Add tests and run**

**Step 4: Commit**

```bash
git add vibes-groove/src/security/policy/
git commit -m "feat(groove): add policy schema types"
```

---

### Task 9: Policy Loader

**Files:**
- Create: `vibes-groove/src/security/policy/loader.rs`

Implement TOML-based policy loading with validation and defaults.

---

### Task 10: Policy Provider

**Files:**
- Create: `vibes-groove/src/security/policy/provider.rs`

Implement PolicyProvider trait with change subscription.

---

## Phase 3: Audit Log Implementation

### Task 11: JSONL Audit Log

**Files:**
- Create: `vibes-groove/src/security/audit/file_log.rs`

Implement append-only JSONL file-based audit logging with rotation.

---

## Phase 4: Content Scanner Implementation

### Task 12: Regex Scanner

**Files:**
- Create: `vibes-groove/src/security/scanning/regex_scanner.rs`

Implement regex-based content scanner using policy patterns.

---

## Phase 5: Secure Injector

### Task 13: SecureInjector Implementation

**Files:**
- Create: `vibes-groove/src/security/injector.rs`

Implement SecureInjector with policy checks, wrapping, and audit.

---

## Phase 6: Storage Integration

### Task 14: Extend Learning Type

**Files:**
- Modify: `vibes-groove/src/types/learning.rs`
- Modify: `vibes-groove/src/store/schema.rs`

Add `trust_level` and `provenance_hash` fields to Learning.

---

### Task 15: Security Metadata Storage

**Files:**
- Create: `vibes-groove/src/security/storage.rs`
- Modify: `vibes-groove/src/store/schema.rs`

Add security_metadata CozoDB relation.

---

## Phase 7: Quarantine Management

### Task 16: Quarantine Storage Operations

**Files:**
- Create: `vibes-groove/src/security/quarantine/store.rs`

Implement quarantine CRUD operations.

---

### Task 17: Policy Rescan Job

**Files:**
- Create: `vibes-groove/src/security/quarantine/rescan.rs`

Implement background rescan when policy changes.

---

## Phase 8: CLI Commands

### Task 18: Groove CLI Commands

**Files:**
- Create: `vibes-cli/src/commands/groove.rs`
- Modify: `vibes-cli/src/commands/mod.rs`

Add `vibes groove quarantine list|show|review` commands.

---

## Phase 9: API Endpoints

### Task 19: Quarantine API

**Files:**
- Create: `vibes-server/src/http/groove.rs`
- Modify: `vibes-server/src/http/mod.rs`

Add REST endpoints for quarantine management.

---

## Phase 10: Web UI

### Task 20: Quarantine Page

**Files:**
- Create: `web-ui/src/pages/Quarantine.tsx`
- Create: `web-ui/src/components/QuarantineList.tsx`
- Create: `web-ui/src/components/QuarantineReview.tsx`
- Modify: `web-ui/src/App.tsx`

Add quarantine management UI.

---

## Phase 11: Finalization

### Task 21: Default Secure Policy

**Files:**
- Create: `vibes-groove/resources/default-secure-policy.toml`

Ship default secure policy with recommended patterns.

---

### Task 22: Update Cargo.toml

**Files:**
- Modify: `vibes-groove/Cargo.toml`

Add new dependencies: `sha2`, `hex`, `regex`, `toml`.

---

### Task 23: Documentation

**Files:**
- Update: `docs/PROGRESS.md`

Mark milestone 4.2.5 as complete.

---

## Summary

| Phase | Tasks | Focus |
|-------|-------|-------|
| 1 | 1-7 | Core security types |
| 2 | 8-10 | Policy system |
| 3 | 11 | Audit logging |
| 4 | 12 | Content scanning |
| 5 | 13 | SecureInjector |
| 6 | 14-15 | Storage integration |
| 7 | 16-17 | Quarantine management |
| 8 | 18 | CLI commands |
| 9 | 19 | API endpoints |
| 10 | 20 | Web UI |
| 11 | 21-23 | Finalization |

**Total Tasks:** 23
**Estimated Commits:** 25-30

Each task follows TDD: write failing test → implement → verify → commit.
