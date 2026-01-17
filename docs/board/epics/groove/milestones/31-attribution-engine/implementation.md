# Milestone 31: Attribution Engine - Implementation Plan

## Stories

| ID | Title | Status | Estimate | Depends On |
|----|-------|--------|----------|------------|
| FEAT0026 | Attribution types and storage | pending | 2h | - |
| FEAT0027 | Activation detection (Layer 1) | pending | 3h | FEAT0026, M30 (embedder) |
| FEAT0028 | Temporal correlation (Layer 2) | pending | 2h | FEAT0026 |
| FEAT0029 | Ablation manager (Layer 3) | pending | 3h | FEAT0026 |
| FEAT0030 | Value aggregation (Layer 4) | pending | 2h | FEAT0026, FEAT0028, FEAT0029 |
| FEAT0031 | Attribution consumer | pending | 3h | FEAT0027, FEAT0028, FEAT0030 |
| FEAT0032 | Auto-deprecation | pending | 1h | FEAT0030 |
| FEAT0033 | CLI commands | pending | 2h | FEAT0031 |

## Dependency Graph

```
                                    M30 (embedder)
                                         │
FEAT0026 (types) ──┬─────────────────────┼──────────────────┐
                   │                     │                  │
                   ▼                     ▼                  ▼
            FEAT0028 (temporal)    FEAT0027 (activation)   FEAT0029 (ablation)
                   │                     │                  │
                   └──────────┬──────────┘                  │
                              ▼                             │
                       FEAT0030 (aggregation) ◄─────────────┘
                              │
                   ┌──────────┴──────────┐
                   ▼                     ▼
            FEAT0032 (deprecation)  FEAT0031 (consumer)
                                         │
                                         ▼
                                   FEAT0033 (CLI)
```

## Execution Order

**Phase 1 - Foundation:**
- FEAT0026: Attribution types and storage

**Phase 2 - Layers (parallel after types):**
- FEAT0027: Activation detection (needs M30 embedder)
- FEAT0028: Temporal correlation
- FEAT0029: Ablation manager

**Phase 3 - Aggregation:**
- FEAT0030: Value aggregation

**Phase 4 - Integration:**
- FEAT0031: Attribution consumer
- FEAT0032: Auto-deprecation

**Phase 5 - CLI:**
- FEAT0033: CLI commands

---

## FEAT0026: Attribution Types and Storage

**Goal:** Define core types and CozoDB storage layer.

### Steps

1. Create `plugins/vibes-groove/src/attribution/` module
2. Define types in `types.rs`:
   - `AttributionRecord`, `ActivationSignal`
   - `LearningValue`, `LearningStatus`
   - `AblationExperiment`, `AblationResult`
   - `TemporalResult`, `ActivationResult`
3. Define `AttributionStore` trait in `store.rs`
4. Implement `CozoAttributionStore`:
   - Schema creation on init
   - CRUD for attribution records
   - Ablation experiment tracking
   - Learning value updates
5. Add tests for all store operations

### Verification

```bash
cargo test -p vibes-groove attribution::store
```

---

## FEAT0027: Activation Detection (Layer 1)

**Goal:** Detect if learnings influenced Claude's behavior.

### Steps

1. Create `attribution/activation.rs`:
   - `ActivationDetector` trait
   - `ActivationResult` struct
2. Implement `HybridActivationDetector`:
   - Embed Claude's responses
   - Compute cosine similarity with learning embeddings
   - Extract keywords from learning insight
   - Search for explicit references
   - Combine signals for confidence
3. Add configuration for thresholds
4. Add tests with sample transcripts

### Verification

```bash
cargo test -p vibes-groove attribution::activation
```

---

## FEAT0028: Temporal Correlation (Layer 2)

**Goal:** Weight signals by proximity to activation.

### Steps

1. Create `attribution/temporal.rs`:
   - `TemporalCorrelator` trait
   - `TemporalResult` struct
2. Implement `ExponentialDecayCorrelator`:
   - Find distance to nearest activation point
   - Apply exponential decay weight
   - Accumulate positive/negative signals
   - Compute net temporal score
3. Add configuration for decay rate and max distance
4. Add tests with sample signal sequences

### Verification

```bash
cargo test -p vibes-groove attribution::temporal
```

---

## FEAT0029: Ablation Manager (Layer 3)

**Goal:** Run A/B experiments by withholding uncertain learnings.

