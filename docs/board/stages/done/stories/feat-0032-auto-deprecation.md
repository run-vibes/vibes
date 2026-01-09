---
id: FEAT0032
title: Auto-deprecation
type: feat
status: done
priority: high
epics: [plugin-system]
depends: [FEAT0030]
estimate: 1h
created: 2026-01-09
milestone: 31-attribution-engine
---

# Auto-deprecation

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement automatic deprecation of harmful learnings based on attribution values.

## Context

Learnings with consistently negative value should be automatically deprecated to protect users. This is reversible via CLI. See [design.md](../../../milestones/31-attribution-engine/design.md).

## Tasks

### Task 1: Add deprecation logic to ValueAggregator

**Files:**
- Modify: `plugins/vibes-groove/src/attribution/aggregation.rs`

**Steps:**
1. Add deprecation check method:
   ```rust
   impl ValueAggregator {
       /// Check if learning should be deprecated based on value
       pub fn should_deprecate(&self, value: &LearningValue) -> bool {
           value.estimated_value < self.deprecation_threshold
               && value.confidence > self.deprecation_confidence
       }

       /// Create deprecation reason
       pub fn deprecation_reason(&self, value: &LearningValue) -> String {
           format!(
               "Auto-deprecated: value={:.2} (threshold={:.2}), confidence={:.2}",
               value.estimated_value,
               self.deprecation_threshold,
               value.confidence
           )
       }
   }
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `feat(groove): add deprecation logic`

### Task 2: Add DeprecationEvent

**Files:**
- Modify: `plugins/vibes-groove/src/attribution/types.rs`

**Steps:**
1. Define deprecation event:
   ```rust
   pub struct DeprecationEvent {
       pub learning_id: LearningId,
       pub timestamp: DateTime<Utc>,
       pub reason: String,
       pub final_value: f64,
       pub confidence: f64,
       pub session_count: u32,
   }
   ```
2. Add serialization for Iggy
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add DeprecationEvent`

### Task 3: Integrate with learning store

**Files:**
- Modify: `plugins/vibes-groove/src/attribution/consumer.rs`
- Modify: `plugins/vibes-groove/src/store.rs`

**Steps:**
1. Add `deprecate_learning()` method to store:
   ```rust
   async fn deprecate_learning(
       &self,
       id: LearningId,
       reason: &str,
   ) -> Result<()>;
   ```
2. Update consumer to check deprecation after value update:
   ```rust
   // After updating learning value
   if self.value_aggregator.should_deprecate(&new_value) {
       self.deprecate_learning(&learning, &new_value).await?;
   }
   ```
3. Publish `DeprecationEvent` to Iggy
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): integrate deprecation`

### Task 4: Add notification/logging

**Files:**
- Modify: `plugins/vibes-groove/src/attribution/consumer.rs`

**Steps:**
1. Add tracing for deprecation events:
   ```rust
   tracing::warn!(
       learning_id = %learning.id,
       value = %new_value.estimated_value,
       "Learning auto-deprecated due to negative value"
   );
   ```
2. Consider user notification mechanism (future)
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add deprecation logging`

### Task 5: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/attribution/aggregation.rs`

**Steps:**
1. Write tests:
   - Test deprecation threshold boundary
   - Test confidence requirement
   - Test deprecation reason formatting
   - Test learning status update
2. Run: `cargo test -p vibes-groove attribution`
3. Commit: `test(groove): add deprecation tests`

## Acceptance Criteria

- [ ] Checks value < threshold AND confidence > threshold
- [ ] Updates learning status to `Deprecated`
- [ ] Records deprecation reason
- [ ] Publishes `DeprecationEvent` to Iggy
- [ ] Excludes deprecated learnings from future injection
- [ ] Logs deprecation events
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0032`
3. Commit, push, and create PR
