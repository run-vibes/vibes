---
created: 2026-01-02
---

# Milestone 36: Firehose Infinite Scroll - Design

> Transform the firehose from a live event buffer to a full-featured log viewer with infinite scroll through the entire eventlog history.

## Overview

The firehose currently loads the last 100 events and buffers up to 500 in memory. Users cannot access older events even though the EventLog (Iggy) stores full history. This milestone adds infinite scroll to access the complete event history with a proper log viewer UX.

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Plugin vs Built-in** | Built-in | Core UI component, not optional |
| **Event ordering** | Oldest-at-top, newest-at-bottom | Standard log viewer convention (`tail -f`) |
| **Pagination trigger** | Scroll-to-top | Infinite scroll is more intuitive than buttons |
| **Auto-follow resume** | Scroll-to-bottom + floating button | Both mechanisms for flexibility |
| **Pause button** | Remove | Scrolling up is natural pause; not needed |
| **Clear button** | Remove | Events are persistent; clearing doesn't make sense |
| **Virtual scrolling** | @tanstack/react-virtual | Handles 100K+ events, proven library |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Firehose Page                           │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Filters: [SESSION] [CLAUDE] [TOOL]  Search: [____]  │   │
│  └─────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ ▲ Loading older...                                   │   │
│  │ ┌─────────────────────────────────────────────────┐ │   │
│  │ │ 09:15:23.456 SESSION session_created            │ │   │
│  │ │ 09:15:24.789 CLAUDE  turn_started               │ │   │
│  │ │ 09:15:25.123 TOOL    tool_use                   │ │   │
│  │ │ ...                                             │ │   │
│  │ │ 09:45:12.345 CLAUDE  turn_complete      ← newest│ │   │
│  │ └─────────────────────────────────────────────────┘ │   │
│  │                              ┌──────────────────┐   │   │
│  │                              │ ↓ Jump to latest │   │   │
│  │                              └──────────────────┘   │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow

```
EventLog (Iggy)                    WebSocket                      React
     │                                │                             │
     │ ◄──── fetch_older ─────────────│◄──── scroll to top ────────│
     │                                │                             │
     │ ────► events_batch ───────────►│────► prepend to list ──────►│
     │       (older events)           │      (preserve scroll)      │
     │                                │                             │
     │ ────► event (live) ───────────►│────► append to list ───────►│
     │                                │      (auto-scroll if        │
     │                                │       following)            │
```

### Component Overview

| Component | Location | Responsibility |
|-----------|----------|----------------|
| `firehose.rs` | vibes-server/src/ws/ | WebSocket handler, pagination, filtering |
| `useFirehose.ts` | web-ui/src/hooks/ | State management, offset tracking, WebSocket |
| `Firehose.tsx` | web-ui/src/pages/ | Page layout, filter controls |
| `StreamView.tsx` | web-ui/src/components/ | Virtualized list, scroll detection |

---

## WebSocket Protocol

### Client → Server

**Fetch older events:**
```json
{ "type": "fetch_older", "before_offset": 12345, "limit": 100 }
```

**Update filters:**
```json
{ "type": "set_filters", "types": ["SESSION", "CLAUDE"], "session": "abc" }
```

### Server → Client

**Event batch (initial load or pagination):**
```json
{
  "type": "events_batch",
  "events": [...],
  "oldest_offset": 12200,
  "has_more": true
}
```

**Live event:**
```json
{ "type": "event", "event": {...}, "offset": 12350 }
```

### Connection Flow

1. Client connects with initial filters in query string
2. Server sends `events_batch` with latest 100 events + `has_more` flag
3. Server streams live events as they occur
4. Client sends `fetch_older` when user scrolls to top
5. Client sends `set_filters` when filters change (server sends fresh latest batch)

---

## Frontend State Model

```typescript
interface FirehoseState {
  events: StreamEvent[];         // All loaded events (sorted by offset)
  oldestOffset: number | null;   // Earliest offset fetched (null = at beginning)
  newestOffset: number;          // Latest offset (for live updates)
  isLoadingOlder: boolean;       // Loading indicator for scroll-up fetch
  isFollowing: boolean;          // Auto-scroll to new events?
  hasMore: boolean;              // More history available?
  filters: {
    types: Set<EventType>;       // Selected event types
    sessionId: string | null;    // Session filter
    search: string;              // Text search (client-side)
  };
}
```

---

## User Experience

### Auto-Follow Behavior

- On load: scroll to bottom, follow new events
- Scroll up: break auto-follow, view freezes
- Scroll to bottom: resume auto-follow
- "Jump to latest" button: appears when not following, one-click resume

### Filter Behavior

- Type chips toggle event types (SERVER, CLAUDE, TOOL, etc.)
- Session filter: text input for session ID prefix match
- Search: client-side text filter across event content
- Changing any filter: jump to latest, resume following

### Timestamps

Display format: `HH:mm:ss.SSS` (millisecond precision)

---

## Dependencies

```toml
# web-ui/package.json
"@tanstack/react-virtual": "^3"
```

No new Rust dependencies required.

---

## Testing Strategy

| Component | Test Coverage |
|-----------|---------------|
| Backend protocol | Unit tests for fetch_older, set_filters handlers |
| useFirehose hook | Mock WebSocket, test offset tracking and state transitions |
| StreamView | Test scroll detection, auto-follow logic |
| Integration | E2E test scrolling through history |

---

## Deliverables

- [ ] Backend: `fetch_older` and `set_filters` message handlers
- [ ] Backend: Offset field in all event messages
- [ ] Frontend: Rewritten useFirehose hook with offset tracking
- [ ] Frontend: Virtualized scroll view with position preservation
- [ ] Frontend: Working filters (type, session, search)
- [ ] Frontend: "Jump to latest" floating button
- [ ] Frontend: Millisecond timestamps
- [ ] Frontend: Remove Pause/Clear buttons
- [ ] Tests passing
