# Milestone 44: Open-World Dashboard - Implementation Plan

## Stories

| ID | Title | Status | Estimate | Depends On |
|----|-------|--------|----------|------------|
| FEAT0079 | OpenWorld page and routing | pending | 2h | M33 |
| FEAT0080 | OpenWorld backend data providers | pending | 3h | FEAT0079 |
| FEAT0081 | Novelty tab components | pending | 2h | FEAT0080 |
| FEAT0082 | Gaps tab with split view | pending | 3h | FEAT0080 |
| FEAT0083 | Solutions tab with actions | pending | 2h | FEAT0080 |
| FEAT0084 | Activity tab with live updates | pending | 2h | FEAT0080 |

## Dependency Graph

```
M33 (Groove Dashboard) ──────────────────────────────────────────────┐
M34 (Open-World Adaptation)                                          │
        │                                                            │
        ▼                                                            │
FEAT0079 (page + routing) ───────────────────────────────────────────┤
        │                                                            │
        ▼                                                            │
FEAT0080 (backend providers) ────────────────────────────────────────┤
        │                                                            │
        ├──────────────┬──────────────┬──────────────┐               │
        ▼              ▼              ▼              ▼               │
FEAT0081 (novelty) FEAT0082 (gaps) FEAT0083 (solutions) FEAT0084 (activity)
```

## Execution Order

**Phase 1 - Foundation:**
- FEAT0079: OpenWorld page and routing

**Phase 2 - Backend:**
- FEAT0080: OpenWorld backend data providers

**Phase 3 - UI (parallel after backend):**
- FEAT0081: Novelty tab components
- FEAT0082: Gaps tab with split view
- FEAT0083: Solutions tab with actions
- FEAT0084: Activity tab with live updates

---

## FEAT0079: OpenWorld Page and Routing

**Goal:** Add OpenWorld tab to dashboard navigation with placeholder content.

### Steps

1. Update `web-ui/src/pages/dashboard/DashboardLayout.tsx`:
   - Add "OpenWorld" to tab list
   - Add route for `/dashboard/openworld`

2. Create `web-ui/src/pages/dashboard/DashboardOpenWorld.tsx`:
   - Internal tabs: Novelty, Gaps, Solutions, Activity
   - Placeholder content for each tab

3. Create `web-ui/src/pages/dashboard/DashboardOpenWorld.css`:
   - Tab styling following CRT design system
   - Layout for split views

4. Update `web-ui/src/pages/dashboard/index.ts`:
   - Export DashboardOpenWorld

5. Add tests for routing

### Verification

```bash
npm test --workspace=web-ui -- --run
npm run build --workspace=web-ui
```

---

## FEAT0080: OpenWorld Backend Data Providers

**Goal:** Add WebSocket topics and handlers for openworld data.

### Steps

1. Update `plugins/vibes-groove/src/dashboard/types.rs`:
   ```rust
   pub enum DashboardTopic {
       // Existing...
       OpenWorldOverview,
       OpenWorldGaps,
       OpenWorldGapDetail(GapId),
       OpenWorldSolutions,
       OpenWorldActivity,
   }

   pub struct OpenWorldOverviewData {
       pub hook_stats: HookStats,
       pub gap_counts: GapCounts,
       pub recent_clusters: Vec<ClusterBrief>,
   }

   pub struct GapCounts {
       pub open: usize,
       pub investigating: usize,
       pub resolved: usize,
       pub critical: usize,
       pub medium: usize,
       pub low: usize,
   }

   pub struct GapListData {
       pub gaps: Vec<GapBrief>,
   }

   pub struct GapBrief {
       pub id: GapId,
       pub description: String,
       pub severity: GapSeverity,
       pub status: GapStatus,
       pub failure_count: usize,
       pub solution_count: usize,
       pub created_at: DateTime<Utc>,
   }

   pub struct GapDetailData {
       pub gap: CapabilityGap,
       pub failure_records: Vec<FailureRecord>,
       pub solutions: Vec<SuggestedSolution>,
   }

   pub struct OpenWorldActivityData {
       pub hook_stats: HookStats,
       pub recent_events: Vec<ActivityEntry>,
   }
   ```

2. Update `plugins/vibes-groove/src/dashboard/handler.rs`:
   - Add `handle_openworld_overview()`
   - Add `handle_openworld_gaps()`
   - Add `handle_openworld_gap_detail()`
   - Add `handle_openworld_solutions()`
   - Add `handle_openworld_activity()`

