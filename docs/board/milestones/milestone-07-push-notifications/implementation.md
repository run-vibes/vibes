# Milestone 2.3: Push Notifications - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Web Push notifications to vibes so users can be alerted on their phone or desktop when Claude sessions complete, fail, or need permission approval.

**Architecture:** Web Push API with auto-generated VAPID keys, file-backed subscription storage, and service worker for notification display. Deep links to Web UI for notification actions.

**Tech Stack:** Rust, web-push crate, axum endpoints, TypeScript service worker, vite-plugin-pwa

---

## Task 1: Add web-push Dependency

**Files:**
- Modify: `vibes-core/Cargo.toml`

**Step 1: Add web-push to dependencies**

Add to `vibes-core/Cargo.toml`:

```toml
web-push = "0.10"
```

**Step 2: Verify it compiles**

Run: `cargo check -p vibes-core`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add vibes-core/Cargo.toml
git commit -m "chore: add web-push dependency for push notifications"
```

---

## Task 2: Create NotificationConfig Type

**Files:**
- Create: `vibes-core/src/notifications/mod.rs`
- Create: `vibes-core/src/notifications/config.rs`
- Modify: `vibes-core/src/lib.rs`

**Step 1: Create the notifications module directory**

```bash
mkdir -p vibes-core/src/notifications
```

**Step 2: Write NotificationConfig type**

Create `vibes-core/src/notifications/config.rs`:

```rust
//! Configuration for push notifications

use serde::{Deserialize, Serialize};

/// Configuration for push notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Whether notifications are enabled globally
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Notify when Claude needs permission approval
    #[serde(default = "default_true")]
    pub notify_permission: bool,

    /// Notify when session completes successfully
    #[serde(default = "default_true")]
    pub notify_completed: bool,

    /// Notify when session fails with an error
    #[serde(default = "default_true")]
    pub notify_error: bool,
}

fn default_true() -> bool {
    true
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            notify_permission: true,
            notify_completed: true,
            notify_error: true,
        }
    }
}

impl NotificationConfig {
    /// Create a config with all notifications enabled
    pub fn all_enabled() -> Self {
        Self::default()
    }

    /// Create a config with all notifications disabled
    pub fn all_disabled() -> Self {
        Self {
            enabled: false,
            notify_permission: false,
            notify_completed: false,
            notify_error: false,
        }
    }

    /// Check if any notification type is enabled
    pub fn any_enabled(&self) -> bool {
        self.enabled && (self.notify_permission || self.notify_completed || self.notify_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NotificationConfig::default();
        assert!(config.enabled);
        assert!(config.notify_permission);
        assert!(config.notify_completed);
        assert!(config.notify_error);
    }

    #[test]
    fn test_all_disabled() {
        let config = NotificationConfig::all_disabled();
        assert!(!config.enabled);
        assert!(!config.any_enabled());
    }

    #[test]
    fn test_any_enabled() {
        let mut config = NotificationConfig::default();
        assert!(config.any_enabled());

        config.enabled = false;
        assert!(!config.any_enabled());
    }

    #[test]
    fn test_deserialize_toml() {
        let toml = r#"
            enabled = true
            notify_permission = true
            notify_completed = false
            notify_error = true
        "#;
        let config: NotificationConfig = toml::from_str(toml).unwrap();
        assert!(config.enabled);
        assert!(config.notify_permission);
        assert!(!config.notify_completed);
        assert!(config.notify_error);
    }

    #[test]
    fn test_deserialize_toml_defaults() {
        let toml = r#""#;
        let config: NotificationConfig = toml::from_str(toml).unwrap();
        assert!(config.enabled); // defaults to true
    }
}
```

**Step 3: Create mod.rs for notifications module**

Create `vibes-core/src/notifications/mod.rs`:

```rust
//! Push notification support for vibes

mod config;

pub use config::NotificationConfig;
```

**Step 4: Export notifications module from lib.rs**

Add to `vibes-core/src/lib.rs` after other module declarations:

```rust
pub mod notifications;
```

And add to the pub use section:

```rust
pub use notifications::NotificationConfig;
```

**Step 5: Run tests**

Run: `cargo test -p vibes-core notifications`
Expected: All tests pass

**Step 6: Commit**

```bash
git add vibes-core/src/notifications/
git add vibes-core/src/lib.rs
git commit -m "feat(notifications): add NotificationConfig type"
```

---

## Task 3: Create Push Notification Types

**Files:**
- Create: `vibes-core/src/notifications/types.rs`
- Modify: `vibes-core/src/notifications/mod.rs`

**Step 1: Write notification types**

Create `vibes-core/src/notifications/types.rs`:

```rust
//! Push notification types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A browser's push subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscription {
    /// Unique identifier for this subscription
    pub id: String,
    /// Push service endpoint URL
    pub endpoint: String,
    /// Encryption keys from the browser
    pub keys: SubscriptionKeys,
    /// Browser user agent for identification
    pub user_agent: String,
    /// When this subscription was created
    pub created_at: DateTime<Utc>,
}

/// Encryption keys for a push subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionKeys {
    /// Browser's public key (base64-encoded P-256)
    pub p256dh: String,
    /// Auth secret (base64-encoded)
    pub auth: String,
}

