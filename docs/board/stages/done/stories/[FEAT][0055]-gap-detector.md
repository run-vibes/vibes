---
id: FEAT0055
title: CapabilityGapDetector
type: feat
status: done
priority: high
scope: plugin-system
depends: [FEAT0052]
estimate: 3h
created: 2026-01-09
---

# CapabilityGapDetector

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement capability gap detection from combined signals including failures, attribution, and confidence.

## Context

The CapabilityGapDetector identifies recurring failure patterns that indicate the system lacks knowledge or capability. It combines multiple signal types (failures, negative attribution, low confidence) to surface actionable gaps. See [design.md](../../../milestones/34-open-world-adaptation/design.md).

## Tasks

### Task 1: Create CapabilityGapDetector struct

**Files:**
- Create: `plugins/vibes-groove/src/openworld/gaps.rs`

**Steps:**
1. Implement `CapabilityGapDetector` struct:
   ```rust
   pub struct CapabilityGapDetector {
       store: Arc<dyn OpenWorldStore>,
       config: GapsConfig,

       // In-memory tracking
       failure_counts: HashMap<u64, Vec<FailureRecord>>,
       active_gaps: HashMap<GapId, CapabilityGap>,
   }
   ```
2. Implement constructor with config loading
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add CapabilityGapDetector struct`

### Task 2: Implement failure detection

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/gaps.rs`

**Steps:**
1. Implement `detect_failure()`:
   ```rust
   pub fn detect_failure(
       &self,
       session: &SessionOutcome,
       attributions: &[AttributionRecord],
   ) -> Option<FailureRecord> {
       // Check for explicit negative feedback
       if session.has_negative_feedback() {
           return Some(FailureRecord {
               failure_type: FailureType::ExplicitFeedback,
               ...
           });
       }

       // Check for negative attribution
       for attr in attributions {
           if attr.attributed_value < -0.3 {
               return Some(FailureRecord {
                   failure_type: FailureType::NegativeAttribution,
                   learning_ids: vec![attr.learning_id],
                   ...
               });
           }
       }

       // Check for low confidence
       if session.outcome_confidence < 0.3 {
           return Some(FailureRecord {
               failure_type: FailureType::LowConfidence,
               ...
           });
       }

       None
   }
   ```
2. Run: `cargo check -p vibes-groove`
3. Commit: `feat(groove): implement failure detection`

### Task 3: Implement gap aggregation

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/gaps.rs`

**Steps:**
1. Implement `record_failure()`:
   ```rust
   pub async fn record_failure(&mut self, failure: FailureRecord) -> Result<()> {
       let context_hash = failure.context_hash;
       self.failure_counts
           .entry(context_hash)
           .or_default()
           .push(failure.clone());

       self.store.save_failure(&failure).await?;
       self.check_for_gap(context_hash).await?;

       Ok(())
   }
   ```
2. Implement `check_for_gap()`:
   ```rust
   async fn check_for_gap(&mut self, context_hash: u64) -> Result<Option<GapId>> {
       let failures = &self.failure_counts[&context_hash];
       if failures.len() < self.config.min_failures_for_gap {
           return Ok(None);
       }

       let gap = self.get_or_create_gap(context_hash, failures).await?;
       self.update_severity(&gap.id)?;

       Ok(Some(gap.id))
   }
   ```
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): implement gap aggregation`

### Task 4: Implement severity escalation

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/gaps.rs`

**Steps:**
1. Implement `update_severity()`:
   ```rust
   fn update_severity(&mut self, gap_id: &GapId) -> Result<()> {
       let gap = self.active_gaps.get_mut(gap_id).unwrap();
       let count = gap.failure_count;

       let new_severity = match count {
           0..=2 => GapSeverity::Low,
           3..=10 => GapSeverity::Medium,
           11..=50 => GapSeverity::High,
           _ => GapSeverity::Critical,
       };

       if new_severity != gap.severity {
           gap.severity = new_severity;
           // Emit event
       }

       Ok(())
   }
   ```
2. Implement category classification from failure patterns
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): implement severity escalation`

### Task 5: Implement main entry point and tests

**Files:**
- Modify: `plugins/vibes-groove/src/openworld/gaps.rs`

**Steps:**
1. Implement `process_outcome()`:
   ```rust
   pub async fn process_outcome(
       &mut self,
       session: &SessionOutcome,
       attributions: &[AttributionRecord],
   ) -> Result<Option<CapabilityGap>> {
       if let Some(failure) = self.detect_failure(session, attributions) {
           self.record_failure(failure).await?;
           // Return gap if one was created/updated
       }
       Ok(None)
   }
   ```
2. Write tests:
   - Test failure detection for each type
   - Test gap creation threshold
   - Test severity escalation
   - Test category classification
3. Run: `cargo test -p vibes-groove openworld::gaps`
4. Commit: `test(groove): add gap detector tests`

## Acceptance Criteria

- [ ] Detects explicit negative feedback
- [ ] Detects negative attribution failures
- [ ] Detects low confidence failures
- [ ] Groups failures by context hash
- [ ] Creates gaps when threshold reached
- [ ] Escalates severity based on failure count
- [ ] Classifies gap category
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0055`
3. Commit, push, and create PR
