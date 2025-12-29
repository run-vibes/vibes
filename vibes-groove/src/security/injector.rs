//! Secure injector pipeline
//!
//! Provides policy-aware injection with trust validation and scanning.

use std::sync::Arc;

use async_trait::async_trait;

use super::{
    ContentScanner, InjectionPolicy, Policy, QuarantineStatus, ScanResult, SecurityError,
    SecurityResult, TrustContext, TrustLevel, TrustSource, WrapperConfig, WrapperType,
};

/// Result of processing content for injection
#[derive(Debug, Clone)]
pub struct InjectionResult {
    /// The processed content (with any wrappers applied)
    pub content: String,
    /// Scan result if scanning was performed
    pub scan_result: Option<ScanResult>,
    /// Whether any transformation was applied
    pub was_transformed: bool,
}

/// Configuration for the secure injector
#[derive(Default)]
pub struct InjectorConfig {
    /// The injection policy to enforce
    pub policy: InjectionPolicy,
    /// Optional content scanner
    pub scanner: Option<Arc<dyn ContentScanner>>,
}

/// Secure content injector
///
/// Enforces policy checks before allowing content injection.
pub struct SecureInjector {
    config: InjectorConfig,
}

impl SecureInjector {
    /// Create a new secure injector with the given configuration
    pub fn new(config: InjectorConfig) -> Self {
        Self { config }
    }

    /// Create from a full policy
    pub fn from_policy(policy: &Policy) -> Self {
        Self::new(InjectorConfig {
            policy: policy.injection.clone(),
            scanner: None,
        })
    }

    /// Create with a scanner
    pub fn with_scanner(mut self, scanner: Arc<dyn ContentScanner>) -> Self {
        self.config.scanner = Some(scanner);
        self
    }

    /// Check if injection is allowed for the given trust context and quarantine status
    pub fn is_injection_allowed(
        &self,
        ctx: &TrustContext,
        quarantine: Option<&QuarantineStatus>,
    ) -> bool {
        // Check quarantine status - block if pending review and policy requires
        if quarantine.is_some_and(|s| s.is_pending_review() && self.config.policy.block_quarantined)
        {
            return false;
        }

        // Check trust level - quarantined content is never allowed
        if ctx.level == TrustLevel::Quarantined {
            return false;
        }

        // Check source-based policy
        match &ctx.source {
            TrustSource::Local { .. } => true, // Always allow local content
            TrustSource::Enterprise { .. } => true, // Enterprise content always allowed
            TrustSource::Imported { .. } => {
                // Imported content requires verification unless policy allows unverified
                if self.config.policy.allow_unverified_injection {
                    true
                } else {
                    ctx.verification.is_some()
                }
            }
            TrustSource::Public { .. } => {
                // Public content follows personal injection policy
                self.config.policy.allow_personal_injection
            }
        }
    }

    /// Get the wrapper config for a trust source
    fn get_wrapper_config(&self, source: &TrustSource) -> &WrapperConfig {
        let presentation = &self.config.policy.presentation;
        match source {
            TrustSource::Local { .. } | TrustSource::Public { .. } => &presentation.personal,
            TrustSource::Enterprise { .. } => &presentation.enterprise,
            TrustSource::Imported { .. } => &presentation.imported,
        }
    }

