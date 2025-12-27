# Milestone 2.2: Cloudflare Access Authentication - Design Document

> Secure remote access to vibes with Cloudflare Access JWT validation.

## Overview

This milestone adds authentication to vibes when accessed through Cloudflare Tunnel. Local access (localhost) remains unauthenticated for frictionless development, while tunnel access requires valid Cloudflare Access credentials.

### Key Decisions

| Decision | Choice | Notes |
|----------|--------|-------|
| Auth scope | Tunnel only | Localhost bypasses auth for dev convenience |
| Configuration | Auto-detect from tunnel | Falls back to manual config if detection fails |
| Identity handling | Display in UI | Show email in header, no persistence |
| Login flow | Rely on Cloudflare | CF Access redirects before requests reach vibes |
| JWT library | jsonwebtoken | Well-maintained Rust JWT library |
| JWKS caching | 1 hour TTL | Refresh on unknown `kid` for key rotation |

---

## Architecture

### ADR-011: Auth Middleware Architecture

**Status:** Decided

**Context:** vibes needs to authenticate requests coming through Cloudflare Tunnel while allowing unauthenticated local access. The authentication layer must integrate cleanly with the existing axum server.

**Decision:** Implement auth as an axum middleware layer that checks request source and validates Cloudflare Access JWTs.

**Architecture:**

```
┌─────────────────────────────────────────────────────────────────────┐
│                         vibes server                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │                    AuthMiddleware                               │ │
│  │  ┌─────────────────┐    ┌─────────────────────────────────────┐│ │
│  │  │ Request arrives │───▶│ Is source localhost?                ││ │
│  │  └─────────────────┘    └───────────────┬─────────────────────┘│ │
│  │                                         │                       │ │
│  │                    ┌────────────────────┴────────────────────┐ │ │
│  │                    │ YES                │ NO                  │ │ │
│  │                    ▼                    ▼                     │ │ │
│  │         ┌──────────────────┐  ┌──────────────────────────┐   │ │ │
│  │         │ Skip auth        │  │ Validate CF Access JWT   │   │ │ │
│  │         │ (pass through)   │  │ from header/cookie       │   │ │ │
│  │         └──────────────────┘  └────────────┬─────────────┘   │ │ │
│  │                    │                       │                  │ │ │
│  │                    │          ┌────────────┴────────────┐    │ │ │
│  │                    │          │ Valid?                  │    │ │ │
│  │                    │          ▼            ▼            │    │ │ │
│  │                    │      ┌───────┐   ┌────────────┐    │    │ │ │
│  │                    │      │  YES  │   │ NO → 401   │    │    │ │ │
│  │                    │      └───┬───┘   └────────────┘    │    │ │ │
│  │                    │          │                         │    │ │ │
│  │                    └──────────┴─────────────────────────┘    │ │ │
│  │                               │                               │ │
│  │                               ▼                               │ │
│  │                    ┌──────────────────────┐                  │ │
│  │                    │ Attach AuthContext   │                  │ │
│  │                    │ (identity, source)   │                  │ │
│  │                    └──────────────────────┘                  │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                               │                                      │
│                               ▼                                      │
│                    ┌──────────────────────┐                         │
│                    │  Route handlers       │                         │
│                    │  (can read identity)  │                         │
│                    └──────────────────────┘                         │
└─────────────────────────────────────────────────────────────────────┘
```

**Key components:**
- **AuthMiddleware** - axum layer that intercepts all requests
- **JwtValidator** - Fetches JWKS from Cloudflare, validates tokens
- **AuthContext** - Request extension containing identity info (or Local for localhost)

**Rationale:**
- Middleware pattern cleanly separates auth from business logic
- Request extensions allow handlers to optionally access identity
- Localhost bypass enables frictionless local development
- Cloudflare handles login redirects, keeping vibes simple

**Alternatives considered:**
- Auth in every handler: Repetitive, error-prone
- Separate auth service: Overkill for single-user tool
- Always require auth: Poor DX for local development

---

### ADR-012: JWT Validation Strategy

**Status:** Decided

