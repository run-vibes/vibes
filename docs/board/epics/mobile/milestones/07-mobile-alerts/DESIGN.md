# Milestone 2.3: Push Notifications - Design Document

> Get notified on your phone when Claude finishes, fails, or needs permission - even when you're away from the terminal.

## Overview

This milestone adds Web Push notifications to vibes, completing Phase 2 (Remote Access). Users can receive notifications on any device when Claude sessions require attention.

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Delivery mechanism | Web Push API only | Single implementation covers mobile + desktop browsers |
| VAPID keys | Auto-generate on first run | Zero setup friction, stored in vibes config |
| Notification actions | Deep link to Web UI | Universal browser support, leverages existing permission UI |
| Default behavior | All events notify | Opt-out model for remote monitoring use case |

### Notification Events

| Event | Default | Description |
|-------|---------|-------------|
| Permission needed | On | Claude needs approval to proceed |
| Session completed | On | Task finished successfully |
| Session error | On | Something went wrong |

---

## Architecture

### ADR-013: Push Notification Architecture

See [VISION.md](../../../../VISION.md#adr-013-push-notification-architecture) for the full ADR.

### Push Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Push Notification Flow                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────────────────┐ │
│  │   Browser    │     │    vibes     │     │   Push Service           │ │
│  │  (Web UI)    │     │   server     │     │  (FCM/Mozilla/Apple)     │ │
│  └──────┬───────┘     └──────┬───────┘     └────────────┬─────────────┘ │
│         │                    │                          │               │
│         │ 1. Enable notifs   │                          │               │
│         │ ──────────────────>│                          │               │
│         │                    │                          │               │
│         │ 2. Subscribe       │                          │               │
│         │    (VAPID pubkey)  │                          │               │
│         │<───────────────────│                          │               │
│         │                    │                          │               │
│         │ 3. PushSubscription│                          │               │
│         │    {endpoint, keys}│                          │               │
│         │ ──────────────────>│                          │               │
│         │                    │ 4. Store subscription    │               │
│         │                    │    (in memory + file)    │               │
│         │                    │                          │               │
│         │    ═══════════════ Later: Event occurs ═══════════════       │
│         │                    │                          │               │
│         │                    │ 5. POST to endpoint      │               │
│         │                    │    (signed with VAPID)   │               │
│         │                    │ ────────────────────────>│               │
│         │                    │                          │               │
│         │                    │                          │ 6. Push       │
│         │<───────────────────────────────────────────────│               │
│         │                    │                          │               │
│         │ 7. Service Worker  │                          │               │
│         │    shows notif     │                          │               │
│         │                    │                          │               │
│         │ 8. User clicks     │                          │               │
│         │    → Open Web UI   │                          │               │
└─────────────────────────────────────────────────────────────────────────┘
```

### Component Overview

| Component | Location | Responsibility |
|-----------|----------|----------------|
| VapidKeyManager | vibes-core | Generate/load VAPID keypair |
| SubscriptionStore | vibes-core | Store push subscriptions (file-backed) |
| NotificationService | vibes-core | Send push notifications on events |
| Service Worker | web-ui | Receive pushes, display notifications |
| NotificationSettings | web-ui | UI for enabling/configuring notifications |

---

## Configuration

### Config Schema

```toml
# ~/.config/vibes/config.toml

[notifications]
enabled = true                    # Master switch for push notifications

# Events (all default to true)
notify_permission = true          # Claude needs approval
notify_completed = true           # Session finished successfully
notify_error = true               # Session failed

# VAPID keys (auto-generated if missing)
# vapid_private_key = "..."       # Base64-encoded, managed by vibes
# vapid_public_key = "..."        # Base64-encoded, shared with browsers
```

---

## Types and Interfaces

### Core Types

```rust
/// VAPID keypair for Web Push authentication
pub struct VapidKeys {
    pub private_key: Vec<u8>,     // P-256 private key
    pub public_key: Vec<u8>,      // P-256 public key (shared with browsers)
}

/// A browser's push subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscription {
    pub id: String,               // UUID for this subscription
    pub endpoint: String,         // Push service URL
    pub keys: SubscriptionKeys,   // p256dh and auth keys
    pub user_agent: String,       // Browser identification
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionKeys {
    pub p256dh: String,           // Browser's public key (base64)
    pub auth: String,             // Auth secret (base64)
}

/// Notification to send
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushNotification {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub tag: String,              // Dedup - same tag replaces previous
    pub data: NotificationData,   // Deep link info
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationData {
    pub url: String,              // Deep link URL
    pub session_id: Option<String>,
    pub event_type: NotificationEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationEvent {
    PermissionNeeded,
    SessionCompleted,
    SessionError,
}
```

### Notification Config

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotificationConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub notify_permission: bool,
    #[serde(default = "default_true")]
    pub notify_completed: bool,
    #[serde(default = "default_true")]
    pub notify_error: bool,
}

fn default_true() -> bool { true }
```

---

## Backend Services

### VapidKeyManager

```rust
pub struct VapidKeyManager {
    keys: VapidKeys,
    config_path: PathBuf,
}

impl VapidKeyManager {
    /// Load existing keys or generate new ones
    pub fn load_or_generate(config_dir: &Path) -> Result<Self>;

    /// Get public key as base64 for browser subscription
    pub fn public_key_base64(&self) -> String;

    /// Sign a push message with VAPID
    pub fn sign(&self, audience: &str) -> Result<VapidSignature>;
}
```

### SubscriptionStore

```rust
pub struct SubscriptionStore {
    subscriptions: RwLock<HashMap<String, PushSubscription>>,
    file_path: PathBuf,
}

impl SubscriptionStore {
    pub fn load(config_dir: &Path) -> Result<Self>;

    pub async fn add(&self, sub: PushSubscription) -> Result<()>;
    pub async fn remove(&self, id: &str) -> Result<()>;
    pub async fn list(&self) -> Vec<PushSubscription>;

    /// Remove subscriptions whose endpoints return 404/410 (expired)
    pub async fn cleanup_stale(&self, stale_ids: &[String]) -> Result<()>;
}
```

### NotificationService

```rust
pub struct NotificationService {
    vapid: Arc<VapidKeyManager>,
    subscriptions: Arc<SubscriptionStore>,
    config: NotificationConfig,
    http_client: reqwest::Client,
}

impl NotificationService {
    pub fn new(
        vapid: Arc<VapidKeyManager>,
        subscriptions: Arc<SubscriptionStore>,
        config: NotificationConfig,
    ) -> Self;

    /// Start listening to EventBus and sending notifications
    pub async fn run(&self, event_rx: broadcast::Receiver<VibesEvent>);

    /// Send a notification to all subscribed browsers
    async fn send_to_all(&self, notification: PushNotification) -> Result<()>;

    /// Send to single subscription, return true if successful
    async fn send_one(&self, sub: &PushSubscription, notif: &PushNotification) -> bool;
}
```

### Event → Notification Mapping

```rust
impl NotificationService {
    fn event_to_notification(&self, event: &VibesEvent) -> Option<PushNotification> {
        match event {
            VibesEvent::PermissionRequest { session_id, tool, .. } if self.config.notify_permission => {
                Some(PushNotification {
                    title: "Claude needs approval".into(),
                    body: format!("{} wants to use {}", session_id, tool),
                    tag: format!("permission-{}", session_id),
                    data: NotificationData {
                        url: format!("/session/{}?permission=pending", session_id),
                        session_id: Some(session_id.clone()),
                        event_type: NotificationEvent::PermissionNeeded,
                    },
                    ..Default::default()
                })
            }
            VibesEvent::SessionCompleted { session_id } if self.config.notify_completed => {
                Some(PushNotification {
                    title: "Session completed".into(),
                    body: "Claude finished the task".into(),
                    tag: format!("completed-{}", session_id),
                    data: NotificationData {
                        url: format!("/session/{}", session_id),
                        session_id: Some(session_id.clone()),
                        event_type: NotificationEvent::SessionCompleted,
                    },
                    ..Default::default()
                })
            }
            VibesEvent::SessionFailed { session_id, error } if self.config.notify_error => {
                Some(PushNotification {
                    title: "Session failed".into(),
                    body: error.to_string(),
                    tag: format!("error-{}", session_id),
                    data: NotificationData {
                        url: format!("/session/{}", session_id),
                        session_id: Some(session_id.clone()),
                        event_type: NotificationEvent::SessionError,
                    },
                    ..Default::default()
                })
            }
            _ => None,
        }
    }
}
```

---

## HTTP API

### New Endpoints

```
GET  /api/push/vapid-key          # Get VAPID public key for subscription
POST /api/push/subscribe          # Register a push subscription
DELETE /api/push/subscribe/:id    # Unsubscribe
GET  /api/push/subscriptions      # List current subscriptions (for settings UI)
```

### Endpoint Details

```rust
/// GET /api/push/vapid-key
/// Returns the VAPID public key needed to create a push subscription
#[derive(Serialize)]
pub struct VapidKeyResponse {
    pub public_key: String,  // Base64-encoded P-256 public key
}

/// POST /api/push/subscribe
/// Browser sends its PushSubscription after calling pushManager.subscribe()
#[derive(Deserialize)]
pub struct SubscribeRequest {
    pub endpoint: String,
    pub keys: SubscriptionKeys,
}

#[derive(Serialize)]
pub struct SubscribeResponse {
    pub id: String,          // Subscription ID for unsubscribing
}

/// GET /api/push/subscriptions
/// List all subscriptions (for settings UI to show connected devices)
#[derive(Serialize)]
pub struct SubscriptionInfo {
    pub id: String,
    pub user_agent: String,
    pub created_at: DateTime<Utc>,
}
```

### WebSocket Addition

```typescript
// Server → Client: notification settings sync
{
  "type": "notification_config",
  "enabled": true,
  "notify_permission": true,
  "notify_completed": true,
  "notify_error": true
}
```

---

## Web UI Components

### Service Worker (`sw.js`)

```javascript
// Receives push events and displays notifications
self.addEventListener('push', (event) => {
  const data = event.data.json();

  event.waitUntil(
    self.registration.showNotification(data.title, {
      body: data.body,
      icon: '/icon-192.png',
      badge: '/badge-72.png',
      tag: data.tag,
      data: data.data,  // Contains deep link URL
      requireInteraction: data.data.event_type === 'PermissionNeeded',
    })
  );
});

// Handle notification click → open/focus Web UI
self.addEventListener('notificationclick', (event) => {
  event.notification.close();

  const url = event.notification.data.url;

  event.waitUntil(
    clients.matchAll({ type: 'window' }).then((windowClients) => {
      // Focus existing tab if open, otherwise open new one
      for (const client of windowClients) {
        if (client.url.includes(self.location.origin)) {
          client.navigate(url);
          return client.focus();
        }
      }
      return clients.openWindow(url);
    })
  );
});
```

### NotificationSettings Component

```tsx
function NotificationSettings() {
  const { subscription, subscribe, unsubscribe } = usePushSubscription();
  const [config, setConfig] = useNotificationConfig();

  return (
    <div className="notification-settings">
      {!subscription ? (
        <button onClick={subscribe}>
          Enable Notifications
        </button>
      ) : (
        <>
          <div className="status">
            ✓ Notifications enabled
            <button onClick={unsubscribe}>Disable</button>
          </div>

          <div className="event-toggles">
            <Toggle
              label="Permission requests"
              checked={config.notify_permission}
              onChange={(v) => setConfig({ ...config, notify_permission: v })}
            />
            <Toggle
              label="Session completed"
              checked={config.notify_completed}
              onChange={(v) => setConfig({ ...config, notify_completed: v })}
            />
            <Toggle
              label="Session errors"
              checked={config.notify_error}
              onChange={(v) => setConfig({ ...config, notify_error: v })}
            />
          </div>
        </>
      )}
    </div>
  );
}
```

### usePushSubscription Hook

```typescript
function usePushSubscription() {
  const [subscription, setSubscription] = useState<PushSubscription | null>(null);

  const subscribe = async () => {
    // 1. Request notification permission
    const permission = await Notification.requestPermission();
    if (permission !== 'granted') return;

    // 2. Get VAPID public key from server
    const { public_key } = await fetch('/api/push/vapid-key').then(r => r.json());

    // 3. Subscribe via Push API
    const reg = await navigator.serviceWorker.ready;
    const sub = await reg.pushManager.subscribe({
      userVisibleOnly: true,
      applicationServerKey: urlBase64ToUint8Array(public_key),
    });

    // 4. Send subscription to server
    await fetch('/api/push/subscribe', {
      method: 'POST',
      body: JSON.stringify({
        endpoint: sub.endpoint,
        keys: {
          p256dh: arrayBufferToBase64(sub.getKey('p256dh')),
          auth: arrayBufferToBase64(sub.getKey('auth')),
        },
      }),
    });

    setSubscription(sub);
  };

  return { subscription, subscribe, unsubscribe };
}
```

---

## CLI Integration

### `--notify` Flag

```bash
# Enable notifications for this session
vibes claude --notify "Build the authentication system"

# Combine with other flags
vibes claude --notify --session-name "auth-feature" "Build auth"
```

### Behavior

| Scenario | `--notify` behavior |
|----------|---------------------|
| No subscriptions exist | Warn user to enable notifications in Web UI |
| Subscriptions exist | Notifications sent normally |
| Server not running | Start server automatically (existing behavior) |

### Implementation

```rust
// vibes-cli/src/commands/claude.rs

#[derive(Parser)]
pub struct ClaudeArgs {
    // ... existing args ...

    /// Send push notifications for this session's events
    #[arg(long)]
    notify: bool,
}

impl ClaudeArgs {
    pub async fn run(self) -> Result<()> {
        // ... existing setup ...

        if self.notify {
            // Check if any subscriptions exist
            let subs = client.get("/api/push/subscriptions").await?;
            if subs.is_empty() {
                eprintln!("⚠ No push subscriptions found.");
                eprintln!("  Open the Web UI and enable notifications first.");
                eprintln!("  Web UI: http://localhost:{}", port);
            }

            // Set session-level notification override
            client.post("/api/sessions/:id/notify", json!({ "enabled": true })).await?;
        }

        // ... rest of session logic ...
    }
}
```

### Session-Level Override

```rust
// Session state includes notification preference
pub struct Session {
    // ... existing fields ...
    notify_override: Option<bool>,  // None = use global config
}
```

---

## Crate Structure

### New/Modified Files

```
vibes/
├── vibes-core/
│   └── src/
│       ├── notifications/              # NEW MODULE
│       │   ├── mod.rs                  # Module exports
│       │   ├── config.rs               # NotificationConfig
│       │   ├── vapid.rs                # VapidKeyManager
│       │   ├── subscription.rs         # PushSubscription, SubscriptionStore
│       │   ├── service.rs              # NotificationService
│       │   └── push.rs                 # Web Push protocol implementation
│       └── lib.rs                      # Export notifications module
│
├── vibes-server/
│   └── src/
│       ├── http/
│       │   ├── mod.rs                  # Add push routes
│       │   └── push.rs                 # NEW: Push subscription endpoints
│       └── state.rs                    # Add NotificationService to AppState
│
├── vibes-cli/
│   └── src/
│       └── commands/
│           └── claude.rs               # Add --notify flag
│
└── web-ui/
    ├── public/
    │   └── sw.js                       # NEW: Service worker
    ├── src/
    │   ├── hooks/
    │   │   └── usePushSubscription.ts  # NEW: Push subscription hook
    │   └── components/
    │       └── NotificationSettings.tsx # NEW: Settings UI
    └── vite.config.ts                  # Configure service worker
```

---

## Dependencies

### vibes-core/Cargo.toml

```toml
[dependencies]
# Web Push protocol
web-push = "0.10"           # Handles VAPID signing and push message encryption

# Already present
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["sync", "fs"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
```

### web-ui/package.json

```json
{
  "devDependencies": {
    "vite-plugin-pwa": "^0.20"
  }
}
```

---

## Testing Strategy

### Unit Tests

| Component | Test Coverage |
|-----------|---------------|
| VapidKeyManager | Key generation, persistence, loading existing keys |
| SubscriptionStore | Add/remove/list subscriptions, file persistence, concurrent access |
| NotificationService | Event filtering, notification mapping, config respect |
| Push message signing | VAPID JWT generation, message encryption |

### Integration Tests

| Test | Description |
|------|-------------|
| Subscription flow | Register subscription via API, verify stored |
| Push delivery (mock) | Mock push endpoint, verify correct payload/headers |
| Stale cleanup | Simulate 410 response, verify subscription removed |
| Event → notification | Trigger VibesEvent, verify notification sent |

### Manual Testing Checklist

- [ ] Fresh install generates VAPID keys automatically
- [ ] Web UI prompts for notification permission
- [ ] Subscription persists across server restart
- [ ] Permission request triggers notification
- [ ] Session completion triggers notification
- [ ] Session error triggers notification
- [ ] Clicking notification opens correct Web UI page
- [ ] Notification replaces previous (same tag)
- [ ] Disabling notification type stops those notifications
- [ ] `--notify` flag warns if no subscriptions exist
- [ ] Works on mobile browser (iOS Safari, Android Chrome)
- [ ] Works through Cloudflare Tunnel

---

## Deliverables

### Milestone 2.3 Checklist

**Backend (vibes-core):**
- [ ] VapidKeyManager with auto-generation
- [ ] SubscriptionStore with file persistence
- [ ] NotificationService subscribing to EventBus
- [ ] Web Push protocol implementation
- [ ] NotificationConfig type

**Server (vibes-server):**
- [ ] GET /api/push/vapid-key endpoint
- [ ] POST /api/push/subscribe endpoint
- [ ] DELETE /api/push/subscribe/:id endpoint
- [ ] GET /api/push/subscriptions endpoint
- [ ] NotificationService integration with AppState

**CLI (vibes-cli):**
- [ ] `--notify` flag on `vibes claude`
- [ ] Subscription check and warning

**Web UI:**
- [ ] Service worker (sw.js)
- [ ] usePushSubscription hook
- [ ] NotificationSettings component
- [ ] Vite PWA plugin configuration

**Documentation:**
- [ ] ADR-013 in PRD
- [ ] Design document
- [ ] Update PROGRESS.md
