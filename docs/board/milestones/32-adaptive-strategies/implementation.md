# Milestone 32: Adaptive Strategies - Implementation Plan

## Stories

| ID | Title | Status | Estimate | Depends On |
|----|-------|--------|----------|------------|
| FEAT0034 | Strategy types and storage | pending | 2h | - |
| FEAT0035 | Strategy distribution hierarchy | pending | 2h | FEAT0034 |
| FEAT0036 | Thompson sampling learner | pending | 3h | FEAT0035, M21 (AdaptiveParam) |
| FEAT0037 | Outcome router | pending | 2h | FEAT0034, M31 (attribution) |
| FEAT0038 | Distribution updater | pending | 2h | FEAT0035, FEAT0037 |
| FEAT0039 | Strategy consumer | pending | 3h | FEAT0036, FEAT0038 |
| FEAT0040 | Novelty hook extension point | pending | 1h | FEAT0039 |
| FEAT0041 | CLI commands | pending | 2h | FEAT0039 |

## Dependency Graph

```
                                    M21 (AdaptiveParam)
                                         │
                                    M31 (Attribution)
                                         │
FEAT0034 (types) ────────────────────────┼────────────────────┐
        │                                │                    │
        ▼                                │                    │
FEAT0035 (distribution)                  │                    │
        │                                │                    │
        ├────────────────────────────────┘                    │
        │                                                     │
        ▼                                                     ▼
FEAT0036 (learner) ◄──────────────────────────────── FEAT0037 (router)
        │                                                     │
        └──────────────────┬──────────────────────────────────┘
                           │
                           ▼
                    FEAT0038 (updater)
                           │
                           ▼
                    FEAT0039 (consumer)
                           │
                ┌──────────┴──────────┐
                ▼                     ▼
        FEAT0040 (hook)        FEAT0041 (CLI)
```

## Execution Order

**Phase 1 - Foundation:**
- FEAT0034: Strategy types and storage

**Phase 2 - Distribution System:**
- FEAT0035: Strategy distribution hierarchy

**Phase 3 - Core Components (parallel after distribution):**
- FEAT0036: Thompson sampling learner (needs M21 AdaptiveParam)
- FEAT0037: Outcome router (needs M31 attribution)

**Phase 4 - Integration:**
- FEAT0038: Distribution updater
- FEAT0039: Strategy consumer

**Phase 5 - Extensions:**
- FEAT0040: Novelty hook extension point
- FEAT0041: CLI commands

---

## FEAT0034: Strategy Types and Storage

**Goal:** Define core types and CozoDB storage layer.

### Steps

1. Create `plugins/vibes-groove/src/strategy/` module
2. Define types in `types.rs`:
   - `InjectionStrategy` enum with all variants
   - `StrategyVariant` for simplified matching
   - `StrategyParams` for per-variant parameters
   - `StrategyOutcome`, `OutcomeSource`
   - `StrategyEvent` for Iggy events
3. Define `StrategyStore` trait in `store.rs`
4. Implement `CozoStrategyStore`:
   - Schema creation on init
   - CRUD for strategy distributions
   - Strategy event persistence
   - Session cache management
5. Add tests for all store operations

### Verification

```bash
cargo test -p vibes-groove strategy::store
```

---

## FEAT0035: Strategy Distribution Hierarchy

**Goal:** Implement hierarchical distributions with category priors and learning specialization.

### Steps

1. Create `strategy/distribution.rs`:
   - `StrategyDistribution` struct
   - `LearningStrategyOverride` struct
   - Helper methods for initialization
2. Implement `StrategyDistribution`:
   - `new()` with default priors from config
   - `get_weight()` for strategy variant
   - `update_weight()` with AdaptiveParam
3. Implement `LearningStrategyOverride`:
   - `new()` inheriting from category
   - `specialize_from()` to copy and diverge
   - `get_effective_weights()` delegation
4. Add serialization for CozoDB storage
5. Add tests for hierarchy behavior

### Verification

```bash
cargo test -p vibes-groove strategy::distribution
```

---

## FEAT0036: Thompson Sampling Learner

**Goal:** Implement strategy selection via Thompson sampling.

### Steps

1. Create `strategy/learner.rs`:
   - `StrategyLearner` struct
   - `StrategyLearnerConfig` struct
2. Implement `StrategyLearner`:
   - `new()` loading distributions from store
   - `select_strategy()` with lazy caching
   - `sample_strategy()` using Thompson sampling
   - `sample_params()` for strategy parameters
   - `get_effective_weights()` hierarchy lookup
   - `clear_session()` cache cleanup
3. Integrate with `AdaptiveParam::sample()` from M21
4. Add configuration loading from TOML
5. Add tests for sampling behavior

