//! Notification service that listens to events and sends push notifications

use std::sync::Arc;

use base64ct::{Base64UrlUnpadded, Encoding};
use http::Uri;
use p256::EncodedPoint;
use p256::elliptic_curve::sec1::FromEncodedPoint;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use web_push_native::jwt_simple::prelude::ES256KeyPair;
use web_push_native::{Auth, WebPushBuilder};

use super::{NotificationConfig, PushNotification, SubscriptionStore, VapidKeyManager};
use crate::NotificationError;
use crate::events::{ClaudeEvent, EventSeq, VibesEvent};

/// Service that sends push notifications based on vibes events
pub struct NotificationService {
    vapid: Arc<VapidKeyManager>,
    subscriptions: Arc<SubscriptionStore>,
    config: NotificationConfig,
    http_client: reqwest::Client,
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
            http_client: reqwest::Client::new(),
        }
    }

    /// Start listening to events and sending notifications
    pub async fn run(&self, mut event_rx: broadcast::Receiver<(EventSeq, VibesEvent)>) {
        info!("NotificationService started");

        loop {
            match event_rx.recv().await {
                Ok((_seq, event)) => {
                    if let Some(notification) = self.event_to_notification(&event)
                        && let Err(e) = self.send_to_all(notification).await
                    {
                        error!("Failed to send notifications: {}", e);
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
            // Permission requests come from Claude events
            VibesEvent::Claude {
                session_id,
                event: ClaudeEvent::PermissionRequest { tool, .. },
            } if self.config.notify_permission => {
                Some(PushNotification::permission_needed(session_id, tool))
            }

            // Errors come from Claude events
            VibesEvent::Claude {
                session_id,
                event: ClaudeEvent::Error { message, .. },
            } if self.config.notify_error => {
                Some(PushNotification::session_error(session_id, message))
            }

            // Session completion comes from state changes
            VibesEvent::SessionStateChanged { session_id, state }
                if self.config.notify_completed && state == "Completed" =>
            {
                Some(PushNotification::session_completed(session_id))
            }

            _ => None,
        }
    }

    /// Send a notification to all subscribed browsers
    async fn send_to_all(&self, notification: PushNotification) -> Result<(), NotificationError> {
        let subscriptions = self.subscriptions.list().await;

        if subscriptions.is_empty() {
            debug!("No subscriptions, skipping notification");
            return Ok(());
        }

        let payload = serde_json::to_string(&notification)
            .map_err(|e| NotificationError::SendFailed(format!("serialization error: {}", e)))?;

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
        if !stale_ids.is_empty()
            && let Err(e) = self.subscriptions.cleanup_stale(&stale_ids).await
        {
            error!("Failed to cleanup stale subscriptions: {}", e);
        }

        Ok(())
    }

    /// Send to a single subscription
    async fn send_one(&self, sub: &super::PushSubscription, payload: &str) -> Result<(), String> {
        // Parse endpoint as URI
        let endpoint: Uri = sub
            .endpoint
            .parse()
            .map_err(|e| format!("invalid endpoint URL: {}", e))?;

        // Decode the subscription keys
        let p256dh_bytes = Base64UrlUnpadded::decode_vec(&sub.keys.p256dh)
            .map_err(|e| format!("invalid p256dh key: {}", e))?;
        let auth_bytes = Base64UrlUnpadded::decode_vec(&sub.keys.auth)
            .map_err(|e| format!("invalid auth key: {}", e))?;

        // Parse p256dh as a P-256 public key (65 bytes uncompressed point)
        let encoded_point = EncodedPoint::from_bytes(&p256dh_bytes)
            .map_err(|e| format!("invalid p256dh point: {}", e))?;
        let ua_public = p256::PublicKey::from_encoded_point(&encoded_point);
        let ua_public =
            Option::from(ua_public).ok_or_else(|| "invalid p256dh public key".to_string())?;

        // Convert auth to Auth type (16 bytes)
        let auth_array: [u8; 16] = auth_bytes
            .try_into()
            .map_err(|_| "auth secret must be 16 bytes")?;
        let auth: Auth = auth_array.into();

        // Create the web push builder
        let builder = WebPushBuilder::new(endpoint, ua_public, auth);

        // Get the signing key and create a key pair for VAPID
        let signing_key = self.vapid.signing_key();
        let key_pair = ES256KeyPair::from_bytes(&signing_key.to_bytes())
            .map_err(|e| format!("failed to create VAPID key pair: {}", e))?;

        // Build the request with VAPID
        let request = builder
            .with_vapid(&key_pair, "mailto:noreply@vibes.local")
            .build(payload.as_bytes())
            .map_err(|e| format!("failed to build push request: {}", e))?;

        // Convert to reqwest request and send
        let (parts, body) = request.into_parts();
        let url = parts.uri.to_string();

        let mut req = self.http_client.post(&url);
        for (name, value) in parts.headers.iter() {
            if let Ok(v) = value.to_str() {
                req = req.header(name.as_str(), v);
            }
        }
        req = req.body(body);

        let response = req.send().await.map_err(|e| format!("HTTP error: {}", e))?;

        let status = response.status();
        if status.is_success() || status.as_u16() == 201 {
            Ok(())
        } else {
            Err(format!("Push failed with status {}", status.as_u16()))
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &NotificationConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::ClaudeEvent;

    #[test]
    fn test_permission_event_pattern() {
        // Test that our pattern matching logic is correct
        let event = VibesEvent::Claude {
            session_id: "sess-123".to_string(),
            event: ClaudeEvent::PermissionRequest {
                id: "req-1".to_string(),
                tool: "Bash".to_string(),
                description: "Run command".to_string(),
            },
        };

        // Verify the pattern matches
        if let VibesEvent::Claude {
            session_id,
            event: ClaudeEvent::PermissionRequest { tool, .. },
        } = &event
        {
            assert_eq!(session_id, "sess-123");
            assert_eq!(tool, "Bash");
        } else {
            panic!("Pattern should match");
        }
    }

    #[test]
    fn test_error_event_pattern() {
        let event = VibesEvent::Claude {
            session_id: "sess-123".to_string(),
            event: ClaudeEvent::Error {
                message: "Out of memory".to_string(),
                recoverable: false,
            },
        };

        if let VibesEvent::Claude {
            session_id,
            event: ClaudeEvent::Error { message, .. },
        } = &event
        {
            assert_eq!(session_id, "sess-123");
            assert_eq!(message, "Out of memory");
        } else {
            panic!("Pattern should match");
        }
    }

    #[test]
    fn test_session_completed_pattern() {
        let event = VibesEvent::SessionStateChanged {
            session_id: "sess-123".to_string(),
            state: "Completed".to_string(),
        };

        if let VibesEvent::SessionStateChanged { session_id, state } = &event {
            assert_eq!(session_id, "sess-123");
            assert_eq!(state, "Completed");
        } else {
            panic!("Pattern should match");
        }
    }
}
