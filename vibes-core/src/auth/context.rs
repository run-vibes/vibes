//! Authentication context types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Authentication context for a request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
