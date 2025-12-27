//! Push subscription storage

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::fs;
use tokio::sync::RwLock;

use super::PushSubscription;
use crate::NotificationError;

/// Subscriptions file name
const SUBSCRIPTIONS_FILE: &str = "push_subscriptions.json";

/// File-backed storage for push subscriptions
pub struct SubscriptionStore {
    subscriptions: Arc<RwLock<HashMap<String, PushSubscription>>>,
    file_path: PathBuf,
}

impl SubscriptionStore {
    /// Load subscriptions from file or create empty store
    pub async fn load(config_dir: &Path) -> Result<Self, NotificationError> {
        let file_path = config_dir.join(SUBSCRIPTIONS_FILE);

        let subscriptions = if file_path.exists() {
            let content = fs::read_to_string(&file_path).await.map_err(|e| {
                NotificationError::Storage(format!("failed to read subscriptions: {}", e))
            })?;
            let subs: Vec<PushSubscription> = serde_json::from_str(&content).unwrap_or_default();
            subs.into_iter().map(|s| (s.id.clone(), s)).collect()
        } else {
            HashMap::new()
        };

        Ok(Self {
            subscriptions: Arc::new(RwLock::new(subscriptions)),
            file_path,
        })
    }

    /// Add a subscription
    pub async fn add(&self, subscription: PushSubscription) -> Result<(), NotificationError> {
        {
            let mut subs = self.subscriptions.write().await;
            subs.insert(subscription.id.clone(), subscription);
        }
        self.persist().await
    }

    /// Remove a subscription by ID
    pub async fn remove(&self, id: &str) -> Result<bool, NotificationError> {
        let removed = {
            let mut subs = self.subscriptions.write().await;
            subs.remove(id).is_some()
        };
        if removed {
            self.persist().await?;
        }
        Ok(removed)
    }

    /// List all subscriptions
    pub async fn list(&self) -> Vec<PushSubscription> {
        let subs = self.subscriptions.read().await;
        subs.values().cloned().collect()
    }

    /// Get subscription count
    pub async fn count(&self) -> usize {
        let subs = self.subscriptions.read().await;
        subs.len()
    }

    /// Check if there are any subscriptions
    pub async fn is_empty(&self) -> bool {
        let subs = self.subscriptions.read().await;
        subs.is_empty()
    }

    /// Remove stale subscriptions (expired or failed endpoints)
    pub async fn cleanup_stale(&self, stale_ids: &[String]) -> Result<usize, NotificationError> {
        let removed = {
            let mut subs = self.subscriptions.write().await;
            let mut count = 0;
            for id in stale_ids {
                if subs.remove(id).is_some() {
                    count += 1;
                }
            }
            count
        };
        if removed > 0 {
            self.persist().await?;
        }
        Ok(removed)
    }

    /// Persist subscriptions to file
    async fn persist(&self) -> Result<(), NotificationError> {
        let subs = self.subscriptions.read().await;
        let list: Vec<_> = subs.values().collect();

        // Ensure parent directory exists
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                NotificationError::Storage(format!("failed to create config dir: {}", e))
            })?;
        }

        let content = serde_json::to_string_pretty(&list).map_err(|e| {
            NotificationError::Storage(format!("failed to serialize subscriptions: {}", e))
        })?;

        fs::write(&self.file_path, content).await.map_err(|e| {
            NotificationError::Storage(format!("failed to write subscriptions: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notifications::SubscriptionKeys;
    use tempfile::tempdir;

    fn create_test_subscription(id: &str) -> PushSubscription {
        PushSubscription {
            id: id.to_string(),
            endpoint: format!("https://push.example.com/{}", id),
            keys: SubscriptionKeys {
                p256dh: "test-key".into(),
                auth: "test-auth".into(),
            },
            user_agent: "Mozilla/5.0".into(),
            created_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_empty_store() {
        let temp_dir = tempdir().unwrap();
        let store = SubscriptionStore::load(temp_dir.path()).await.unwrap();

        assert!(store.is_empty().await);
        assert_eq!(store.count().await, 0);
    }

    #[tokio::test]
    async fn test_add_subscription() {
        let temp_dir = tempdir().unwrap();
        let store = SubscriptionStore::load(temp_dir.path()).await.unwrap();

        let sub = create_test_subscription("test-1");
        store.add(sub).await.unwrap();

        assert!(!store.is_empty().await);
        assert_eq!(store.count().await, 1);
    }

    #[tokio::test]
    async fn test_remove_subscription() {
        let temp_dir = tempdir().unwrap();
        let store = SubscriptionStore::load(temp_dir.path()).await.unwrap();

        let sub = create_test_subscription("test-1");
        store.add(sub).await.unwrap();

        assert!(store.remove("test-1").await.unwrap());
        assert!(store.is_empty().await);

        // Removing again should return false
        assert!(!store.remove("test-1").await.unwrap());
    }

    #[tokio::test]
    async fn test_persistence() {
        let temp_dir = tempdir().unwrap();

        // Add subscription
        {
            let store = SubscriptionStore::load(temp_dir.path()).await.unwrap();
            let sub = create_test_subscription("test-1");
            store.add(sub).await.unwrap();
        }

        // Load again and verify
        {
            let store = SubscriptionStore::load(temp_dir.path()).await.unwrap();
            assert_eq!(store.count().await, 1);
        }
    }

    #[tokio::test]
    async fn test_cleanup_stale() {
        let temp_dir = tempdir().unwrap();
        let store = SubscriptionStore::load(temp_dir.path()).await.unwrap();

        store.add(create_test_subscription("test-1")).await.unwrap();
        store.add(create_test_subscription("test-2")).await.unwrap();
        store.add(create_test_subscription("test-3")).await.unwrap();

        let removed = store
            .cleanup_stale(&["test-1".into(), "test-3".into()])
            .await
            .unwrap();
        assert_eq!(removed, 2);
        assert_eq!(store.count().await, 1);
    }
}
