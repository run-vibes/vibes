//! Policy schema types
//!
//! Comprehensive policy configuration for enterprise control.

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

/// Identity and versioning for the policy
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IdentityPolicy {
    pub enterprise_id: Option<String>,
    pub policy_version: Option<String>,
}

/// Which storage tiers are allowed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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

/// Learning capture policy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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

/// Learning injection policy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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

/// Presentation wrapper configuration per source type
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

/// Configuration for injection presentation wrapper
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

/// Type of wrapper to use for injected content
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WrapperType {
    #[default]
    None,
    SourceTag,
    Warning,
    StrongWarning,
}

/// Import/export policy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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

/// Content scanning policy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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

/// Regex patterns for content scanning
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanPatterns {
    #[serde(default)]
    pub prompt_injection: Vec<String>,
    #[serde(default)]
    pub data_exfiltration: Vec<String>,
    #[serde(default)]
    pub secrets: Vec<String>,
}

/// What to do when policy changes affect existing learnings
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

/// Action to take for policy violations
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuarantineAction {
    #[default]
    Quarantine,
    SoftDelete,
    HardDelete,
}

/// Audit logging policy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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

/// Quarantine management policy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_default() {
        let policy = Policy::default();
        assert!(policy.tiers.allow_personal_tier);
        assert!(policy.injection.allow_personal_injection);
        assert_eq!(policy.audit.retention_days, 30);
    }

    #[test]
    fn test_policy_toml_roundtrip() {
        let policy = Policy::default();
        let toml = toml::to_string(&policy).unwrap();
        let parsed: Policy = toml::from_str(&toml).unwrap();

        assert_eq!(
            parsed.tiers.allow_personal_tier,
            policy.tiers.allow_personal_tier
        );
        assert_eq!(parsed.audit.retention_days, policy.audit.retention_days);
    }

    #[test]
    fn test_policy_json_roundtrip() {
        let policy = Policy::default();
        let json = serde_json::to_string(&policy).unwrap();
        let parsed: Policy = serde_json::from_str(&json).unwrap();

        assert_eq!(
            parsed.injection.block_quarantined,
            policy.injection.block_quarantined
        );
    }

    #[test]
    fn test_wrapper_type_serialization() {
        assert_eq!(
            serde_json::to_string(&WrapperType::None).unwrap(),
            "\"none\""
        );
        assert_eq!(
            serde_json::to_string(&WrapperType::SourceTag).unwrap(),
            "\"source_tag\""
        );
        assert_eq!(
            serde_json::to_string(&WrapperType::Warning).unwrap(),
            "\"warning\""
        );
    }

    #[test]
    fn test_quarantine_action_serialization() {
        assert_eq!(
            serde_json::to_string(&QuarantineAction::Quarantine).unwrap(),
            "\"quarantine\""
        );
        assert_eq!(
            serde_json::to_string(&QuarantineAction::HardDelete).unwrap(),
            "\"hard_delete\""
        );
    }

    #[test]
    fn test_scan_patterns_default_empty() {
        let patterns = ScanPatterns::default();
        assert!(patterns.prompt_injection.is_empty());
        assert!(patterns.data_exfiltration.is_empty());
        assert!(patterns.secrets.is_empty());
    }

    #[test]
    fn test_policy_with_custom_patterns() {
        let toml = r#"
[scanning.patterns]
prompt_injection = ["ignore previous", "system prompt"]
secrets = ["AKIA[0-9A-Z]{16}"]
"#;
        let policy: Policy = toml::from_str(toml).unwrap();
        assert_eq!(policy.scanning.patterns.prompt_injection.len(), 2);
        assert_eq!(policy.scanning.patterns.secrets.len(), 1);
    }

    #[test]
    fn test_presentation_policy() {
        let config = WrapperConfig {
            wrapper: WrapperType::Warning,
            show_author: true,
            show_verification: true,
            warning_text: Some("Imported content".into()),
            sanitize: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: WrapperConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.wrapper, WrapperType::Warning);
        assert!(parsed.show_author);
        assert_eq!(parsed.warning_text, Some("Imported content".into()));
    }
}
