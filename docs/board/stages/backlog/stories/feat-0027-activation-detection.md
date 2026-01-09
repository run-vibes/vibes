---
id: FEAT0027
title: Activation detection (Layer 1)
type: feat
status: pending
priority: high
epics: [plugin-system]
depends: [FEAT0026]
estimate: 3h
created: 2026-01-09
milestone: 31-attribution-engine
---

# Activation detection (Layer 1)

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Summary

Implement activation detection to determine if learnings influenced Claude's behavior.

## Context

Layer 1 of the attribution engine detects whether injected learnings were actually used by Claude. Uses embedding similarity and explicit keyword references. See [design.md](../../../milestones/31-attribution-engine/design.md).

## Tasks

### Task 1: Define ActivationDetector trait

**Files:**
- Create: `plugins/vibes-groove/src/attribution/activation.rs`

**Steps:**
1. Define the trait and result types:
   ```rust
   #[async_trait]
   pub trait ActivationDetector: Send + Sync {
       async fn detect(
           &self,
           learning: &Learning,
           transcript: &Transcript,
           embedder: &dyn Embedder,
       ) -> Result<ActivationResult>;
   }

   pub struct ActivationResult {
       pub was_activated: bool,
       pub confidence: f64,
       pub signals: Vec<ActivationSignal>,
   }
   ```
2. Add to `attribution/mod.rs`
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add ActivationDetector trait`

### Task 2: Implement HybridActivationDetector

**Files:**
- Modify: `plugins/vibes-groove/src/attribution/activation.rs`

**Steps:**
1. Create `HybridActivationDetector` struct:
   ```rust
   pub struct HybridActivationDetector {
       similarity_threshold: f64,  // default 0.75
       reference_boost: f64,       // default 0.15
   }
   ```
2. Implement embedding similarity detection:
   - Embed learning description/insight
   - Embed each Claude response in transcript
   - Find max cosine similarity
   - Record message indices above threshold
3. Implement explicit reference detection:
   - Extract key phrases from learning
   - Search Claude responses for phrase matches
   - Record matching message indices
4. Combine signals:
   - If explicit reference found: confidence = similarity + reference_boost
   - If only embedding match: confidence = similarity
   - `was_activated` = confidence > similarity_threshold
5. Run: `cargo check -p vibes-groove`
6. Commit: `feat(groove): implement HybridActivationDetector`

### Task 3: Add configuration

**Files:**
- Modify: `plugins/vibes-groove/src/config.rs`

**Steps:**
1. Add activation config:
   ```rust
   pub struct ActivationConfig {
       pub similarity_threshold: f64,
       pub reference_boost: f64,
   }

   impl Default for ActivationConfig {
       fn default() -> Self {
           Self {
               similarity_threshold: 0.75,
               reference_boost: 0.15,
           }
       }
   }
   ```
2. Wire into `GrooveConfig` under `attribution` section
3. Run: `cargo check -p vibes-groove`
4. Commit: `feat(groove): add activation config`

### Task 4: Add tests

**Files:**
- Modify: `plugins/vibes-groove/src/attribution/activation.rs`

**Steps:**
1. Create mock transcript builder for tests
2. Write tests:
   - Test high similarity triggers activation
   - Test low similarity no activation
   - Test explicit reference boosts confidence
   - Test multiple signals in single session
   - Test confidence thresholds
3. Run: `cargo test -p vibes-groove attribution::activation`
4. Commit: `test(groove): add activation detection tests`

## Acceptance Criteria

- [ ] `ActivationDetector` trait defined
- [ ] `HybridActivationDetector` uses embedding similarity
- [ ] Explicit reference detection boosts confidence
- [ ] Configurable similarity threshold
- [ ] Returns detailed activation signals
- [ ] All tests pass

## Completion

> **IMPORTANT:** After all acceptance criteria are met:

1. Update this file's frontmatter: `status: done`
2. Move story: `just board done FEAT0027`
3. Commit, push, and create PR
