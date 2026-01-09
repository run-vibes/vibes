---
id: FEAT0028
title: Temporal correlation (Layer 2)
type: feat
status: pending
priority: high
epics: [plugin-system]
depends: [FEAT0026]
estimate: 2h
created: 2026-01-09
milestone: 31-attribution-engine
---

# Temporal correlation (Layer 2)

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement temporal correlation to weight signals by proximity to activation points.

## Context

Layer 2 of the attribution engine measures positive and negative signals near activation points, applying exponential decay based on distance. See [design.md](../../../milestones/31-attribution-engine/design.md).

## Tasks

### Task 1: Define TemporalCorrelator trait

**Files:**
- Create: `plugins/vibes-groove/src/attribution/temporal.rs`

**Steps:**
1. Define the trait and result types:
   ```rust
   pub trait TemporalCorrelator: Send + Sync {
       fn correlate(
           &self,
           activation_points: &[u32],
           lightweight_events: &[LightweightEvent],
       ) -> TemporalResult;
   }

   pub struct TemporalResult {
       pub positive_score: f64,
       pub negative_score: f64,
       pub net_score: f64,
   }
   ```
2. Add to `attribution/mod.rs`
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add TemporalCorrelator trait`

### Task 2: Implement ExponentialDecayCorrelator

**Files:**
- Modify: `plugins/vibes-groove/src/attribution/temporal.rs`

**Steps:**
1. Create `ExponentialDecayCorrelator` struct:
   ```rust
   pub struct ExponentialDecayCorrelator {
       decay_rate: f64,    // lambda, default 0.2
       max_distance: u32,  // default 10 messages
   }
   ```
2. Implement decay weight calculation:
   ```rust
   fn weight(&self, distance: u32) -> f64 {
       if distance > self.max_distance {
           return 0.0;
       }
       (-self.decay_rate * distance as f64).exp()
   }
   ```
3. Implement `correlate()`:
   - For each lightweight event (acceptance/rejection signal)
   - Find distance to nearest activation point
   - Apply decay weight
   - Accumulate into positive or negative score
   - Compute net score = positive - negative
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): implement ExponentialDecayCorrelator`

### Task 3: Add configuration

**Files:**
- Modify: `plugins/vibes-groove/src/config.rs`

**Steps:**
1. Add temporal config:
   ```rust
   pub struct TemporalConfig {
       pub decay_rate: f64,
       pub max_distance: u32,
   }

   impl Default for TemporalConfig {
       fn default() -> Self {
           Self {
               decay_rate: 0.2,
               max_distance: 10,
           }
       }
   }
   ```
2. Wire into `GrooveConfig` under `attribution` section
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add temporal config`

### Task 4: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/attribution/temporal.rs`

**Steps:**
1. Write tests:
   - Test decay weight calculation at various distances
   - Test signals at activation point have full weight
   - Test signals beyond max_distance have zero weight
   - Test positive and negative signal accumulation
   - Test net score calculation
   - Test with multiple activation points
2. Run: `cargo test -p vibes-groove attribution::temporal`
3. Commit: `test(groove): add temporal correlation tests`

## Acceptance Criteria

- [ ] `TemporalCorrelator` trait defined
- [ ] `ExponentialDecayCorrelator` applies decay based on distance
- [ ] Signals beyond max_distance ignored
- [ ] Configurable decay rate and max distance
- [ ] Net score correctly computed
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0028`
3. Commit, push, and create PR
