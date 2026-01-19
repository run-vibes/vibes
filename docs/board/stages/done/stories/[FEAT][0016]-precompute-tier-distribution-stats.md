---
id: FEAT0016
title: Pre-compute tier distribution stats for dashboards
type: feat
status: done
priority: medium
scope: plugin-system
depends: []
estimate: 4h
created: 2026-01-08
---

# Pre-compute tier distribution stats for dashboards

## Summary

The "Tier Distribution" stats on `/groove/assessment/status` currently only show data from the last 100 assessments. Stats should be pre-computed as assessments stream in, enabling fast retrieval for dashboards without re-scanning all events.

## Requirements

- Emit a stats snapshot on every assessment event (real-time updates)
- Persist snapshots to a dedicated Iggy topic with message-count retention
- On recovery, read latest snapshot and replay from that offset to rebuild current state
- Precompute both global and per-session tier distributions

## Technical Approach

### Stats Snapshot Structure
```rust
struct StatsSnapshot {
    // Global tier distribution
    tier_distribution: TierDistribution,
    // Per-session tier distribution
    session_stats: HashMap<SessionId, TierDistribution>,
    // Total assessment count
    total_assessments: usize,
    // Last processed assessment event offset (for recovery)
    last_offset: u64,
    // Timestamp of this snapshot
    timestamp: DateTime<Utc>,
}
```

### Persistence Pattern (Standard for Aggregations)
1. **Stats Accumulator**: Maintain in-memory `StatsSnapshot` that updates incrementally
2. **Emit on Every Event**: After processing each assessment, serialize snapshot to Iggy topic
3. **Topic Retention**: Keep last 100 snapshots (configurable via `stats_retention_count`)
4. **Recovery**: On startup, read latest snapshot, set offset, replay any events after that offset

### Implementation Steps
1. Add `StatsAccumulator` struct with `update(result: &PluginAssessmentResult)` method
2. Create `assessment-stats` Iggy topic with retention config
3. Wire accumulator into `SyncAssessmentProcessor` to emit after each result
4. Update `/assess/stats` endpoint to read from accumulator instead of scanning
5. Add recovery logic to consumer startup

## Acceptance Criteria

- [x] Tier distribution reflects all assessments, not just last 100
- [x] Stats endpoint returns in <50ms regardless of total assessment count
- [x] Stats survive server restart (rebuilds from last snapshot + replay)
- [x] Per-session stats are available without re-scanning events
- [x] Retention is configurable (default: 100 snapshots)
