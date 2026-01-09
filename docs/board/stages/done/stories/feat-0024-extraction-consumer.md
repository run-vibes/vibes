---
id: FEAT0024
title: Extraction consumer
type: feat
status: done
priority: high
epics: [plugin-system]
depends: [FEAT0019, FEAT0021]
estimate: 3h
created: 2026-01-08
updated: 2026-01-09
milestone: 30-learning-extraction
---

# Extraction consumer

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Iggy consumer that orchestrates the extraction pipeline, processing heavy events and pattern detection.

## Context

The extraction consumer subscribes to `groove.assessment.heavy` events, runs the extraction pipeline (LLM candidates, pattern detectors, deduplication), and persists learnings. See [design.md](../../../milestones/30-learning-extraction/design.md).

## Tasks

### Task 1: Create consumer structure

**Files:**
- Create: `plugins/vibes-groove/src/extraction/consumer.rs`

**Steps:**
1. Create `ExtractionConsumer` struct:
   ```rust
   pub struct ExtractionConsumer {
       store: Arc<dyn LearningStore>,
       embedder: Arc<dyn Embedder>,
       dedup: Arc<dyn DeduplicationStrategy>,
       correction_detector: CorrectionDetector,
       error_recovery_detector: ErrorRecoveryDetector,
       config: ExtractionConfig,
       iggy_client: Arc<IggyClient>,
   }
   ```
2. Create `ExtractionConfig` struct with all options
3. Implement `new()` constructor
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): add ExtractionConsumer structure`

### Task 2: Implement Iggy subscription

**Files:**
- Modify: `plugins/vibes-groove/src/extraction/consumer.rs`

**Steps:**
1. Implement `subscribe()`:
   - Create consumer group for extraction
   - Subscribe to `groove.assessment.heavy`
   - Resume from last acknowledged offset
2. Implement message polling loop:
   - Poll for messages
   - Deserialize `HeavyEvent`
   - Process each event
   - Acknowledge on success
3. Add graceful shutdown handling
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): add Iggy subscription for extraction`

### Task 3: Implement extraction pipeline

**Files:**
- Modify: `plugins/vibes-groove/src/extraction/consumer.rs`

**Steps:**
1. Implement `process_heavy_event()`:
   ```rust
   async fn process_heavy_event(&self, event: HeavyEvent) -> Result<()> {
       let mut candidates = Vec::new();

       // Collect LLM-extracted candidates from event
       candidates.extend(event.extraction_candidates);

       // Run pattern detectors if transcript available
       if let Some(transcript) = &event.transcript {
           candidates.extend(self.correction_detector.detect(transcript).await?);
           candidates.extend(self.error_recovery_detector.detect(transcript).await?);
       }

       // Process each candidate
       for candidate in candidates {
           self.process_candidate(candidate).await?;
       }

       Ok(())
   }
   ```
2. Implement `process_candidate()`:
   - Filter by minimum confidence
   - Embed the learning
   - Check for duplicates
   - Merge or save as new
   - Write `ExtractionEvent` to Iggy
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): implement extraction pipeline`

### Task 4: Add Iggy producer

**Files:**
- Modify: `plugins/vibes-groove/src/extraction/consumer.rs`

**Steps:**
1. Create `groove.extraction` topic
2. Define `ExtractionEvent` messages:
   - `LearningCreated { learning_id, category, confidence }`
   - `LearningMerged { learning_id, merged_from }`
   - `ExtractionFailed { reason, source }`
3. Emit events after each extraction
4. Run: `cargo check -p vibes-groove`
5. Commit: `feat(groove): add extraction event producer`

### Task 5: Wire into plugin lifecycle

**Files:**
- Modify: `plugins/vibes-groove/src/plugin.rs`

**Steps:**
1. Create `ExtractionConsumer` on plugin init
2. Start consumer in background task
3. Add shutdown handling
4. Add health check endpoint
5. Run: `cargo check -p vibes-groove`
6. Commit: `feat(groove): wire extraction consumer into plugin`

### Task 6: Add integration tests

**Files:**
- Create: `plugins/vibes-groove/tests/extraction_integration.rs`

**Steps:**
1. Write integration tests:
   - Test full pipeline with mock heavy event
   - Test duplicate detection
   - Test pattern detection
   - Test event emission
2. Run: `cargo test -p vibes-groove --test extraction_integration`
3. Commit: `test(groove): add extraction integration tests`

## Acceptance Criteria

- [ ] Subscribes to `groove.assessment.heavy`
- [ ] Processes LLM extraction candidates
- [ ] Runs pattern detectors on transcripts
- [ ] Deduplicates before storing
- [ ] Emits `ExtractionEvent` to Iggy
- [ ] Starts automatically with plugin
- [ ] Handles shutdown gracefully
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0024`
3. Commit, push, and create PR
