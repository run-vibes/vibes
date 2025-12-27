//! Authentication middleware for axum

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use vibes_core::{AccessConfig, AuthContext, AuthError, JwtValidator};

/// Header name for Cloudflare Access JWT
const CF_ACCESS_JWT_HEADER: &str = "cf-access-jwt-assertion";

/// Cookie name for Cloudflare Access JWT
const CF_AUTHORIZATION_COOKIE: &str = "CF_Authorization";

/// Authentication layer state
#[derive(Clone)]
pub struct AuthLayer {
    validator: Option<Arc<JwtValidator>>,
    config: AccessConfig,
}

impl AuthLayer {
    /// Create a new AuthLayer with the given configuration
    pub fn new(config: AccessConfig) -> Self {
        let validator = if config.enabled && config.is_valid() {
            Some(Arc::new(JwtValidator::new(config.clone())))
        } else {
            None
        };

        Self { validator, config }
    }

    /// Create a disabled AuthLayer (for testing or when auth is not configured)
    pub fn disabled() -> Self {
        Self {
            validator: None,
            config: AccessConfig::default(),
        }
    }
}

/// Check if the request is from localhost
fn is_localhost(addr: &SocketAddr) -> bool {
    let ip = addr.ip();
    ip.is_loopback()
}

/// Extract JWT from request headers or cookies
fn extract_jwt(request: &Request) -> Option<String> {
    // Try header first
    if let Some(header) = request.headers().get(CF_ACCESS_JWT_HEADER) {
        if let Ok(value) = header.to_str() {
            return Some(value.to_string());
        }
    }

    // Fall back to cookie
    if let Some(cookie_header) = request.headers().get("cookie") {
        if let Ok(cookies) = cookie_header.to_str() {
            for cookie in cookies.split(';') {
                let cookie = cookie.trim();
                if let Some(value) = cookie.strip_prefix(&format!("{}=", CF_AUTHORIZATION_COOKIE)) {
                    return Some(value.to_string());
                }
            }
        }
    }

    None
}

/// Authentication middleware function
pub async fn auth_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    axum::Extension(auth_layer): axum::Extension<AuthLayer>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_context = if auth_layer.config.bypass_localhost && is_localhost(&addr) {
        // Localhost bypass
        AuthContext::Local
    } else if let Some(ref validator) = auth_layer.validator {
        // Auth is enabled, validate JWT
        match extract_jwt(&request) {
            Some(token) => match validator.validate(&token).await {
                Ok(identity) => AuthContext::Authenticated { identity },
                Err(AuthError::Expired) => {
                    tracing::debug!("JWT expired");
                    return Err(StatusCode::UNAUTHORIZED);
                }
                Err(AuthError::InvalidAudience) => {
                    tracing::debug!("Invalid JWT audience");
                    return Err(StatusCode::UNAUTHORIZED);
                }
                Err(e) => {
                    tracing::debug!("JWT validation failed: {}", e);
                    return Err(StatusCode::UNAUTHORIZED);
                }
            },
            None => {
                // No token provided on non-localhost request with auth enabled
                tracing::debug!("No JWT token provided");
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    } else {
        // Auth not enabled, treat as local
        AuthContext::Local
    };

    // Attach auth context to request extensions
    request.extensions_mut().insert(auth_context);

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_is_localhost_loopback() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        assert!(is_localhost(&addr));
    }

    #[test]
    fn test_is_localhost_not_loopback() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 8080);
        assert!(!is_localhost(&addr));
    }

    #[test]
    fn test_auth_layer_disabled() {
        let layer = AuthLayer::disabled();
        assert!(layer.validator.is_none());
        assert!(!layer.config.enabled);
    }

    #[test]
    fn test_auth_layer_enabled() {
        let config = AccessConfig::new("team", "aud");
        let layer = AuthLayer::new(config);
        assert!(layer.validator.is_some());
        assert!(layer.config.enabled);
    }
}
