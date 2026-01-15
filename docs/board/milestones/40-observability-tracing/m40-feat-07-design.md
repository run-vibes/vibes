# Observe Web UI Design

## Overview

Add a `/traces` page to the web-ui for viewing real-time distributed traces. Users can filter by session, agent, or log level, and inspect span hierarchies with inline expansion.

## Data Flow

```
┌─────────────────────────────────────────────────────────┐
│  Web UI                                                  │
│  ┌─────────────────────────────────────────────────┐    │
│  │  useTraces hook                                  │    │
│  │  - Sends SubscribeTraces with filters           │    │
│  │  - Receives TraceEvent stream                   │    │
│  │  - Groups spans by trace_id into trees          │    │
│  └─────────────────────────────────────────────────┘    │
│                         │                                │
│                         ▼                                │
│  ┌─────────────────────────────────────────────────┐    │
│  │  TracesPage component                           │    │
│  │  - Filter controls (session, agent, level)      │    │
│  │  - Trace list with tree rendering               │    │
│  │  - Auto-follow with pause/resume                │    │
│  └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Trace grouping | Tree view per trace | Best for understanding request flow, similar to Jaeger/Zipkin |
| Filtering | Server-side only | Efficient for high-throughput, less client state |
| Live streaming | Auto-follow with pause | Matches Firehose pattern, good UX for live data |
| Detail view | Inline expansion | No navigation away from list, focused experience |

## TypeScript Types

Add to `web-ui/src/lib/types.ts`:

```typescript
// === Trace Types ===

export type SpanStatus = 'ok' | 'error';

export interface TraceEvent {
  trace_id: string;
  span_id: string;
  parent_span_id?: string;
  name: string;
  level: string;  // trace, debug, info, warn, error
  timestamp: string;  // ISO 8601
  duration_ms?: number;
  session_id?: string;
  agent_id?: string;
  attributes: Record<string, string>;
  status: SpanStatus;
}

// Grouped trace for UI rendering
export interface TraceTree {
  trace_id: string;
  root_span: SpanNode;
  session_id?: string;
  agent_id?: string;
  timestamp: string;
  total_duration_ms?: number;
  has_errors: boolean;
}

export interface SpanNode {
  event: TraceEvent;
  children: SpanNode[];
}
```

**Client/Server message additions:**

```typescript
// Add to ClientMessage union
| { type: 'subscribe_traces'; session_id?: string; agent_id?: string; level?: string }
| { type: 'unsubscribe_traces' }

// Add to ServerMessage union
| { type: 'trace_event'; /* fields from TraceEvent */ }
| { type: 'trace_subscribed' }
| { type: 'trace_unsubscribed' }
```

## useTraces Hook

**File:** `web-ui/src/hooks/useTraces.ts`

```typescript
interface UseTracesOptions {
  send: (message: ClientMessage) => void;
  addMessageHandler: (handler: (msg: ServerMessage) => void) => () => void;
  isConnected: boolean;
}

interface UseTracesFilters {
  sessionId?: string;
  agentId?: string;
  level?: string;  // default: 'info'
}

interface UseTracesReturn {
  traces: TraceTree[];           // Grouped traces, newest first
  isSubscribed: boolean;
  isFollowing: boolean;
  subscribe: (filters: UseTracesFilters) => void;
  unsubscribe: () => void;
  setFollowing: (following: boolean) => void;
  clear: () => void;
}
```

**Key behaviors:**

1. **Span grouping** — Incoming `TraceEvent` spans are grouped by `trace_id`. When a span arrives, it's inserted into its trace's tree based on `parent_span_id`.

2. **Buffer management** — Keep last 100 traces (configurable). Evict oldest when limit reached.

3. **Tree assembly** — Root spans have no `parent_span_id`. Child spans are nested under their parent. Orphaned spans (parent not yet received) wait in a pending queue.

4. **Filter changes** — When filters change, unsubscribe + clear + resubscribe. Server sends only matching spans.

## Page Layout

```
┌────────────────────────────────────────────────────────────┐
│  TRACES                              [Connected ●]         │
├────────────────────────────────────────────────────────────┤
│  Filters: [Session ▼] [Agent ▼] [Level: info ▼]  [Clear]  │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  ▼ Trace 019abc12... | 14:23:05 | 5.2s | session-xyz      │
│    └─ server::handle_ws_message (2.3ms) ✓                 │
│       └─ session::process_event (1.8ms) ✓                 │
│          ├─ model::inference (1.2ms) tokens=450 ✓         │
│          └─ tool::execute (0.4ms) tool=read_file ✓        │
│                                                            │
│  ▶ Trace 019abc13... | 14:23:02 | 1.1s | session-xyz      │
│                                                            │
│  ▶ Trace 019abc14... | 14:23:00 | 0.8s | session-abc  ✗   │
│                                                            │
├────────────────────────────────────────────────────────────┤
│  [↓ Following]  or  [Resume ↓] when paused                │
└────────────────────────────────────────────────────────────┘
```

**Components:**
- `TracesPage` — Page container with header, filters, list
- `TraceFilters` — Dropdowns for session/agent/level (inline in TracesPage)
- `TraceList` — Scrollable container with auto-follow logic
- `TraceRow` — Collapsible trace header with summary
- `SpanTree` — Recursive span rendering with indentation

**Visual indicators:**
- Green checkmark for `ok` status
- Red X for `error` status
- Dim text for duration and attributes
- CRT aesthetic with phosphor glow

## Files

**Create:**

| File | Purpose |
|------|---------|
| `web-ui/src/pages/Traces.tsx` | Main page component |
| `web-ui/src/pages/Traces.css` | Page styles |
| `web-ui/src/hooks/useTraces.ts` | WebSocket subscription hook |

**Modify:**

| File | Change |
|------|--------|
| `web-ui/src/lib/types.ts` | Add trace types and type guards |
| `web-ui/src/App.tsx` | Add `/traces` route and nav item |

## Implementation Plan

1. Add TypeScript types to `types.ts`
2. Create `useTraces` hook with span grouping logic
3. Create `Traces.tsx` page with filters and trace list
4. Add route and navigation in `App.tsx`
5. Style with CRT aesthetic in `Traces.css`

## Acceptance Criteria

- [ ] Traces page shows live traces
- [ ] Trace tree renders spans correctly
- [ ] Session/agent filters work
- [ ] Level filter works
- [ ] Real-time streaming updates
- [ ] Auto-follow with pause/resume works
