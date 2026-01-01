# Milestone 19: Event CLI - Implementation Plan

> Step-by-step tasks for implementing `vibes event send` CLI command.

## Phase 1: CLI Implementation

### Task 1.1: Add event subcommand structure

**Files:**
- Create: `vibes-cli/src/commands/event.rs`
- Modify: `vibes-cli/src/commands/mod.rs`

Add the subcommand structure with clap:

```rust
// vibes-cli/src/commands/event.rs
use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum EventCommand {
    /// Send an event to the EventLog
    Send(SendArgs),
}

#[derive(Debug, Args)]
pub struct SendArgs {
    /// Event type: hook, session-state, claude
    #[arg(short = 't', long)]
    pub event_type: String,

    /// Session ID for event attribution
    #[arg(short, long)]
    pub session: Option<String>,

    /// Event payload as JSON (reads from stdin if omitted)
    #[arg(short, long)]
    pub data: Option<String>,

    /// Iggy topic name
    #[arg(long, default_value = "events")]
    pub topic: String,
}
```

**Tests:**
- [ ] Command parses with all options
- [ ] Command parses with stdin (no --data)

---

### Task 1.2: Add Iggy configuration to vibes-core

**Files:**
- Modify: `vibes-core/src/config.rs` (or create if needed)

Add Iggy client configuration:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct IggyClientConfig {
    pub host: String,
    pub http_port: u16,
    pub username: String,
    pub password: String,
}

impl Default for IggyClientConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            http_port: 3000,
            username: "iggy".to_string(),
            password: "iggy".to_string(),
        }
    }
}

impl IggyClientConfig {
    /// Load from environment variables, falling back to defaults
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("VIBES_IGGY_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            http_port: std::env::var("VIBES_IGGY_HTTP_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3000),
            username: std::env::var("VIBES_IGGY_USERNAME")
                .unwrap_or_else(|_| "iggy".to_string()),
            password: std::env::var("VIBES_IGGY_PASSWORD")
                .unwrap_or_else(|_| "iggy".to_string()),
        }
    }
}
```

**Tests:**
- [ ] Default config has correct values
- [ ] from_env reads environment variables
- [ ] from_env falls back to defaults

---

### Task 1.3: Implement Iggy HTTP client

**Files:**
- Create: `vibes-cli/src/iggy_client.rs`
- Modify: `vibes-cli/Cargo.toml` (add reqwest if needed)

Simple HTTP client for Iggy:

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct IggyHttpClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

impl IggyHttpClient {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            client: Client::new(),
            base_url: format!("http://{}:{}", host, port),
            token: None,
        }
    }

    /// Login and store token
    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), Error> {
        // POST /users/login
        // Store JWT token
    }

    /// Send messages to a topic
    pub async fn send_message(
        &self,
        stream: &str,
        topic: &str,
        payload: &[u8],
    ) -> Result<(), Error> {
        // POST /streams/{stream}/topics/{topic}/messages
        // Include Authorization header
    }
}
```

**Tests:**
- [ ] Client constructs correct URLs
- [ ] Login parses token from response
- [ ] send_message includes auth header

---

### Task 1.4: Implement token caching

**Files:**
- Modify: `vibes-cli/src/iggy_client.rs`

Cache token to avoid login on every call:

```rust
const TOKEN_CACHE_PATH: &str = ".cache/vibes/iggy-token";

impl IggyHttpClient {
    /// Load cached token or login fresh
    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<(), Error> {
        // Try to load from cache
        if let Some(token) = self.load_cached_token() {
            self.token = Some(token);
            return Ok(());
        }

        // Login and cache
        self.login(username, password).await?;
        self.cache_token()?;
        Ok(())
    }

    fn load_cached_token(&self) -> Option<String> {
        let path = dirs::home_dir()?.join(TOKEN_CACHE_PATH);
        std::fs::read_to_string(path).ok()
    }

    fn cache_token(&self) -> Result<(), Error> {
        let path = dirs::home_dir()
            .ok_or(Error::NoHomeDir)?
            .join(TOKEN_CACHE_PATH);
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(path, self.token.as_ref().unwrap())?;
        Ok(())
    }
}
```

**Tests:**
- [ ] Token loaded from cache when present
- [ ] Token cached after login
- [ ] Expired token triggers re-login

---

### Task 1.5: Implement event send command

**Files:**
- Modify: `vibes-cli/src/commands/event.rs`