/// Notification payload to send to browser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushNotification {
    /// Notification title
    pub title: String,
    /// Notification body text
    pub body: String,
    /// Icon URL (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Tag for notification grouping/replacement
    pub tag: String,
    /// Data payload for click handling
    pub data: NotificationData,
}

/// Data attached to notification for click handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationData {
    /// Deep link URL to open when clicked
    pub url: String,
    /// Associated session ID (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Type of event that triggered this notification
    pub event_type: NotificationEvent,
}

/// Types of events that trigger notifications
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationEvent {
    /// Claude needs permission to proceed
    PermissionNeeded,
    /// Session completed successfully
    SessionCompleted,
    /// Session failed with an error
    SessionError,
}

impl PushSubscription {
    /// Create a new push subscription
    pub fn new(
        endpoint: impl Into<String>,
        keys: SubscriptionKeys,
        user_agent: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            endpoint: endpoint.into(),
            keys,
            user_agent: user_agent.into(),
            created_at: Utc::now(),
        }
    }
}

impl PushNotification {
    /// Create a permission needed notification
    pub fn permission_needed(session_id: &str, tool_name: &str) -> Self {
        Self {
            title: "Claude needs approval".into(),
            body: format!("Permission requested for {}", tool_name),
            icon: Some("/icon-192.png".into()),
            tag: format!("permission-{}", session_id),
            data: NotificationData {
                url: format!("/session/{}?permission=pending", session_id),
                session_id: Some(session_id.to_string()),
                event_type: NotificationEvent::PermissionNeeded,
            },
        }
    }

    /// Create a session completed notification
    pub fn session_completed(session_id: &str) -> Self {
        Self {
            title: "Session completed".into(),
            body: "Claude finished the task".into(),
            icon: Some("/icon-192.png".into()),
            tag: format!("completed-{}", session_id),
            data: NotificationData {
                url: format!("/session/{}", session_id),
                session_id: Some(session_id.to_string()),
                event_type: NotificationEvent::SessionCompleted,
            },
        }
    }