    /// Apply wrapper to content based on policy
    fn apply_wrapper(&self, content: &str, ctx: &TrustContext) -> String {
        let wrapper_config = self.get_wrapper_config(&ctx.source);

        match wrapper_config.wrapper {
            WrapperType::None => content.to_string(),
            WrapperType::SourceTag => {
                let source_type = match &ctx.source {
                    TrustSource::Local { .. } => "Local",
                    TrustSource::Enterprise { .. } => "Enterprise",
                    TrustSource::Imported { .. } => "Imported",
                    TrustSource::Public { .. } => "Public",
                };
                let mut tag_parts = vec![format!("[Source: {}]", source_type)];
                if let Some(author) = wrapper_config
                    .show_author
                    .then(|| ctx.source.author_id())
                    .flatten()
                {
                    tag_parts.push(format!("[Author: {}]", author));
                }
                if let Some(verification) = wrapper_config
                    .show_verification
                    .then_some(ctx.verification.as_ref())
                    .flatten()
                {
                    let verifier = match &verification.verified_by {
                        super::VerifiedBy::Curator { curator_id } => {
                            format!("Curator({})", curator_id)
                        }
                        super::VerifiedBy::CommunityVote { votes_received, .. } => {
                            format!("Community({} votes)", votes_received)
                        }
                        super::VerifiedBy::Automated { checker } => {
                            format!("Automated({})", checker)
                        }
                    };
                    tag_parts.push(format!("[Verified by: {}]", verifier));
                }
                format!("{}\n{}", tag_parts.join(" "), content)
            }
            WrapperType::Warning => {
                let warning = wrapper_config
                    .warning_text
                    .as_deref()
                    .unwrap_or("Content from external source");
                format!("⚠️ {}\n\n{}", warning, content)
            }
            WrapperType::StrongWarning => {
                let warning = wrapper_config
                    .warning_text
                    .as_deref()
                    .unwrap_or("CAUTION: Unverified content");
                format!(
                    "🚨 {} 🚨\n━━━━━━━━━━━━━━━━━━━━\n{}\n━━━━━━━━━━━━━━━━━━━━",
                    warning, content
                )
            }
        }
    }

    /// Process content for injection
    ///
    /// This checks policy, optionally scans content, and applies wrappers.
    pub async fn process(
        &self,
        content: &str,
        ctx: &TrustContext,
        quarantine: Option<&QuarantineStatus>,
    ) -> SecurityResult<InjectionResult> {
        // Check if injection is allowed
        if !self.is_injection_allowed(ctx, quarantine) {
            let reason = if ctx.level == TrustLevel::Quarantined {
                "content is quarantined"
            } else if quarantine.map(|q| q.is_pending_review()).unwrap_or(false) {
                "content pending quarantine review"
            } else {
                "injection not allowed by policy"
            };
            return Err(SecurityError::PolicyViolation(reason.to_string()));
        }

        // Scan content if scanner is available and trust level requires it
        let scan_result = if let Some(ref scanner) = self.config.scanner {
            if ctx.level.requires_scanning() {
                let result = scanner.scan(content, ctx.level).await?;
                if !result.passed {
                    return Err(SecurityError::ScanFailed(format!(
                        "content failed security scan: {} findings",
                        result.findings.len()
                    )));
                }
                Some(result)
            } else {
                None
            }
        } else {
            None
        };

        // Apply wrapper
        let wrapper_config = self.get_wrapper_config(&ctx.source);
        let processed_content = if wrapper_config.wrapper != WrapperType::None {
            self.apply_wrapper(content, ctx)
        } else {
            content.to_string()
        };

        Ok(InjectionResult {
            content: processed_content,
            scan_result,
            was_transformed: wrapper_config.wrapper != WrapperType::None,
        })
    }
}

/// Trait for injectable content providers
#[async_trait]
pub trait InjectableContent: Send + Sync {
    /// Get the content to inject
    fn content(&self) -> &str;

    /// Get the trust context
    fn trust_context(&self) -> &TrustContext;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::{
        PresentationPolicy, QuarantineReason, ScanFinding, Severity, Verification, VerifiedBy,
    };
    use chrono::Utc;

    fn local_ctx() -> TrustContext {
        TrustContext::local("test-user")
    }

    fn imported_ctx() -> TrustContext {
        TrustContext::imported("test-source")
    }

    #[test]
    fn test_injection_allowed_local() {
        let injector = SecureInjector::new(InjectorConfig::default());
        let ctx = local_ctx();
        assert!(injector.is_injection_allowed(&ctx, None));
    }

