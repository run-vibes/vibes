# Milestone 2.2: Cloudflare Access Authentication - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Cloudflare Access JWT authentication to vibes, protecting tunnel access while allowing unauthenticated localhost access.

**Architecture:** Auth middleware layer in axum that validates JWTs from Cloudflare Access headers/cookies. JWKS cached with 1-hour TTL and automatic refresh on unknown key IDs. Localhost requests bypass auth entirely.

**Tech Stack:** Rust, axum middleware, jsonwebtoken crate, reqwest for JWKS fetch, TanStack React for UI

---

## Task 1: Add jsonwebtoken Dependency

**Files:**
- Modify: `vibes-core/Cargo.toml`

**Step 1: Add jsonwebtoken to dependencies**

Add to `vibes-core/Cargo.toml`:

```toml
jsonwebtoken = "9"
```

**Step 2: Verify it compiles**

Run: `cargo check -p vibes-core`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add vibes-core/Cargo.toml
git commit -m "chore: add jsonwebtoken dependency for JWT validation"
```

---

## Task 2: Create AuthError Type

**Files:**
- Create: `vibes-core/src/auth/error.rs`
- Create: `vibes-core/src/auth/mod.rs`
- Modify: `vibes-core/src/lib.rs`

**Step 1: Create the auth module directory**

```bash
mkdir -p vibes-core/src/auth
```

**Step 2: Write AuthError type**

Create `vibes-core/src/auth/error.rs`:

```rust
//! Authentication error types

use thiserror::Error;

/// Errors that can occur during authentication
#[derive(Debug, Error)]
pub enum AuthError {
    /// No authentication token was provided in the request
    #[error("no authentication token provided")]
    MissingToken,

    /// The token format is invalid (not a valid JWT)
    #[error("invalid token format: {0}")]
    InvalidFormat(String),

    /// The token signature verification failed
    #[error("token signature verification failed")]
    InvalidSignature,

    /// The token has expired
    #[error("token has expired")]
    Expired,

    /// The token's audience claim doesn't match the expected value
    #[error("invalid audience claim")]
    InvalidAudience,

    /// The key ID in the token doesn't match any known keys
    #[error("unknown key ID: {0}")]
    UnknownKeyId(String),

    /// Failed to fetch JWKS from Cloudflare
    #[error("failed to fetch JWKS: {0}")]
    JwksFetchError(String),

    /// JWT decoding error from jsonwebtoken crate
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_error_display() {
        let err = AuthError::MissingToken;
        assert_eq!(err.to_string(), "no authentication token provided");
    }

    #[test]
    fn test_auth_error_unknown_key_id() {
        let err = AuthError::UnknownKeyId("abc123".to_string());
        assert_eq!(err.to_string(), "unknown key ID: abc123");
    }
}
```

**Step 3: Create mod.rs for auth module**

Create `vibes-core/src/auth/mod.rs`:

```rust
//! Authentication module for Cloudflare Access JWT validation

mod error;

pub use error::AuthError;
```

**Step 4: Export auth module from lib.rs**

Add to `vibes-core/src/lib.rs` after other module declarations:

```rust
pub mod auth;
```

And add to the pub use section:

```rust
pub use auth::AuthError;
```

**Step 5: Run tests**

Run: `cargo test -p vibes-core auth`
Expected: 2 tests pass

**Step 6: Commit**

```bash
git add vibes-core/src/auth/
git add vibes-core/src/lib.rs
git commit -m "feat(auth): add AuthError type"
```

---

## Task 3: Create AccessConfig Type

**Files:**
- Create: `vibes-core/src/auth/config.rs`
- Modify: `vibes-core/src/auth/mod.rs`

**Step 1: Write AccessConfig with tests**

Create `vibes-core/src/auth/config.rs`:

```rust
//! Configuration for Cloudflare Access authentication

use serde::{Deserialize, Serialize};

