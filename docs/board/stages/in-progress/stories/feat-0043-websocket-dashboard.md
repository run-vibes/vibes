---
id: FEAT0043
title: WebSocket dashboard endpoint
type: feat
status: in-progress
priority: high
epics: [plugin-system]
depends: [FEAT0042]
estimate: 3h
created: 2026-01-09
milestone: 33-groove-dashboard
---

# WebSocket dashboard endpoint

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Create `/ws/groove/dashboard` WebSocket endpoint with topic-based subscriptions for real-time dashboard updates.

## Context

The dashboard needs real-time updates for learnings, attribution, strategy, and health data. Uses a topic-based subscription model where clients subscribe to specific data feeds. See [design.md](../../../milestones/33-groove-dashboard/design.md).

## Tasks

### Task 1: Define dashboard types

**Files:**
- Create: `plugins/vibes-groove/src/dashboard/mod.rs`
- Create: `plugins/vibes-groove/src/dashboard/types.rs`

**Steps:**
1. Create dashboard module directory
2. Define `DashboardTopic` enum:
   ```rust
   pub enum DashboardTopic {
       Overview,
       Learnings { filters: LearningsFilter },
       LearningDetail { id: LearningId },
       Attribution { period: Period },
       SessionTimeline { period: Period },
       StrategyDistributions,
       StrategyOverrides,
       Health,
   }
   ```
3. Define `DashboardMessage` enum for server→client messages
4. Define `DashboardRequest` enum for client→server messages
5. Define data structures for each topic
6. Run: `cargo check -p vibes-groove`
7. Commit: `feat(groove): add dashboard types`

### Task 2: Create dashboard handler

**Files:**
- Create: `plugins/vibes-groove/src/dashboard/handler.rs`

**Steps:**
1. Implement `DashboardHandler` struct:
   ```rust
   pub struct DashboardHandler {
       store: Arc<dyn LearningStore>,
       attribution_store: Arc<dyn AttributionStore>,
       strategy_store: Arc<dyn StrategyStore>,
       subscriptions: HashMap<ConnectionId, HashSet<DashboardTopic>>,
   }
   ```
2. Implement subscribe/unsubscribe logic
3. Implement topic-specific data providers:
   - `get_overview_data()`
   - `get_learnings_data(filters)`
   - `get_learning_detail(id)`
   - `get_attribution_data(period)`
   - `get_strategy_data()`
   - `get_health_data()`
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): add DashboardHandler`

### Task 3: Create WebSocket endpoint

**Files:**
- Create: `plugins/vibes-groove/src/dashboard/websocket.rs`

**Steps:**
1. Implement WebSocket upgrade handler
2. Implement message routing loop
3. Implement connection management
4. Handle reconnection with subscription replay
5. Wire into server routes
6. Run: `cargo check -p vibes-groove`
7. Commit: `feat(groove): add dashboard WebSocket endpoint`

### Task 4: Create frontend hook

**Files:**
- Create: `web-ui/src/hooks/useDashboard.ts`

**Steps:**
1. Implement WebSocket connection management:
   - Auto-connect on mount
   - Auto-reconnect on disconnect
   - Connection state tracking
2. Implement topic subscription helpers:
   - `subscribe(topic)`
   - `unsubscribe(topic)`
3. Implement data caching per topic
4. Export typed hooks for each topic
5. Run: `npm run typecheck --workspace=web-ui`
6. Commit: `feat(web-ui): add useDashboard hook`

### Task 5: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/dashboard/websocket.rs`
- Create: `web-ui/src/hooks/__tests__/useDashboard.test.ts`

**Steps:**
1. Write backend tests:
   - Test subscription management
   - Test message routing
   - Test data provider responses
2. Write frontend tests:
   - Test connection lifecycle
   - Test subscription helpers
   - Test data caching
3. Run: `cargo test -p vibes-groove dashboard`
4. Run: `npm test --workspace=web-ui -- --run`
5. Commit: `test(groove): add dashboard WebSocket tests`

## Acceptance Criteria

- [ ] `/ws/groove/dashboard` endpoint accepts connections
- [ ] Topic subscription/unsubscription works
- [ ] Data providers return correct data for each topic
- [ ] Frontend hook manages connection lifecycle
- [ ] Data caching works correctly
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0043`
3. Commit, push, and create PR