    #[test]
    fn test_injection_allowed_enterprise() {
        let injector = SecureInjector::new(InjectorConfig::default());
        let ctx = TrustContext::enterprise("acme", "alice", true);
        assert!(injector.is_injection_allowed(&ctx, None));
    }

    #[test]
    fn test_injection_blocked_quarantined_level() {
        let injector = SecureInjector::new(InjectorConfig::default());
        let ctx = TrustContext {
            level: TrustLevel::Quarantined,
            source: TrustSource::Local {
                user_id: "test".into(),
            },
            verification: None,
        };
        assert!(!injector.is_injection_allowed(&ctx, None));
    }

    #[test]
    fn test_injection_blocked_pending_quarantine() {
        let injector = SecureInjector::new(InjectorConfig::default());
        let ctx = local_ctx();
        let quarantine = QuarantineStatus::new(QuarantineReason::ImportScanFailed, vec![]);
        assert!(!injector.is_injection_allowed(&ctx, Some(&quarantine)));
    }

    #[test]
    fn test_injection_allowed_reviewed_quarantine() {
        let injector = SecureInjector::new(InjectorConfig::default());
        let ctx = local_ctx();
        let mut quarantine = QuarantineStatus::new(QuarantineReason::ImportScanFailed, vec![]);
        quarantine.review("admin", crate::security::ReviewOutcome::Approved);
        assert!(injector.is_injection_allowed(&ctx, Some(&quarantine)));
    }

    #[test]
    fn test_injection_allowed_quarantine_when_policy_allows() {
        let config = InjectorConfig {
            policy: InjectionPolicy {
                block_quarantined: false,
                ..Default::default()
            },
            scanner: None,
        };
        let injector = SecureInjector::new(config);
        let ctx = local_ctx();
        let quarantine = QuarantineStatus::new(QuarantineReason::ImportScanFailed, vec![]);
        assert!(injector.is_injection_allowed(&ctx, Some(&quarantine)));
    }

    #[test]
    fn test_injection_imported_unverified_blocked() {
        let config = InjectorConfig {
            policy: InjectionPolicy {
                allow_unverified_injection: false,
                ..Default::default()
            },
            scanner: None,
        };
        let injector = SecureInjector::new(config);
        let ctx = imported_ctx();
        assert!(!injector.is_injection_allowed(&ctx, None));
    }

    #[test]
    fn test_injection_imported_verified_allowed() {
        let config = InjectorConfig {
            policy: InjectionPolicy {
                allow_unverified_injection: false,
                ..Default::default()
            },
            scanner: None,
        };
        let injector = SecureInjector::new(config);
        let mut ctx = imported_ctx();
        ctx.verification = Some(Verification {
            verified_by: VerifiedBy::Automated {
                checker: "test".into(),
            },
            verified_at: Utc::now(),
            expires_at: None,
        });
        assert!(injector.is_injection_allowed(&ctx, None));
    }

    #[test]
    fn test_injection_public_follows_personal_policy() {
        let config = InjectorConfig {
            policy: InjectionPolicy {
                allow_personal_injection: false,
                ..Default::default()
            },
            scanner: None,
        };
        let injector = SecureInjector::new(config);
        let ctx = TrustContext {
            level: TrustLevel::PublicUnverified,
            source: TrustSource::Public {
                author_id: Some("community-user".into()),
                source_url: None,
            },
            verification: None,
        };
        assert!(!injector.is_injection_allowed(&ctx, None));
    }

    #[tokio::test]
    async fn test_process_no_wrapper() {
        let injector = SecureInjector::new(InjectorConfig::default());
        let ctx = local_ctx();
        let result = injector.process("test content", &ctx, None).await.unwrap();

        assert_eq!(result.content, "test content");
        assert!(!result.was_transformed);
        assert!(result.scan_result.is_none());
    }