/// Configuration for Cloudflare Access authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessConfig {
    /// Whether authentication is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Cloudflare Access team name (e.g., "mycompany" for mycompany.cloudflareaccess.com)
    #[serde(default)]
    pub team: String,

    /// Application audience (AUD) tag from Cloudflare Access
    #[serde(default)]
    pub aud: String,

    /// Whether to bypass authentication for localhost requests
    #[serde(default = "default_bypass_localhost")]
    pub bypass_localhost: bool,

    /// Clock skew leeway in seconds for token expiry validation
    #[serde(default = "default_clock_skew")]
    pub clock_skew_seconds: u64,
}

fn default_bypass_localhost() -> bool {
    true
}

fn default_clock_skew() -> u64 {
    60
}

impl Default for AccessConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            team: String::new(),
            aud: String::new(),
            bypass_localhost: default_bypass_localhost(),
            clock_skew_seconds: default_clock_skew(),
        }
    }
}

impl AccessConfig {
    /// Create a new AccessConfig with the given team and AUD
    pub fn new(team: impl Into<String>, aud: impl Into<String>) -> Self {
        Self {
            enabled: true,
            team: team.into(),
            aud: aud.into(),
            bypass_localhost: true,
            clock_skew_seconds: 60,
        }
    }

    /// Returns the JWKS URL for this team
    pub fn jwks_url(&self) -> String {
        format!(
            "https://{}.cloudflareaccess.com/cdn-cgi/access/certs",
            self.team
        )
    }

    /// Check if the config is valid (has required fields when enabled)
    pub fn is_valid(&self) -> bool {
        if !self.enabled {
            return true;
        }
        !self.team.is_empty() && !self.aud.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AccessConfig::default();
        assert!(!config.enabled);
        assert!(config.bypass_localhost);
        assert_eq!(config.clock_skew_seconds, 60);
    }

    #[test]
    fn test_new_config() {
        let config = AccessConfig::new("myteam", "aud123");
        assert!(config.enabled);
        assert_eq!(config.team, "myteam");
        assert_eq!(config.aud, "aud123");
    }

    #[test]
    fn test_jwks_url() {
        let config = AccessConfig::new("myteam", "aud123");
        assert_eq!(
            config.jwks_url(),
            "https://myteam.cloudflareaccess.com/cdn-cgi/access/certs"
        );
    }

    #[test]
    fn test_is_valid_disabled() {
        let config = AccessConfig::default();
        assert!(config.is_valid());
    }

    #[test]
    fn test_is_valid_enabled_with_fields() {
        let config = AccessConfig::new("team", "aud");
        assert!(config.is_valid());
    }

    #[test]
    fn test_is_valid_enabled_missing_fields() {
        let mut config = AccessConfig::default();
        config.enabled = true;
        assert!(!config.is_valid());
    }

    #[test]
    fn test_deserialize_toml() {
        let toml = r#"
            enabled = true
            team = "mycompany"
            aud = "abc123"
        "#;
        let config: AccessConfig = toml::from_str(toml).unwrap();
        assert!(config.enabled);
        assert_eq!(config.team, "mycompany");
        assert_eq!(config.aud, "abc123");
        assert!(config.bypass_localhost); // default
    }
}
```

**Step 2: Export from mod.rs**

Update `vibes-core/src/auth/mod.rs`:

```rust
//! Authentication module for Cloudflare Access JWT validation

mod config;
mod error;

pub use config::AccessConfig;
pub use error::AuthError;
```

**Step 3: Export from lib.rs**

Update the pub use in `vibes-core/src/lib.rs`:

```rust
pub use auth::{AccessConfig, AuthError};
```

**Step 4: Run tests**

Run: `cargo test -p vibes-core auth`
Expected: All tests pass

**Step 5: Commit**

```bash
git add vibes-core/src/auth/config.rs
git add vibes-core/src/auth/mod.rs
git add vibes-core/src/lib.rs
git commit -m "feat(auth): add AccessConfig type with TOML parsing"
```

---

## Task 4: Create AuthContext and AccessIdentity Types

**Files:**
- Create: `vibes-core/src/auth/context.rs`
- Modify: `vibes-core/src/auth/mod.rs`
- Modify: `vibes-core/src/lib.rs`

**Step 1: Write context types**

Create `vibes-core/src/auth/context.rs`:

```rust
//! Authentication context types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Authentication context for a request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum AuthContext {
    /// Request from localhost, authentication bypassed
    Local,
    /// Authenticated via Cloudflare Access
    Authenticated {
        /// The authenticated user's identity
        identity: AccessIdentity,
    },
    /// No valid authentication (should have been rejected by middleware)
    Anonymous,
}

