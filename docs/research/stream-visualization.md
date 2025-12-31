# Stream Visualization Research

> Research document exploring visualization approaches for Iggy event streams in vibes.

## Overview

This document analyzes use cases and visualization approaches for the event streams flowing through Apache Iggy in the vibes system. The goal is to provide developers and users with powerful tools to observe, debug, and understand session behavior.

## Event Stream Structure

Events flowing through Iggy follow the `VibesEvent` schema:

```
VibesEvent
├── SessionCreated { session_id, harness, timestamp }
├── SessionStateChanged { session_id, old, new }
├── PtyOutput { session_id, data, source }
├── UserInput { session_id, content }
├── PermissionRequest { session_id, tool, args }
├── PermissionResponse { session_id, approved }
├── TunnelStateChanged { session_id, state }
├── ClaudeEvent { session_id, event_type, payload }
└── ClientEvent { client_id, action }
```

---

## Use Cases

### 1. Firehose

**Purpose:** See everything happening in real-time

**Who needs it:** Developers during development, ops during incidents

**Key requirements:**
- Handle high volume without UI lag (virtualized scrolling)
- Color-coded by event type for quick scanning
- Pause without losing events (buffer continues)
- Basic filtering (by type, session, search)

**Visualization:**
```
┌─────────────────────────────────────────────────────────────────┐
│ 🔴 LIVE │ 247/sec │ [Filter ▾] [Session ▾] │ ⏸ Pause │ 🔍      │
├─────────────────────────────────────────────────────────────────┤
│ 12:34:56.789 │ abc-123 │ 🟦 PtyOutput    │ "Compiling vi..." │
│ 12:34:56.801 │ xyz-789 │ 🟩 UserInput    │ "git status"      │
│ 12:34:56.812 │ abc-123 │ 🟨 Permission   │ Bash: cargo test  │
│ 12:34:56.823 │ abc-123 │ 🟪 Claude       │ thinking...       │
│ 12:34:56.834 │ xyz-789 │ 🟦 PtyOutput    │ "On branch main"  │
│ ▼ auto-scrolling...                                            │
└─────────────────────────────────────────────────────────────────┘
```

**Technical considerations:**
- Virtual list with 10k+ event buffer
- WebSocket streaming from server
- Client-side filtering for responsiveness
- Rate limiting display updates (batch renders)

---

### 2. Debugging

**Purpose:** Find what went wrong in a specific session/timeframe

**Who needs it:** Developers investigating bugs, users wondering "why did it do that?"

**Key requirements:**
- Time-range selection with zoom
- Grep-like search across payloads
- Session isolation ("show only session X")
- Correlation ("show events ±5s around this error")
- Event detail expansion (full JSON)

**Visualization:**
```
┌─────────────────────────────────────────────────────────────────┐
│ DEBUG │ Session: abc-123 │ 12:30 - 12:45 │ 🔍 "error"          │
├─────────────────────────────────────────────────────────────────┤
│ Timeline (zoom with scroll)                                     │
│ ├──────────────────●━━━━━━━━━●──────────────────┤               │
│ 12:30            12:34:56  12:38:12           12:45             │
│                   error     error                               │
├─────────────────────────────────────────────────────────────────┤
│ ▸ 12:34:52.001 │ UserInput    │ "cargo test"                   │
│ ▸ 12:34:52.234 │ PtyOutput    │ "running 42 tests..."          │
│ ▾ 12:34:56.789 │ 🔴 PtyOutput │ "error[E0382]: borrow..."      │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │ {                                                       │   │
│   │   "type": "PtyOutput",                                  │   │
│   │   "session_id": "abc-123",                              │   │
│   │   "data": "error[E0382]: borrow of moved value...",     │   │
│   │   "source": "stderr"                                    │   │
│   │ }                                                       │   │
│   └─────────────────────────────────────────────────────────┘   │
│ ▸ 12:34:57.001 │ Claude       │ "I see the error..."           │
└─────────────────────────────────────────────────────────────────┘
```

**Technical considerations:**
- Query API with offset/partition seeking
- Full-text search on payloads (leverage existing FTS5?)
- Efficient range queries by timestamp
- Event correlation by session_id + time window

---

### 3. Actions

**Purpose:** Surface events requiring human intervention

**Who needs it:** Users monitoring sessions, operators

**Key requirements:**
- Filter to actionable event types only
- Priority/urgency indicators
- One-click actions (approve, deny, restart)
- Acknowledgment (mark as handled)
- Notification integration (web push already exists)

**Visualization:**
```
┌─────────────────────────────────────────────────────────────────┐
│ ACTIONS │ 3 pending │ [Mark all read]                           │
├─────────────────────────────────────────────────────────────────┤
│ 🟡 PENDING │ 2 min ago │ Session abc-123                        │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ Permission Request: Bash                                    │ │
│ │ Command: rm -rf ./node_modules                              │ │
│ │                                                             │ │
│ │ [✓ Approve] [✗ Deny] [👁 View Context]                      │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│ 🔴 ERROR │ 5 min ago │ Session xyz-789                          │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ Session crashed: PTY process exited unexpectedly            │ │
│ │ Exit code: 137 (SIGKILL - out of memory?)                   │ │
│ │                                                             │ │
│ │ [🔄 Restart Session] [📋 View Logs] [✓ Acknowledge]         │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│ ⚪ RESOLVED │ 12 min ago │ Permission approved                  │
└─────────────────────────────────────────────────────────────────┘
```

