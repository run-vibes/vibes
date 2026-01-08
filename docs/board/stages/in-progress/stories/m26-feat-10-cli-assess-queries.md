---
id: F011
title: Feature: Wire CLI Assess Commands to Query Real Data
type: feat
status: backlog
priority: medium
epics: [core, cli, plugin-system]
depends: []
estimate:
created: 2026-01-04
updated: 2026-01-07
milestone: milestone-26-assessment-framework
---

# Feature: Wire CLI Assess Commands to Query Real Data

## Problem

The `vibes groove assess status` and `vibes groove assess history` CLI commands currently return **hardcoded dummy data** instead of querying actual assessment state.

Current implementation (`plugin.rs:857-908`) returns static strings like:
- "Active sessions: 0"
- "Events today: 0"
- "No assessments found for this session."

## Goal

Wire CLI commands to query actual data from:
1. Iggy event log (for event counts, session activity)
2. Assessment processor state (circuit breaker status, sampling config)
3. Assessment log (for session history)

## Tasks

### Task 1: Add State Query API

Add methods to query assessment state:
- `get_circuit_breaker_status() -> CircuitBreakerStatus`
- `get_active_sessions() -> Vec<SessionId>`
- `get_session_history(session_id) -> Vec<AssessmentEvent>`

### Task 2: Wire `assess status` Command

Replace hardcoded values with real queries:
- Circuit breaker state from processor
- Active session count from session tracking
- Event count from Iggy high water mark
- Checkpoint count from assessment log

### Task 3: Wire `assess history` Command

Query actual session history:
- List recent sessions with assessment data
- Show assessment events for specific session
- Include timestamps and outcomes

## Considerations

- Plugin runs in CLI context without direct access to server state
- May need to add HTTP API endpoints to query assessment data
- Or persist assessment state to queryable storage (SQLite, file)

## Acceptance Criteria

- [ ] `assess status` shows real circuit breaker state
- [ ] `assess status` shows actual event/session counts
- [ ] `assess history` shows real session data
- [ ] No hardcoded values remain in assess commands
