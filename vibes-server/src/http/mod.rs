//! HTTP server module

mod api;
mod groove;
mod push;
mod static_files;

use std::sync::Arc;

use axum::{
    Extension, Router, middleware,
    routing::{delete, get, post},
};

use crate::AppState;
use crate::middleware::auth_middleware;
use crate::ws::ws_handler;

pub use api::{
    AuthIdentityResponse, AuthStatusResponse, HealthResponse, SessionListResponse, SessionSummary,
    TunnelStatusResponse,
};
pub use push::{
    PushErrorResponse, SubscribeRequest, SubscribeResponse, SubscriptionInfo,
    SubscriptionListResponse, VapidKeyResponse,
};

/// Create the HTTP router with all routes configured
pub fn create_router(state: Arc<AppState>) -> Router {
    let auth_layer = state.auth_layer.clone();

    Router::new()
        .route("/api/health", get(api::health))
        .route("/api/claude/sessions", get(api::list_sessions))
        .route("/api/tunnel/status", get(api::get_tunnel_status))
        .route("/api/auth/status", get(api::get_auth_status))
        // Push notification endpoints
        .route("/api/push/vapid-key", get(push::get_vapid_key))
        .route("/api/push/subscribe", post(push::subscribe))
        .route("/api/push/subscribe/:id", delete(push::unsubscribe))
        .route("/api/push/subscriptions", get(push::list_subscriptions))
        // Groove security endpoints
        .route("/api/groove/policy", get(groove::get_policy))
        .route("/api/groove/trust/levels", get(groove::get_trust_levels))
        .route(
            "/api/groove/trust/role/:role",
            get(groove::get_role_permissions),
        )
        .route("/api/groove/quarantine", get(groove::list_quarantined))
        .route(
            "/api/groove/quarantine/stats",
            get(groove::get_quarantine_stats),
        )
        .route(
            "/api/groove/quarantine/:id/review",
            post(groove::review_quarantined),
        )
        .route("/ws", get(ws_handler))
        .layer(middleware::from_fn(auth_middleware))
        .layer(Extension(auth_layer))
        .with_state(state)
        // Fallback serves embedded web-ui for SPA routing
        .fallback(static_files::static_handler)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn test_router_has_health_endpoint() {
        let state = Arc::new(AppState::new());
        let router = create_router(state);
        // Use into_make_service_with_connect_info to provide ConnectInfo<SocketAddr>
        // for the auth middleware (requires HTTP transport, not mock)
        let server =
            TestServer::new(router.into_make_service_with_connect_info::<SocketAddr>()).unwrap();

        let response = server.get("/api/health").await;
        response.assert_status_ok();
    }
}
