# Milestone 34: Open-World Adaptation - Implementation Plan

## Stories

| ID | Title | Status | Estimate | Depends On |
|----|-------|--------|----------|------------|
| FEAT0052 | Core types and traits | pending | 2h | - |
| FEAT0053 | NoveltyDetector with embedding reuse | pending | 4h | FEAT0052, M30 |
| FEAT0054 | Incremental DBSCAN clustering | pending | 3h | FEAT0053 |
| FEAT0055 | CapabilityGapDetector | pending | 3h | FEAT0052, M31 |
| FEAT0056 | GraduatedResponse system | pending | 3h | FEAT0053, FEAT0055 |
| FEAT0057 | SolutionGenerator | pending | 2h | FEAT0055 |
| FEAT0058 | OpenWorldHook (M32 integration) | pending | 3h | FEAT0056, FEAT0057, M32 |
| FEAT0059 | CozoDB schema and store | pending | 2h | FEAT0052 |
| FEAT0060 | Iggy consumer and events | pending | 2h | FEAT0058, FEAT0059 |
| FEAT0061 | CLI commands (novelty) | pending | 2h | FEAT0053 |
| FEAT0062 | CLI commands (gaps) | pending | 2h | FEAT0055, FEAT0057 |
| FEAT0063 | Configuration and wiring | pending | 2h | FEAT0060 |

## Dependency Graph

```
                    M30 (Learning Extraction - embedder)
                              │
                    M31 (Attribution Engine)
                              │
                    M32 (Adaptive Strategies - NoveltyHook)
                              │
FEAT0052 (types) ─────────────┼────────────────────────────────────┐
        │                     │                                    │
        ├─────────────────────┼────────────────────────────────────┤
        ▼                     ▼                                    │
FEAT0053 (NoveltyDetector)   FEAT0055 (GapDetector)               │
        │                     │                                    │
        ▼                     │                                    │
FEAT0054 (DBSCAN)            │                                    │
        │                     │                                    │
        └────────┬────────────┘                                    │
                 ▼                                                 │
        FEAT0056 (GraduatedResponse)    FEAT0057 (SolutionGenerator)
                 │                              │                  │
                 └──────────────┬───────────────┘                  │
                                ▼                                  │
                    FEAT0058 (OpenWorldHook)                       │
                                │                                  │
        ┌───────────────────────┼───────────────────────┐          │
        ▼                       ▼                       ▼          │
FEAT0059 (CozoDB)       FEAT0060 (Iggy)         FEAT0061 (CLI novelty)
        │                       │                       │          │
        └───────────────────────┴───────────────────────┘          │
                                │                                  │
                    FEAT0062 (CLI gaps) ◄──────────────────────────┤
                                │                                  │
                    FEAT0063 (config) ◄────────────────────────────┘
```

## Execution Order

**Phase 1 - Foundation:**
- FEAT0052: Core types and traits
- FEAT0059: CozoDB schema and store

**Phase 2 - Detection (parallel):**
- FEAT0053: NoveltyDetector with embedding reuse
- FEAT0055: CapabilityGapDetector

**Phase 3 - Algorithms:**
- FEAT0054: Incremental DBSCAN clustering
- FEAT0057: SolutionGenerator

**Phase 4 - Response:**
- FEAT0056: GraduatedResponse system

**Phase 5 - Integration:**
- FEAT0058: OpenWorldHook (M32 integration)
- FEAT0060: Iggy consumer and events

**Phase 6 - CLI:**
- FEAT0061: CLI commands (novelty)
- FEAT0062: CLI commands (gaps)

**Phase 7 - Finalization:**
- FEAT0063: Configuration and wiring

---

## FEAT0052: Core Types and Traits

**Goal:** Define all core types for open-world adaptation.

### Steps

1. Create `plugins/vibes-groove/src/openworld/` module
2. Create `types.rs`:
   - `PatternFingerprint` struct
   - `AnomalyCluster` struct
   - `NoveltyResult` enum
   - `CapabilityGap` struct
   - `GapCategory`, `GapSeverity`, `GapStatus` enums
   - `FailureRecord` struct
   - `FailureType` enum
   - `SuggestedSolution` struct
   - `SolutionAction` enum
   - `SolutionSource` enum
   - `ResponseAction` enum
   - `ResponseStage` enum
   - `OpenWorldEvent` enum (for Iggy)
3. Create `traits.rs`:
   - `OpenWorldStore` trait
4. Create `mod.rs` with module exports
5. Add tests for type serialization

### Verification

```bash
cargo test -p vibes-groove openworld::types
cargo clippy -p vibes-groove
```

