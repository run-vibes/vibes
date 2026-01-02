# Firehose Infinite Scroll Design

> Transform the firehose from a live event buffer to a full-featured log viewer with infinite scroll through the entire eventlog history.

## Overview

The firehose currently loads the last 100 events and buffers up to 500 in memory. Users cannot access older events even though the EventLog (Iggy) stores full history. This design adds infinite scroll to access the complete event history.

## User Experience

### Log Viewer Behavior

Events display in chronological order: **oldest at top, newest at bottom** (like `tail -f`).

**Auto-Follow:**
- On load, scroll to bottom and follow new events as they arrive
- Scrolling up breaks auto-follow — view freezes at your position
- Scrolling back to bottom resumes auto-follow
- A floating "Jump to latest" button appears when not following

**Infinite Scroll:**
- Scroll to the top to trigger loading older events (100 at a time)
- Loading indicator while fetching
- Smooth scroll position preservation after older events load
- Can scroll all the way back to the first event in the eventlog

### Removed Controls

- **Pause button** — Scrolling up is the natural pause
- **Clear button** — Events are persistent; clearing doesn't make sense

### Filter Behavior

- Type filters (SESSION, CLAUDE, TOOL, etc.) and session filter work
- Search input filters events by content
- Changing any filter jumps to latest and resumes auto-follow
- Filters apply to both historical loading and live events

### Timestamps

Display format: `HH:mm:ss.SSS` (millisecond precision)

## Frontend Architecture

### Virtualized List

Replace the current simple list with a virtualized scroller (`@tanstack/react-virtual`):
- Only renders visible rows plus a small buffer
- Handles 100K+ events without performance issues
- Fixed row height for predictable scrolling

### State Model

```typescript
interface FirehoseState {
  events: StreamEvent[];         // All loaded events (sorted by offset)
  oldestOffset: number | null;   // Earliest offset fetched (null = at beginning)
  newestOffset: number;          // Latest offset (for live updates)
  isLoadingOlder: boolean;       // Loading indicator for scroll-up fetch
  isFollowing: boolean;          // Auto-scroll to new events?
  filters: {
    types: Set<EventType>;       // Selected event types
    sessionId: string | null;    // Session filter
    search: string;              // Text search in event content
  };
}
```

### Filter Implementation

- **Type filters:** Chip toggles. Filter applied client-side AND sent to server for historical fetches.
- **Session filter:** Text input for session ID prefix match
- **Search:** Text search across event type, summary, and payload. Client-side only.

## Backend Protocol

### WebSocket Messages

**Client → Server:**

```json
{ "type": "fetch_older", "before_offset": 12345, "limit": 100 }
```
Request events older than the given offset.

```json
{ "type": "set_filters", "types": ["SESSION", "CLAUDE"], "session": "abc" }
```
Update server-side filters for live events and subsequent fetches.

**Server → Client:**

```json
{
  "type": "events_batch",
  "events": [...],
  "oldest_offset": 12200,
  "has_more": true
}
```
Response to `fetch_older` or initial connection.

```json
{ "type": "event", "event": {...}, "offset": 12350 }
```
Live event with explicit offset.

### Connection Flow

1. Client connects with initial filters in query string
2. Server sends `events_batch` with latest 100 events + `has_more` flag
3. Server streams live events as they occur
4. Client sends `fetch_older` when user scrolls to top
5. Client sends `set_filters` when filters change (server sends fresh latest batch)

## Implementation

### Backend Changes

**`vibes-server/src/ws/firehose.rs`:**
- Add message handler for `fetch_older` and `set_filters`
- `fetch_older`: Create ephemeral consumer, seek to `offset - limit`, poll, return batch
- `set_filters`: Store filters in connection state, apply to live + fetches
- Include `offset` field in all event messages

### Frontend Changes

| File | Changes |
|------|---------|
| `hooks/useFirehose.ts` | Rewrite: offset tracking, `fetchOlder()`, `setFilters()`, remove pause/buffer |
| `pages/Firehose.tsx` | Wire up filter chips, add search input, remove Pause/Clear buttons |
| `components/StreamView.tsx` | Virtualized list, scroll detection for auto-follow and fetch-older |
| `components/EventCard.tsx` | Timestamp format `HH:mm:ss.SSS` |

### Scroll Position Preservation

When older events load, adjust scroll position to compensate for new content above. Virtual scrollers handle this with `scrollToOffset` adjustments.

### Edge Cases

- **Reached beginning:** `has_more: false` — show "Beginning of history" marker
- **WebSocket reconnect:** Re-fetch from last known offset to fill gaps
- **Filter change:** Jump to latest, clear loaded history, start fresh

## Acceptance Criteria

### Core Functionality
- [ ] Events display oldest-at-top, newest-at-bottom
- [ ] New events appear at bottom with auto-scroll (when following)
- [ ] Scrolling up loads older events infinitely (until beginning)
- [ ] Scrolling to bottom resumes auto-follow
- [ ] "Jump to latest" floating button appears when not following
- [ ] Timestamps show millisecond precision (`HH:mm:ss.SSS`)

### Controls
- [ ] Pause button removed
- [ ] Clear button removed
- [ ] Type filter chips work (toggle event types)
- [ ] Session filter input works
- [ ] Search input filters events by content
- [ ] Changing filters jumps to latest

### Performance
- [ ] Virtualized scrolling handles 10K+ events smoothly
- [ ] Scroll position preserved when older events load
- [ ] Loading indicator shown while fetching older events

### Edge Cases
- [ ] "Beginning of history" indicator when scrolled to first event
- [ ] Graceful reconnect after WebSocket disconnect
- [ ] Empty state when no events match filters