    #[tokio::test]
    async fn test_process_with_source_tag() {
        let config = InjectorConfig {
            policy: InjectionPolicy {
                presentation: PresentationPolicy {
                    personal: WrapperConfig {
                        wrapper: WrapperType::SourceTag,
                        show_author: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            scanner: None,
        };
        let injector = SecureInjector::new(config);
        let ctx = local_ctx();

        let result = injector.process("test content", &ctx, None).await.unwrap();

        assert!(result.content.contains("[Source: Local]"));
        assert!(result.content.contains("[Author: test-user]"));
        assert!(result.content.contains("test content"));
        assert!(result.was_transformed);
    }

    #[tokio::test]
    async fn test_process_with_warning() {
        let config = InjectorConfig {
            policy: InjectionPolicy {
                presentation: PresentationPolicy {
                    imported: WrapperConfig {
                        wrapper: WrapperType::Warning,
                        warning_text: Some("External content".into()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                allow_unverified_injection: true,
                ..Default::default()
            },
            scanner: None,
        };
        let injector = SecureInjector::new(config);
        let ctx = imported_ctx();

        let result = injector.process("test content", &ctx, None).await.unwrap();

        assert!(result.content.contains("⚠️"));
        assert!(result.content.contains("External content"));
        assert!(result.was_transformed);
    }

    #[tokio::test]
    async fn test_process_blocked_by_policy() {
        let config = InjectorConfig {
            policy: InjectionPolicy {
                allow_unverified_injection: false,
                ..Default::default()
            },
            scanner: None,
        };
        let injector = SecureInjector::new(config);
        let ctx = imported_ctx();

        let result = injector.process("test content", &ctx, None).await;
        assert!(matches!(result, Err(SecurityError::PolicyViolation(_))));
    }

    // Mock scanner for testing
    struct MockScanner {
        should_pass: bool,
    }

    #[async_trait]
    impl ContentScanner for MockScanner {
        async fn scan(
            &self,
            _content: &str,
            _trust_level: TrustLevel,
        ) -> SecurityResult<ScanResult> {
            if self.should_pass {
                Ok(ScanResult::passed())
            } else {
                Ok(ScanResult::failed(vec![ScanFinding {
                    severity: Severity::Critical,
                    category: "test".into(),
                    pattern_matched: "test".into(),
                    location: None,
                }]))
            }
        }

        fn name(&self) -> &'static str {
            "mock"
        }
    }

    #[tokio::test]
    async fn test_process_with_scanner_pass() {
        let scanner = Arc::new(MockScanner { should_pass: true });
        let injector = SecureInjector::new(InjectorConfig::default()).with_scanner(scanner);
        // Use a trust level that requires scanning
        let ctx = TrustContext {
            level: TrustLevel::PublicUnverified,
            source: TrustSource::Public {
                author_id: None,
                source_url: None,
            },
            verification: None,
        };

        let result = injector.process("test content", &ctx, None).await.unwrap();

        assert!(result.scan_result.is_some());
        assert!(result.scan_result.unwrap().passed);
    }

    #[tokio::test]
    async fn test_process_with_scanner_fail() {
        let scanner = Arc::new(MockScanner { should_pass: false });
        let injector = SecureInjector::new(InjectorConfig::default()).with_scanner(scanner);
        // Use a trust level that requires scanning
        let ctx = TrustContext {
            level: TrustLevel::PublicUnverified,
            source: TrustSource::Public {
                author_id: None,
                source_url: None,
            },
            verification: None,
        };

        let result = injector.process("test content", &ctx, None).await;
        assert!(matches!(result, Err(SecurityError::ScanFailed(_))));
    }

    #[tokio::test]
    async fn test_process_skips_scanning_for_trusted_content() {
        let scanner = Arc::new(MockScanner { should_pass: false });
        let injector = SecureInjector::new(InjectorConfig::default()).with_scanner(scanner);
        let ctx = local_ctx(); // Local content doesn't require scanning

        // Should pass even though scanner would fail, because Local doesn't require scanning
        let result = injector.process("test content", &ctx, None).await.unwrap();
        assert!(result.scan_result.is_none());
    }
}
