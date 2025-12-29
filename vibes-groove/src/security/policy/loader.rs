//! Policy loading from TOML files
//!
//! Provides file-based policy loading with validation.

use std::path::Path;

use super::Policy;
use crate::security::{SecurityError, SecurityResult};

/// Load policy from a TOML file
pub fn load_policy_from_file(path: impl AsRef<Path>) -> SecurityResult<Policy> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path).map_err(|e| {
        SecurityError::PolicyLoad(format!("failed to read policy file {:?}: {}", path, e))
    })?;

    parse_policy(&content)
}

/// Parse policy from TOML string
pub fn parse_policy(toml_content: &str) -> SecurityResult<Policy> {
    toml::from_str(toml_content)
        .map_err(|e| SecurityError::PolicyLoad(format!("invalid policy TOML: {}", e)))
}

/// Load policy with fallback to default
pub fn load_policy_or_default(path: impl AsRef<Path>) -> Policy {
    load_policy_from_file(path).unwrap_or_default()
}

/// Validate a policy for consistency
pub fn validate_policy(policy: &Policy) -> SecurityResult<()> {
    // Check for contradictory settings
    if policy.tiers.enterprise_tier_required && policy.tiers.allow_personal_tier {
        // This is a warning-level issue, not an error
        // Enterprise can override personal, so this is technically allowed
    }

    // Ensure quarantine reviewers exist if quarantine action is enabled
    if policy.scanning.on_policy_change.action == super::QuarantineAction::Quarantine
        && policy.quarantine.reviewers.is_empty()
    {
        return Err(SecurityError::PolicyLoad(
            "quarantine action requires at least one reviewer".into(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_policy_empty() {
        let policy = parse_policy("").unwrap();
        // Should use all defaults
        assert!(policy.tiers.allow_personal_tier);
        assert_eq!(policy.audit.retention_days, 30);
    }

    #[test]
    fn test_parse_policy_partial() {
        let toml = r#"
[tiers]
allow_personal_tier = false

[audit]
retention_days = 90
"#;
        let policy = parse_policy(toml).unwrap();
        assert!(!policy.tiers.allow_personal_tier);
        assert_eq!(policy.audit.retention_days, 90);
        // Others should be default
        assert!(policy.injection.allow_personal_injection);
    }

    #[test]
    fn test_parse_policy_with_patterns() {
        let toml = r#"
[scanning.patterns]
prompt_injection = ["ignore.*instructions", "system\\s+prompt"]
secrets = ["AKIA[A-Z0-9]{16}"]
"#;
        let policy = parse_policy(toml).unwrap();
        assert_eq!(policy.scanning.patterns.prompt_injection.len(), 2);
        assert_eq!(policy.scanning.patterns.secrets.len(), 1);
    }

    #[test]
    fn test_parse_policy_invalid_toml() {
        let result = parse_policy("invalid { toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_policy_valid() {
        let policy = Policy::default();
        assert!(validate_policy(&policy).is_ok());
    }

    #[test]
    fn test_validate_policy_no_reviewers() {
        let mut policy = Policy::default();
        policy.quarantine.reviewers = vec![];
        // Should fail because quarantine action requires reviewers
        assert!(validate_policy(&policy).is_err());
    }

    #[test]
    fn test_load_policy_or_default_missing_file() {
        let policy = load_policy_or_default("/nonexistent/path/policy.toml");
        // Should return default
        assert!(policy.tiers.allow_personal_tier);
    }
}