impl AuthContext {
    /// Returns the identity if authenticated, None otherwise
    pub fn identity(&self) -> Option<&AccessIdentity> {
        match self {
            AuthContext::Authenticated { identity } => Some(identity),
            _ => None,
        }
    }

    /// Returns true if the request is from localhost
    pub fn is_local(&self) -> bool {
        matches!(self, AuthContext::Local)
    }

    /// Returns true if the request is authenticated
    pub fn is_authenticated(&self) -> bool {
        matches!(self, AuthContext::Authenticated { .. })
    }
}

/// Identity information from Cloudflare Access JWT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessIdentity {
    /// User's email address
    pub email: String,
    /// User's display name (if available)
    pub name: Option<String>,
    /// Identity provider used (e.g., "github", "google")
    pub identity_provider: Option<String>,
    /// When the token expires
    pub expires_at: DateTime<Utc>,
}

impl AccessIdentity {
    /// Create a new AccessIdentity
    pub fn new(email: impl Into<String>, expires_at: DateTime<Utc>) -> Self {
        Self {
            email: email.into(),
            name: None,
            identity_provider: None,
            expires_at,
        }
    }

    /// Set the display name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the identity provider
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.identity_provider = Some(provider.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_context_local() {
        let ctx = AuthContext::Local;
        assert!(ctx.is_local());
        assert!(!ctx.is_authenticated());
        assert!(ctx.identity().is_none());
    }

    #[test]
    fn test_auth_context_authenticated() {
        let identity = AccessIdentity::new("user@example.com", Utc::now());
        let ctx = AuthContext::Authenticated {
            identity: identity.clone(),
        };
        assert!(!ctx.is_local());
        assert!(ctx.is_authenticated());
        assert_eq!(ctx.identity().unwrap().email, "user@example.com");
    }

    #[test]
    fn test_auth_context_anonymous() {
        let ctx = AuthContext::Anonymous;
        assert!(!ctx.is_local());
        assert!(!ctx.is_authenticated());
        assert!(ctx.identity().is_none());
    }

    #[test]
    fn test_access_identity_builder() {
        let identity = AccessIdentity::new("user@example.com", Utc::now())
            .with_name("Test User")
            .with_provider("github");

        assert_eq!(identity.email, "user@example.com");
        assert_eq!(identity.name, Some("Test User".to_string()));
        assert_eq!(identity.identity_provider, Some("github".to_string()));
    }

    #[test]
    fn test_auth_context_serialize_local() {
        let ctx = AuthContext::Local;
        let json = serde_json::to_string(&ctx).unwrap();
        assert!(json.contains("\"source\":\"local\""));
    }

    #[test]
    fn test_auth_context_serialize_authenticated() {
        let identity = AccessIdentity::new("user@example.com", Utc::now());
        let ctx = AuthContext::Authenticated { identity };
        let json = serde_json::to_string(&ctx).unwrap();
        assert!(json.contains("\"source\":\"authenticated\""));
        assert!(json.contains("user@example.com"));
    }
}
```

**Step 2: Update mod.rs**

Update `vibes-core/src/auth/mod.rs`:

```rust
//! Authentication module for Cloudflare Access JWT validation

mod config;
mod context;
mod error;

pub use config::AccessConfig;
pub use context::{AccessIdentity, AuthContext};
pub use error::AuthError;
```

**Step 3: Update lib.rs exports**

Update the pub use in `vibes-core/src/lib.rs`:

```rust
pub use auth::{AccessConfig, AccessIdentity, AuthContext, AuthError};
```

**Step 4: Run tests**

Run: `cargo test -p vibes-core auth`
Expected: All tests pass

**Step 5: Commit**

```bash
git add vibes-core/src/auth/context.rs
git add vibes-core/src/auth/mod.rs
git add vibes-core/src/lib.rs
git commit -m "feat(auth): add AuthContext and AccessIdentity types"
```

---

## Task 5: Create JwtValidator with JWKS Caching

**Files:**
- Create: `vibes-core/src/auth/validator.rs`
- Modify: `vibes-core/src/auth/mod.rs`
- Modify: `vibes-core/src/lib.rs`

**Step 1: Write JwtValidator**

Create `vibes-core/src/auth/validator.rs`:

```rust
//! JWT validation with JWKS caching

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::{AccessConfig, AccessIdentity, AuthError};

/// JWKS cache TTL (1 hour)
const JWKS_CACHE_TTL: Duration = Duration::from_secs(3600);

/// JWT validator with JWKS caching
pub struct JwtValidator {
    config: AccessConfig,
    jwks_cache: Arc<RwLock<JwksCache>>,
    http_client: reqwest::Client,
}

struct JwksCache {
    keys: HashMap<String, DecodingKey>,
    fetched_at: Option<Instant>,
}

impl JwksCache {
    fn new() -> Self {
        Self {
            keys: HashMap::new(),
            fetched_at: None,
        }
    }