**Technical considerations:**
- Filtered consumer with state tracking
- Action dispatch via existing WebSocket protocol
- Persistence of acknowledgment state
- Integration with web push notifications

---

### 4. Insights

**Purpose:** Understand patterns, usage, and system health over time

**Who needs it:** Product owners, developers optimizing, capacity planning

**Key requirements:**
- Time-series aggregations (events/hour, sessions/day)
- Distribution breakdowns (event types, durations)
- Anomaly highlighting
- Exportable reports

**Visualization:**
```
┌─────────────────────────────────────────────────────────────────┐
│ INSIGHTS │ [Last 24h ▾] [Last 7d] [Last 30d] │ [Export CSV]     │
├─────────────────────────────────────────────────────────────────┤
│ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐              │
│ │    12.4K     │ │      23      │ │    4.2 min   │              │
│ │ Total Events │ │   Sessions   │ │ Avg Duration │              │
│ │   ▲ 12%      │ │   ▲ 8%       │ │   ▼ 15%      │              │
│ └──────────────┘ └──────────────┘ └──────────────┘              │
├─────────────────────────────────────────────────────────────────┤
│ Events Over Time                     Event Distribution          │
│ 800│    ▄▄                          ┌────────────────────┐      │
│    │   ████  ▄▄                     │ ████████░░ 67% Pty │      │
│ 400│▄▄██████████▄▄                  │ ████░░░░░░ 18% Input│     │
│    │████████████████▄▄              │ ███░░░░░░░ 12% Perm │     │
│   0└──────────────────              │ █░░░░░░░░░  3% Other│     │
│    00  06  12  18  24               └────────────────────┘      │
├─────────────────────────────────────────────────────────────────┤
│ Session Duration Distribution        Peak Hours                  │
│ ▁▂▄█▆▃▂▁                            🔥 14:00-16:00 (42%)        │
│ 0  5  10  15  20+ min               💤 02:00-06:00 (3%)         │
└─────────────────────────────────────────────────────────────────┘
```

**Technical considerations:**
- Aggregation consumer writing to time-series storage
- Could use Iggy topics for aggregates or separate store
- Pre-computed rollups for common time ranges
- Chart library integration (e.g., Recharts, Chart.js)

---

### 5. Replay

**Purpose:** Re-experience a session exactly as it happened

**Who needs it:** Debugging, demos, training, compliance audit

**Key requirements:**
- VCR-style controls (play, pause, speed, seek)
- Event-by-event stepping (forensic mode)
- State reconstruction (terminal at time T)
- Bookmarks/annotations
- Export/share capability

**Visualization:**
```
┌─────────────────────────────────────────────────────────────────┐
│ REPLAY │ Session: abc-123 │ "Add authentication" │ 12:34 total  │
├─────────────────────────────────────────────────────────────────┤
│ ◀◀  ◀  ▶  ▶  ▶▶ │ [0.5x] [1x] [2x] [4x] │ Event ◀ 234 ▶       │
│ ━━━━━━━━━━━━━━━━━━━●━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  │
│ 00:00            05:23                                   12:34  │
│                    ▲                                            │
│              Current: UserInput                                 │
├─────────────────────────────────────────────────────────────────┤
│ Terminal at 05:23                                               │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ $ cargo test                                                │ │
│ │ running 42 tests                                            │ │
│ │ test auth::tests::login_success ... ok                      │ │
│ │ test auth::tests::login_failure ... ok                      │ │
│ │ █                                                           │ │
│ └─────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│ Event Details                        │ Event List               │
│ ┌──────────────────────────────────┐ │ ▸ 05:22 PtyOutput       │
│ │ Type: UserInput                  │ │ ▾ 05:23 UserInput ◀     │
│ │ Content: "cargo test"            │ │ ▸ 05:23 PtyOutput       │
│ │ Offset: 4,521                    │ │ ▸ 05:24 PtyOutput       │
│ └──────────────────────────────────┘ │ ▸ 05:31 Claude          │
└─────────────────────────────────────────────────────────────────┘
```

**Technical considerations:**
- Offset-based seeking in Iggy partitions
- Terminal state reconstruction from cumulative PtyOutput events
- Efficient binary search for timestamp-to-offset mapping
- Video-like buffering for smooth playback

---

## Additional Use Cases

### 6. Topology

**Purpose:** Live view of sessions, connections, and event flow

