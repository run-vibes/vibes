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