    fn is_expired(&self) -> bool {
        match self.fetched_at {
            Some(fetched_at) => fetched_at.elapsed() > JWKS_CACHE_TTL,
            None => true,
        }
    }
}

/// JWKS response from Cloudflare
#[derive(Debug, Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}

/// Individual JWK
#[derive(Debug, Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    n: String,
    e: String,
}

/// JWT claims from Cloudflare Access
#[derive(Debug, Serialize, Deserialize)]
struct AccessClaims {
    aud: Vec<String>,
    email: String,
    exp: i64,
    iat: i64,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    identity_nonce: Option<String>,
    #[serde(rename = "custom")]
    #[serde(default)]
    custom: Option<CustomClaims>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct CustomClaims {
    #[serde(default)]
    identity_provider: Option<String>,
}

impl JwtValidator {
    /// Create a new JwtValidator with the given configuration
    pub fn new(config: AccessConfig) -> Self {
        Self {
            config,
            jwks_cache: Arc::new(RwLock::new(JwksCache::new())),
            http_client: reqwest::Client::new(),
        }
    }

    /// Validate a JWT token and return the identity
    pub async fn validate(&self, token: &str) -> Result<AccessIdentity, AuthError> {
        // Decode header to get kid
        let header = decode_header(token)?;
        let kid = header.kid.ok_or_else(|| {
            AuthError::InvalidFormat("missing kid in token header".to_string())
        })?;

        // Get the decoding key
        let key = self.get_key(&kid).await?;

        // Set up validation
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.config.aud]);
        validation.leeway = self.config.clock_skew_seconds;

        // Decode and validate
        let token_data = decode::<AccessClaims>(token, &key, &validation)?;
        let claims = token_data.claims;

        // Build identity
        let expires_at = DateTime::from_timestamp(claims.exp, 0)
            .unwrap_or_else(Utc::now);

        let mut identity = AccessIdentity::new(claims.email, expires_at);

        if let Some(name) = claims.name {
            identity = identity.with_name(name);
        }

        if let Some(custom) = claims.custom {
            if let Some(provider) = custom.identity_provider {
                identity = identity.with_provider(provider);
            }
        }

