//! HTTP server module

mod api;
mod static_files;

use std::sync::Arc;

use axum::{Router, routing::get};

use crate::AppState;
use crate::ws::ws_handler;

pub use api::{
    AuthIdentityResponse, AuthStatusResponse, HealthResponse, SessionListResponse, SessionSummary,
    TunnelStatusResponse,
};

/// Create the HTTP router with all routes configured
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(api::health))
        .route("/api/claude/sessions", get(api::list_sessions))
        .route("/api/tunnel/status", get(api::get_tunnel_status))
        .route("/api/auth/status", get(api::get_auth_status))
        .route("/ws", get(ws_handler))
        .with_state(state)
        // Fallback serves embedded web-ui for SPA routing
        .fallback(static_files::static_handler)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;

    #[tokio::test]
    async fn test_router_has_health_endpoint() {
        let state = Arc::new(AppState::new());
        let router = create_router(state);
        let server = TestServer::new(router).unwrap();

        let response = server.get("/api/health").await;
        response.assert_status_ok();
    }
}
