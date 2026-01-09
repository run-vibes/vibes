---
id: FEAT0030
title: Value aggregation (Layer 4)
type: feat
status: pending
priority: high
epics: [plugin-system]
depends: [FEAT0026, FEAT0028, FEAT0029]
estimate: 2h
created: 2026-01-09
milestone: 31-attribution-engine
---

# Value aggregation (Layer 4)

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement value aggregation to combine signals from all attribution layers into final learning value.

## Context

Layer 4 of the attribution engine aggregates temporal correlation and ablation results into a single estimated value with confidence. See [design.md](../../../milestones/31-attribution-engine/design.md).

## Tasks

### Task 1: Create ValueAggregator

**Files:**
- Create: `plugins/vibes-groove/src/attribution/aggregation.rs`

**Steps:**
1. Create `ValueAggregator` struct:
   ```rust
   pub struct ValueAggregator {
       temporal_weight: f64,        // default 0.6
       ablation_weight: f64,        // default 0.4
       deprecation_threshold: f64,  // default -0.3
       deprecation_confidence: f64, // default 0.8
   }
   ```
2. Add to `attribution/mod.rs`
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add ValueAggregator struct`

### Task 2: Implement running average update

**Files:**
- Modify: `plugins/vibes-groove/src/attribution/aggregation.rs`

**Steps:**
1. Implement `update_temporal_value()`:
   ```rust
   /// Update temporal value with new observation using running weighted average
   pub fn update_temporal_value(
       &self,
       current: &LearningValue,
       new_temporal: &TemporalResult,
   ) -> (f64, f64) {
       // Returns (new_value, new_confidence)
       // Confidence increases with session count
   }
   ```
2. Implement helper functions:
   ```rust
   fn weighted_update(old_value: f64, new_value: f64, weight: f64) -> f64;
   fn confidence_from_count(count: u32) -> f64;
   ```
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): implement temporal value update`

### Task 3: Implement multi-source aggregation

**Files:**
- Modify: `plugins/vibes-groove/src/attribution/aggregation.rs`

**Steps:**
1. Implement `aggregate_value()`:
   ```rust
   /// Combine temporal and ablation values into final estimate
   pub fn aggregate_value(&self, value: &LearningValue) -> (f64, f64) {
       // If ablation result exists and is significant, use weighted average
       // Otherwise, use temporal value only
       // Returns (estimated_value, confidence)
   }
   ```
2. Implement `combine_estimates()`:
   ```rust
   fn combine_estimates(
       temporal: (f64, f64),    // (value, confidence)
       ablation: Option<(f64, f64)>,
       temporal_weight: f64,
       ablation_weight: f64,
   ) -> (f64, f64);
   ```
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): implement multi-source aggregation`

### Task 4: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/attribution/aggregation.rs`

**Steps:**
1. Write tests:
   - Test running average converges correctly
   - Test confidence increases with session count
   - Test temporal-only aggregation
   - Test temporal + ablation aggregation
   - Test weight balancing
   - Test deprecation threshold detection
2. Run: `cargo test -p vibes-groove attribution::aggregation`
3. Commit: `test(groove): add aggregation tests`

## Acceptance Criteria

- [ ] `ValueAggregator` combines signals
- [ ] Running average for temporal values
- [ ] Confidence-weighted combination of sources
- [ ] Configurable weights
- [ ] Deprecation threshold detection
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0030`
3. Commit, push, and create PR
