//! Quarantine management
//!
//! Provides quarantine workflow for suspicious learnings.

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

    /// Check if the learning should be restored
    pub fn should_restore(&self) -> bool {
        matches!(self.review_outcome, Some(ReviewOutcome::Approved))
    }

    /// Check if the learning should be deleted
    pub fn should_delete(&self) -> bool {
        matches!(self.review_outcome, Some(ReviewOutcome::Rejected))
    }
}

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

    #[test]
    fn test_quarantine_should_restore() {
        let mut status = QuarantineStatus::new(QuarantineReason::PolicyRescanFailed, vec![]);
        assert!(!status.should_restore());

        status.review("curator", ReviewOutcome::Approved);
        assert!(status.should_restore());
        assert!(!status.should_delete());
    }

    #[test]
    fn test_quarantine_should_delete() {
        let mut status = QuarantineStatus::new(
            QuarantineReason::UserReported {
                reporter_id: "user1".into(),
                reason: "suspicious content".into(),
            },
            vec![],
        );

        status.review("admin", ReviewOutcome::Rejected);
        assert!(status.should_delete());
        assert!(!status.should_restore());
    }

    #[test]
    fn test_quarantine_escalated() {
        let mut status = QuarantineStatus::new(
            QuarantineReason::ManualQuarantine {
                admin_id: "admin1".into(),
            },
            vec![],
        );

        status.review("curator", ReviewOutcome::Escalated);
        assert!(!status.is_pending_review());
        assert!(!status.should_restore());
        assert!(!status.should_delete());
    }
}
