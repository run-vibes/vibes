//! HTTP types for plugin route registration

use std::collections::HashMap;

use serde::Serialize;

use crate::error::PluginError;

/// HTTP method for route registration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

/// Specification for an HTTP route
#[derive(Debug, Clone)]
pub struct RouteSpec {
    /// HTTP method
    pub method: HttpMethod,
    /// Path pattern, e.g., "/policy" or "/quarantine/:id"
    pub path: String,
}

/// Incoming HTTP request passed to plugin handler
#[derive(Debug)]
pub struct RouteRequest {
    /// Path parameters extracted from route pattern (e.g., ":id" -> "123")
    pub params: HashMap<String, String>,
    /// Query parameters
    pub query: HashMap<String, String>,
    /// Request body as bytes
    pub body: Vec<u8>,
    /// Request headers
    pub headers: HashMap<String, String>,
}

/// HTTP response from plugin handler
#[derive(Debug)]
pub struct RouteResponse {
    /// HTTP status code
    pub status: u16,
    /// Response body
    pub body: Vec<u8>,
    /// Content-Type header
    pub content_type: String,
}

impl RouteResponse {
    /// Create a JSON response
    pub fn json<T: Serialize>(status: u16, data: &T) -> Result<Self, PluginError> {
        Ok(Self {
            status,
            body: serde_json::to_vec(data).map_err(|e| PluginError::Json(e.to_string()))?,
            content_type: "application/json".to_string(),
        })
    }

    /// Create a plain text response
    pub fn text(status: u16, text: impl Into<String>) -> Self {
        Self {
            status,
            body: text.into().into_bytes(),
            content_type: "text/plain".to_string(),
        }
    }

    /// Create an empty response with status code
    pub fn empty(status: u16) -> Self {
        Self {
            status,
            body: vec![],
            content_type: "application/json".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_equality() {
        assert_eq!(HttpMethod::Get, HttpMethod::Get);
        assert_ne!(HttpMethod::Get, HttpMethod::Post);
    }

    #[test]
    fn test_http_method_debug() {
        assert_eq!(format!("{:?}", HttpMethod::Get), "Get");
    }

    #[test]
    fn test_route_spec_creation() {
        let spec = RouteSpec {
            method: HttpMethod::Get,
            path: "/policy".into(),
        };
        assert_eq!(spec.method, HttpMethod::Get);
        assert_eq!(spec.path, "/policy");
    }

    #[test]
    fn test_route_request_params() {
        use std::collections::HashMap;
        let request = RouteRequest {
            params: [("id".into(), "123".into())].into_iter().collect(),
            query: HashMap::new(),
            body: vec![],
            headers: HashMap::new(),
        };
        assert_eq!(request.params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_route_response_json() {
        use serde::Serialize;

        #[derive(Serialize)]
        struct Data {
            value: i32,
        }

        let resp = RouteResponse::json(200, &Data { value: 42 }).unwrap();
        assert_eq!(resp.status, 200);
        assert_eq!(resp.content_type, "application/json");
        assert!(String::from_utf8_lossy(&resp.body).contains("42"));
    }
}