3. Wire handlers in message routing

4. Update `web-ui/src/hooks/useDashboard.ts`:
   - Add types for openworld data
   - Add subscription helpers

5. Add tests

### Verification

```bash
cargo test -p vibes-groove dashboard
npm test --workspace=web-ui -- --run
```

---

## FEAT0081: Novelty Tab Components

**Goal:** Implement novelty detection visualization.

### Steps

1. Create `web-ui/src/components/dashboard/openworld/`:
   - `NoveltyStats.tsx` - Stats cards (threshold, pending, clusters)
   - `ClusterList.tsx` - Recent clusters table
   - `ClusterItem.tsx` - Single cluster row

2. Update `DashboardOpenWorld.tsx`:
   - Subscribe to OpenWorldOverview topic
   - Render NoveltyStats and ClusterList

3. Style with CRT design system

4. Add tests

### Verification

```bash
npm test --workspace=web-ui -- --run
```

---

## FEAT0082: Gaps Tab with Split View

**Goal:** Implement capability gaps browser with detail panel.

### Steps

1. Create `web-ui/src/components/dashboard/openworld/`:
   - `GapsList.tsx` - Filterable gap list
   - `GapsFilters.tsx` - Severity, status, category filters
   - `GapItem.tsx` - Single gap row
   - `GapDetail.tsx` - Full gap detail panel
   - `FailureRecordList.tsx` - Failure records in detail
   - `GapSeverityBadge.tsx` - Severity indicator

2. Update `DashboardOpenWorld.tsx`:
   - Subscribe to OpenWorldGaps topic
   - Subscribe to OpenWorldGapDetail on selection
   - Split panel layout

3. Implement filters:
   - Severity: Critical, Medium, Low
   - Status: Open, Investigating, Resolved, WontFix
   - Category: CodePattern, Error, Performance, Security

4. Add tests

### Verification

```bash
npm test --workspace=web-ui -- --run
```

---

## FEAT0083: Solutions Tab with Actions

**Goal:** Implement solutions viewer with apply/dismiss actions.

### Steps

1. Create `web-ui/src/components/dashboard/openworld/`:
   - `SolutionsList.tsx` - Solutions grouped by status
   - `SolutionItem.tsx` - Single solution with actions
   - `SolutionActions.tsx` - Apply/Dismiss buttons
   - `SolutionConfidenceBadge.tsx` - Confidence indicator

2. Update `DashboardOpenWorld.tsx`:
   - Subscribe to OpenWorldSolutions topic
   - Group by status (Pending, Applied, Dismissed)

3. Implement actions in `useDashboard.ts`:
   - `applySolution(id)`
   - `dismissSolution(id)`

4. Add backend handlers for actions

5. Add confirmation dialogs

6. Add tests

### Verification

```bash
cargo test -p vibes-groove dashboard::solutions
npm test --workspace=web-ui -- --run
```

---

## FEAT0084: Activity Tab with Live Updates

**Goal:** Implement real-time response activity feed.

### Steps

1. Create `web-ui/src/components/dashboard/openworld/`:
   - `ActivityStats.tsx` - Stats cards (outcomes, negative rate, exploration)
   - `ActivityFeed.tsx` - Live event list
   - `ActivityItem.tsx` - Single activity entry
   - `ResponseActionBadge.tsx` - Action type indicator

2. Update `DashboardOpenWorld.tsx`:
   - Subscribe to OpenWorldActivity topic
   - Real-time updates from OPENWORLD_STREAM

3. Implement activity types:
   - OutlierDetected
   - ClusterFormed
   - GapCreated
   - GapEscalated
   - SolutionGenerated
   - AdjustExploration
   - Monitor

4. Add live indicator (pulsing dot)

5. Add tests

### Verification

```bash
npm test --workspace=web-ui -- --run
```

---

## Completion Checklist

- [ ] FEAT0079: OpenWorld page and routing
- [ ] FEAT0080: OpenWorld backend data providers
- [ ] FEAT0081: Novelty tab components
- [ ] FEAT0082: Gaps tab with split view
- [ ] FEAT0083: Solutions tab with actions
- [ ] FEAT0084: Activity tab with live updates
- [ ] All tests passing (`just pre-commit`)
- [ ] Documentation updated
