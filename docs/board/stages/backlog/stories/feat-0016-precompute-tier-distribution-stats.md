---
id: FEAT0016
title: Pre-compute tier distribution stats for dashboards
type: feat
status: backlog
priority: medium
epics: [core,plugin-system]
depends: []
estimate: 4h
created: 2026-01-08
updated: 2026-01-08
milestone: 29-assessment-framework
---

# Pre-compute tier distribution stats for dashboards

## Summary

The "Tier Distribution" stats on `/groove/assessment/status` currently only show data from the last 100 assessments. Stats should be pre-computed as assessments stream in, enabling fast retrieval for dashboards without re-scanning all events.

## Requirements

- Maintain running counters for tier distribution as assessments are processed
- Store aggregated stats in a queryable format (in-memory or persisted)
- Support time-windowed stats (e.g., last hour, last day, all time)
- Expose stats via dedicated endpoint for fast dashboard queries

## Technical Approach

- Add stats accumulator in the assessment processor
- Update counters on each new assessment result
- Consider using Iggy consumer offset tracking for recovery after restart

## Acceptance Criteria

- [ ] Tier distribution reflects all assessments, not just last 100
- [ ] Stats endpoint returns in <50ms regardless of total assessment count
- [ ] Stats survive server restart (or rebuild quickly on startup)
