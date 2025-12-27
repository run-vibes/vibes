//! REST API handlers

use std::sync::Arc;

use axum::{Json, extract::State};
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

/// Summary of a session for list views
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionSummary {
    /// Session ID
    pub id: String,
    /// Session name
    pub name: Option<String>,
    /// Current state
    pub state: String,
    /// When the session was created (ISO 8601 format)
    pub created_at: String,
}

/// Response for listing sessions
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionListResponse {
    /// List of sessions
    pub sessions: Vec<SessionSummary>,
}

/// List all Claude sessions
pub async fn list_sessions(State(state): State<Arc<AppState>>) -> Json<SessionListResponse> {
    let sessions_full = state.session_manager.list_sessions_full().await;

    let sessions = sessions_full
        .into_iter()
        .map(|(id, name, session_state, created_at)| {
            // Convert SystemTime to ISO 8601 format
            let created_at_str = created_at
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| {
                    let secs = d.as_secs();
                    let datetime =
                        chrono::DateTime::from_timestamp(secs as i64, 0).unwrap_or_default();
                    datetime.to_rfc3339()
                })
                .unwrap_or_else(|_| "unknown".to_string());

            SessionSummary {
                id,
                name,
                state: format!("{:?}", session_state),
                created_at: created_at_str,
            }
        })
        .collect();

    Json(SessionListResponse { sessions })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, routing::get};
    use axum_test::TestServer;

    fn create_test_app() -> Router {
        let state = Arc::new(AppState::new());
        Router::new()
            .route("/api/health", get(health))
            .route("/api/claude/sessions", get(list_sessions))
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

    #[tokio::test]
    async fn test_list_sessions_empty() {
        let server = TestServer::new(create_test_app()).unwrap();

        let response = server.get("/api/claude/sessions").await;
        response.assert_status_ok();

        let body: SessionListResponse = response.json();
        assert!(body.sessions.is_empty());
    }
}
