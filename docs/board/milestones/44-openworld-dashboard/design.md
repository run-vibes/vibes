---
created: 2026-01-11
---

# Milestone 44: Open-World Dashboard - Design

> Add OpenWorld page to groove dashboard for visualizing novelty detection, capability gaps, solutions, and response activity.

## Overview

Extends the groove dashboard (M33) with a dedicated OpenWorld tab that surfaces data from the open-world adaptation system (M34). Users can monitor novelty detection, browse capability gaps, review suggested solutions, and observe graduated response activity in real-time.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Integration point | New tab in existing dashboard | Reuses WebSocket infrastructure, consistent UX |
| Tab structure | 4 tabs (Novelty, Gaps, Solutions, Activity) | Matches mental model of openworld pipeline |
| Real-time updates | Subscribe to OPENWORLD_STREAM | Leverage existing Iggy event system |
| Gap detail | Split view (list + detail) | Follows Learnings page pattern |

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Groove Dashboard                              │
├─────────┬──────────┬─────────────┬──────────┬─────────┬────────────┤
│ Overview│ Learnings│ Attribution │ Strategy │ Health  │ OpenWorld  │
└─────────┴──────────┴─────────────┴──────────┴─────────┴────────────┘
                                                              │
                                                              ▼
                    ┌─────────────────────────────────────────────────┐
                    │              DashboardOpenWorld                  │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │              Tabs                          │   │
                    │  │  [Novelty] [Gaps] [Solutions] [Activity]  │   │
                    │  └──────────────────────────────────────────┘   │
                    │                                                  │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │           Tab Content                      │   │
                    │  │  - NoveltyPanel (stats, clusters)         │   │
                    │  │  - GapsPanel (list, detail)               │   │
                    │  │  - SolutionsPanel (per-gap solutions)     │   │
                    │  │  - ActivityPanel (response log)           │   │
                    │  └──────────────────────────────────────────┘   │
                    └─────────────────────────────────────────────────┘
                                          │
                                          ▼
                    ┌─────────────────────────────────────────────────┐
                    │            WebSocket Dashboard                   │
                    │  Topics: OpenWorldOverview, GapDetail,          │
                    │          OpenWorldActivity                       │
                    └─────────────────────────────────────────────────┘
                                          │
                                          ▼
                    ┌─────────────────────────────────────────────────┐
                    │              DashboardHandler                    │
                    │  - Query OpenWorldStore (gaps, solutions)       │
                    │  - Query OpenWorldHook (stats, tracking)        │
                    │  - Subscribe OPENWORLD_STREAM (activity)        │
                    └─────────────────────────────────────────────────┘
```

## Data Sources

### From OpenWorldStore
- `list_gaps()` - All capability gaps with filters
- `get_gap(id)` - Gap details with failure records
- `list_solutions(gap_id)` - Solutions for a gap

### From OpenWorldHook
- `stats()` - HookStats (outcomes_processed, negative_outcomes, gaps_created)

### From OPENWORLD_STREAM (Iggy)
- `OpenWorldEvent::OutlierDetected`
- `OpenWorldEvent::ClusterFormed`
- `OpenWorldEvent::GapCreated`
- `OpenWorldEvent::GapEscalated`
- `OpenWorldEvent::SolutionGenerated`
- `OpenWorldEvent::ResponseAction`

## UI Components

### 1. Novelty Tab

Stats cards showing threshold, pending outliers, cluster count. Table of recent clusters with category and member count.

### 2. Gaps Tab

Split view with filterable gap list (by severity, status, category) and detail panel showing failure records and solutions.

### 3. Solutions Tab

List of suggested solutions grouped by status (Pending Review, Applied, Dismissed). Actions to apply or dismiss solutions.

### 4. Activity Tab

Live feed of response actions with stats cards for outcomes processed, negative rate, and exploration adjustment.

## WebSocket Topics

```rust
pub enum DashboardTopic {
    // Existing...

    // New for openworld
    OpenWorldOverview,          // Stats + recent clusters + gap counts
    OpenWorldGaps,              // Gap list with filters
    OpenWorldGapDetail(GapId),  // Single gap with failures + solutions
    OpenWorldSolutions,         // All solutions with status
    OpenWorldActivity,          // Live event stream
}
```

## Styling

Follow CRT design system (M27):
- Phosphor green (#00ff41) for positive
- Amber (#ffb000) for warnings
- Red (#ff4136) for critical
- Monospace font, scan line effects
- Status indicators: ● (good), ◐ (warning), ○ (error)

## Deliverables

- [ ] OpenWorld tab added to dashboard navigation
- [ ] Backend data providers for all topics
- [ ] Novelty panel with stats and cluster list
- [ ] Gaps panel with split view
- [ ] Solutions panel with actions
- [ ] Activity panel with live updates
- [ ] Tests for all components
