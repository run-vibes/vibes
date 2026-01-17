# Milestone 33: Groove Dashboard - Implementation Plan

## Stories

| ID | Title | Status | Estimate | Depends On |
|----|-------|--------|----------|------------|
| FEAT0042 | Dashboard layout and routing | pending | 2h | - |
| FEAT0043 | WebSocket dashboard endpoint | pending | 3h | FEAT0042 |
| FEAT0044 | Overview page with cards | pending | 3h | FEAT0043 |
| FEAT0045 | Learnings page (split view) | pending | 4h | FEAT0043, M30 |
| FEAT0046 | Learning actions (enable/disable/delete) | pending | 2h | FEAT0045 |
| FEAT0047 | Attribution page (leaderboard + timeline) | pending | 4h | FEAT0043, M31 |
| FEAT0048 | Strategy page (distributions + overrides) | pending | 3h | FEAT0043, M32 |
| FEAT0049 | Health page | pending | 2h | FEAT0043 |
| FEAT0050 | Sparkline and chart components | pending | 3h | FEAT0044 |
| FEAT0051 | Learning indicator (Settings + Header) | pending | 2h | FEAT0043 |

## Dependency Graph

```
                    M30 (Learning Extraction)
                              │
                    M31 (Attribution Engine)
                              │
                    M32 (Adaptive Strategies)
                              │
FEAT0042 (layout) ────────────┼────────────────────────────────────┐
        │                     │                                    │
        ▼                     │                                    │
FEAT0043 (WebSocket) ─────────┴────────────────────────────────────┤
        │                                                          │
        ├──────────────┬──────────────┬──────────────┬────────────┤
        ▼              ▼              ▼              ▼            │
FEAT0044 (overview)  FEAT0045 (learnings)  FEAT0047 (attr)  FEAT0048 (strategy)
        │                   │              │              │        │
        │                   ▼              │              │        │
        │            FEAT0046 (actions)    │              │        │
        │                                  │              │        │
        ├──────────────────────────────────┴──────────────┘        │
        ▼                                                          │
FEAT0050 (charts) ─────────────────────────────────────────────────┤
                                                                   │
FEAT0049 (health) ◄────────────────────────────────────────────────┤
                                                                   │
FEAT0051 (indicator) ◄─────────────────────────────────────────────┘
```

## Execution Order

**Phase 1 - Foundation:**
- FEAT0042: Dashboard layout and routing

**Phase 2 - Backend:**
- FEAT0043: WebSocket dashboard endpoint

**Phase 3 - Core Pages (parallel after WebSocket):**
- FEAT0044: Overview page with cards
- FEAT0045: Learnings page (split view)
- FEAT0047: Attribution page
- FEAT0048: Strategy page
- FEAT0049: Health page

**Phase 4 - Features:**
- FEAT0046: Learning actions
- FEAT0050: Sparkline and chart components
- FEAT0051: Learning indicator

---

## FEAT0042: Dashboard Layout and Routing

**Goal:** Create dashboard navigation structure extending groove subnav.

### Steps

1. Update `web-ui/src/App.tsx`:
   - Add Dashboard to grooveSubnavItems
   - Create dashboard layout route
   - Add child routes for each page
2. Create `web-ui/src/pages/dashboard/DashboardLayout.tsx`:
   - Internal tabs: Overview, Learnings, Attribution, Strategy, Health
   - Follow AssessmentLayout pattern
3. Create `web-ui/src/pages/dashboard/DashboardLayout.css`
4. Create placeholder pages:
   - `DashboardOverview.tsx`
   - `DashboardLearnings.tsx`
   - `DashboardAttribution.tsx`
   - `DashboardStrategy.tsx`
   - `DashboardHealth.tsx`
5. Create `web-ui/src/pages/dashboard/index.ts` exports
6. Add tests for routing

### Verification

```bash
npm test --workspace=web-ui -- --run
npm run build --workspace=web-ui
```

---

## FEAT0043: WebSocket Dashboard Endpoint

**Goal:** Create `/ws/groove/dashboard` WebSocket endpoint with topic subscriptions.

### Steps

1. Create `plugins/vibes-groove/src/dashboard/` module
2. Define types in `types.rs`:
   - `DashboardTopic` enum
   - `DashboardMessage` enum
   - `DashboardRequest` enum
   - Data structures for each topic