**Context:** Cloudflare Access sends JWTs that must be validated against Cloudflare's public keys. Keys rotate every 6 weeks, and we need to handle this gracefully.

**Decision:** Implement JWT validation with JWKS caching and automatic refresh on unknown key IDs.

**Validation Flow:**

```
┌─────────────────────────────────────────────────────────────────────┐
│                       JwtValidator                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │ 1. Extract JWT                                                   ││
│  │    - Check Cf-Access-Jwt-Assertion header                       ││
│  │    - Fallback to CF_Authorization cookie                        ││
│  └─────────────────────────────────────────────────────────────────┘│
│                               │                                      │
│                               ▼                                      │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │ 2. Decode JWT header (without verification)                      ││
│  │    - Extract `kid` (key ID)                                      ││
│  │    - Extract `alg` (should be RS256)                            ││
│  └─────────────────────────────────────────────────────────────────┘│
│                               │                                      │
│                               ▼                                      │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │ 3. Fetch JWKS (cached)                                          ││
│  │    GET https://<team>.cloudflareaccess.com/cdn-cgi/access/certs ││
│  │    - Cache for 1 hour                                           ││
│  │    - Refresh on cache miss or `kid` not found                   ││
│  └─────────────────────────────────────────────────────────────────┘│
│                               │                                      │
│                               ▼                                      │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │ 4. Find matching key by `kid`                                    ││
│  │    - If not found after refresh → reject                        ││
│  └─────────────────────────────────────────────────────────────────┘│
│                               │                                      │
│                               ▼                                      │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │ 5. Verify signature + claims                                     ││
│  │    - Signature valid with public key                            ││
│  │    - `aud` matches configured Application AUD                   ││
│  │    - `exp` not expired (with 60s leeway for clock skew)         ││
│  │    - `iat` not in future                                        ││
│  └─────────────────────────────────────────────────────────────────┘│
│                               │                                      │
│                               ▼                                      │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │ 6. Extract identity                                              ││
│  │    - email, name from claims                                    ││
│  │    - identity_provider (GitHub, Google, etc.)                   ││
│  └─────────────────────────────────────────────────────────────────┘│
│                               │                                      │
│                               ▼                                      │
│                        AccessIdentity                                │
└─────────────────────────────────────────────────────────────────────┘
```

**Caching strategy:**
- JWKS cached in-memory for 1 hour
- If a JWT arrives with an unknown `kid`, force refresh (handles key rotation)
- Use `tokio::sync::RwLock` for thread-safe cache access

**Rationale:**
- Caching reduces latency and Cloudflare API load
- Automatic refresh on unknown `kid` handles key rotation seamlessly
- Clock skew leeway prevents false rejections due to time sync issues

