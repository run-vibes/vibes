//! History REST API endpoints

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use vibes_core::history::{MessageQuery, MessageRole, SessionQuery, SortField, SortOrder};
use vibes_core::session::SessionState;

use crate::state::AppState;

/// Query params for session list
#[derive(Debug, Deserialize)]
pub struct ListSessionsQuery {
    pub q: Option<String>,
    pub name: Option<String>,
    pub state: Option<String>,
    pub tool: Option<String>,
    pub min_tokens: Option<u32>,
    pub after: Option<i64>,
    pub before: Option<i64>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

impl From<ListSessionsQuery> for SessionQuery {
    fn from(q: ListSessionsQuery) -> Self {
        Self {
            search: q.q,
            name: q.name,
            state: q.state.and_then(|s| parse_session_state(&s)),
            tool: q.tool,
            min_tokens: q.min_tokens,
            after: q.after,
            before: q.before,
            limit: q.limit.unwrap_or(20),
            offset: q.offset.unwrap_or(0),
            sort: q
                .sort
                .map(|s| match s.as_str() {
                    "last_accessed_at" => SortField::LastAccessedAt,
                    "message_count" => SortField::MessageCount,
                    "total_tokens" => SortField::TotalTokens,
                    _ => SortField::CreatedAt,
                })
                .unwrap_or_default(),
            order: q
                .order
                .map(|o| match o.as_str() {
                    "asc" => SortOrder::Asc,
                    _ => SortOrder::Desc,
                })
                .unwrap_or_default(),
        }
    }
}

/// Parse state string to SessionState for filtering queries.
///
/// Note: WaitingPermission and Failed use placeholder values for their inner fields
/// because these are used only for state-based filtering in list queries. The database
/// stores only the state variant name, and actual field values are preserved separately
/// in the session's error_message column. This allows filtering by state category
/// (e.g., "show all failed sessions") without needing the specific error details.
fn parse_session_state(s: &str) -> Option<SessionState> {
    match s.to_lowercase().as_str() {
        "idle" => Some(SessionState::Idle),
        "processing" => Some(SessionState::Processing),
        "waiting_permission" => Some(SessionState::WaitingPermission {
            // Placeholder values - filtering matches on variant, not inner fields
            request_id: String::new(),
            tool: String::new(),
        }),
        "finished" => Some(SessionState::Finished),
        "failed" => Some(SessionState::Failed {
            // Placeholder values - actual error stored in session's error_message field
            message: String::new(),
            recoverable: false,
        }),
        _ => None,
    }
}

/// Query params for message list
#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub role: Option<String>,
}

impl From<ListMessagesQuery> for MessageQuery {
    fn from(q: ListMessagesQuery) -> Self {
        Self {
            limit: q.limit.unwrap_or(50),
            offset: q.offset.unwrap_or(0),
            role: q.role.and_then(|r| MessageRole::parse(&r)),
            before_id: None,
        }
    }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

/// GET /api/history/sessions
pub async fn list_sessions(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListSessionsQuery>,
) -> impl IntoResponse {
    let Some(ref history) = state.history else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "History not configured".into(),
                code: "NOT_CONFIGURED".into(),
            }),
        )
            .into_response();
    };

    match history.list_sessions(&query.into()) {
        Ok(result) => Json(result).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "INTERNAL_ERROR".into(),
            }),
        )
            .into_response(),
    }
}

/// GET /api/history/sessions/:id
pub async fn get_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let Some(ref history) = state.history else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "History not configured".into(),
                code: "NOT_CONFIGURED".into(),
            }),
        )
            .into_response();
    };

    match history.get_session(&id) {
        Ok(Some(session)) => Json(session).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Session not found: {}", id),
                code: "NOT_FOUND".into(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "INTERNAL_ERROR".into(),
            }),
        )
            .into_response(),
    }
}

/// GET /api/history/sessions/:id/messages
pub async fn get_messages(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(query): Query<ListMessagesQuery>,
) -> impl IntoResponse {
    let Some(ref history) = state.history else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "History not configured".into(),
                code: "NOT_CONFIGURED".into(),
            }),
        )
            .into_response();
    };

    match history.get_messages(&id, &query.into()) {
        Ok(result) => Json(result).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "INTERNAL_ERROR".into(),
            }),
        )
            .into_response(),
    }
}

#[derive(Serialize)]
pub struct ResumeResponse {
    pub session_id: String,
    pub claude_session_id: Option<String>,
}

/// POST /api/history/sessions/:id/resume
pub async fn resume_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let Some(ref history) = state.history else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "History not configured".into(),
                code: "NOT_CONFIGURED".into(),
            }),
        )
            .into_response();
    };

    // Get the Claude session ID from history
    match history.get_claude_session_id(&id) {
        Ok(Some(claude_id)) => {
            // Return the Claude session ID for the client to use with --resume
            Json(ResumeResponse {
                session_id: id,
                claude_session_id: Some(claude_id),
            })
            .into_response()
        }
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Session has no Claude session ID for resume".into(),
                code: "NOT_RESUMABLE".into(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "INTERNAL_ERROR".into(),
            }),
        )
            .into_response(),
    }
}

/// DELETE /api/history/sessions/:id
pub async fn delete_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let Some(ref history) = state.history else {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    };

    match history.delete_session(&id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "INTERNAL_ERROR".into(),
            }),
        )
            .into_response(),
    }
}