        Ok(identity)
    }

    /// Get a decoding key by kid, fetching JWKS if needed
    async fn get_key(&self, kid: &str) -> Result<DecodingKey, AuthError> {
        // First, try to get from cache
        {
            let cache = self.jwks_cache.read().await;
            if !cache.is_expired() {
                if let Some(key) = cache.keys.get(kid) {
                    return Ok(key.clone());
                }
            }
        }

        // Cache miss or expired, refresh
        self.refresh_jwks().await?;

        // Try again
        let cache = self.jwks_cache.read().await;
        cache
            .keys
            .get(kid)
            .cloned()
            .ok_or_else(|| AuthError::UnknownKeyId(kid.to_string()))
    }

    /// Force refresh the JWKS cache
    pub async fn refresh_jwks(&self) -> Result<(), AuthError> {
        let url = self.config.jwks_url();

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AuthError::JwksFetchError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthError::JwksFetchError(format!(
                "HTTP {}: {}",
                response.status(),
                url
            )));
        }

        let jwks: JwksResponse = response
            .json()
            .await
            .map_err(|e| AuthError::JwksFetchError(e.to_string()))?;

        let mut keys = HashMap::new();
        for jwk in jwks.keys {
            if jwk.kty == "RSA" {
                let key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
                    .map_err(|e| AuthError::JwksFetchError(e.to_string()))?;
                keys.insert(jwk.kid, key);
            }
        }

        let mut cache = self.jwks_cache.write().await;
        cache.keys = keys;
        cache.fetched_at = Some(Instant::now());

        Ok(())
    }

    /// Get the configuration
    pub fn config(&self) -> &AccessConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwks_cache_new_is_expired() {
        let cache = JwksCache::new();
        assert!(cache.is_expired());
    }

    #[test]
    fn test_jwks_cache_fresh_not_expired() {
        let mut cache = JwksCache::new();
        cache.fetched_at = Some(Instant::now());
        assert!(!cache.is_expired());
    }

    #[test]
    fn test_validator_new() {
        let config = AccessConfig::new("team", "aud");
        let validator = JwtValidator::new(config);
        assert_eq!(validator.config().team, "team");
    }
}
```

**Step 2: Update mod.rs**

Update `vibes-core/src/auth/mod.rs`:

```rust
//! Authentication module for Cloudflare Access JWT validation

mod config;
mod context;
mod error;
mod validator;

pub use config::AccessConfig;
pub use context::{AccessIdentity, AuthContext};
pub use error::AuthError;
pub use validator::JwtValidator;
```

**Step 3: Update lib.rs exports**

Update the pub use in `vibes-core/src/lib.rs`:

```rust
pub use auth::{AccessConfig, AccessIdentity, AuthContext, AuthError, JwtValidator};
```

**Step 4: Run tests**

Run: `cargo test -p vibes-core auth`
Expected: All tests pass

**Step 5: Commit**

```bash
git add vibes-core/src/auth/validator.rs
git add vibes-core/src/auth/mod.rs
git add vibes-core/src/lib.rs
git commit -m "feat(auth): add JwtValidator with JWKS caching"
```

---

## Task 6: Create AuthMiddleware for axum

**Files:**
- Create: `vibes-server/src/middleware/mod.rs`
- Create: `vibes-server/src/middleware/auth.rs`
- Modify: `vibes-server/src/lib.rs`

**Step 1: Create middleware directory**

```bash
mkdir -p vibes-server/src/middleware
```

**Step 2: Write AuthMiddleware**

Create `vibes-server/src/middleware/auth.rs`:

```rust
//! Authentication middleware for axum

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    body::Body,
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
    auth_layer: axum::Extension<AuthLayer>,
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
```

**Step 3: Create mod.rs**

Create `vibes-server/src/middleware/mod.rs`:

```rust
//! Middleware for the vibes server

mod auth;

pub use auth::{auth_middleware, AuthLayer};
```

**Step 4: Update lib.rs**

Add to `vibes-server/src/lib.rs` after other module declarations:

```rust
pub mod middleware;
```

And update the pub use section to include:

```rust
pub use middleware::{auth_middleware, AuthLayer};
```

**Step 5: Run tests**

Run: `cargo test -p vibes-server middleware`
Expected: Tests pass

**Step 6: Commit**

```bash
git add vibes-server/src/middleware/
git add vibes-server/src/lib.rs
git commit -m "feat(auth): add AuthMiddleware for axum"
```

---

## Task 7: Add Auth Status API Endpoint

**Files:**
- Modify: `vibes-server/src/http/api.rs`
- Modify: `vibes-server/src/http/mod.rs`

**Step 1: Add auth status endpoint**

Add to `vibes-server/src/http/api.rs`:

```rust
use vibes_core::AuthContext;

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
    pub email: String,
    pub name: Option<String>,
    pub identity_provider: Option<String>,
}

