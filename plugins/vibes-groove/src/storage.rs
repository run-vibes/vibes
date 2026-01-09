//! Multi-tier storage facade for continual learning
//!
//! This module provides unified access to user, project, and enterprise learning stores.
//! Reads check all tiers in priority order; writes go to user tier.

use std::sync::Arc;

use crate::{CozoStore, GrooveConfig, ProjectContext, Result};

/// Multi-tier storage facade
///
/// Provides unified access to user, project, and enterprise learning stores.
/// Reads check all tiers in priority order; writes go to user tier.
pub struct GrooveStorage {
    /// User's personal store (always present)
    user_store: Arc<CozoStore>,

    /// Project-specific store (optional)
    project_store: Option<Arc<CozoStore>>,

    /// Enterprise/org store (optional)
    enterprise_store: Option<Arc<CozoStore>>,

    /// Current project context
    context: ProjectContext,
}

impl GrooveStorage {
    /// Create new storage with user store only
    pub async fn new(config: &GrooveConfig) -> Result<Self> {
        let user_store = CozoStore::open(&config.user_db_path).await?;
        Ok(Self {
            user_store: Arc::new(user_store),
            project_store: None,
            enterprise_store: None,
            context: ProjectContext::Personal,
        })
    }

    /// Set project context (opens project store if path provided)
    pub async fn with_project(mut self, project_db_path: Option<&std::path::Path>) -> Result<Self> {
        if let Some(path) = project_db_path {
            let store = CozoStore::open(path).await?;
            self.project_store = Some(Arc::new(store));
        }
        Ok(self)
    }

    /// Set enterprise context
    pub async fn with_enterprise(mut self, org_id: &str, config: &GrooveConfig) -> Result<Self> {
        if let Some(enterprise_config) = config.enterprises.get(org_id) {
            let store = CozoStore::open(&enterprise_config.db_path).await?;
            self.enterprise_store = Some(Arc::new(store));
            self.context = ProjectContext::Enterprise {
                org_id: org_id.to_string(),
            };
        }
        Ok(self)
    }

    /// Get stores in read priority order (user > project > enterprise)
    pub fn get_read_tiers(&self) -> Vec<Arc<CozoStore>> {
        let mut tiers = vec![Arc::clone(&self.user_store)];
        if let Some(ref store) = self.project_store {
            tiers.push(Arc::clone(store));
        }
        if let Some(ref store) = self.enterprise_store {
            tiers.push(Arc::clone(store));
        }
        tiers
    }

    /// Get the user store (write target)
    pub fn user_store(&self) -> &CozoStore {
        &self.user_store
    }

    /// Get current project context
    pub fn context(&self) -> &ProjectContext {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn make_test_config(path: &std::path::Path) -> GrooveConfig {
        GrooveConfig {
            user_db_path: path.to_path_buf(),
            enterprises: HashMap::new(),
            deduplication: crate::DeduplicationConfig::default(),
            correction: crate::CorrectionConfig::default(),
            temporal: crate::TemporalConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_new_creates_user_store() {
        let tmp = TempDir::new().unwrap();
        let config = make_test_config(&tmp.path().join("user.db"));

        let storage = GrooveStorage::new(&config).await.unwrap();

        assert!(storage.user_store().is_initialized());
        assert_eq!(storage.get_read_tiers().len(), 1);
    }

    #[tokio::test]
    async fn test_with_project_adds_tier() {
        let tmp = TempDir::new().unwrap();
        let config = make_test_config(&tmp.path().join("user.db"));
        let project_path = tmp.path().join("project.db");

        let storage = GrooveStorage::new(&config)
            .await
            .unwrap()
            .with_project(Some(&project_path))
            .await
            .unwrap();

        assert_eq!(storage.get_read_tiers().len(), 2);
    }

    #[tokio::test]
    async fn test_with_project_none_keeps_single_tier() {
        let tmp = TempDir::new().unwrap();
        let config = make_test_config(&tmp.path().join("user.db"));

        let storage = GrooveStorage::new(&config)
            .await
            .unwrap()
            .with_project(None)
            .await
            .unwrap();

        assert_eq!(storage.get_read_tiers().len(), 1);
    }

    #[tokio::test]
    async fn test_with_enterprise_adds_tier() {
        let tmp = TempDir::new().unwrap();
        let user_path = tmp.path().join("user.db");
        let enterprise_path = tmp.path().join("enterprise.db");

        let config = GrooveConfig {
            user_db_path: user_path,
            enterprises: HashMap::from([(
                "acme".to_string(),
                crate::EnterpriseConfig {
                    org_id: "acme".to_string(),
                    db_path: enterprise_path,
                    sync_url: None,
                },
            )]),
            deduplication: crate::DeduplicationConfig::default(),
            correction: crate::CorrectionConfig::default(),
            temporal: crate::TemporalConfig::default(),
        };

        let storage = GrooveStorage::new(&config)
            .await
            .unwrap()
            .with_enterprise("acme", &config)
            .await
            .unwrap();

        assert_eq!(storage.get_read_tiers().len(), 2);
        assert!(matches!(
            storage.context(),
            ProjectContext::Enterprise { .. }
        ));
    }

    #[tokio::test]
    async fn test_with_enterprise_unknown_org_keeps_context() {
        let tmp = TempDir::new().unwrap();
        let config = make_test_config(&tmp.path().join("user.db"));

        let storage = GrooveStorage::new(&config)
            .await
            .unwrap()
            .with_enterprise("unknown", &config)
            .await
            .unwrap();

        assert_eq!(storage.get_read_tiers().len(), 1);
        assert!(matches!(storage.context(), ProjectContext::Personal));
    }

    #[tokio::test]
    async fn test_read_tiers_order() {
        let tmp = TempDir::new().unwrap();
        let user_path = tmp.path().join("user.db");
        let project_path = tmp.path().join("project.db");
        let enterprise_path = tmp.path().join("enterprise.db");

        let config = GrooveConfig {
            user_db_path: user_path,
            enterprises: HashMap::from([(
                "acme".to_string(),
                crate::EnterpriseConfig {
                    org_id: "acme".to_string(),
                    db_path: enterprise_path,
                    sync_url: None,
                },
            )]),
            deduplication: crate::DeduplicationConfig::default(),
            correction: crate::CorrectionConfig::default(),
            temporal: crate::TemporalConfig::default(),
        };

        let storage = GrooveStorage::new(&config)
            .await
            .unwrap()
            .with_project(Some(&project_path))
            .await
            .unwrap()
            .with_enterprise("acme", &config)
            .await
            .unwrap();

        // Should have all 3 tiers
        assert_eq!(storage.get_read_tiers().len(), 3);
    }

    #[tokio::test]
    async fn test_context_starts_as_personal() {
        let tmp = TempDir::new().unwrap();
        let config = make_test_config(&tmp.path().join("user.db"));

        let storage = GrooveStorage::new(&config).await.unwrap();

        assert!(matches!(storage.context(), ProjectContext::Personal));
    }

    #[tokio::test]
    async fn test_user_store_accessor() {
        let tmp = TempDir::new().unwrap();
        let config = make_test_config(&tmp.path().join("user.db"));

        let storage = GrooveStorage::new(&config).await.unwrap();

        // User store should be accessible and initialized
        let user_store = storage.user_store();
        assert!(user_store.is_initialized());
    }
}