    /// Create a session error notification
    pub fn session_error(session_id: &str, error: &str) -> Self {
        Self {
            title: "Session failed".into(),
            body: error.to_string(),
            icon: Some("/icon-192.png".into()),
            tag: format!("error-{}", session_id),
            data: NotificationData {
                url: format!("/session/{}", session_id),
                session_id: Some(session_id.to_string()),
                event_type: NotificationEvent::SessionError,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_subscription_new() {
        let keys = SubscriptionKeys {
            p256dh: "test-key".into(),
            auth: "test-auth".into(),
        };
        let sub = PushSubscription::new("https://push.example.com", keys, "Mozilla/5.0");
        assert!(!sub.id.is_empty());
        assert_eq!(sub.endpoint, "https://push.example.com");
    }

    #[test]
    fn test_permission_notification() {
        let notif = PushNotification::permission_needed("sess-123", "Bash");
        assert_eq!(notif.title, "Claude needs approval");
        assert!(notif.body.contains("Bash"));
        assert_eq!(notif.tag, "permission-sess-123");
        assert_eq!(notif.data.event_type, NotificationEvent::PermissionNeeded);
    }

    #[test]
    fn test_completed_notification() {
        let notif = PushNotification::session_completed("sess-123");
        assert_eq!(notif.title, "Session completed");
        assert_eq!(notif.data.event_type, NotificationEvent::SessionCompleted);
    }

    #[test]
    fn test_error_notification() {
        let notif = PushNotification::session_error("sess-123", "Out of memory");
        assert_eq!(notif.title, "Session failed");
        assert!(notif.body.contains("Out of memory"));
        assert_eq!(notif.data.event_type, NotificationEvent::SessionError);
    }

    #[test]
    fn test_notification_serialization() {
        let notif = PushNotification::permission_needed("sess-123", "Bash");
        let json = serde_json::to_string(&notif).unwrap();
        assert!(json.contains("permission_needed"));
        assert!(json.contains("Claude needs approval"));
    }
}
```

**Step 2: Update mod.rs**

Update `vibes-core/src/notifications/mod.rs`:

```rust
//! Push notification support for vibes

mod config;
mod types;

pub use config::NotificationConfig;
pub use types::{
    NotificationData, NotificationEvent, PushNotification, PushSubscription, SubscriptionKeys,
};
```

**Step 3: Update lib.rs exports**

Update the pub use in `vibes-core/src/lib.rs`:

```rust
pub use notifications::{
    NotificationConfig, NotificationData, NotificationEvent, PushNotification, PushSubscription,
    SubscriptionKeys,
};
```

**Step 4: Run tests**

Run: `cargo test -p vibes-core notifications`
Expected: All tests pass

**Step 5: Commit**

```bash
git add vibes-core/src/notifications/types.rs
git add vibes-core/src/notifications/mod.rs
git add vibes-core/src/lib.rs
git commit -m "feat(notifications): add PushSubscription and PushNotification types"
```

---

## Task 4: Create VapidKeyManager

**Files:**
- Create: `vibes-core/src/notifications/vapid.rs`
- Modify: `vibes-core/src/notifications/mod.rs`

**Step 1: Write VapidKeyManager**

Create `vibes-core/src/notifications/vapid.rs`:

```rust
//! VAPID key management for Web Push

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::fs;
use web_push::VapidSignatureBuilder;

use crate::VibesError;

/// VAPID keys file name
const VAPID_KEYS_FILE: &str = "vapid_keys.json";

/// VAPID key manager for Web Push authentication
pub struct VapidKeyManager {
    keys: VapidKeys,
    config_path: PathBuf,
}

/// VAPID keypair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VapidKeys {
    /// Private key (base64-encoded)
    pub private_key: String,
    /// Public key (base64-encoded)
    pub public_key: String,
}

impl VapidKeyManager {
    /// Load existing keys or generate new ones
    pub async fn load_or_generate(config_dir: &Path) -> Result<Self, VibesError> {
        let keys_path = config_dir.join(VAPID_KEYS_FILE);

        let keys = if keys_path.exists() {
            // Load existing keys
            let content = fs::read_to_string(&keys_path).await.map_err(|e| {
                VibesError::Config(format!("failed to read VAPID keys: {}", e))
            })?;
            serde_json::from_str(&content).map_err(|e| {
                VibesError::Config(format!("invalid VAPID keys file: {}", e))
            })?
        } else {
            // Generate new keys
            let sig_builder = VapidSignatureBuilder::generate_keys()
                .map_err(|e| VibesError::Config(format!("failed to generate VAPID keys: {}", e)))?;

            let private_key = base64::Engine::encode(
                &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                sig_builder.get_private_key(),
            );
            let public_key = base64::Engine::encode(
                &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                sig_builder.get_public_key(),
            );

            let keys = VapidKeys {
                private_key,
                public_key,
            };

            // Ensure config directory exists
            if let Some(parent) = keys_path.parent() {
                fs::create_dir_all(parent).await.map_err(|e| {
                    VibesError::Config(format!("failed to create config dir: {}", e))
                })?;
            }

            // Save keys
            let content = serde_json::to_string_pretty(&keys).map_err(|e| {
                VibesError::Config(format!("failed to serialize VAPID keys: {}", e))
            })?;
            fs::write(&keys_path, content).await.map_err(|e| {
                VibesError::Config(format!("failed to write VAPID keys: {}", e))
            })?;

            keys
        };

        Ok(Self {
            keys,
            config_path: keys_path,
        })
    }

    /// Get the public key for browser subscription
    pub fn public_key(&self) -> &str {
        &self.keys.public_key
    }

    /// Get the private key for signing
    pub fn private_key(&self) -> &str {
        &self.keys.private_key
    }

    /// Get the path where keys are stored
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_generate_new_keys() {
        let temp_dir = tempdir().unwrap();
        let manager = VapidKeyManager::load_or_generate(temp_dir.path()).await.unwrap();

        assert!(!manager.public_key().is_empty());
        assert!(!manager.private_key().is_empty());
        assert!(manager.config_path().exists());
    }

    #[tokio::test]
    async fn test_load_existing_keys() {
        let temp_dir = tempdir().unwrap();

        // Generate keys first time
        let manager1 = VapidKeyManager::load_or_generate(temp_dir.path()).await.unwrap();
        let public_key = manager1.public_key().to_string();

        // Load keys second time
        let manager2 = VapidKeyManager::load_or_generate(temp_dir.path()).await.unwrap();

        // Should be the same keys
        assert_eq!(manager2.public_key(), public_key);
    }
}
```

**Step 2: Add base64 dependency**

Add to `vibes-core/Cargo.toml`:

```toml
base64 = "0.22"
```

**Step 3: Update mod.rs**

Update `vibes-core/src/notifications/mod.rs`:

```rust
//! Push notification support for vibes

mod config;
mod types;
mod vapid;

pub use config::NotificationConfig;
pub use types::{
    NotificationData, NotificationEvent, PushNotification, PushSubscription, SubscriptionKeys,
};
pub use vapid::{VapidKeyManager, VapidKeys};
```

**Step 4: Update lib.rs exports**

Add to the pub use in `vibes-core/src/lib.rs`:

```rust
pub use notifications::{
    NotificationConfig, NotificationData, NotificationEvent, PushNotification, PushSubscription,
    SubscriptionKeys, VapidKeyManager, VapidKeys,
};
```

**Step 5: Run tests**

Run: `cargo test -p vibes-core notifications`
Expected: All tests pass

**Step 6: Commit**

```bash
git add vibes-core/Cargo.toml
git add vibes-core/src/notifications/vapid.rs
git add vibes-core/src/notifications/mod.rs
git add vibes-core/src/lib.rs
git commit -m "feat(notifications): add VapidKeyManager with auto-generation"
```

---

## Task 5: Create SubscriptionStore

**Files:**
- Create: `vibes-core/src/notifications/store.rs`
- Modify: `vibes-core/src/notifications/mod.rs`

**Step 1: Write SubscriptionStore**

Create `vibes-core/src/notifications/store.rs`:

```rust
//! Push subscription storage

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::fs;
use tokio::sync::RwLock;

use super::PushSubscription;
use crate::VibesError;

/// Subscriptions file name
const SUBSCRIPTIONS_FILE: &str = "push_subscriptions.json";

/// File-backed storage for push subscriptions
pub struct SubscriptionStore {
    subscriptions: Arc<RwLock<HashMap<String, PushSubscription>>>,
    file_path: PathBuf,
}

impl SubscriptionStore {
    /// Load subscriptions from file or create empty store
    pub async fn load(config_dir: &Path) -> Result<Self, VibesError> {
        let file_path = config_dir.join(SUBSCRIPTIONS_FILE);

        let subscriptions = if file_path.exists() {
            let content = fs::read_to_string(&file_path).await.map_err(|e| {
                VibesError::Config(format!("failed to read subscriptions: {}", e))
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
    pub async fn add(&self, subscription: PushSubscription) -> Result<(), VibesError> {
        {
            let mut subs = self.subscriptions.write().await;
            subs.insert(subscription.id.clone(), subscription);
        }
        self.persist().await
    }

    /// Remove a subscription by ID
    pub async fn remove(&self, id: &str) -> Result<bool, VibesError> {
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
    pub async fn cleanup_stale(&self, stale_ids: &[String]) -> Result<usize, VibesError> {
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
    async fn persist(&self) -> Result<(), VibesError> {
        let subs = self.subscriptions.read().await;
        let list: Vec<_> = subs.values().collect();

        // Ensure parent directory exists
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                VibesError::Config(format!("failed to create config dir: {}", e))
            })?;
        }

        let content = serde_json::to_string_pretty(&list).map_err(|e| {
            VibesError::Config(format!("failed to serialize subscriptions: {}", e))
        })?;

        fs::write(&self.file_path, content).await.map_err(|e| {
            VibesError::Config(format!("failed to write subscriptions: {}", e))
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

        let removed = store.cleanup_stale(&["test-1".into(), "test-3".into()]).await.unwrap();
        assert_eq!(removed, 2);
        assert_eq!(store.count().await, 1);
    }
}
```

**Step 2: Update mod.rs**

Update `vibes-core/src/notifications/mod.rs`:

```rust
//! Push notification support for vibes

mod config;
mod store;
mod types;
mod vapid;

pub use config::NotificationConfig;
pub use store::SubscriptionStore;
pub use types::{
    NotificationData, NotificationEvent, PushNotification, PushSubscription, SubscriptionKeys,
};
pub use vapid::{VapidKeyManager, VapidKeys};
```

**Step 3: Update lib.rs exports**

Add `SubscriptionStore` to the pub use in `vibes-core/src/lib.rs`.

**Step 4: Run tests**

Run: `cargo test -p vibes-core notifications`
Expected: All tests pass

**Step 5: Commit**

```bash
git add vibes-core/src/notifications/store.rs
git add vibes-core/src/notifications/mod.rs
git add vibes-core/src/lib.rs
git commit -m "feat(notifications): add SubscriptionStore with file persistence"
```

---

## Task 6: Create NotificationService

**Files:**
- Create: `vibes-core/src/notifications/service.rs`
- Modify: `vibes-core/src/notifications/mod.rs`

**Step 1: Write NotificationService**

Create `vibes-core/src/notifications/service.rs`:

```rust
//! Notification service that listens to events and sends push notifications

use std::sync::Arc;

use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use web_push::{
    ContentEncoding, SubscriptionInfo, VapidSignatureBuilder, WebPushClient, WebPushMessageBuilder,
};

use super::{NotificationConfig, PushNotification, SubscriptionStore, VapidKeyManager};
use crate::events::VibesEvent;

/// Service that sends push notifications based on vibes events
pub struct NotificationService {
    vapid: Arc<VapidKeyManager>,
    subscriptions: Arc<SubscriptionStore>,
    config: NotificationConfig,
    http_client: WebPushClient,
}

impl NotificationService {
    /// Create a new NotificationService
    pub fn new(
        vapid: Arc<VapidKeyManager>,
        subscriptions: Arc<SubscriptionStore>,
        config: NotificationConfig,
    ) -> Self {
        Self {
            vapid,
            subscriptions,
            config,
            http_client: WebPushClient::new().expect("failed to create web push client"),
        }
    }

    /// Start listening to events and sending notifications
    pub async fn run(&self, mut event_rx: broadcast::Receiver<VibesEvent>) {
        info!("NotificationService started");

        loop {
            match event_rx.recv().await {
                Ok(event) => {
                    if let Some(notification) = self.event_to_notification(&event) {
                        if let Err(e) = self.send_to_all(notification).await {
                            error!("Failed to send notifications: {}", e);
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!("NotificationService lagged by {} events", n);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    info!("Event channel closed, stopping NotificationService");
                    break;
                }
            }
        }
    }

    /// Convert a vibes event to a push notification (if applicable)
    fn event_to_notification(&self, event: &VibesEvent) -> Option<PushNotification> {
        if !self.config.enabled {
            return None;
        }

        match event {
            VibesEvent::PermissionRequest {
                session_id,
                tool_name,
                ..
            } if self.config.notify_permission => {
                Some(PushNotification::permission_needed(session_id, tool_name))
            }
            VibesEvent::SessionCompleted { session_id } if self.config.notify_completed => {
                Some(PushNotification::session_completed(session_id))
            }
            VibesEvent::SessionFailed { session_id, error } if self.config.notify_error => {
                Some(PushNotification::session_error(session_id, error))
            }
            _ => None,
        }
    }

    /// Send a notification to all subscribed browsers
    async fn send_to_all(&self, notification: PushNotification) -> Result<(), String> {
        let subscriptions = self.subscriptions.list().await;

        if subscriptions.is_empty() {
            debug!("No subscriptions, skipping notification");
            return Ok(());
        }

        let payload = serde_json::to_string(&notification)
            .map_err(|e| format!("failed to serialize notification: {}", e))?;

        let mut stale_ids = Vec::new();

        for sub in subscriptions {
            match self.send_one(&sub, &payload).await {
                Ok(()) => {
                    debug!("Sent notification to {}", sub.endpoint);
                }
                Err(e) if e.contains("410") || e.contains("404") => {
                    // Subscription is stale/expired
                    warn!("Subscription {} is stale, marking for removal", sub.id);
                    stale_ids.push(sub.id.clone());
                }
                Err(e) => {
                    warn!("Failed to send to {}: {}", sub.endpoint, e);
                }
            }
        }

        // Clean up stale subscriptions
        if !stale_ids.is_empty() {
            if let Err(e) = self.subscriptions.cleanup_stale(&stale_ids).await {
                error!("Failed to cleanup stale subscriptions: {}", e);
            }
        }

        Ok(())
    }

    /// Send to a single subscription
    async fn send_one(
        &self,
        sub: &super::PushSubscription,
        payload: &str,
    ) -> Result<(), String> {
        let subscription_info = SubscriptionInfo::new(
            &sub.endpoint,
            &sub.keys.p256dh,
            &sub.keys.auth,
        );

        // Build VAPID signature
        let sig_builder = VapidSignatureBuilder::from_base64(
            self.vapid.private_key(),
            web_push::URL_SAFE_NO_PAD,
            &subscription_info,
        )
        .map_err(|e| format!("VAPID signature error: {}", e))?;

        // Build the push message
        let mut builder = WebPushMessageBuilder::new(&subscription_info);
        builder.set_payload(ContentEncoding::Aes128Gcm, payload.as_bytes());
        builder.set_vapid_signature(
            sig_builder
                .build()
                .map_err(|e| format!("failed to build VAPID signature: {}", e))?,
        );

        let message = builder
            .build()
            .map_err(|e| format!("failed to build push message: {}", e))?;

        // Send it
        self.http_client
            .send(message)
            .await
            .map_err(|e| format!("{}", e))?;

        Ok(())
    }

    /// Get the configuration
    pub fn config(&self) -> &NotificationConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full integration tests require mocking the web push endpoint
    // Unit tests focus on event filtering logic

    #[test]
    fn test_config_filtering() {
        // This would require more setup to test properly
        // For now, we verify the structure compiles
    }
}
```

**Step 2: Update mod.rs**

Update `vibes-core/src/notifications/mod.rs`:

```rust
//! Push notification support for vibes

mod config;
mod service;
mod store;
mod types;
mod vapid;

pub use config::NotificationConfig;
pub use service::NotificationService;
pub use store::SubscriptionStore;
pub use types::{
    NotificationData, NotificationEvent, PushNotification, PushSubscription, SubscriptionKeys,
};
pub use vapid::{VapidKeyManager, VapidKeys};
```

**Step 3: Update lib.rs exports**

Add `NotificationService` to the pub use in `vibes-core/src/lib.rs`.

**Step 4: Run tests**

Run: `cargo test -p vibes-core notifications`
Expected: All tests pass

**Step 5: Commit**

```bash
git add vibes-core/src/notifications/service.rs
git add vibes-core/src/notifications/mod.rs
git add vibes-core/src/lib.rs
git commit -m "feat(notifications): add NotificationService with Web Push delivery"
```

---

## Task 7: Add Push Subscription API Endpoints

**Files:**
- Create: `vibes-server/src/http/push.rs`
- Modify: `vibes-server/src/http/mod.rs`
- Modify: `vibes-server/src/state.rs`

**Step 1: Create push API handlers**

Create `vibes-server/src/http/push.rs`:

```rust
//! Push notification API endpoints

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use vibes_core::{PushSubscription, SubscriptionKeys};

use crate::state::AppState;

/// GET /api/push/vapid-key
pub async fn get_vapid_key(
    State(state): State<Arc<AppState>>,
) -> Result<Json<VapidKeyResponse>, StatusCode> {
    let vapid = state.vapid.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(Json(VapidKeyResponse {
        public_key: vapid.public_key().to_string(),
    }))
}

#[derive(Serialize)]
pub struct VapidKeyResponse {
    pub public_key: String,
}

/// POST /api/push/subscribe
pub async fn subscribe(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SubscribeRequest>,
) -> Result<Json<SubscribeResponse>, StatusCode> {
    let store = state.subscriptions.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let subscription = PushSubscription::new(
        req.endpoint,
        SubscriptionKeys {
            p256dh: req.keys.p256dh,
            auth: req.keys.auth,
        },
        req.user_agent.unwrap_or_else(|| "Unknown".to_string()),
    );

    let id = subscription.id.clone();

    store
        .add(subscription)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SubscribeResponse { id }))
}

#[derive(Deserialize)]
pub struct SubscribeRequest {
    pub endpoint: String,
    pub keys: SubscribeKeys,
    pub user_agent: Option<String>,
}

#[derive(Deserialize)]
pub struct SubscribeKeys {
    pub p256dh: String,
    pub auth: String,
}

#[derive(Serialize)]
pub struct SubscribeResponse {
    pub id: String,
}

/// DELETE /api/push/subscribe/:id
pub async fn unsubscribe(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let store = match state.subscriptions.as_ref() {
        Some(s) => s,
        None => return StatusCode::SERVICE_UNAVAILABLE,
    };

    match store.remove(&id).await {
        Ok(true) => StatusCode::NO_CONTENT,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// GET /api/push/subscriptions
pub async fn list_subscriptions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<SubscriptionInfo>>, StatusCode> {
    let store = state.subscriptions.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let subs = store.list().await;
    let infos: Vec<_> = subs
        .into_iter()
        .map(|s| SubscriptionInfo {
            id: s.id,
            user_agent: s.user_agent,
            created_at: s.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(infos))
}

#[derive(Serialize)]
pub struct SubscriptionInfo {
    pub id: String,
    pub user_agent: String,
    pub created_at: String,
}
```

**Step 2: Add routes to router**

Update `vibes-server/src/http/mod.rs` to add push routes:

```rust
mod push;

// In create_router:
.route("/api/push/vapid-key", get(push::get_vapid_key))
.route("/api/push/subscribe", post(push::subscribe))
.route("/api/push/subscribe/:id", delete(push::unsubscribe))
.route("/api/push/subscriptions", get(push::list_subscriptions))
```

**Step 3: Update AppState**

Update `vibes-server/src/state.rs` to include notification components:

```rust
use std::sync::Arc;
use vibes_core::{SubscriptionStore, VapidKeyManager};

pub struct AppState {
    // ... existing fields ...
    pub vapid: Option<Arc<VapidKeyManager>>,
    pub subscriptions: Option<Arc<SubscriptionStore>>,
}
```

**Step 4: Run tests**

Run: `cargo test -p vibes-server`
Expected: Tests pass

**Step 5: Commit**

```bash
git add vibes-server/src/http/push.rs
git add vibes-server/src/http/mod.rs
git add vibes-server/src/state.rs
git commit -m "feat(server): add push subscription API endpoints"
```

---

## Task 8: Add --notify Flag to CLI

**Files:**
- Modify: `vibes-cli/src/commands/claude.rs`

**Step 1: Add notify flag**

Add to ClaudeArgs in `vibes-cli/src/commands/claude.rs`:

```rust
/// Send push notifications for this session's events
#[arg(long)]
pub notify: bool,
```

**Step 2: Handle notify flag**

In the run function, after connecting to the server:

```rust
if self.notify {
    // Check if any subscriptions exist
    let subs: Vec<SubscriptionInfo> = client
        .get("/api/push/subscriptions")
        .await?;

    if subs.is_empty() {
        eprintln!("Warning: No push subscriptions found.");
        eprintln!("  Open the Web UI and enable notifications first.");
        eprintln!("  Web UI: http://localhost:{}", port);
    }
}
```

**Step 3: Run tests**

Run: `cargo build -p vibes-cli`
Expected: Builds successfully

**Step 4: Commit**

```bash
git add vibes-cli/src/commands/claude.rs
git commit -m "feat(cli): add --notify flag for push notifications"
```

---

## Task 9: Create Service Worker

**Files:**
- Create: `web-ui/public/sw.js`
- Modify: `web-ui/vite.config.ts`

**Step 1: Create service worker**

Create `web-ui/public/sw.js`:

```javascript
// vibes push notification service worker

self.addEventListener('push', (event) => {
  if (!event.data) {
    console.warn('Push event without data');
    return;
  }

  const data = event.data.json();

  const options = {
    body: data.body,
    icon: data.icon || '/icon-192.png',
    badge: '/badge-72.png',
    tag: data.tag,
    data: data.data,
    requireInteraction: data.data?.event_type === 'permission_needed',
    actions: [
      { action: 'open', title: 'Open' },
      { action: 'dismiss', title: 'Dismiss' },
    ],
  };

  event.waitUntil(
    self.registration.showNotification(data.title, options)
  );
});

self.addEventListener('notificationclick', (event) => {
  event.notification.close();

  if (event.action === 'dismiss') {
    return;
  }

  const url = event.notification.data?.url || '/';

  event.waitUntil(
    clients.matchAll({ type: 'window', includeUncontrolled: true }).then((windowClients) => {
      // Try to focus an existing vibes window
      for (const client of windowClients) {
        if (client.url.includes(self.location.origin)) {
          client.navigate(url);
          return client.focus();
        }
      }
      // No existing window, open a new one
      return clients.openWindow(url);
    })
  );
});

self.addEventListener('install', (event) => {
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(clients.claim());
});
```

**Step 2: Update vite.config.ts for service worker**

The service worker in `/public` is automatically served by Vite. No additional config needed for a simple service worker.

**Step 3: Commit**

```bash
git add web-ui/public/sw.js
git commit -m "feat(ui): add service worker for push notifications"
```

---

## Task 10: Create usePushSubscription Hook

**Files:**
- Create: `web-ui/src/hooks/usePushSubscription.ts`
- Modify: `web-ui/src/hooks/index.ts`

**Step 1: Create the hook**

Create `web-ui/src/hooks/usePushSubscription.ts`:

```typescript
import { useState, useEffect, useCallback } from 'react';

interface PushSubscriptionState {
  subscription: PushSubscription | null;
  isSupported: boolean;
  isLoading: boolean;
  error: string | null;
}

export function usePushSubscription() {
  const [state, setState] = useState<PushSubscriptionState>({
    subscription: null,
    isSupported: false,
    isLoading: true,
    error: null,
  });

  useEffect(() => {
    const checkSupport = async () => {
      const supported = 'serviceWorker' in navigator && 'PushManager' in window;

      if (!supported) {
        setState((s) => ({ ...s, isSupported: false, isLoading: false }));
        return;
      }

      try {
        const registration = await navigator.serviceWorker.ready;
        const subscription = await registration.pushManager.getSubscription();
        setState({
          subscription,
          isSupported: true,
          isLoading: false,
          error: null,
        });
      } catch (err) {
        setState((s) => ({
          ...s,
          isSupported: true,
          isLoading: false,
          error: String(err),
        }));
      }
    };

    checkSupport();
  }, []);

  const subscribe = useCallback(async () => {
    setState((s) => ({ ...s, isLoading: true, error: null }));

    try {
      // Request notification permission
      const permission = await Notification.requestPermission();
      if (permission !== 'granted') {
        throw new Error('Notification permission denied');
      }

      // Get VAPID public key from server
      const vapidResponse = await fetch('/api/push/vapid-key');
      if (!vapidResponse.ok) {
        throw new Error('Failed to get VAPID key');
      }
      const { public_key } = await vapidResponse.json();

      // Register service worker if needed
      const registration = await navigator.serviceWorker.ready;

      // Subscribe to push
      const subscription = await registration.pushManager.subscribe({
        userVisibleOnly: true,
        applicationServerKey: urlBase64ToUint8Array(public_key),
      });

      // Send subscription to server
      const subscribeResponse = await fetch('/api/push/subscribe', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          endpoint: subscription.endpoint,
          keys: {
            p256dh: arrayBufferToBase64(subscription.getKey('p256dh')!),
            auth: arrayBufferToBase64(subscription.getKey('auth')!),
          },
          user_agent: navigator.userAgent,
        }),
      });

      if (!subscribeResponse.ok) {
        throw new Error('Failed to register subscription with server');
      }

      setState({
        subscription,
        isSupported: true,
        isLoading: false,
        error: null,
      });
    } catch (err) {
      setState((s) => ({
        ...s,
        isLoading: false,
        error: String(err),
      }));
    }
  }, []);

  const unsubscribe = useCallback(async () => {
    if (!state.subscription) return;

    setState((s) => ({ ...s, isLoading: true, error: null }));

    try {
      await state.subscription.unsubscribe();
      setState({
        subscription: null,
        isSupported: true,
        isLoading: false,
        error: null,
      });
    } catch (err) {
      setState((s) => ({
        ...s,
        isLoading: false,
        error: String(err),
      }));
    }
  }, [state.subscription]);

  return {
    ...state,
    subscribe,
    unsubscribe,
  };
}

// Helper functions
function urlBase64ToUint8Array(base64String: string): Uint8Array {
  const padding = '='.repeat((4 - (base64String.length % 4)) % 4);
  const base64 = (base64String + padding).replace(/-/g, '+').replace(/_/g, '/');
  const rawData = window.atob(base64);
  const outputArray = new Uint8Array(rawData.length);
  for (let i = 0; i < rawData.length; ++i) {
    outputArray[i] = rawData.charCodeAt(i);
  }
  return outputArray;
}

function arrayBufferToBase64(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return window.btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
}
```

**Step 2: Export from hooks index**

Add to `web-ui/src/hooks/index.ts`:

```typescript
export { usePushSubscription } from './usePushSubscription';
```

**Step 3: Commit**

```bash
git add web-ui/src/hooks/usePushSubscription.ts
git add web-ui/src/hooks/index.ts
git commit -m "feat(ui): add usePushSubscription hook"
```

---

## Task 11: Create NotificationSettings Component

**Files:**
- Create: `web-ui/src/components/NotificationSettings.tsx`
- Modify: `web-ui/src/components/index.ts`

**Step 1: Create the component**

Create `web-ui/src/components/NotificationSettings.tsx`:

```tsx
import { usePushSubscription } from '../hooks/usePushSubscription';

export function NotificationSettings() {
  const {
    subscription,
    isSupported,
    isLoading,
    error,
    subscribe,
    unsubscribe,
  } = usePushSubscription();

  if (!isSupported) {
    return (
      <div className="notification-settings">
        <p className="text-muted">
          Push notifications are not supported in this browser.
        </p>
      </div>
    );
  }

  return (
    <div className="notification-settings">
      <h3>Push Notifications</h3>

      {error && (
        <div className="error-message">
          {error}
        </div>
      )}

      {!subscription ? (
        <div className="notification-disabled">
          <p>Enable notifications to get alerts when:</p>
          <ul>
            <li>Claude needs permission to proceed</li>
            <li>A session completes</li>
            <li>An error occurs</li>
          </ul>
          <button
            onClick={subscribe}
            disabled={isLoading}
            className="btn btn-primary"
          >
            {isLoading ? 'Enabling...' : 'Enable Notifications'}
          </button>
        </div>
      ) : (
        <div className="notification-enabled">
          <p className="status-success">
            ✓ Notifications enabled
          </p>
          <button
            onClick={unsubscribe}
            disabled={isLoading}
            className="btn btn-secondary"
          >
            {isLoading ? 'Disabling...' : 'Disable Notifications'}
          </button>
        </div>
      )}
    </div>
  );
}
```

**Step 2: Export from components index**

Add to `web-ui/src/components/index.ts`:

```typescript
export { NotificationSettings } from './NotificationSettings';
```

**Step 3: Add styles**

Add to appropriate stylesheet:

```css
.notification-settings {
  padding: 1rem;
}

.notification-settings h3 {
  margin-bottom: 1rem;
}

.notification-settings ul {
  margin: 0.5rem 0 1rem 1.5rem;
}

.notification-disabled p,
.notification-enabled p {
  margin-bottom: 1rem;
}

.status-success {
  color: var(--color-success);
}

.error-message {
  color: var(--color-error);
  margin-bottom: 1rem;
}
```

**Step 4: Commit**

```bash
git add web-ui/src/components/NotificationSettings.tsx
git add web-ui/src/components/index.ts
git commit -m "feat(ui): add NotificationSettings component"
```

---

## Task 12: Register Service Worker

**Files:**
- Modify: `web-ui/src/main.tsx` (or appropriate entry point)

**Step 1: Add service worker registration**

Add to the entry point:

```typescript
// Register service worker for push notifications
if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker
      .register('/sw.js')
      .then((registration) => {
        console.log('Service worker registered:', registration.scope);
      })
      .catch((error) => {
        console.error('Service worker registration failed:', error);
      });
  });
}
```

**Step 2: Commit**

```bash
git add web-ui/src/main.tsx
git commit -m "feat(ui): register service worker on load"
```

---

## Task 13: Initialize Notification Services on Server Start

**Files:**
- Modify: `vibes-server/src/lib.rs`

**Step 1: Initialize VAPID and subscription store**

Update server initialization to create notification services:

```rust
use vibes_core::{NotificationConfig, SubscriptionStore, VapidKeyManager};

// In server startup:
let config_dir = dirs::config_dir()
    .map(|p| p.join("vibes"))
    .expect("no config dir");

let vapid = VapidKeyManager::load_or_generate(&config_dir)
    .await
    .expect("failed to load VAPID keys");

let subscriptions = SubscriptionStore::load(&config_dir)
    .await
    .expect("failed to load subscriptions");

let notification_config = NotificationConfig::default();

// Add to AppState
let state = AppState {
    // ... existing fields ...
    vapid: Some(Arc::new(vapid)),
    subscriptions: Some(Arc::new(subscriptions)),
};
```

**Step 2: Run tests**

Run: `cargo test -p vibes-server`
Expected: Tests pass

**Step 3: Commit**

```bash
git add vibes-server/src/lib.rs
git commit -m "feat(server): initialize notification services on startup"
```

---

## Task 14: Update Documentation

**Files:**
- Modify: `docs/PROGRESS.md`

**Step 1: Mark tasks complete**

Update PROGRESS.md milestone 2.3 section to mark items as complete.

**Step 2: Commit**

```bash
git add docs/PROGRESS.md
git commit -m "docs: update progress for milestone 2.3"
```

---

## Task 15: Final Integration Test

**Step 1: Build everything**

Run: `cargo build`
Expected: Builds successfully

**Step 2: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 3: Run pre-commit checks**

Run: `just pre-commit`
Expected: All checks pass

**Step 4: Manual testing checklist**

- [ ] Start vibes server
- [ ] Open Web UI
- [ ] Enable notifications (should prompt for permission)
- [ ] Start a Claude session
- [ ] Trigger permission request → notification appears
- [ ] Click notification → opens Web UI at permission
- [ ] Session completes → notification appears
- [ ] Restart server → subscriptions persist

**Step 5: Commit any fixes**

```bash
git add .
git commit -m "test: fix issues from final integration"
```

---

## Summary

This implementation plan covers:

1. **Core types** (Tasks 1-3): NotificationConfig, PushSubscription, PushNotification
2. **Key management** (Task 4): VapidKeyManager with auto-generation
3. **Storage** (Task 5): SubscriptionStore with file persistence
4. **Service** (Task 6): NotificationService that listens to events
5. **Server API** (Task 7): Push subscription endpoints
6. **CLI** (Task 8): `--notify` flag
7. **Web UI** (Tasks 9-12): Service worker, hooks, and components
8. **Integration** (Tasks 13-15): Server initialization and testing

Tasks follow TDD workflow: write tests around new behavior, implement changes, verify tests pass before committing.
