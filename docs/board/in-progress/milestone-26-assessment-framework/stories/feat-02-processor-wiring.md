---
created: 2026-01-01
status: in_progress
---

# Feature: Wire Assessment Processor Pipeline

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Goal

Connect the assessment components (LightweightDetector, CircuitBreaker, SessionBuffer, CheckpointManager) in the AssessmentProcessor so events flow through the full pipeline.

## Context

All assessment components are implemented and tested individually, but `processor.process_event()` is currently a stub with TODOs. This story wires them together.

The processor should route events through:
1. **LightweightDetector** (B1) - Pattern matching, EMA signals
2. **CircuitBreaker** (B2) - Intervention decisions
3. **SessionBuffer** (B3) - Per-session event collection
4. **CheckpointManager** (B4) - Checkpoint triggers

See [design.md](../design.md) for the three-tier assessment model.

## Tasks

### Task 1: Wire LightweightDetector

**File:** `plugins/vibes-groove/src/assessment/processor.rs`

**Steps:**
1. Add `LightweightDetector` as a field in `AssessmentProcessor`
2. In `process_event()`, call `detector.process(&event)`
3. Emit `LightweightEvent` to the assessment log

**Verification:**
```bash
cargo test -p vibes-groove processor
```

**Commit:** `feat(groove): wire LightweightDetector in processor`

### Task 2: Wire CircuitBreaker

**File:** `plugins/vibes-groove/src/assessment/processor.rs`

**Steps:**
1. Add `CircuitBreaker` as a field (per-session via HashMap)
2. Feed lightweight signals to circuit breaker
3. When circuit breaker triggers, call intervention handler

**Verification:**
```bash
cargo test -p vibes-groove circuit_breaker
```

**Commit:** `feat(groove): wire CircuitBreaker in processor`

### Task 3: Wire SessionBuffer

**File:** `plugins/vibes-groove/src/assessment/processor.rs`

**Steps:**
1. Add `SessionBuffer` as a field
2. Buffer events per session
3. Use buffer for checkpoint context

**Commit:** `feat(groove): wire SessionBuffer in processor`

### Task 4: Wire CheckpointManager

**File:** `plugins/vibes-groove/src/assessment/processor.rs`

**Steps:**
1. Add `CheckpointManager` as a field
2. Check for checkpoint triggers after each event
3. When triggered, emit `MediumEvent` with segment summary

**Commit:** `feat(groove): wire CheckpointManager in processor`

### Task 5: Integration Test

**Steps:**
1. Create test that sends events through full pipeline
2. Verify lightweight events emitted
3. Verify circuit breaker state changes
4. Verify checkpoints triggered

**Commit:** `test(groove): add processor integration tests`

## Acceptance Criteria

- [ ] Events flow through LightweightDetector
- [ ] CircuitBreaker receives signals and can trigger interventions
- [ ] SessionBuffer collects events per session
- [ ] CheckpointManager triggers at appropriate times
- [ ] All existing tests still pass
- [ ] New integration test validates full pipeline
