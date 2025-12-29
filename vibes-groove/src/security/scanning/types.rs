//! Scanning types
//!
//! Core types for content scanning.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::security::{SecurityResult, TrustLevel};

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
    fn test_severity_should_block() {
        assert!(!Severity::Low.should_block());
        assert!(!Severity::Medium.should_block());
        assert!(Severity::High.should_block());
        assert!(Severity::Critical.should_block());
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

    #[test]
    fn test_scan_result_low_severity_doesnt_fail() {
        let mut result = ScanResult::passed();
        result.add_finding(ScanFinding {
            severity: Severity::Low,
            category: "style".to_string(),
            pattern_matched: "minor issue".to_string(),
            location: None,
        });

        assert!(result.passed); // Low severity doesn't fail
        assert_eq!(result.findings.len(), 1);
    }

    #[test]
    fn test_scan_result_max_severity() {
        let mut result = ScanResult::passed();
        result.add_finding(ScanFinding {
            severity: Severity::Low,
            category: "a".to_string(),
            pattern_matched: "x".to_string(),
            location: None,
        });
        result.add_finding(ScanFinding {
            severity: Severity::High,
            category: "b".to_string(),
            pattern_matched: "y".to_string(),
            location: None,
        });
        result.add_finding(ScanFinding {
            severity: Severity::Medium,
            category: "c".to_string(),
            pattern_matched: "z".to_string(),
            location: None,
        });

        assert_eq!(result.max_severity(), Some(Severity::High));
    }

    #[tokio::test]
    async fn test_noop_dlp_scanner() {
        let scanner = NoOpDlpScanner;
        let findings = scanner.scan("test content").await.unwrap();
        assert!(findings.is_empty());
    }

    #[tokio::test]
    async fn test_noop_injection_detector() {
        let detector = NoOpInjectionDetector;
        let findings = detector.detect("test content").await.unwrap();
        assert!(findings.is_empty());
    }
}