/// GET /api/auth/status - Get current auth status
pub async fn get_auth_status(
    axum::Extension(auth_context): axum::Extension<AuthContext>,
) -> Json<AuthStatusResponse> {
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
```

**Step 2: Add route**

Update `vibes-server/src/http/mod.rs` to add the route:

```rust
.route("/api/auth/status", get(api::get_auth_status))
```

**Step 3: Run tests**

Run: `cargo test -p vibes-server`
Expected: Tests pass

**Step 4: Commit**

```bash
git add vibes-server/src/http/api.rs
git add vibes-server/src/http/mod.rs
git commit -m "feat(auth): add GET /api/auth/status endpoint"
```

---

## Task 8: Add vibes auth CLI Commands

**Files:**
- Create: `vibes-cli/src/commands/auth.rs`
- Modify: `vibes-cli/src/commands/mod.rs`
- Modify: `vibes-cli/src/main.rs`

**Step 1: Create auth command**

Create `vibes-cli/src/commands/auth.rs`:

```rust
//! Auth subcommands for vibes CLI

use clap::{Args, Subcommand};
use vibes_core::AccessConfig;

use crate::config::VibesConfig;

#[derive(Debug, Args)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommand,
}

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// Show current auth configuration and status
    Status,
    /// Test auth configuration by fetching JWKS
    Test,
}

pub async fn run(args: AuthArgs) -> anyhow::Result<()> {
    match args.command {
        AuthCommand::Status => status().await,
        AuthCommand::Test => test().await,
    }
}

async fn status() -> anyhow::Result<()> {
    let config = VibesConfig::load()?;

    println!("Auth Configuration:");
    println!("  Enabled: {}", config.auth.enabled);

    if config.auth.enabled {
        println!("  Team: {}", config.auth.team);
        println!("  AUD: {}", if config.auth.aud.is_empty() { "(not set)" } else { &config.auth.aud });
        println!("  Bypass localhost: {}", config.auth.bypass_localhost);
        println!("  Clock skew: {}s", config.auth.clock_skew_seconds);
        println!();

        if config.auth.is_valid() {
            println!("Status: Ready");
        } else {
            println!("Status: Invalid configuration (missing team or aud)");
        }
    } else {
        println!("Status: Disabled");
    }

    Ok(())
}

async fn test() -> anyhow::Result<()> {
    let config = VibesConfig::load()?;

    if !config.auth.enabled {
        println!("Auth is disabled. Enable it in config to test.");
        return Ok(());
    }

    if !config.auth.is_valid() {
        anyhow::bail!("Auth configuration is invalid (missing team or aud)");
    }

    println!("Testing auth configuration...");
    println!("Fetching JWKS from: {}", config.auth.jwks_url());

    let validator = vibes_core::JwtValidator::new(config.auth);
    validator.refresh_jwks().await?;

    println!("Success! JWKS fetched and cached.");

    Ok(())
}
```

**Step 2: Export from commands/mod.rs**

Add to `vibes-cli/src/commands/mod.rs`:

```rust
pub mod auth;
```

**Step 3: Add to main.rs CLI**

Add the auth subcommand to the CLI enum in `vibes-cli/src/main.rs`:

```rust
/// Auth management
Auth(commands::auth::AuthArgs),
```

And handle it in the match:

```rust
Commands::Auth(args) => commands::auth::run(args).await?,
```

**Step 4: Update VibesConfig to include auth**

Update `vibes-cli/src/config.rs` to include auth config:

```rust
use vibes_core::AccessConfig;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VibesConfig {
    // ... existing fields ...

    #[serde(default)]
    pub auth: AccessConfig,
}
```

**Step 5: Run tests**

Run: `cargo build -p vibes-cli`
Expected: Builds successfully

**Step 6: Commit**

```bash
git add vibes-cli/src/commands/auth.rs
git add vibes-cli/src/commands/mod.rs
git add vibes-cli/src/main.rs
git add vibes-cli/src/config.rs
git commit -m "feat(cli): add vibes auth status and test commands"
```

---

## Task 9: Integrate AuthMiddleware into Server Router

**Files:**
- Modify: `vibes-server/src/http/mod.rs`
- Modify: `vibes-server/src/state.rs`
- Modify: `vibes-server/src/lib.rs`

**Step 1: Add AuthLayer to AppState**

Update `vibes-server/src/state.rs`:

```rust
use crate::middleware::AuthLayer;
use vibes_core::AccessConfig;

