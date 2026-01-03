//! HTTP server module

mod api;
pub mod plugins;
mod push;
mod static_files;

use std::sync::Arc;

use axum::{
    Extension, Router, middleware,
    routing::{delete, get, post},
};

use crate::AppState;
use crate::middleware::auth_middleware;
use crate::ws::{firehose_ws, ws_handler};

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
        .route("/ws", get(ws_handler))
        .route("/ws/firehose", get(firehose_ws))
        .layer(middleware::from_fn(auth_middleware))
        .layer(Extension(auth_layer))
        // Plugin routes (checked before static fallback)
        .merge(plugins::plugin_router())
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

    #[tokio::test]
    async fn test_plugin_routes_return_json_not_html() {
        let state = Arc::new(AppState::new());
        let router = create_router(state);
        let server =
            TestServer::new(router.into_make_service_with_connect_info::<SocketAddr>()).unwrap();

        // Request an unregistered plugin route - should return JSON 404, not HTML
        let response = server.get("/api/groove/policy").await;

        // Plugin router returns 404 for unregistered routes (not the SPA HTML)
        response.assert_status_not_found();

        // Verify it's JSON, not HTML
        let body = response.text();
        assert!(
            body.contains("error") && !body.contains("<!DOCTYPE"),
            "Plugin route should return JSON error, not HTML. Got: {}",
            body
        );
    }
}
