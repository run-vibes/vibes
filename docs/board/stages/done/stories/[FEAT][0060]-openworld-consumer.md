---
id: FEAT0060
title: Iggy consumer and events
type: feat
status: done
priority: high
scope: plugin-system
depends: [FEAT0058, FEAT0059]
estimate: 2h
created: 2026-01-09
---

# Iggy consumer and events

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement event streaming for open-world system using Iggy topics.

## Context

The open-world system emits events for novelty detection, gap lifecycle, and strategy feedback. These events are streamed via Iggy for audit trail and downstream processing. See [design.md](../../../milestones/34-open-world-adaptation/design.md).

## Tasks

### Task 1: Define Iggy stream and topics

**Files:**
- Create: `plugins/vibes-groove/src/openworld/consumer.rs`

**Steps:**
1. Define stream and topics:
   ```rust
   pub const OPENWORLD_STREAM: &str = "groove.openworld";

   pub mod topics {
       pub const NOVELTY: &str = "novelty";      // Detection events
       pub const GAPS: &str = "gaps";            // Gap lifecycle events
       pub const FEEDBACK: &str = "feedback";    // Strategy feedback events
   }
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `feat(groove): define openworld Iggy stream`

### Task 2: Implement OpenWorldProducer

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/consumer.rs`

**Steps:**
1. Implement `OpenWorldProducer`:
   ```rust
   pub struct OpenWorldProducer {
       client: Arc<IggyClient>,
       stream_id: StreamId,
   }

   impl OpenWorldProducer {
       pub async fn emit_novelty_detected(
           &self,
           fingerprint: &PatternFingerprint,
           cluster: Option<ClusterId>,
       ) -> Result<()> {
           let event = OpenWorldEvent::NoveltyDetected {
               fingerprint: fingerprint.clone(),
               cluster,
           };
           self.emit(topics::NOVELTY, &event).await
       }

       pub async fn emit_cluster_updated(&self, cluster: &AnomalyCluster) -> Result<()> {
           let event = OpenWorldEvent::ClusterUpdated {
               cluster: cluster.clone(),
           };
           self.emit(topics::NOVELTY, &event).await
       }

       pub async fn emit_gap_created(&self, gap: &CapabilityGap) -> Result<()> {
           let event = OpenWorldEvent::GapCreated { gap: gap.clone() };
           self.emit(topics::GAPS, &event).await
       }

       pub async fn emit_gap_status_changed(
           &self,
           gap_id: GapId,
           old: GapStatus,
           new: GapStatus,
       ) -> Result<()> {
           let event = OpenWorldEvent::GapStatusChanged { gap_id, old, new };
           self.emit(topics::GAPS, &event).await
       }

       pub async fn emit_solutions_generated(
           &self,
           gap_id: GapId,
           solutions: &[SuggestedSolution],
       ) -> Result<()> {
           let event = OpenWorldEvent::SolutionsGenerated {
               gap_id,
               solutions: solutions.to_vec(),
           };
           self.emit(topics::GAPS, &event).await
       }

       pub async fn emit_strategy_feedback(
           &self,
           learning_id: LearningId,
           adjustment: f64,
       ) -> Result<()> {
           let event = OpenWorldEvent::StrategyFeedback {
               learning_id,
               adjustment,
           };
           self.emit(topics::FEEDBACK, &event).await
       }
   }
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `feat(groove): implement OpenWorldProducer`

### Task 3: Wire producer into OpenWorldHook

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/hook.rs`

**Steps:**
1. Add producer to `OpenWorldHook`
2. Emit events at appropriate points:
   - After novelty detection
   - After cluster updates
   - After gap creation/status change
   - After solution generation
   - After strategy feedback
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): wire producer into OpenWorldHook`

### Task 4: Initialize stream in plugin

**Files:**
- Modify: `plugins/vibes-groove/src/plugin.rs`

**Steps:**
1. Create Iggy stream on plugin initialization
2. Create topics within stream
3. Initialize `OpenWorldProducer`
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): initialize openworld stream`

### Task 5: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/consumer.rs`

**Steps:**
1. Write tests:
   - Test event serialization
   - Test producer emit methods
   - Test stream initialization
2. Run: `cargo test -p vibes-groove openworld::consumer`
3. Commit: `test(groove): add openworld consumer tests`

## Acceptance Criteria

- [ ] `groove.openworld` stream with novelty, gaps, feedback topics
- [ ] `OpenWorldProducer` emits all event types
- [ ] Events emitted at correct points in hook
- [ ] Stream initialized on plugin startup
- [ ] Event serialization/deserialization works
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0060`
3. Commit, push, and create PR