pub struct AppState {
    // ... existing fields ...
    pub auth_layer: AuthLayer,
}

impl AppState {
    pub fn new() -> Self {
        // ... existing code ...
        let auth_layer = AuthLayer::disabled();

        Self {
            // ... existing fields ...
            auth_layer,
        }
    }

    pub fn with_auth(mut self, config: AccessConfig) -> Self {
        self.auth_layer = AuthLayer::new(config);
        self
    }
}
```

**Step 2: Apply middleware to router**

Update `vibes-server/src/http/mod.rs`:

```rust
use axum::middleware;
use crate::middleware::{auth_middleware, AuthLayer};

pub fn create_router(state: Arc<AppState>) -> Router {
    let auth_layer = state.auth_layer.clone();

    Router::new()
        // ... existing routes ...
        .layer(middleware::from_fn_with_state(
            axum::Extension(auth_layer),
            auth_middleware,
        ))
        .with_state(state)
}
```

**Step 3: Update ServerConfig**

Update `vibes-server/src/lib.rs` to accept auth config:

```rust
use vibes_core::AccessConfig;

pub struct ServerConfig {
    // ... existing fields ...
    pub auth: AccessConfig,
}
```

**Step 4: Run tests**

Run: `cargo test -p vibes-server`
Expected: Tests pass

**Step 5: Commit**

```bash
git add vibes-server/src/http/mod.rs
git add vibes-server/src/state.rs
git add vibes-server/src/lib.rs
git commit -m "feat(server): integrate AuthMiddleware into router"
```

---

## Task 10: Send auth_context on WebSocket Connection

**Files:**
- Modify: `vibes-server/src/ws/connection.rs`
- Modify: `vibes-server/src/ws/protocol.rs`

**Step 1: Add auth_context message type**

Update `vibes-server/src/ws/protocol.rs`:

```rust
use vibes_core::AuthContext;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    // ... existing variants ...

    /// Auth context sent on connection
    AuthContext(AuthContext),
}
```

**Step 2: Send auth_context on connect**

Update WebSocket connection handler in `vibes-server/src/ws/connection.rs` to send auth context immediately after connection:

```rust
// After connection established
let auth_msg = ServerMessage::AuthContext(auth_context.clone());
if let Ok(json) = serde_json::to_string(&auth_msg) {
    let _ = sender.send(Message::Text(json)).await;
}
```

**Step 3: Run tests**

Run: `cargo test -p vibes-server`
Expected: Tests pass

**Step 4: Commit**

```bash
git add vibes-server/src/ws/connection.rs
git add vibes-server/src/ws/protocol.rs
git commit -m "feat(ws): send auth_context on WebSocket connection"
```

---

## Task 11: Add useAuth Hook to Web UI

**Files:**
- Create: `web-ui/src/hooks/useAuth.ts`
- Modify: `web-ui/src/hooks/index.ts`

**Step 1: Create useAuth hook**

Create `web-ui/src/hooks/useAuth.ts`:

```typescript
import { useState, useEffect } from 'react';
import { useWebSocket } from './useWebSocket';