**References:**
- [Cloudflare Access JWT Validation](https://developers.cloudflare.com/cloudflare-one/access-controls/applications/http-apps/authorization-cookie/validating-json/)
- [jsonwebtoken crate](https://docs.rs/jsonwebtoken/latest/jsonwebtoken/)

---

## Configuration

### Config Schema

```toml
# ~/.config/vibes/config.toml

[auth]
enabled = true                          # Enable auth (auto-enabled when tunnel is named)

# Auto-detected from tunnel config, or set manually:
team = "mycompany"                       # → mycompany.cloudflareaccess.com
aud = "abc123..."                        # Application AUD tag

# Optional overrides
bypass_localhost = true                  # Default: true (skip auth for 127.0.0.1)
clock_skew_seconds = 60                  # Leeway for exp/iat validation
```

### Auto-Detection Flow

1. When `vibes serve --tunnel` starts with a named tunnel
2. Read tunnel hostname from config (e.g., `vibes.example.com`)
3. Derive team name from cloudflared credentials
4. Fetch AUD from Cloudflare API: `GET /zones/:zone/access/apps` filtered by hostname
5. Cache in config if successful, warn if detection fails

### CLI Commands

```bash
vibes auth status              # Show current auth configuration and state
vibes auth setup               # Interactive setup wizard (if auto-detect fails)
vibes auth test                # Validate configuration by fetching JWKS
```

---

## Types and Interfaces

### Core Types

```rust
/// Authentication result for a request
#[derive(Debug, Clone)]
pub enum AuthContext {
    /// Request from localhost, auth bypassed
    Local,
    /// Authenticated via Cloudflare Access
    Authenticated(AccessIdentity),
    /// No valid auth (should have been rejected by middleware)
    Anonymous,
}

/// Identity from Cloudflare Access JWT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessIdentity {
    pub email: String,
    pub name: Option<String>,
    pub identity_provider: Option<String>,
    pub expires_at: DateTime<Utc>,
}

/// Configuration for Cloudflare Access auth
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccessConfig {
    pub enabled: bool,
    pub team: String,
    pub aud: String,
    #[serde(default = "default_bypass_localhost")]
    pub bypass_localhost: bool,
    #[serde(default = "default_clock_skew")]
    pub clock_skew_seconds: u64,
}

fn default_bypass_localhost() -> bool { true }
fn default_clock_skew() -> u64 { 60 }
```

### JWT Validator

```rust
/// JWT validator with JWKS caching
pub struct JwtValidator {
    config: AccessConfig,
    jwks_cache: Arc<RwLock<JwksCache>>,
    http_client: reqwest::Client,
}

struct JwksCache {
    keys: HashMap<String, DecodingKey>,  // kid -> key
    fetched_at: Instant,
    ttl: Duration,
}

impl JwtValidator {
    pub fn new(config: AccessConfig) -> Self;

    /// Validate a JWT and return the identity
    pub async fn validate(&self, token: &str) -> Result<AccessIdentity, AuthError>;

    /// Force refresh the JWKS cache
    pub async fn refresh_jwks(&self) -> Result<(), AuthError>;
}
```

### Auth Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("No authentication token provided")]
    MissingToken,

    #[error("Invalid token format")]
    InvalidFormat,

    #[error("Token signature verification failed")]
    InvalidSignature,

    #[error("Token has expired")]
    Expired,

    #[error("Invalid audience claim")]
    InvalidAudience,

    #[error("Unknown key ID: {0}")]
    UnknownKeyId(String),

    #[error("Failed to fetch JWKS: {0}")]
    JwksFetchError(String),
}
```

---

## Crate Changes

```
vibes/
├── vibes-core/
│   └── src/
│       ├── auth/                    # NEW MODULE
│       │   ├── mod.rs               # Module exports
│       │   ├── config.rs            # AccessConfig
│       │   ├── context.rs           # AuthContext, AccessIdentity
│       │   ├── validator.rs         # JwtValidator, JWKS caching
│       │   └── error.rs             # AuthError
│       └── lib.rs                   # Export auth module
│
├── vibes-server/
│   └── src/
│       ├── middleware/              # NEW MODULE
│       │   ├── mod.rs
│       │   └── auth.rs              # AuthMiddleware layer
│       ├── http/
│       │   └── mod.rs               # Apply auth middleware to router
│       └── state.rs                 # Add JwtValidator to AppState
│
├── vibes-cli/
│   └── src/
│       └── commands/
│           └── auth.rs              # NEW: auth subcommands
│
└── web-ui/
    └── src/
        ├── hooks/
        │   └── useAuth.ts           # NEW: auth context hook
        └── components/
            └── Header.tsx           # Add identity display
```

---

## HTTP API

### Existing Endpoints (with auth)

All existing endpoints now include auth context. Handlers can optionally access identity:

```rust
pub async fn health(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthContext>,  // NEW
) -> Json<HealthResponse> {
    // auth.identity() returns Option<&AccessIdentity>
}
```

### New Endpoint

```
GET /api/auth/status    # Current auth state
```

**Response:**
```json
{
  "authenticated": true,
  "source": "tunnel",
  "identity": {
    "email": "user@example.com",
    "name": "User Name",
    "identity_provider": "github"
  }
}
```

---

## WebSocket Protocol

### New Message Types

```typescript
// Server → Client on connection
{
  "type": "auth_context",
  "source": "tunnel",           // "local" | "tunnel"
  "identity": {                 // null if source is "local"
    "email": "user@example.com",
    "name": "User Name",
    "identity_provider": "github"
  }
}
```

---

## Web UI Changes

### Header Component

```tsx
// Header.tsx - show identity when authenticated
function Header() {
  const { identity, source } = useAuth();

  return (
    <header>
      <Logo />
      <TunnelBadge />
      {identity && (
        <div className="identity">
          <span>{identity.email}</span>
          <IdentityProviderIcon provider={identity.identity_provider} />
        </div>
      )}
      {source === 'local' && (
        <span className="local-badge">Local</span>
      )}
    </header>
  );
}
```

### Auth Hook

```typescript
// hooks/useAuth.ts
interface AuthState {
  source: 'local' | 'tunnel';
  identity: AccessIdentity | null;
}

function useAuth(): AuthState {
  // Populated from auth_context WebSocket message
}
```

---

## Dependencies

### vibes-core/Cargo.toml

```toml
[dependencies]
jsonwebtoken = "9"              # JWT validation
reqwest = { version = "0.12", features = ["json"] }  # Already present, for JWKS fetch
```

### web-ui/package.json

No new dependencies required.

---

## Testing Strategy

### Unit Tests

| Component | Test Coverage |
|-----------|---------------|
| AccessConfig | Parse valid/invalid TOML, defaults |
| JwtValidator | Valid token, expired, wrong aud, unknown kid |
| AuthMiddleware | Localhost bypass, valid auth, missing auth, invalid auth |
| JWKS cache | TTL expiry, force refresh, concurrent access |

### Integration Tests

| Test | Description |
|------|-------------|
| Auth middleware with mock validator | Full request flow |
| JWKS fetch from mock server | Network error handling |
| WebSocket auth context | Message sent on connect |

### Manual Testing

- [ ] Local access works without any auth config
- [ ] Tunnel access rejected without Cloudflare Access
- [ ] Tunnel access works with valid CF Access session
- [ ] Identity displayed correctly in Web UI
- [ ] Key rotation handled gracefully
- [ ] Auto-detection works with named tunnel

---

## Deliverables

### New Files

| File | Description |
|------|-------------|
| vibes-core/src/auth/mod.rs | Module exports |
| vibes-core/src/auth/config.rs | AccessConfig type |
| vibes-core/src/auth/context.rs | AuthContext, AccessIdentity |
| vibes-core/src/auth/validator.rs | JwtValidator with JWKS caching |
| vibes-core/src/auth/error.rs | AuthError type |
| vibes-server/src/middleware/mod.rs | Middleware module |
| vibes-server/src/middleware/auth.rs | AuthMiddleware layer |
| vibes-cli/src/commands/auth.rs | Auth CLI commands |
| web-ui/src/hooks/useAuth.ts | Auth context hook |

### Modified Files

| File | Changes |
|------|---------|
| vibes-core/src/lib.rs | Export auth module |
| vibes-core/src/config/mod.rs | Add AuthConfig parsing |
| vibes-server/src/lib.rs | Export middleware |
| vibes-server/src/http/mod.rs | Apply auth middleware |
| vibes-server/src/state.rs | Add JwtValidator to AppState |
| vibes-server/src/ws/connection.rs | Send auth_context on connect |
| vibes-cli/src/main.rs | Add auth subcommand |
| web-ui/src/components/Header.tsx | Display identity |

---

## Milestone 2.2 Checklist

- [ ] AccessConfig type with TOML parsing
- [ ] JwtValidator with JWKS caching
- [ ] AuthMiddleware layer for axum
- [ ] Localhost bypass logic
- [ ] AuthContext request extension
- [ ] GET /api/auth/status endpoint
- [ ] WebSocket auth_context message
- [ ] vibes auth status command
- [ ] vibes auth setup wizard
- [ ] Auto-detect team/aud from tunnel config
- [ ] Web UI identity display
- [ ] Unit tests for validator
- [ ] Integration tests for middleware
- [ ] Documentation updates