---

## FEAT0053: NoveltyDetector with Embedding Reuse

**Goal:** Implement novelty detection using M30's embedder.

### Steps

1. Create `plugins/vibes-groove/src/openworld/novelty.rs`
2. Implement `NoveltyDetector` struct:
   - Constructor that takes `Arc<dyn Embedder>` from M30
   - `detect()` method with embedding + fingerprint matching
   - `mark_known()` method to add successful patterns
   - `hash_context()` helper for fast pre-filtering
   - `find_nearest_cluster()` for cluster assignment
3. Implement cosine similarity helper
4. Add `AdaptiveParam` for similarity threshold
5. Wire embedder from groove plugin initialization
6. Add unit tests with mock embedder

### Verification

```bash
cargo test -p vibes-groove openworld::novelty
```

---

## FEAT0054: Incremental DBSCAN Clustering

**Goal:** Implement online clustering for novel patterns.

### Steps

1. Create `plugins/vibes-groove/src/openworld/clustering.rs`
2. Implement `incremental_dbscan()` function:
   - Take pending outliers + existing clusters
   - Find density-reachable points
   - Form new clusters or merge into existing
   - Return new/updated clusters
3. Implement helper functions:
   - `euclidean_distance()` for embedding space
   - `region_query()` for neighborhood search
   - `expand_cluster()` for cluster growth
4. Add `maybe_recluster()` to NoveltyDetector
5. Add tests with synthetic embeddings

### Verification

```bash
cargo test -p vibes-groove openworld::clustering
```

---

## FEAT0055: CapabilityGapDetector

**Goal:** Implement capability gap detection from combined signals.

### Steps

1. Create `plugins/vibes-groove/src/openworld/gaps.rs`
2. Implement `CapabilityGapDetector` struct:
   - `process_outcome()` main entry point
   - `detect_failure()` to classify failure type
   - `record_failure()` to track failures
   - `check_for_gap()` to aggregate into gaps
   - `update_severity()` for escalation
   - `get_or_create_gap()` for gap lifecycle
3. Implement failure clustering by context hash
4. Add configurable thresholds
5. Add tests for each failure type detection

### Verification

```bash
cargo test -p vibes-groove openworld::gaps
```

---

## FEAT0056: GraduatedResponse System

**Goal:** Implement progressive response to novelty.

### Steps

1. Create `plugins/vibes-groove/src/openworld/response.rs`
2. Implement `GraduatedResponse` struct:
   - `respond()` main entry point
   - `determine_stage()` based on cluster size
   - `respond_to_cluster()` stage-specific actions
   - `adjust_exploration()` feedback to M32
   - `create_gap_from_cluster()` for persistent novelty
   - `emit_gap_event()` for notifications
3. Implement `ResponseStages` configuration
4. Add `Arc<RwLock<StrategyLearner>>` for M32 integration
5. Add tests for each response stage

### Verification

```bash
cargo test -p vibes-groove openworld::response
```

---

## FEAT0057: SolutionGenerator

**Goal:** Implement solution generation for capability gaps.

### Steps

1. Create `plugins/vibes-groove/src/openworld/solutions.rs`
2. Implement `SolutionGenerator` struct:
   - `new()` with default templates
   - `default_templates()` for each gap category
   - `generate()` to produce solutions
   - `specialize_action()` to fill in gap-specific details
3. Implement `PatternAnalyzer` struct:
   - `find_solutions_from_similar_contexts()` using embeddings
4. Add templates for all four gap categories
5. Add tests for solution generation

### Verification

```bash
cargo test -p vibes-groove openworld::solutions
```

---

## FEAT0058: OpenWorldHook (M32 Integration)

**Goal:** Implement NoveltyHook trait for M32 integration.

### Steps

1. Create `plugins/vibes-groove/src/openworld/hook.rs`
2. Implement `OpenWorldHook` struct
3. Implement `NoveltyHook` trait:
   - `on_strategy_outcome()` full pipeline
4. Add helper methods:
   - `feedback_to_strategy_learner()` for exploration bonus
   - `adjust_category_priors()` for gap feedback
   - `emit_gap_confirmed()` for notifications
5. Wire into M32's `StrategyConsumer`
6. Add integration tests

### Verification

```bash
cargo test -p vibes-groove openworld::hook
```

---

## FEAT0059: CozoDB Schema and Store

**Goal:** Implement persistence for open-world data.

### Steps