3. Create `handler.rs`:
   - `DashboardHandler` struct
   - Subscribe/unsubscribe logic
   - Topic-specific data providers
4. Create `websocket.rs`:
   - WebSocket upgrade handler
   - Message routing
   - Connection management
5. Wire into server routes
6. Create `web-ui/src/hooks/useDashboard.ts`:
   - WebSocket connection management
   - Topic subscription helpers
   - Data caching
7. Add tests for WebSocket protocol

### Verification

```bash
cargo test -p vibes-groove dashboard
npm test --workspace=web-ui -- --run
```

---

## FEAT0044: Overview Page with Cards

**Goal:** Implement grid of summary cards with drill-down links.

### Steps

1. Create card components in `web-ui/src/components/dashboard/`:
   - `TrendCard.tsx` - Sparkline placeholder, metrics, trend
   - `LearningsCard.tsx` - Counts, recent list
   - `AttributionCard.tsx` - Top contributors, warnings
   - `HealthCard.tsx` - Progress bars, status
2. Update `DashboardOverview.tsx`:
   - Subscribe to Overview topic
   - Grid layout with responsive breakpoints
   - Card click navigates to detail pages
3. Add CSS for card grid layout
4. Add loading and error states
5. Add tests for overview components

### Verification

```bash
npm test --workspace=web-ui -- --run
```

---

## FEAT0045: Learnings Page (Split View)

**Goal:** Implement split view with filterable list and detail panel.

### Steps

1. Create `web-ui/src/components/dashboard/learnings/`:
   - `LearningsList.tsx` - Filterable, sortable list
   - `LearningsFilters.tsx` - Scope, category, status, sort dropdowns
   - `LearningDetail.tsx` - Full metrics, source, actions
   - `LearningStatusBadge.tsx` - Status indicators
   - `ValueBar.tsx` - Visual value indicator
2. Update `DashboardLearnings.tsx`:
   - Subscribe to Learnings topic with filters
   - Subscribe to LearningDetail on selection
   - Split panel layout
3. Implement filtering:
   - Scope: Project, User, Global
   - Category: Correction, ErrorRecovery, Pattern, Preference
   - Status: Active, Disabled, UnderReview, Deprecated
   - Sort: Value, Confidence, Usage, Recency
4. Add responsive behavior (stack on mobile)
5. Add tests

### Verification

```bash
npm test --workspace=web-ui -- --run
```

---

## FEAT0046: Learning Actions (Enable/Disable/Delete)

**Goal:** Add learning management actions with confirmation dialogs.

### Steps

1. Create `web-ui/src/components/dashboard/learnings/`:
   - `LearningActions.tsx` - Action buttons
   - `ConfirmDialog.tsx` - Reusable confirmation modal
2. Implement actions in `useDashboard.ts`:
   - `disableLearning(id)`
   - `enableLearning(id)`
   - `deleteLearning(id)`
3. Add backend handlers in `plugins/vibes-groove/src/dashboard/handler.rs`:
   - Process action requests
   - Update learning store
   - Broadcast updates to subscribers
4. Add confirmation dialogs:
   - Disable: "This learning won't be injected. Continue?"
   - Delete: "This will permanently remove the learning. Continue?"
5. Add optimistic updates with rollback on error
6. Add tests

### Verification

```bash
cargo test -p vibes-groove dashboard::actions
npm test --workspace=web-ui -- --run
```

---

## FEAT0047: Attribution Page (Leaderboard + Timeline)

**Goal:** Implement dual-view attribution page.

### Steps

1. Create `web-ui/src/components/dashboard/attribution/`:
   - `AttributionTabs.tsx` - Leaderboard / Timeline toggle
   - `Leaderboard.tsx` - Ranked contributors list
   - `ContributorCard.tsx` - Learning with value, confidence
   - `NegativeImpact.tsx` - Negative learnings section
   - `AblationCoverage.tsx` - Progress bar with stats
   - `SessionTimeline.tsx` - Timeline view
   - `SessionTimelineItem.tsx` - Individual session row
2. Update `DashboardAttribution.tsx`:
   - Tab state management
   - Subscribe to Attribution or SessionTimeline topic
   - Period selector (7/30/90 days)
3. Implement timeline grouping (Today, Yesterday, etc.)
4. Add tests

