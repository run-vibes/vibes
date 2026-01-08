---
id: FEAT0014
title: Production Iggy polling for assessment queries
type: feat
status: backlog
priority: medium
epics: [core,plugin-system]
depends: []
estimate: 4h
created: 2026-01-08
updated: 2026-01-08
---

# Production Iggy polling for assessment queries

## Summary

Wire the assessment history query endpoint to poll real Iggy event log data instead of returning mock data. The HTTP routes exist (`/groove/assess/history`) but currently the implementation uses placeholder data.

## Current State

- `GroovePlugin::get_assessment_history()` returns mock data
- EventLog is available via `SharedState`
- Need to query Iggy for assessment-related events

## Acceptance Criteria

- [ ] `/groove/assess/history` returns real assessment events from Iggy
- [ ] Pagination works with cursor-based navigation
- [ ] Filtering by session ID supported
- [ ] Response format matches existing API contract
- [ ] Unit tests cover happy path and edge cases

## Implementation Notes

### Query Pattern

```rust
// In GroovePlugin::get_assessment_history()
let events = self.eventlog
    .query_events(QueryOptions {
        topic: "groove.assessments",
        cursor: params.cursor,
        limit: params.limit,
        filter: params.session_id.map(|s| EventFilter::Session(s)),
    })
    .await?;
```

### Key Files

- `plugins/vibes-groove/src/plugin.rs` - History query implementation
- `plugins/vibes-groove/src/routes.rs` - HTTP handler (already wired)

### Event Schema

Need to define or use existing assessment event schema:
- `assessment.lightweight` - Pattern detection events
- `assessment.checkpoint` - Medium assessments
- `assessment.complete` - Full session assessments