1. Update `plugins/vibes-groove/src/store/schema.rs`:
   - Add `pattern_fingerprint` relation
   - Add `anomaly_cluster` relation
   - Add `cluster_member` relation
   - Add `capability_gap` relation
   - Add `gap_solution` relation
   - Add `failure_record` relation
   - Add `novelty_event` relation
   - Add HNSW index for fingerprint embeddings
   - Add standard indexes
2. Implement `OpenWorldStore` trait in `cozo.rs`:
   - `save_fingerprint()`, `load_fingerprints()`
   - `save_cluster()`, `load_clusters()`
   - `save_gap()`, `load_gaps()`, `update_gap_solutions()`
   - `save_failure()`, `load_failures_by_context()`
   - `save_novelty_event()`
3. Add migration for new relations
4. Add tests for store operations

### Verification

```bash
cargo test -p vibes-groove store::openworld
```

---

## FEAT0060: Iggy Consumer and Events

**Goal:** Implement event streaming for open-world.

### Steps

1. Create `plugins/vibes-groove/src/openworld/consumer.rs`
2. Define Iggy stream `groove.openworld` with topics:
   - `novelty` - detection events
   - `gaps` - gap lifecycle events
   - `feedback` - strategy feedback events
3. Implement `OpenWorldProducer`:
   - `emit_novelty_detected()`
   - `emit_cluster_updated()`
   - `emit_gap_status_changed()`
   - `emit_solutions_generated()`
   - `emit_strategy_feedback()`
4. Wire producer into OpenWorldHook
5. Add tests for event serialization

### Verification

```bash
cargo test -p vibes-groove openworld::consumer
```

---

## FEAT0061: CLI Commands (Novelty)

**Goal:** Implement novelty detection CLI commands.

### Steps

1. Create `plugins/vibes-groove/src/cli/novelty.rs`
2. Implement commands:
   - `vibes groove novelty status`
   - `vibes groove novelty clusters`
   - `vibes groove novelty cluster <id>`
   - `vibes groove novelty fingerprints`
   - `vibes groove novelty mark-known <hash>`
   - `vibes groove novelty reset`
3. Add clap subcommand definitions
4. Wire into groove CLI module
5. Add output formatting (table for lists)

### Verification

```bash
cargo build -p vibes-groove
./target/debug/vibes groove novelty --help
```

---

## FEAT0062: CLI Commands (Gaps)

**Goal:** Implement capability gap CLI commands.

### Steps

1. Create `plugins/vibes-groove/src/cli/gaps.rs`
2. Implement commands:
   - `vibes groove gaps status`
   - `vibes groove gaps list`
   - `vibes groove gaps show <id>`
   - `vibes groove gaps dismiss <id>`
   - `vibes groove gaps resolve <id>`
   - `vibes groove gaps apply <gap> <solution>`
3. Add clap subcommand definitions
4. Add combined commands:
   - `vibes groove openworld status`
   - `vibes groove openworld history`
5. Wire into groove CLI module

### Verification

```bash
cargo build -p vibes-groove
./target/debug/vibes groove gaps --help
./target/debug/vibes groove openworld --help
```

---

## FEAT0063: Configuration and Wiring

**Goal:** Wire all components together with configuration.

### Steps

1. Update `plugins/vibes-groove/src/config.rs`:
   - Add `OpenWorldConfig` struct
   - Add `NoveltyConfig` nested struct
   - Add `GapsConfig` nested struct
   - Add `ResponseConfig` nested struct
   - Add `SolutionsConfig` nested struct
2. Update groove plugin initialization:
   - Create `NoveltyDetector` with M30 embedder
   - Create `CapabilityGapDetector`
   - Create `GraduatedResponse` with M32 learner
   - Create `SolutionGenerator`
   - Create `OpenWorldHook` and register with M32
3. Add default configuration values
4. Add integration test for full pipeline

### Verification

```bash
cargo test -p vibes-groove openworld::integration
just test
```

---

## Completion Checklist

- [ ] FEAT0052: Core types and traits
- [ ] FEAT0053: NoveltyDetector with embedding reuse
- [ ] FEAT0054: Incremental DBSCAN clustering
- [ ] FEAT0055: CapabilityGapDetector
- [ ] FEAT0056: GraduatedResponse system
- [ ] FEAT0057: SolutionGenerator
- [ ] FEAT0058: OpenWorldHook (M32 integration)
- [ ] FEAT0059: CozoDB schema and store
- [ ] FEAT0060: Iggy consumer and events
- [ ] FEAT0061: CLI commands (novelty)
- [ ] FEAT0062: CLI commands (gaps)
- [ ] FEAT0063: Configuration and wiring
- [ ] All tests passing (`just test`)
- [ ] Pre-commit checks passing (`just pre-commit`)
- [ ] Documentation updated
