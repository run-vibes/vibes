---
created: 2026-01-02
status: pending
---

# feat-0004: Firehose Infinite Scroll

## Summary

Transform the firehose from a live event buffer to a full-featured log viewer with infinite scroll through the entire eventlog history. Events display oldest-at-top, newest-at-bottom with auto-follow behavior.

See [design document](../../plans/2026-01-02-firehose-infinite-scroll-design.md) for full details.

## Key Changes

- **Infinite scroll** — Load older events on scroll-up, all the way to first event
- **Reverse chronological** — Oldest at top, newest at bottom (like `tail -f`)
- **Auto-follow** — New events auto-scroll unless user scrolls up
- **"Jump to latest" button** — Floating button to resume following
- **Working filters** — Type chips, session filter, and search all functional
- **Millisecond timestamps** — `HH:mm:ss.SSS` format
- **Remove Pause/Clear** — Not needed with persistent history

## Tasks

### Task 1: Backend Protocol
- Add `fetch_older` message handler (seek + poll EventLog)
- Add `set_filters` message handler
- Include `offset` in all event messages
- Return `events_batch` with `has_more` flag

### Task 2: useFirehose Hook Rewrite
- Track oldest/newest offsets
- Implement `fetchOlder()` for pagination
- Implement `setFilters()` for filter changes
- Remove pause/buffer logic
- Track `isFollowing` state

### Task 3: Virtualized ScrollView
- Add `@tanstack/react-virtual` dependency
- Replace list with virtualized scroller
- Scroll position preservation on older events load
- Scroll detection for fetch-older trigger
- Auto-follow detection (at bottom?)

### Task 4: UI Updates
- Remove Pause and Clear buttons
- Wire up type filter chips
- Wire up session filter input
- Add search input with client-side filtering
- Add floating "Jump to latest" button
- Update timestamp format to `HH:mm:ss.SSS`

### Task 5: Edge Cases
- "Beginning of history" marker
- Loading indicator while fetching
- Empty state for no matching events
- Reconnection handling

## Acceptance Criteria

- [ ] Events display oldest-at-top, newest-at-bottom
- [ ] New events appear at bottom with auto-scroll (when following)
- [ ] Scrolling up loads older events infinitely (until beginning)
- [ ] Scrolling to bottom resumes auto-follow
- [ ] "Jump to latest" floating button appears when not following
- [ ] Timestamps show millisecond precision (`HH:mm:ss.SSS`)
- [ ] Pause button removed
- [ ] Clear button removed
- [ ] Type filter chips work (toggle event types)
- [ ] Session filter input works
- [ ] Search input filters events by content
- [ ] Changing filters jumps to latest
- [ ] Virtualized scrolling handles 10K+ events smoothly
- [ ] Scroll position preserved when older events load
- [ ] "Beginning of history" indicator when scrolled to first event
