---
id: FEAT0177
title: WebSocket real-time updates
type: feat
status: done
priority: high
scope: tui/02-terminal-dashboard
depends: [m42-feat-01, m42-feat-02, m42-feat-03]
estimate: 4h
---

# WebSocket Real-Time Updates

## Summary

Connect the dashboard widgets to the vibes-server WebSocket for real-time event streaming. Subscribe to relevant topics and update widget state as events arrive.

## Features

### Event Subscriptions

Subscribe to these event types:

- `session.*` - Session lifecycle events
- `agent.*` - Agent status and action events
- `swarm.*` - Swarm coordination events
- `metrics.*` - Cost and usage updates

### State Updates

Events trigger widget updates:

| Event | Widget Update |
|-------|---------------|
| session.created | Add to session list |
| session.ended | Remove/gray out in list |
| agent.started | Increment agent count |
| agent.completed | Activity feed entry |
| agent.permission_requested | Activity feed warning |
| metrics.cost_updated | Stats bar cost |

### Connection Management

- Auto-reconnect on disconnect
- Show connection status indicator
- Queue events during reconnection

## Implementation

### Message Handler

```rust
impl App {
    pub async fn handle_ws_message(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::Event(event) => {
                self.process_event(event);
                self.refresh_widgets();
            }
            ServerMessage::Snapshot(snapshot) => {
                self.state = snapshot.into();
                self.refresh_widgets();
            }
        }
    }

    fn process_event(&mut self, event: DomainEvent) {
        match event {
            DomainEvent::SessionCreated { id, .. } => {
                self.state.sessions.insert(id, SessionInfo::new(id));
            }
            DomainEvent::AgentStarted { id, session_id, .. } => {
                self.state.agent_count += 1;
                self.activity_feed.push(ActivityEntry::agent_started(id));
            }
            // ... other events
        }
    }
}
```

### Steps

1. Create `src/ws.rs` for WebSocket client wrapper
2. Define `WsClient` with connection and subscription management
3. Add message deserialization for server events
4. Implement event-to-state mapping in App
5. Add reconnection logic with exponential backoff
6. Show connection status in stats bar or status line
7. Add integration tests with mock server

## Acceptance Criteria

- [ ] TUI connects to vibes-server WebSocket on startup
- [ ] Session list updates when sessions created/ended
- [ ] Agent count updates with agent lifecycle
- [ ] Activity feed populates from real events
- [ ] Cost updates in real-time
- [ ] Connection status visible in UI
- [ ] Auto-reconnect on disconnect
- [ ] Integration tests pass