### Steps

1. Create `attribution/ablation.rs`:
   - `AblationStrategy` trait
   - `AblationExperiment` struct
   - `AblationResult` struct
2. Implement `ConservativeAblation`:
   - `should_withhold()` - check confidence, random selection
   - `is_experiment_complete()` - minimum sessions per arm
   - `compute_marginal_value()` - Welch's t-test
3. Add experiment persistence to store
4. Add configuration for thresholds and rates
5. Add tests for experiment lifecycle

### Verification

```bash
cargo test -p vibes-groove attribution::ablation
```

---

## FEAT0030: Value Aggregation (Layer 4)

**Goal:** Combine signals into final learning value.

### Steps

1. Create `attribution/aggregation.rs`:
   - `ValueAggregator` struct
2. Implement `update_value()`:
   - Running weighted average for temporal
   - Incorporate ablation results when significant
   - Confidence-weighted combination
   - Compute final estimated value
3. Add helper functions:
   - `weighted_update()` for running averages
   - `confidence_from_count()` for sample size
   - `combine_estimates()` for multi-source
4. Add tests for aggregation scenarios

### Verification

```bash
cargo test -p vibes-groove attribution::aggregation
```

---

## FEAT0031: Attribution Consumer

**Goal:** Iggy consumer that orchestrates attribution pipeline.

### Steps

1. Create `attribution/consumer.rs`:
   - `AttributionConsumer` struct
   - `AttributionConfig` struct
2. Implement Iggy consumer setup:
   - Subscribe to `groove.assessment.heavy`
   - Resume from last acknowledged offset
3. Implement `process_heavy_event()`:
   - Load active learnings from event
   - Load transcript (if available)
   - For each learning:
     - Run activation detection
     - Run temporal correlation
     - Check ablation status
     - Compute attributed value
     - Update learning value
   - Write attribution event to Iggy
4. Add Iggy topic `groove.attribution`
5. Wire consumer startup into plugin lifecycle
6. Add integration tests

### Verification

```bash
cargo test -p vibes-groove attribution::consumer
```

---

## FEAT0032: Auto-Deprecation

**Goal:** Automatically deprecate harmful learnings.

### Steps

1. Add deprecation logic to `ValueAggregator`:
   - Check value < threshold
   - Check confidence > threshold
   - Set status to `Deprecated`
2. Add `DeprecationEvent` to Iggy stream
3. Integrate with learning store:
   - Update learning status
   - Exclude from future injection
4. Add notification/logging for deprecations
5. Add tests for deprecation scenarios

### Verification

```bash
cargo test -p vibes-groove attribution::deprecation
```

---

## FEAT0033: CLI Commands

**Goal:** `vibes groove attr` subcommands.

### Steps

1. Add `attr` subcommand to groove CLI:
   - `status` - Show attribution engine status
   - `values` - List learning values with sort options
   - `show <learning-id>` - Detailed attribution breakdown
   - `explain <learning> <session>` - Why this attribution?
2. Add `learn enable/disable` commands:
   - `enable <id>` - Re-enable deprecated learning
   - `disable <id>` - Manually deprecate
3. Implement `status`:
   - Query engine configuration
   - Show learning value summary
   - Show recent activity
4. Implement `values`:
   - Query learning values from store
   - Sort by value/confidence/sessions
   - Table output
5. Implement `show`:
   - Full learning value breakdown
   - Per-source attribution
   - Recent sessions
6. Implement `explain`:
   - Load specific attribution record
   - Show activation signals
   - Show temporal correlation details
7. Add HTTP routes for CLI queries

### Verification

```bash
cargo test -p vibes-groove -- cli::attr
vibes groove attr status
vibes groove attr values --sort value
```

---

## Completion Checklist

- [ ] FEAT0026: Attribution types and storage
- [ ] FEAT0027: Activation detection (Layer 1)
- [ ] FEAT0028: Temporal correlation (Layer 2)
- [ ] FEAT0029: Ablation manager (Layer 3)
- [ ] FEAT0030: Value aggregation (Layer 4)
- [ ] FEAT0031: Attribution consumer
- [ ] FEAT0032: Auto-deprecation
- [ ] FEAT0033: CLI commands
- [ ] All tests passing (`just test`)
- [ ] Pre-commit checks passing (`just pre-commit`)
- [ ] Documentation updated
