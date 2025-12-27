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
    /// Browser's public key (base64url-encoded P-256)
    pub p256dh: String,
    /// Auth secret (base64url-encoded)
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
