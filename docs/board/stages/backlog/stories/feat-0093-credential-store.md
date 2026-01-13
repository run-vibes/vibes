---
id: FEAT0093
title: CredentialStore for API key management
type: feat
status: pending
priority: high
epics: [models]
depends: [FEAT0090]
estimate: 3h
created: 2026-01-13
milestone: 37-models-registry-auth
---

# CredentialStore for API key management

## Summary

Implement secure credential storage using the system keyring with environment variable fallback.

## Requirements

- Store API keys in system keyring (macOS Keychain, Linux Secret Service, Windows Credential Manager)
- Fall back to environment variables (ANTHROPIC_API_KEY, OPENAI_API_KEY, etc.)
- CRUD operations for credentials
- List configured providers
- Secure handling (no logging of keys)

## Implementation

```rust
pub struct CredentialStore {
    keyring: Option<SystemKeyring>,
    env_fallback: bool,
    service_name: String,
}

impl CredentialStore {
    pub fn new(service_name: &str) -> Self;
    pub fn with_env_fallback(self) -> Self;

    pub fn get(&self, provider: &str) -> Result<ApiKey>;
    pub fn set(&self, provider: &str, key: &str) -> Result<()>;
    pub fn delete(&self, provider: &str) -> Result<()>;
    pub fn has(&self, provider: &str) -> bool;
    pub fn list_providers(&self) -> Vec<String>;
}

pub struct ApiKey(SecretString);
```

## Environment Variables

| Provider | Env Var |
|----------|---------|
| Anthropic | ANTHROPIC_API_KEY |
| OpenAI | OPENAI_API_KEY |
| Google | GOOGLE_API_KEY |
| Groq | GROQ_API_KEY |

## Dependencies

- `keyring` crate for system keyring
- `secrecy` crate for secure string handling

## Acceptance Criteria

- [ ] System keyring integration (cross-platform)
- [ ] Environment variable fallback
- [ ] Secure API key type (no Debug, no logging)
- [ ] CRUD operations for credentials
- [ ] Unit tests with mock keyring