**Visualization:**
```
┌─────────────────────────────────────────────────────────────────┐
│              ┌─────────┐                                        │
│              │ Iggy    │                                        │
│              │ Stream  │                                        │
│              └────┬────┘                                        │
│         ┌────────┼────────┐                                     │
│         ▼        ▼        ▼                                     │
│    ┌────────┐┌────────┐┌────────┐                               │
│    │Session ││Session ││Session │                               │
│    │ abc    ││ xyz    ││ 123    │                               │
│    │ 🟢     ││ 🟡     ││ ⚪     │                               │
│    │CLI+Web ││CLI only││ idle   │                               │
│    └────────┘└────────┘└────────┘                               │
└─────────────────────────────────────────────────────────────────┘
```

**Use:** Understanding system state at a glance, identifying connection issues

### 7. Consumer Health

**Purpose:** Monitor Iggy consumer lag and processing health

**Visualization:**
```
Consumer         Offset      Head       Lag      Rate
────────────────────────────────────────────────────
websocket-bc     45,231     45,234        3     120/s ✓
metrics-agg      45,100     45,234      134      45/s ⚠
assessment       45,230     45,234        4      80/s ✓
```

**Use:** Ops monitoring, identifying backpressure issues

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Web UI (TanStack)                       │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────┐│
│  │ Firehose │ │ Debugger │ │ Actions  │ │ Insights │ │ Replay ││
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └───┬────┘│
│       │            │            │            │           │      │
│       └────────────┴────────────┴────────────┴───────────┘      │
│                              │                                   │
│                     WebSocket + REST API                         │
└──────────────────────────────┬──────────────────────────────────┘
                               │
┌──────────────────────────────┴──────────────────────────────────┐
│                        vibes-server                              │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐     │
│  │ Stream Relay   │  │ Query Service  │  │ Metrics Agg    │     │
│  │ (real-time)    │  │ (seek/filter)  │  │ (time-series)  │     │
│  └───────┬────────┘  └───────┬────────┘  └───────┬────────┘     │
│          └───────────────────┴───────────────────┘               │
│                              │                                   │
│                    ┌─────────┴─────────┐                        │
│                    │   IggyEventLog    │                        │
│                    └─────────┬─────────┘                        │
└──────────────────────────────┬──────────────────────────────────┘
                               │
                    ┌──────────┴──────────┐
                    │   Iggy Server       │
                    │   (persistent log)  │
                    └─────────────────────┘
```

### Server Components

| Component | Responsibility |
|-----------|----------------|
| Stream Relay | Real-time WebSocket broadcast of events |
| Query Service | Time-range queries, filtering, search |
| Metrics Aggregator | Pre-compute time-series aggregations |

### Data Flow

1. **Write path:** Session events → IggyEventLog → Iggy partitions
2. **Real-time path:** Iggy consumer → Stream Relay → WebSocket → UI
3. **Query path:** UI request → Query Service → Iggy seek → Response
4. **Aggregation path:** Iggy consumer → Metrics Agg → Time-series store

---

## Implementation Priority

| Use Case | User Value | Complexity | Priority | Notes |
|----------|------------|------------|----------|-------|
| Firehose | High | Low | P0 | Validates streaming infrastructure |
| Actions | High | Medium | P0 | Direct user value, leverages existing notifications |
| Debugging | High | Medium | P1 | Essential for development |
| Replay | Medium | High | P2 | Killer feature, but complex state reconstruction |
| Insights | Medium | Medium | P2 | Can defer, less urgent |
| Topology | Low | Medium | P3 | Nice-to-have |
| Consumer Health | Low | Low | P3 | Ops tooling |

### Suggested Phases

**Phase 1: Foundation**
- Firehose view (validates real-time streaming)
- Basic filtering and search
- Event detail expansion

**Phase 2: Actionability**
- Actions queue with approve/deny
- Integration with web push
- Session-specific views

**Phase 3: Time Travel**
- Debugging timeline with zoom
- Replay with VCR controls
- State reconstruction

**Phase 4: Analytics**
- Insights dashboard
- Metrics aggregation
- Export capabilities

---

## Open Questions

1. **State reconstruction:** How do we efficiently reconstruct terminal state at arbitrary points? Options:
   - Replay all PtyOutput from session start (simple, slow)
   - Periodic snapshots with delta replay (complex, fast)
   - Client-side terminal emulator with seek (xterm.js?)

2. **Search scalability:** Full-text search across all event payloads at scale?
   - Leverage existing FTS5 in chat history?
   - Separate search index?
   - Iggy-native search (if available)?

3. **Retention:** How long do we keep events?
   - Configurable per-topic retention in Iggy
   - Tiered storage (hot/warm/cold)?
   - User-configurable per session?

4. **Multi-tenancy:** If vibes supports multiple users, how do we partition/secure streams?
   - Separate Iggy topics per user?
   - Filter at query time?
   - Encryption at rest?

---

## References

- [Apache Iggy Documentation](https://iggy.rs/)
- [xterm.js](https://xtermjs.org/) - Terminal emulator for replay
- [TanStack Virtual](https://tanstack.com/virtual) - Virtual scrolling for firehose
- [Recharts](https://recharts.org/) - Charts for insights dashboard
