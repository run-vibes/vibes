//! HTTP server module

mod api;

use std::sync::Arc;

use axum::{routing::get, Router};

use crate::AppState;

pub use api::{HealthResponse, SessionListResponse, SessionSummary};

/// Create the HTTP router with all routes configured
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(api::health))
        .route("/api/claude/sessions", get(api::list_sessions))
        .with_state(state)
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
