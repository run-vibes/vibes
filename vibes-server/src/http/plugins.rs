//! Plugin HTTP route handler

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use vibes_plugin_api::{HttpMethod, RouteRequest};

use crate::AppState;

/// Convert axum Method to plugin HttpMethod
fn to_http_method(method: &axum::http::Method) -> Option<HttpMethod> {
    match *method {
        axum::http::Method::GET => Some(HttpMethod::Get),
        axum::http::Method::POST => Some(HttpMethod::Post),
        axum::http::Method::PUT => Some(HttpMethod::Put),
        axum::http::Method::DELETE => Some(HttpMethod::Delete),
        axum::http::Method::PATCH => Some(HttpMethod::Patch),
        _ => None,
    }
}

/// Plugin route handler
pub async fn handle_plugin_route(State(state): State<Arc<AppState>>, request: Request) -> Response {
    let method = match to_http_method(request.method()) {
        Some(m) => m,
        None => {
            return (StatusCode::METHOD_NOT_ALLOWED, "Method not allowed").into_response();
        }
    };

    let path = request.uri().path().to_string();
    let query = parse_query(request.uri().query());
    let headers = extract_headers(request.headers());

    // Get plugin host and find matching route
    let plugin_host = state.plugin_host().read().await;
    let Some((route, params)) = plugin_host.route_registry().match_route(method, &path) else {
        return (StatusCode::NOT_FOUND, r#"{"error":"Not found"}"#).into_response();
    };

    let plugin_name = route.plugin_name.clone();
    let route_path = route.spec.path.clone();
    drop(plugin_host);

    // Extract body
    let body = match axum::body::to_bytes(request.into_body(), 1024 * 1024).await {
        Ok(b) => b.to_vec(),
        Err(_) => {
            return (StatusCode::BAD_REQUEST, "Failed to read body").into_response();
        }
    };

    let route_request = RouteRequest {
        params,
        query,
        body,
        headers,
    };

    // Dispatch to plugin
    let mut plugin_host = state.plugin_host().write().await;
    match plugin_host.dispatch_route(&plugin_name, method, &route_path, route_request) {
        Ok(resp) => Response::builder()
            .status(resp.status)
            .header("Content-Type", resp.content_type)
            .body(Body::from(resp.body))
            .unwrap_or_else(|e| {
                tracing::error!("Failed to build HTTP response: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
            }),
        Err(e) => {
            let error_json = json!({"error": e.to_string()});
            Response::builder()
                .status(500)
                .header("Content-Type", "application/json")
                .body(Body::from(error_json.to_string()))
                .unwrap_or_else(|e| {
                    tracing::error!("Failed to build HTTP error response: {e}");
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
                })
        }
    }
}

fn parse_query(query: Option<&str>) -> HashMap<String, String> {
    query
        .map(|q| {
            url::form_urlencoded::parse(q.as_bytes())
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn extract_headers(headers: &axum::http::HeaderMap) -> HashMap<String, String> {
    headers
        .iter()
        .filter_map(|(k, v)| v.to_str().ok().map(|val| (k.to_string(), val.to_string())))
        .collect()
}

/// Create router for plugin routes
pub fn plugin_router() -> Router<Arc<AppState>> {
    Router::new().fallback(handle_plugin_route)
}
