---
created: 2026-01-02
---

# Milestone 36: Firehose Infinite Scroll - Implementation Plan

> **For Claude:** Work through stories in order. Use superpowers:executing-plans for each story.

**Goal:** Transform the firehose from a live event buffer to a full-featured log viewer with infinite scroll through the entire eventlog history.

**Design:** See [design.md](design.md) for architecture decisions.

---

## Stories

| # | Story | Description |
|---|-------|-------------|
| 1 | [feat-01-backend-pagination](stories/feat-01-backend-pagination.md) | WebSocket protocol for fetching older events |
| 2 | [feat-02-backend-filters](stories/feat-02-backend-filters.md) | Server-side filter handling |
| 3 | [feat-03-frontend-hook](stories/feat-03-frontend-hook.md) | Rewrite useFirehose with offset tracking |
| 4 | [feat-04-virtual-scroll](stories/feat-04-virtual-scroll.md) | Virtualized scroll view with @tanstack/react-virtual |
| 5 | [feat-05-ui-polish](stories/feat-05-ui-polish.md) | Filters, timestamps, remove pause/clear |

> **Status:** Check story frontmatter or run `just board` for current status.

## Dependencies

```
Story 1 (backend pagination)
    ↓
Story 2 (backend filters) ──┐
    ↓                       │
Story 3 (frontend hook) ◄───┘
    ↓
Story 4 (virtual scroll)
    ↓
Story 5 (UI polish)
```

- Story 3 depends on Stories 1 & 2 (backend must support pagination/filters)
- Story 4 depends on Story 3 (hook must provide data)
- Story 5 depends on Story 4 (view must exist for polish)

## Completion Criteria

- [ ] All stories merged
- [ ] Full eventlog history accessible via scroll
- [ ] Filters working (type, session, search)
- [ ] Auto-follow behavior correct
- [ ] Integration tests passing
