---
id: FEAT0012
title: Wire circuit breaker intervention
type: feat
status: done
priority: high
epics: [core,plugin-system]
depends: []
estimate: 2h
created: 2026-01-08
updated: 2026-01-08
milestone: 29-assessment-framework
---

# Wire circuit breaker intervention

## Summary

Complete the intervention pipeline by connecting the `CircuitBreaker` state transitions to the `HookIntervention` system. Currently, when the circuit breaker opens due to high frustration signals, it only logs the event (line 302 in `processor.rs` has `// TODO: Trigger actual intervention via InterventionHandler`). The intervention system exists but isn't called.

This is the critical missing piece that enables vibes-groove to automatically inject learnings when sessions go badly.

## Acceptance Criteria

- [x] When `CircuitTransition::Opened` fires, `HookIntervention.intervene()` is called
- [ ] Learnings are retrieved from CozoDB storage based on session patterns (deferred - uses default learning)
- [x] Hook files are written to `.claude/hooks/` directory
- [x] Intervention count is tracked per session
- [x] Circuit breaker cooldown is respected
- [x] Tests verify end-to-end intervention flow
- [x] CLI `assess status` shows intervention count

## Implementation Notes

### Current State

1. **CircuitBreaker** (`circuit_breaker.rs`): Detects frustration signals, returns `CircuitTransition::Opened`
2. **HookIntervention** (`intervention.rs`): Writes learning hooks to disk, fully implemented
3. **Gap**: `processor.rs:302` logs but doesn't call intervention

### Required Changes

1. Add `HookIntervention` as a field on `AssessmentProcessor`
2. When `CircuitTransition::Opened` fires:
   - Query CozoDB for relevant learnings (by pattern tags)
   - Call `intervention.intervene(session_id, learning)`
   - Track the result
3. Add intervention status to `AssessmentStatus` API response
4. Update CLI output to show intervention history

### Key Files

- `plugins/vibes-groove/src/assessment/processor.rs` - Add intervention call
- `plugins/vibes-groove/src/assessment/sync_processor.rs` - Mirror changes
- `plugins/vibes-groove/src/plugin.rs` - Add intervention to status API

### Test Plan

```bash
cargo test -p vibes-groove intervention
cargo test -p vibes-groove e2e_circuit_breaker
```