### Verification

```bash
npm test --workspace=web-ui -- --run
```

---

## FEAT0048: Strategy Page (Distributions + Overrides)

**Goal:** Implement strategy visualization page.

### Steps

1. Create `web-ui/src/components/dashboard/strategy/`:
   - `StrategyTabs.tsx` - Distributions / Overrides toggle
   - `DistributionCard.tsx` - Category distribution with bars
   - `StrategyBar.tsx` - Visual weight bar
   - `OverridesList.tsx` - Learning overrides list
   - `OverrideItem.tsx` - Individual override row
2. Update `DashboardStrategy.tsx`:
   - Tab state management
   - Subscribe to StrategyDistributions or StrategyOverrides
   - Filter for specialized only
3. Implement distribution visualization:
   - Bar chart per strategy variant
   - Session count display
4. Add tests

### Verification

```bash
npm test --workspace=web-ui -- --run
```

---

## FEAT0049: Health Page

**Goal:** Implement system health monitoring page.

### Steps

1. Create `web-ui/src/components/dashboard/health/`:
   - `SystemStatusBanner.tsx` - Overall status with indicator
   - `SubsystemCard.tsx` - Individual subsystem status
   - `AdaptiveParamsTable.tsx` - Parameter table with trends
   - `RecentActivity.tsx` - Activity feed
2. Update `DashboardHealth.tsx`:
   - Subscribe to Health topic
   - Grid of subsystem cards
   - Parameter table
   - Activity feed
3. Implement status indicators:
   - Green (●) - Operational
   - Yellow (◐) - Degraded
   - Red (○) - Error
4. Add tests

### Verification

```bash
npm test --workspace=web-ui -- --run
```

---

## FEAT0050: Sparkline and Chart Components

**Goal:** Create reusable chart components using visx.

### Steps

1. Create `web-ui/src/components/charts/`:
   - `Sparkline.tsx` - Compact inline chart for cards
   - `TrendChart.tsx` - Full session trend chart
   - `ProgressBar.tsx` - Horizontal progress indicator
   - `ValueBar.tsx` - Value indicator (-1 to +1 range)
2. Add visx imports (already in project):
   - `@visx/shape` for lines
   - `@visx/scale` for axes
   - `@visx/responsive` for sizing
3. Apply CRT styling:
   - Phosphor green colors
   - Scan line effects (optional)
   - Monospace labels
4. Create stories/tests for each component
5. Update Overview cards to use real sparklines
6. Add tests

### Verification

```bash
npm test --workspace=web-ui -- --run
```

---

## FEAT0051: Learning Indicator (Settings + Header)

**Goal:** Add opt-in learning indicator with Settings toggle.

### Steps

1. Update `web-ui/src/pages/Settings.tsx`:
   - Add GROOVE section
   - Learning Indicator toggle
   - Dashboard Auto-Refresh toggle
2. Create `web-ui/src/hooks/useGrooveSettings.ts`:
   - Load/save groove preferences
   - localStorage persistence
3. Create `web-ui/src/components/LearningIndicator.tsx`:
   - 🧠 icon with states (idle, active, error)
   - Pulsing animation for active state
   - Click to expand status
4. Update Header component:
   - Show indicator when enabled
   - Subscribe to groove activity events
5. Add indicator states:
   - Hidden (default)
   - Idle (static 🧠)
   - Active (pulsing 🧠)
   - Error (red 🧠)
6. Add tests

### Verification

```bash
npm test --workspace=web-ui -- --run
```

---

## Completion Checklist

- [ ] FEAT0042: Dashboard layout and routing
- [ ] FEAT0043: WebSocket dashboard endpoint
- [ ] FEAT0044: Overview page with cards
- [ ] FEAT0045: Learnings page (split view)
- [ ] FEAT0046: Learning actions (enable/disable/delete)
- [ ] FEAT0047: Attribution page (leaderboard + timeline)
- [ ] FEAT0048: Strategy page (distributions + overrides)
- [ ] FEAT0049: Health page
- [ ] FEAT0050: Sparkline and chart components
- [ ] FEAT0051: Learning indicator (Settings + Header)
- [ ] All tests passing (`just test`)
- [ ] Pre-commit checks passing (`just pre-commit`)
- [ ] Documentation updated
