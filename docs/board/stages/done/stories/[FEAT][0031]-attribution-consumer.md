---
id: FEAT0031
title: Attribution consumer
type: feat
status: done
priority: high
epics: [plugin-system]
depends: [FEAT0027, FEAT0028, FEAT0030]
estimate: 3h
created: 2026-01-09
milestone: 31-attribution-engine
---

# Attribution consumer

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement Iggy consumer that orchestrates the full attribution pipeline.

## Context

The attribution consumer subscribes to heavy assessment events and runs the 4-layer attribution pipeline for each session. See [design.md](../../../milestones/31-attribution-engine/design.md).

## Tasks

### Task 1: Create AttributionConsumer struct

**Files:**
- Create: `plugins/vibes-groove/src/attribution/consumer.rs`

**Steps:**
1. Create the consumer struct:
   ```rust
   pub struct AttributionConsumer {
       activation_detector: Arc<dyn ActivationDetector>,
       temporal_correlator: Arc<dyn TemporalCorrelator>,
       ablation_manager: AblationManager<ConservativeAblation>,
       value_aggregator: ValueAggregator,
       store: Arc<dyn AttributionStore>,
       embedder: Arc<dyn Embedder>,
       config: AttributionConfig,
   }
   ```
2. Add configuration struct:
   ```rust
   pub struct AttributionConfig {
       pub activation: ActivationConfig,
       pub temporal: TemporalConfig,
       pub ablation: AblationConfig,
       pub enabled: bool,
   }
   ```
3. Add to `attribution/mod.rs`
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): add AttributionConsumer struct`

### Task 2: Implement heavy event processing

**Files:**
- Modify: `plugins/vibes-groove/src/attribution/consumer.rs`

**Steps:**
1. Implement `process_heavy_event()`:
   ```rust
   async fn process_heavy_event(&self, event: &HeavyEvent) -> Result<()> {
       let transcript = event.transcript.as_ref();
       let outcome = event.outcome;

       for learning_id in &event.active_learnings {
           // 1. Load learning
           let learning = self.load_learning(learning_id).await?;

           // 2. Run activation detection
           let activation = self.activation_detector
               .detect(&learning, transcript, &*self.embedder)
               .await?;

           // 3. Run temporal correlation
           let temporal = self.temporal_correlator
               .correlate(&activation.signals_indices(), &event.lightweight_events);

           // 4. Check ablation status
           let was_withheld = event.withheld_learnings.contains(learning_id);

           // 5. Compute attributed value
           let attributed_value = self.compute_attribution(
               &activation, &temporal, outcome
           );

           // 6. Store attribution record
           self.store_attribution_record(/*...*/);

           // 7. Update learning value
           self.update_learning_value(/*...*/);
       }

       Ok(())
   }
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `feat(groove): implement heavy event processing`

### Task 3: Wire Iggy consumer

**Files:**
- Modify: `plugins/vibes-groove/src/attribution/consumer.rs`

**Steps:**
1. Implement Iggy subscription setup:
   ```rust
   impl AttributionConsumer {
       pub async fn start(&self, iggy: &IggyClient) -> Result<JoinHandle<()>> {
           // Subscribe to groove.assessment.heavy
           // Resume from last acknowledged offset
           // Spawn consumer task
       }
   }
   ```
2. Add Iggy topic `groove.attribution` for output events
3. Implement consumer loop with error handling
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): wire Iggy consumer`

### Task 4: Integrate with plugin lifecycle

**Files:**
- Modify: `plugins/vibes-groove/src/plugin.rs`

**Steps:**
1. Add `AttributionConsumer` to `GroovePlugin`
2. Start consumer in plugin initialization
3. Graceful shutdown handling
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): integrate attribution consumer`

### Task 5: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/attribution/consumer.rs`

**Steps:**
1. Write tests:
   - Test heavy event processing pipeline
   - Test attribution record creation
   - Test learning value updates
   - Test handling of withheld learnings
   - Test error handling
2. Run: `cargo test -p vibes-groove attribution::consumer`
3. Commit: `test(groove): add consumer tests`

## Acceptance Criteria

- [ ] Subscribes to `groove.assessment.heavy`
- [ ] Runs full 4-layer pipeline for each learning
- [ ] Stores attribution records
- [ ] Updates learning values
- [ ] Publishes to `groove.attribution`
- [ ] Integrated with plugin lifecycle
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0031`
3. Commit, push, and create PR
