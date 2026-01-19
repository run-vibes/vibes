---
id: FEAT0037
title: Outcome router
type: feat
status: done
priority: high
scope: plugin-system
depends: [FEAT0034]
estimate: 2h
created: 2026-01-09
---

# Outcome router

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement outcome routing to combine attribution and direct signals into strategy outcomes.

## Context

The outcome router combines signals from the attribution engine (M31) and direct lightweight events to compute strategy effectiveness. Attribution is weighted higher but direct signals provide faster feedback. See [design.md](../../../milestones/32-adaptive-strategies/design.md).

## Tasks

### Task 1: Create OutcomeRouter struct

**Files:**
- Create: `plugins/vibes-groove/src/strategy/router.rs`

**Steps:**
1. Define `OutcomeRouter` struct:
   ```rust
   pub struct OutcomeRouter {
       /// Weight for attributed value vs direct signals
       attribution_weight: f64,  // default 0.7
       direct_weight: f64,       // default 0.3
       min_outcome_confidence: f64, // default 0.3
   }

   impl Default for OutcomeRouter {
       fn default() -> Self {
           Self {
               attribution_weight: 0.7,
               direct_weight: 0.3,
               min_outcome_confidence: 0.3,
           }
       }
   }
   ```
2. Add config struct:
   ```rust
   pub struct OutcomeRouterConfig {
       pub attribution_weight: f64,
       pub direct_weight: f64,
       pub min_outcome_confidence: f64,
   }
   ```
3. Add to `strategy/mod.rs`
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): add OutcomeRouter struct`

### Task 2: Implement outcome computation

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/router.rs`

**Steps:**
1. Implement `compute_outcome()`:
   ```rust
   pub fn compute_outcome(
       &self,
       attribution: Option<&AttributionRecord>,
       direct_signals: &[LightweightEvent],
       strategy_used: &InjectionStrategy,
   ) -> Option<StrategyOutcome> {
       let attributed_value = attribution
           .filter(|a| a.was_activated)
           .map(|a| a.attributed_value);

       let direct_value = Self::aggregate_direct_signals(direct_signals);

       match (attributed_value, direct_value) {
           (Some(av), Some(dv)) => Some(StrategyOutcome {
               value: self.attribution_weight * av + self.direct_weight * dv,
               confidence: 0.9,  // Both sources available
               source: OutcomeSource::Both,
           }),
           (Some(av), None) => Some(StrategyOutcome {
               value: av,
               confidence: 0.8,
               source: OutcomeSource::Attribution,
           }),
           (None, Some(dv)) => Some(StrategyOutcome {
               value: dv,
               confidence: 0.5,  // Lower confidence for direct only
               source: OutcomeSource::Direct,
           }),
           (None, None) => None,  // No signal, skip update
       }
   }
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `feat(groove): implement outcome computation`

### Task 3: Implement direct signal aggregation

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/router.rs`

**Steps:**
1. Implement `aggregate_direct_signals()`:
   ```rust
   fn aggregate_direct_signals(signals: &[LightweightEvent]) -> Option<f64> {
       if signals.is_empty() {
           return None;
       }

       let mut positive_count = 0;
       let mut negative_count = 0;

       for signal in signals {
           match signal.signal_type {
               SignalType::Acceptance | SignalType::Completion => positive_count += 1,
               SignalType::Rejection | SignalType::Abandonment => negative_count += 1,
               SignalType::Neutral => {}
           }
       }

       let total = positive_count + negative_count;
       if total == 0 {
           return None;
       }

       // Normalize to [-1, 1] range
       Some((positive_count as f64 - negative_count as f64) / total as f64)
   }
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `feat(groove): implement direct signal aggregation`

### Task 4: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/strategy/router.rs`

**Steps:**
1. Write tests:
   - Test both attribution and direct signals (weighted average)
   - Test attribution only (full weight, 0.8 confidence)
   - Test direct only (lower 0.5 confidence)
   - Test no signals (returns None)
   - Test direct signal aggregation
   - Test confidence thresholds
2. Run: `cargo test -p vibes-groove strategy::router`
3. Commit: `test(groove): add outcome router tests`

## Acceptance Criteria

- [ ] `OutcomeRouter` with configurable weights
- [ ] Weighted combination of attribution and direct
- [ ] Confidence varies by source availability
- [ ] Direct signal aggregation to normalized value
- [ ] Returns None when no usable signals
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0037`
3. Commit, push, and create PR
