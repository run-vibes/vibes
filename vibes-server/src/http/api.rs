//! REST API handlers

use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use serde::{Deserialize, Serialize};
use vibes_core::AuthContext;

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

/// Tunnel status response
#[derive(Debug, Serialize, Deserialize)]
pub struct TunnelStatusResponse {
    /// Current tunnel state
    pub state: String,
    /// Tunnel mode (quick or named)
    pub mode: Option<String>,
    /// Public URL when connected
    pub url: Option<String>,
    /// Tunnel name for named tunnels
    pub tunnel_name: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
}

/// GET /api/tunnel/status - Get tunnel status
pub async fn get_tunnel_status(State(state): State<Arc<AppState>>) -> Json<TunnelStatusResponse> {
    let manager = state.tunnel_manager.read().await;
    let tunnel_state = manager.state().await;

    let (status, url, error) = match &tunnel_state {
        vibes_core::TunnelState::Disabled => ("disabled", None, None),
        vibes_core::TunnelState::Starting => ("starting", None, None),
        vibes_core::TunnelState::Connected { url, .. } => ("connected", Some(url.clone()), None),
        vibes_core::TunnelState::Reconnecting { last_error, .. } => {
            ("reconnecting", None, Some(last_error.clone()))
        }
        vibes_core::TunnelState::Failed { error, .. } => ("failed", None, Some(error.clone())),
        vibes_core::TunnelState::Stopped => ("stopped", None, None),
    };

    let mode = if manager.is_enabled() {
        Some(manager.config().mode.as_str())
    } else {
        None
    };

    Json(TunnelStatusResponse {
        state: status.to_string(),
        mode: mode.map(|s| s.to_string()),
        url,
        tunnel_name: manager.config().tunnel_name().map(|s| s.to_string()),
        error,
    })
}

/// Auth status response
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthStatusResponse {
    /// Whether the request is authenticated
    pub authenticated: bool,
    /// Request source
    pub source: String,
    /// Identity info (if authenticated)
    pub identity: Option<AuthIdentityResponse>,
}

/// Identity in auth response
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthIdentityResponse {
    /// User's email address
    pub email: String,
    /// User's display name
    pub name: Option<String>,
    /// Identity provider used
    pub identity_provider: Option<String>,
}

/// GET /api/auth/status - Get current auth status
pub async fn get_auth_status(
    auth_context: Option<Extension<AuthContext>>,
) -> Json<AuthStatusResponse> {
    let auth_context = auth_context
        .map(|Extension(ctx)| ctx)
        .unwrap_or(AuthContext::Local);

    let (authenticated, source, identity) = match &auth_context {
        AuthContext::Local => (false, "local", None),
        AuthContext::Authenticated { identity } => (
            true,
            "tunnel",
            Some(AuthIdentityResponse {
                email: identity.email.clone(),
                name: identity.name.clone(),
                identity_provider: identity.identity_provider.clone(),
            }),
        ),
        AuthContext::Anonymous => (false, "anonymous", None),
    };

    Json(AuthStatusResponse {
        authenticated,
        source: source.to_string(),
        identity,
    })
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
            .route("/api/tunnel/status", get(get_tunnel_status))
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

    #[tokio::test]
    async fn test_get_tunnel_status_disabled() {
        let server = TestServer::new(create_test_app()).unwrap();

        let response = server.get("/api/tunnel/status").await;
        response.assert_status_ok();

        let body: TunnelStatusResponse = response.json();
        assert_eq!(body.state, "disabled");
        assert!(body.mode.is_none());
        assert!(body.url.is_none());
        assert!(body.tunnel_name.is_none());
        assert!(body.error.is_none());
    }
}
