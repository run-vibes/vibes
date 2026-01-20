---
id: FEAT0134
title: "Feature: Add Assessment Query UI to Web Dashboard"
type: feat
status: done
priority: medium
scope: groove/06-assessment-pipeline
depends: [m26-feat-10-cli-assess-queries]
estimate:
created: 2026-01-07
---

# Feature: Add Assessment Query UI to Web Dashboard

## Problem

The web UI Assessment page (`/groove/assessment`) only shows a real-time stream of assessment events. Users cannot:
- View assessment history for a specific session
- See circuit breaker status and health
- Query past assessments with filters
- View aggregate statistics

The CLI is getting these capabilities via `assess status` and `assess history` commands (m26-feat-10), but there's no web UI equivalent.

## Goal

Add assessment query and status features to the web UI, complementing the real-time stream with historical queries and system health visibility.

## Design

### Page Structure & Navigation

Separate pages with distinct URLs and shared subnav:

```
/groove/assessment          → AssessmentStream (existing, renamed)
/groove/assessment/status   → AssessmentStatus (new)
/groove/assessment/history  → AssessmentHistory (new)
/groove/assessment/stats    → AssessmentStats (new)
```

Subnav bar on all assessment pages:
```
[Stream]  [Status]  [History]  [Stats]
```

File organization:
```
web-ui/src/pages/assessment/
├── AssessmentLayout.tsx      # Shared layout with subnav
├── AssessmentStream.tsx      # Existing page, moved here
├── AssessmentStatus.tsx      # Task 1
├── AssessmentHistory.tsx     # Task 2
├── AssessmentStats.tsx       # Task 4
├── components/
│   ├── StatusPanel.tsx       # Reusable status display
│   ├── SessionSelector.tsx   # Session picker for history
│   └── charts/               # visx chart components
└── hooks/
    └── useAssessmentApi.ts   # React Query hooks for new endpoints
```

### Status Page (`/groove/assessment/status`)

Dashboard of status panels:

```
┌─────────────────────┬─────────────────────┬─────────────────┐
│  CIRCUIT BREAKER    │  SAMPLING           │  ACTIVITY       │
│  State: ● Closed    │  Base rate: 20%     │  Sessions: 12   │
│  Cooldown: 120s     │  Burnin: 10         │  Events: 401    │
│  Max/session: 3     │                     │                 │
└─────────────────────┴─────────────────────┴─────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  ACTIVE SESSIONS                                            │
│  cf4b3915-23be-490d-b3b5-84ff3d82ad74  →  [View History]   │
│  9841098a-5d70-49f6-8408-e92c7178513e  →  [View History]   │
└─────────────────────────────────────────────────────────────┘
```

- Fetch from `GET /api/groove/assess/status`
- Auto-refresh every 5 seconds via React Query
- Circuit breaker state uses semantic status colors

### History Page (`/groove/assessment/history`)

Two-panel layout: session selector + event timeline.

```
┌──────────────────────┬──────────────────────────────────────┐
│  SESSIONS            │  SESSION TIMELINE                    │
│  🔍 [Search...]      │  cf4b3915-23be-490d-b3b5-84ff3d82ad74│
│                      │                                      │
│  ● cf4b3915 (5)      │  12:34:56  lightweight  +2 signals   │
│    9841098a (1)      │  12:34:42  checkpoint   pattern match│
│    14482c79 (3)      │  12:33:18  lightweight  -1 signal    │
└──────────────────────┴──────────────────────────────────────┘
```

- Session list from `/api/groove/assess/history`
- Session detail from `/api/groove/assess/history?session=<id>`
- URL state: `/groove/assessment/history?session=cf4b3915-...`
- Reuse existing `toDisplayEvent()` and `EventInspector` components

### Stats Page (`/groove/assessment/stats`)

Dashboard with visx charts styled with CRT phosphor colors.

```
┌─────────────────────────────┬─────────────────────────────────┐
│  ASSESSMENTS BY TIER        │  ASSESSMENTS OVER TIME          │
│      ██████ lightweight 72% │    ▄      ▄▄                    │
│      ███    medium      18% │   ▄█▄    ▄██▄   ▄               │
│      █      heavy       10% │  ▄███▄  ▄████▄ ▄█▄              │
└─────────────────────────────┴─────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  TOP SESSIONS BY ASSESSMENT COUNT                               │
│  cf4b3915-23be-490d...  ████████████████████████████  28       │
│  9841098a-5d70-49f6...  ██████████████████            18       │
└─────────────────────────────────────────────────────────────────┘
```

Chart components using visx:
- **TierDistribution** - Horizontal bar chart (% by tier)
- **AssessmentsOverTime** - Area/line chart (daily counts)
- **TopSessions** - Horizontal bar chart (clickable → history)

visx packages: `@visx/shape`, `@visx/scale`, `@visx/axis`, `@visx/group`, `@visx/responsive`

### New API Endpoint

Add `GET /api/groove/assess/stats`:

```json
{
  "by_tier": { "lightweight": 72, "medium": 18, "heavy": 10 },
  "over_time": [{ "date": "2026-01-07", "count": 45 }, ...],
  "top_sessions": [{ "session_id": "...", "count": 28 }, ...]
}
```

## Implementation Order

1. **Restructure files** - Create `assessment/` directory, move existing stream page
2. **Shared layout** - Create `AssessmentLayout` with subnav, update routes
3. **API endpoint** - Add `/assess/stats` endpoint, test with curl
4. **Status page** - Fetching and displaying data
5. **History page** - Session selector + timeline
6. **Stats page** - Charts with visx

## Acceptance Criteria

- [ ] Subnav shows on all assessment pages with active state
- [ ] Status page shows real circuit breaker state with semantic colors
- [ ] Status page auto-refreshes every 5 seconds
- [ ] History page lists all sessions with event counts
- [ ] Can view assessment timeline for any session
- [ ] History page supports URL state for direct linking
- [ ] Stats page shows tier distribution chart
- [ ] Stats page shows assessments over time chart
- [ ] Stats page shows top sessions chart
- [ ] All charts use CRT design system tokens
- [ ] New `/assess/stats` API endpoint returns aggregate data