### Verification

```bash
cargo test -p vibes-groove strategy::learner
```

---

## FEAT0037: Outcome Router

**Goal:** Route attribution and direct signals to strategy outcomes.

### Steps

1. Create `strategy/router.rs`:
   - `OutcomeRouter` struct
   - `OutcomeRouterConfig` struct
2. Implement `OutcomeRouter`:
   - `new()` with configurable weights
   - `compute_outcome()` combining signals
   - `aggregate_direct_signals()` for lightweight events
3. Handle all source combinations:
   - Both attribution and direct (weighted average)
   - Attribution only (full weight)
   - Direct only (lower confidence)
   - Neither (skip update)
4. Add configuration loading
5. Add tests for all signal combinations

### Verification

```bash
cargo test -p vibes-groove strategy::router
```

---

## FEAT0038: Distribution Updater

**Goal:** Update distributions based on strategy outcomes.

### Steps

1. Create `strategy/updater.rs`:
   - `DistributionUpdater` struct
   - `UpdaterConfig` struct
2. Implement `DistributionUpdater`:
   - `new()` with specialization threshold
   - `update()` main update logic
   - Category distribution update
   - Learning override creation/update
   - Specialization trigger logic
3. Implement specialization:
   - Check session count vs threshold
   - Copy weights from category
   - Mark as specialized
4. Add tests for update scenarios

### Verification

```bash
cargo test -p vibes-groove strategy::updater
```

---

## FEAT0039: Strategy Consumer

**Goal:** Iggy consumer that orchestrates strategy learning pipeline.

### Steps

1. Create `strategy/consumer.rs`:
   - `StrategyConsumer` struct
   - `StrategyConsumerConfig` struct
2. Implement Iggy consumer setup:
   - Subscribe to `groove.attribution`
   - Resume from last acknowledged offset
3. Implement `process_attribution_event()`:
   - Load current distributions
   - For each attribution record:
     - Route outcome from signals
     - Update distributions
     - Write strategy event to Iggy
     - Invoke novelty hook if present
   - Persist updated distributions
4. Add Iggy topic `groove.strategy`
5. Wire consumer startup into plugin lifecycle
6. Add integration tests

### Verification

```bash
cargo test -p vibes-groove strategy::consumer
```

---

## FEAT0040: Novelty Hook Extension Point

**Goal:** Add clean extension point for future novelty detection.

### Steps

1. Create `strategy/novelty.rs`:
   - `NoveltyHook` trait definition
   - `NoOpNoveltyHook` default implementation
2. Define trait methods:
   - `on_strategy_outcome()` - called for each outcome
   - `on_session_end()` - called at session boundary
3. Add `with_novelty_hook()` builder to `StrategyConsumer`
4. Document extension point usage
5. Add tests for hook invocation

### Verification

```bash
cargo test -p vibes-groove strategy::novelty
```

---

## FEAT0041: CLI Commands

**Goal:** `vibes groove strategy` subcommands.

### Steps

1. Add `strategy` subcommand to groove CLI:
   - `status` - Show strategy learner status
   - `distributions` - List category distributions
   - `show <category>` - Detailed distribution breakdown
   - `learning <id>` - Show learning's strategy override
   - `history <learning>` - Strategy selection history
   - `reset <category>` - Reset category to default priors
   - `reset-learning <id>` - Clear learning specialization
2. Implement `status`:
   - Query distribution counts
   - Show top performing strategies
   - Show recent activity
3. Implement `distributions`:
   - List all category/context combinations
   - Show session counts and top strategy
4. Implement `show`:
   - Full distribution breakdown with Beta params
   - Visual bar chart of weights
   - List specialized learnings
5. Implement `learning`:
   - Show override status (inheriting vs specialized)
   - Show effective weights
   - Show session history
6. Implement `history`:
   - Query strategy events for learning
   - Show selection timeline
7. Implement `reset` commands:
   - Reset to default priors
   - Require confirmation
8. Add HTTP routes for CLI queries

### Verification

```bash
cargo test -p vibes-groove -- cli::strategy
vibes groove strategy status
vibes groove strategy distributions
```

---

## Completion Checklist

- [ ] FEAT0034: Strategy types and storage
- [ ] FEAT0035: Strategy distribution hierarchy
- [ ] FEAT0036: Thompson sampling learner
- [ ] FEAT0037: Outcome router
- [ ] FEAT0038: Distribution updater
- [ ] FEAT0039: Strategy consumer
- [ ] FEAT0040: Novelty hook extension point
- [ ] FEAT0041: CLI commands
- [ ] All tests passing (`just test`)
- [ ] Pre-commit checks passing (`just pre-commit`)
- [ ] Documentation updated
