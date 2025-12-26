//! REST API handlers

use std::sync::Arc;

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::AppState;

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Status of the server
    pub status: String,
    /// Server version
    pub version: String,
    /// Seconds since server started
    pub uptime_seconds: i64,
    /// Number of active sessions
    pub active_sessions: usize,
}

/// Health check endpoint
///
/// Returns server status, version, uptime, and active session count.
pub async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let active_sessions = state.session_manager.session_count().await;

    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.uptime_seconds(),
        active_sessions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router};
    use axum_test::TestServer;

    fn create_test_app() -> Router {
        let state = Arc::new(AppState::new());
        Router::new()
            .route("/api/health", get(health))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let server = TestServer::new(create_test_app()).unwrap();

        let response = server.get("/api/health").await;
        response.assert_status_ok();

        let body: HealthResponse = response.json();
        assert_eq!(body.status, "ok");
        assert_eq!(body.version, env!("CARGO_PKG_VERSION"));
        assert!(body.uptime_seconds >= 0);
        assert_eq!(body.active_sessions, 0);
    }
}
