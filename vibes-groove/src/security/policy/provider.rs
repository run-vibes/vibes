//! Policy provider trait and implementations
//!
//! Provides runtime policy access with change notification.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{RwLock, broadcast};

use super::{loader, Policy};
use crate::security::{SecurityResult, SecurityError};

/// Policy provider trait
#[async_trait]
pub trait PolicyProvider: Send + Sync {
    /// Get the current policy
    async fn get_policy(&self) -> Policy;

    /// Subscribe to policy changes
    fn subscribe(&self) -> broadcast::Receiver<Policy>;

    /// Reload policy from source
    async fn reload(&self) -> SecurityResult<()>;
}

/// File-based policy provider
pub struct FilePolicyProvider {
    path: PathBuf,
    policy: Arc<RwLock<Policy>>,
    sender: broadcast::Sender<Policy>,
}

impl FilePolicyProvider {
    /// Create a new file-based policy provider
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let policy = loader::load_policy_or_default(&path);
        let (sender, _) = broadcast::channel(16);

        Self {
            path,
            policy: Arc::new(RwLock::new(policy)),
            sender,
        }
    }

    /// Create with a specific initial policy
    pub fn with_policy(path: impl Into<PathBuf>, policy: Policy) -> Self {
        let (sender, _) = broadcast::channel(16);

        Self {
            path: path.into(),
            policy: Arc::new(RwLock::new(policy)),
            sender,
        }
    }
}

#[async_trait]
impl PolicyProvider for FilePolicyProvider {
    async fn get_policy(&self) -> Policy {
        self.policy.read().await.clone()
    }

    fn subscribe(&self) -> broadcast::Receiver<Policy> {
        self.sender.subscribe()
    }

    async fn reload(&self) -> SecurityResult<()> {
        let new_policy = loader::load_policy_from_file(&self.path)?;
        loader::validate_policy(&new_policy)?;

        let mut policy = self.policy.write().await;
        *policy = new_policy.clone();

        // Notify subscribers (ignore if no receivers)
        let _ = self.sender.send(new_policy);

        Ok(())
    }
}

/// In-memory policy provider (for testing)
pub struct MemoryPolicyProvider {
    policy: Arc<RwLock<Policy>>,
    sender: broadcast::Sender<Policy>,
}

impl MemoryPolicyProvider {
    /// Create with default policy
    pub fn new() -> Self {
        Self::with_policy(Policy::default())
    }

    /// Create with specific policy
    pub fn with_policy(policy: Policy) -> Self {
        let (sender, _) = broadcast::channel(16);
        Self {
            policy: Arc::new(RwLock::new(policy)),
            sender,
        }
    }

    /// Update the policy
    pub async fn set_policy(&self, policy: Policy) -> SecurityResult<()> {
        loader::validate_policy(&policy)?;

        let mut current = self.policy.write().await;
        *current = policy.clone();

        // Notify subscribers
        let _ = self.sender.send(policy);

        Ok(())
    }
}

impl Default for MemoryPolicyProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PolicyProvider for MemoryPolicyProvider {
    async fn get_policy(&self) -> Policy {
        self.policy.read().await.clone()
    }

    fn subscribe(&self) -> broadcast::Receiver<Policy> {
        self.sender.subscribe()
    }

    async fn reload(&self) -> SecurityResult<()> {
        // Memory provider doesn't have a source to reload from
        Err(SecurityError::PolicyLoad(
            "memory provider does not support reload".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_memory_provider_default() {
        let provider = MemoryPolicyProvider::new();
        let policy = provider.get_policy().await;
        assert!(policy.tiers.allow_personal_tier);
    }

    #[tokio::test]
    async fn test_memory_provider_set_policy() {
        let provider = MemoryPolicyProvider::new();

        let mut new_policy = Policy::default();
        new_policy.audit.retention_days = 60;
        provider.set_policy(new_policy).await.unwrap();

        let policy = provider.get_policy().await;
        assert_eq!(policy.audit.retention_days, 60);
    }

    #[tokio::test]
    async fn test_memory_provider_subscribe() {
        let provider = MemoryPolicyProvider::new();
        let mut rx = provider.subscribe();

        let mut new_policy = Policy::default();
        new_policy.audit.retention_days = 45;
        provider.set_policy(new_policy).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.audit.retention_days, 45);
    }

    #[tokio::test]
    async fn test_file_provider_missing_file() {
        let provider = FilePolicyProvider::new("/nonexistent/policy.toml");
        let policy = provider.get_policy().await;
        // Should use defaults
        assert!(policy.tiers.allow_personal_tier);
    }

    #[tokio::test]
    async fn test_file_provider_reload() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
[audit]
retention_days = 30
"#
        )
        .unwrap();

        let provider = FilePolicyProvider::new(file.path());
        assert_eq!(provider.get_policy().await.audit.retention_days, 30);

        // Rewrite file completely (truncate first)
        let path = file.path().to_path_buf();
        std::fs::write(
            &path,
            r#"
[audit]
retention_days = 90
"#,
        )
        .unwrap();

        provider.reload().await.unwrap();
        assert_eq!(provider.get_policy().await.audit.retention_days, 90);
    }

    #[tokio::test]
    async fn test_memory_provider_reload_fails() {
        let provider = MemoryPolicyProvider::new();
        let result = provider.reload().await;
        assert!(result.is_err());
    }
}
