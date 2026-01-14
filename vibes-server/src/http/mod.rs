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
use tower_http::compression::CompressionLayer;

use crate::AppState;
use crate::middleware::auth_middleware;
use crate::ws::{assessment_ws, firehose_ws, traces_ws, ws_handler};

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
        .route("/ws/assessment", get(assessment_ws))
        .route("/ws/traces", get(traces_ws))
        .layer(middleware::from_fn(auth_middleware))
        .layer(Extension(auth_layer))
        // Plugin routes (checked before static fallback)
        .merge(plugins::plugin_router())
        .with_state(state)
        // Fallback serves embedded web-ui for SPA routing
        .fallback(static_files::static_handler)
        // Compression layer (gzip and brotli) applied to all responses
        .layer(CompressionLayer::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::header::{ACCEPT_ENCODING, CONTENT_ENCODING};
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

    #[tokio::test]
    async fn test_compression_gzip_enabled() {
        let state = Arc::new(AppState::new());
        let router = create_router(state);
        let server =
            TestServer::new(router.into_make_service_with_connect_info::<SocketAddr>()).unwrap();

        let response = server
            .get("/api/health")
            .add_header(ACCEPT_ENCODING, "gzip")
            .await;

        response.assert_status_ok();

        // Should have Content-Encoding: gzip when compression is enabled
        let content_encoding = response.header(CONTENT_ENCODING);
        assert_eq!(
            content_encoding.to_str().unwrap(),
            "gzip",
            "Response should be gzip compressed when Accept-Encoding: gzip is sent"
        );
    }

    #[tokio::test]
    async fn test_compression_brotli_enabled() {
        let state = Arc::new(AppState::new());
        let router = create_router(state);
        let server =
            TestServer::new(router.into_make_service_with_connect_info::<SocketAddr>()).unwrap();

        let response = server
            .get("/api/health")
            .add_header(ACCEPT_ENCODING, "br")
            .await;

        response.assert_status_ok();

        // Should have Content-Encoding: br when compression is enabled
        let content_encoding = response.header(CONTENT_ENCODING);
        assert_eq!(
            content_encoding.to_str().unwrap(),
            "br",
            "Response should be brotli compressed when Accept-Encoding: br is sent"
        );
    }

    #[tokio::test]
    #[ignore = "Requires plugins installed; dynamically loaded plugins with background tasks cause issues in tests"]
    async fn test_groove_routes_work_when_plugin_loaded() {
        use std::sync::Arc;
        use tokio::sync::RwLock;
        use vibes_core::{PluginHost, PluginHostConfig};

        // Create plugin host and load plugins
        let config = PluginHostConfig::default();
        let mut host = PluginHost::new(config);

        // Load all plugins (including groove)
        host.load_all().expect("Failed to load plugins");

        // Create state with the loaded plugin host
        let state = AppState::with_plugin_host(Arc::new(RwLock::new(host)));
        let router = create_router(Arc::new(state));
        let server =
            TestServer::new(router.into_make_service_with_connect_info::<SocketAddr>()).unwrap();

        // Request groove trust levels - should return 200 with JSON
        let response = server.get("/api/groove/trust/levels").await;
        response.assert_status_ok();

        // Verify it's JSON with trust level data
        let body = response.text();
        assert!(
            body.contains("levels") && body.contains("Local"),
            "Groove route should return trust levels JSON. Got: {}",
            body
        );
    }
}