Main command implementation:

```rust
pub async fn execute_send(args: SendArgs) -> Result<(), Error> {
    // 1. Read payload from --data or stdin
    let payload = match args.data {
        Some(data) => data,
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };

    // 2. Parse and wrap in VibesEvent
    let event = match args.event_type.as_str() {
        "hook" => {
            let hook: HookEvent = serde_json::from_str(&payload)?;
            VibesEvent::Hook {
                session_id: args.session,
                event: hook,
            }
        }
        "session-state" => {
            let state: StatePayload = serde_json::from_str(&payload)?;
            VibesEvent::SessionStateChanged {
                session_id: args.session.ok_or(Error::SessionRequired)?,
                state: state.state,
            }
        }
        _ => return Err(Error::UnknownEventType),
    };

    // 3. Connect to Iggy and send
    let config = IggyClientConfig::from_env();
    let mut client = IggyHttpClient::new(&config.host, config.http_port);
    client.authenticate(&config.username, &config.password).await?;

    let serialized = serde_json::to_vec(&event)?;
    client.send_message("vibes", &args.topic, &serialized).await?;

    Ok(())
}
```

**Tests:**
- [ ] Parses hook event correctly
- [ ] Parses session-state event correctly
- [ ] Reads from stdin when --data omitted
- [ ] Returns error on invalid JSON

---

## Phase 2: Cleanup

### Task 2.1: Remove HookReceiver

**Files:**
- Delete: `vibes-core/src/hooks/receiver.rs`
- Modify: `vibes-core/src/hooks/mod.rs`
- Modify: `vibes-core/src/lib.rs`

Remove all HookReceiver code:

```rust
// vibes-core/src/hooks/mod.rs
// DELETE these lines:
mod receiver;
pub use receiver::{HookReceiver, HookReceiverConfig};

// vibes-core/src/lib.rs
// DELETE from re-exports:
HookReceiver, HookReceiverConfig,
```

**Tests:**
- [ ] Crate compiles without HookReceiver
- [ ] No dead code warnings

---

### Task 2.2: Update hook scripts

**Files:**
- Modify: `vibes-core/src/hooks/scripts/*.sh` (all hook scripts)

Update scripts to use new CLI:

```bash
#!/bin/bash
# Old: echo "$1" | vibes-hook-send
# New:
vibes event send --type hook --session "$VIBES_SESSION_ID" --data "$1"
```

**Tests:**
- [ ] Hook scripts use correct CLI command
- [ ] VIBES_SESSION_ID passed correctly

---

### Task 2.3: Remove vibes-hook-send binary (if exists)

**Files:**
- Delete: `vibes-cli/src/bin/vibes-hook-send.rs` (if exists)
- Modify: `vibes-cli/Cargo.toml` (remove binary entry if exists)

**Tests:**
- [ ] No references to vibes-hook-send remain

---

## Phase 3: Integration Testing

### Task 3.1: Add integration test

**Files:**
- Create: `vibes-cli/tests/event_send.rs`

```rust
#[tokio::test]
#[ignore] // Requires running Iggy
async fn test_event_send_writes_to_iggy() {
    // 1. Start Iggy (or use existing)
    // 2. Run: vibes event send --type hook --data '...'
    // 3. Poll topic and verify message received
}
```

---

### Task 3.2: Manual verification

1. Start vibes server with Iggy: `vibes serve`
2. Send test event: `vibes event send --type hook --session test --data '{"type":"session_start"}'`
3. Verify in Iggy logs or via consumer

---

## Checklist

### Phase 1: CLI Implementation
- [x] Task 1.1: Add event subcommand structure
- [x] Task 1.2: Add Iggy configuration
- [x] Task 1.3: Implement Iggy HTTP client
- [x] Task 1.4: Implement token caching
- [x] Task 1.5: Implement event send command

### Phase 2: Cleanup
- [x] Task 2.1: Remove HookReceiver
- [x] Task 2.2: Update hook scripts
- [x] Task 2.3: Remove vibes-hook-send binary (N/A - didn't exist)

### Phase 3: Integration
- [x] Task 3.1: Add integration test
- [x] Task 3.2: Manual verification
- [x] `just pre-commit` passes

---

## Exit Criteria

- [x] `vibes event send` works with --data and stdin
- [x] Events appear in Iggy "vibes/events" topic
- [x] HookReceiver code deleted
- [x] Hook scripts updated
- [x] All tests pass
