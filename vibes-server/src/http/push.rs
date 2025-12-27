//! Push notification API handlers

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use vibes_core::{PushSubscription, SubscriptionKeys};

use crate::AppState;

/// Response for GET /api/push/vapid-key
#[derive(Debug, Serialize, Deserialize)]
pub struct VapidKeyResponse {
    /// Base64url-encoded public key for VAPID
    pub public_key: String,
}

/// Request body for POST /api/push/subscribe
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscribeRequest {
    /// Push subscription endpoint URL
    pub endpoint: String,
    /// Subscription keys
    pub keys: SubscriptionKeysRequest,
    /// User agent string (optional)
    pub user_agent: Option<String>,
}

/// Subscription keys in request
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionKeysRequest {
    /// P-256 ECDH public key (base64url)
    pub p256dh: String,
    /// Auth secret (base64url)
    pub auth: String,
}

/// Response for POST /api/push/subscribe
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscribeResponse {
    /// Subscription ID for later management
    pub id: String,
}

/// Response for GET /api/push/subscriptions
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionListResponse {
    /// Number of subscriptions
    pub count: usize,
    /// List of subscription IDs
    pub subscriptions: Vec<SubscriptionInfo>,
}

/// Summary info for a subscription
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionInfo {
    /// Subscription ID
    pub id: String,
    /// Push endpoint (partial, for display)
    pub endpoint_preview: String,
    /// When the subscription was created
    pub created_at: String,
}

/// Error response
#[derive(Debug, Serialize, Deserialize)]
pub struct PushErrorResponse {
    /// Error message
    pub error: String,
}

/// GET /api/push/vapid-key - Get public VAPID key for push subscription
pub async fn get_vapid_key(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match &state.vapid {
        Some(vapid) => {
            let response = VapidKeyResponse {
                public_key: vapid.public_key().to_string(),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(PushErrorResponse {
                error: "Push notifications not configured".to_string(),
            }),
        )
            .into_response(),
    }
}

/// POST /api/push/subscribe - Subscribe to push notifications
pub async fn subscribe(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SubscribeRequest>,
) -> impl IntoResponse {
    let subscriptions = match &state.subscriptions {
        Some(s) => s,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(PushErrorResponse {
                    error: "Push notifications not configured".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Generate a unique subscription ID
    let id = uuid::Uuid::new_v4().to_string();

    let subscription = PushSubscription {
        id: id.clone(),
        endpoint: request.endpoint,
        keys: SubscriptionKeys {
            p256dh: request.keys.p256dh,
            auth: request.keys.auth,
        },
        user_agent: request.user_agent.unwrap_or_else(|| "Unknown".to_string()),
        created_at: chrono::Utc::now(),
    };

    match subscriptions.add(subscription).await {
        Ok(()) => (StatusCode::CREATED, Json(SubscribeResponse { id })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(PushErrorResponse {
                error: format!("Failed to save subscription: {}", e),
            }),
        )
            .into_response(),
    }
}

/// DELETE /api/push/subscribe/:id - Unsubscribe from push notifications
pub async fn unsubscribe(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let subscriptions = match &state.subscriptions {
        Some(s) => s,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(PushErrorResponse {
                    error: "Push notifications not configured".to_string(),
                }),
            )
                .into_response();
        }
    };

    match subscriptions.remove(&id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(PushErrorResponse {
                error: "Subscription not found".to_string(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(PushErrorResponse {
                error: format!("Failed to remove subscription: {}", e),
            }),
        )
            .into_response(),
    }
}

/// GET /api/push/subscriptions - List all subscriptions
pub async fn list_subscriptions(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let subscriptions = match &state.subscriptions {
        Some(s) => s,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(PushErrorResponse {
                    error: "Push notifications not configured".to_string(),
                }),
            )
                .into_response();
        }
    };

    let subs = subscriptions.list().await;
    let count = subs.len();

    let subscription_infos: Vec<SubscriptionInfo> = subs
        .into_iter()
        .map(|s| {
            // Show partial endpoint for privacy
            let endpoint_preview = if s.endpoint.len() > 50 {
                format!("{}...", &s.endpoint[..50])
            } else {
                s.endpoint.clone()
            };

            SubscriptionInfo {
                id: s.id,
                endpoint_preview,
                created_at: s.created_at.to_rfc3339(),
            }
        })
        .collect();

    (
        StatusCode::OK,
        Json(SubscriptionListResponse {
            count,
            subscriptions: subscription_infos,
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        routing::{delete, get, post},
    };
    use axum_test::TestServer;

    fn create_test_app() -> Router {
        let state = Arc::new(AppState::new());
        Router::new()
            .route("/api/push/vapid-key", get(get_vapid_key))
            .route("/api/push/subscribe", post(subscribe))
            .route("/api/push/subscribe/:id", delete(unsubscribe))
            .route("/api/push/subscriptions", get(list_subscriptions))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_vapid_key_not_configured() {
        let server = TestServer::new(create_test_app()).unwrap();

        let response = server.get("/api/push/vapid-key").await;
        response.assert_status(StatusCode::SERVICE_UNAVAILABLE);

        let body: PushErrorResponse = response.json();
        assert!(body.error.contains("not configured"));
    }

    #[tokio::test]
    async fn test_subscribe_not_configured() {
        let server = TestServer::new(create_test_app()).unwrap();

        let response = server
            .post("/api/push/subscribe")
            .json(&SubscribeRequest {
                endpoint: "https://push.example.com/test".to_string(),
                keys: SubscriptionKeysRequest {
                    p256dh: "test-key".to_string(),
                    auth: "test-auth".to_string(),
                },
                user_agent: None,
            })
            .await;

        response.assert_status(StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_unsubscribe_not_configured() {
        let server = TestServer::new(create_test_app()).unwrap();

        let response = server.delete("/api/push/subscribe/some-id").await;
        response.assert_status(StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_list_subscriptions_not_configured() {
        let server = TestServer::new(create_test_app()).unwrap();

        let response = server.get("/api/push/subscriptions").await;
        response.assert_status(StatusCode::SERVICE_UNAVAILABLE);
    }
}
