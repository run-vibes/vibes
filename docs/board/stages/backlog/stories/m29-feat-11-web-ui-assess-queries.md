---
id: F012
title: "Feature: Add Assessment Query UI to Web Dashboard"
type: feat
status: backlog
priority: medium
epics: [web-ui, plugin-system]
depends: [m26-feat-10-cli-assess-queries]
estimate:
created: 2026-01-07
updated: 2026-01-07
milestone: 29-assessment-framework
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

## Tasks

### Task 1: Assessment Status Panel

Add a status panel showing:
- Circuit breaker state (closed/open/half-open)
- Active session count
- Today's event count
- Assessment checkpoint count

### Task 2: Session History View

Add ability to view assessment history for a specific session:
- Session selector/search
- Timeline of assessments for selected session
- Assessment details (tier, outcome, signals)

### Task 3: Query Filters

Extend existing filter UI:
- Date range filter
- Session ID filter
- Outcome filter (positive/negative/neutral)
- Tier filter (already exists, enhance)

### Task 4: Statistics Dashboard

Add aggregate statistics view:
- Assessments by tier (pie/bar chart)
- Assessments over time (line chart)
- Top sessions by assessment count

## API Requirements

This depends on the HTTP API endpoints being added in m26-feat-10-cli-assess-queries:
- `GET /api/groove/assess/status` - system status
- `GET /api/groove/assess/history?session_id=X` - session history
- `GET /api/groove/assess/stats` - aggregate statistics

## Acceptance Criteria

- [ ] Status panel shows real circuit breaker state
- [ ] Can view assessment history for any session
- [ ] Date range and session filters work
- [ ] Statistics show meaningful aggregate data
- [ ] UI follows CRT design system patterns