export interface AccessIdentity {
  email: string;
  name?: string;
  identity_provider?: string;
}

export interface AuthState {
  source: 'local' | 'tunnel' | 'anonymous';
  identity: AccessIdentity | null;
  isAuthenticated: boolean;
  isLocal: boolean;
}

const initialState: AuthState = {
  source: 'local',
  identity: null,
  isAuthenticated: false,
  isLocal: true,
};

export function useAuth(): AuthState {
  const [authState, setAuthState] = useState<AuthState>(initialState);
  const { lastMessage } = useWebSocket();

  useEffect(() => {
    if (lastMessage?.type === 'auth_context') {
      const { source, identity } = lastMessage;
      setAuthState({
        source,
        identity: identity || null,
        isAuthenticated: source === 'authenticated',
        isLocal: source === 'local',
      });
    }
  }, [lastMessage]);

  return authState;
}
```

**Step 2: Export from hooks index**

Add to `web-ui/src/hooks/index.ts`:

```typescript
export { useAuth } from './useAuth';
export type { AuthState, AccessIdentity } from './useAuth';
```

**Step 3: Commit**

```bash
git add web-ui/src/hooks/useAuth.ts
git add web-ui/src/hooks/index.ts
git commit -m "feat(ui): add useAuth hook for auth context"
```

---

## Task 12: Display Identity in Header

**Files:**
- Modify: `web-ui/src/components/Header.tsx`

**Step 1: Add identity display**

Update `web-ui/src/components/Header.tsx`:

```tsx
import { useAuth } from '../hooks/useAuth';

export function Header() {
  const { identity, isLocal, isAuthenticated } = useAuth();

  return (
    <header className="header">
      <Logo />
      <div className="header-right">
        <TunnelBadge />
        {isLocal && (
          <span className="badge badge-local">Local</span>
        )}
        {isAuthenticated && identity && (
          <div className="identity">
            <span className="identity-email">{identity.email}</span>
            {identity.identity_provider && (
              <span className="identity-provider">
                via {identity.identity_provider}
              </span>
            )}
          </div>
        )}
      </div>
    </header>
  );
}
```

**Step 2: Add styles**

Add to header styles:

```css
.identity {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.875rem;
}

.identity-email {
  color: var(--text-primary);
}

.identity-provider {
  color: var(--text-secondary);
  font-size: 0.75rem;
}

.badge-local {
  background: var(--color-gray-200);
  color: var(--color-gray-700);
}
```

**Step 3: Commit**

```bash
git add web-ui/src/components/Header.tsx
git commit -m "feat(ui): display identity in header when authenticated"
```

---

## Task 13: Update Documentation

**Files:**
- Modify: `docs/PROGRESS.md`

**Step 1: Mark tasks complete**

Update PROGRESS.md to mark 2.2 items as in progress or complete as implemented.

**Step 2: Commit**

```bash
git add docs/PROGRESS.md
git commit -m "docs: update progress for milestone 2.2"
```

---

## Task 14: Final Integration Test

**Step 1: Build everything**

Run: `cargo build`
Expected: Builds successfully

**Step 2: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 3: Run clippy**

Run: `cargo clippy`
Expected: No warnings

**Step 4: Run formatter**

Run: `cargo fmt -- --check`
Expected: No formatting issues

**Step 5: Commit any fixes**

```bash
git add .
git commit -m "test: fix any issues from final integration"
```

---

## Summary

This implementation plan covers:

1. **Core types** (Tasks 1-5): AuthError, AccessConfig, AuthContext, AccessIdentity, JwtValidator
2. **Server integration** (Tasks 6-10): AuthMiddleware, auth status endpoint, router integration, WebSocket auth
3. **CLI** (Task 8): `vibes auth status` and `vibes auth test` commands
4. **Web UI** (Tasks 11-12): useAuth hook and Header identity display
5. **Documentation** (Task 13): Progress tracking updates

Tasks follow a test-focused workflow: write or update tests around new behavior, implement the changes, then ensure tests pass before committing.
