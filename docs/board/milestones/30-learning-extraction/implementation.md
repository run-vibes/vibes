# Milestone 30: Learning Extraction - Implementation Plan

## Stories

| ID | Title | Status | Estimate | Depends On |
|----|-------|--------|----------|------------|
| FEAT0019 | Learning types and storage | pending | 2h | - |
| FEAT0020 | Local embedder | pending | 3h | - |
| FEAT0021 | Semantic deduplication | pending | 2h | FEAT0020 |
| FEAT0022 | Correction detector | pending | 2h | - |
| FEAT0023 | Error recovery detector | pending | 2h | - |
| FEAT0024 | Extraction consumer | pending | 3h | FEAT0019, FEAT0021 |
| FEAT0025 | CLI commands | pending | 2h | FEAT0024 |

## Dependency Graph

```
FEAT0019 (types/storage) ──────────────────┐
                                           ├──► FEAT0024 (consumer) ──► FEAT0025 (CLI)
FEAT0020 (embedder) ──► FEAT0021 (dedup) ──┘

FEAT0022 (corrections) ────────────────────┘ (parallel, feeds into consumer)
FEAT0023 (error recovery) ─────────────────┘ (parallel, feeds into consumer)
```

## Execution Order

**Phase 1 - Foundation (parallel):**
- FEAT0019: Learning types and storage
- FEAT0020: Local embedder

**Phase 2 - Deduplication:**
- FEAT0021: Semantic deduplication (needs FEAT0020)

**Phase 3 - Pattern Detection (parallel):**
- FEAT0022: Correction detector
- FEAT0023: Error recovery detector

**Phase 4 - Integration:**
- FEAT0024: Extraction consumer (needs FEAT0019, FEAT0021)

**Phase 5 - CLI:**
- FEAT0025: CLI commands (needs FEAT0024)

---

## FEAT0019: Learning Types and Storage

**Goal:** Define core types and CozoDB storage layer.

### Steps

1. Create `plugins/vibes-groove/src/extraction/` module
2. Define types in `types.rs`:
   - `Learning`, `LearningId`, `LearningScope`, `LearningCategory`
   - `LearningPattern`, `LearningSource`, `ExtractionMethod`
   - `ExtractionEvent`
3. Define `LearningStore` trait in `store.rs`
4. Implement `CozoLearningStore`:
   - Schema creation on init
   - CRUD operations
   - `find_by_scope`, `find_similar`, `find_for_injection`
5. Add tests for all store operations

### Verification

```bash
cargo test -p vibes-groove extraction::store
```

---

## FEAT0020: Local Embedder

**Goal:** Implement local embedding using gte-small model.

### Steps

1. Add dependencies to `Cargo.toml`:
   - `ort` (ONNX Runtime)
   - `tokenizers`
2. Create `extraction/embedder.rs`:
   - `Embedder` trait definition
   - `LocalEmbedder` struct
3. Implement model download on first use:
   - Check `~/.cache/vibes/models/gte-small/`
   - Download from HuggingFace if missing
   - Show progress indicator
4. Implement `embed()` and `embed_batch()`:
   - Tokenization
   - ONNX inference
   - Mean pooling
5. Add `health_check()` for model validation
6. Add tests with sample texts

### Verification

```bash
cargo test -p vibes-groove extraction::embedder
```

---

## FEAT0021: Semantic Deduplication

**Goal:** Implement configurable deduplication strategy.

### Steps

1. Create `extraction/dedup.rs`:
   - `DeduplicationStrategy` trait
   - `SemanticDedup` implementation
2. Implement `find_duplicate()`:
   - Embed candidate description
   - Query HNSW index for similar learnings
   - Return match if above threshold
3. Implement `merge()`:
   - Average confidence scores
   - Update timestamp
   - Preserve original description
4. Add configuration for threshold
5. Add tests for dedup scenarios

### Verification

```bash
cargo test -p vibes-groove extraction::dedup
```

---

## FEAT0022: Correction Detector

**Goal:** Detect user corrections in transcripts.

### Steps

1. Create `extraction/patterns/mod.rs` and `correction.rs`:
   - `PatternDetector` trait
   - `CorrectionDetector` struct
   - `CorrectionCandidate` struct
2. Define default correction patterns:
   - "no, " / "No, "
   - "actually, " / "Actually, "
   - "I meant " / "I want "
   - "not X, Y" pattern
3. Implement `detect()`:
   - Scan user messages for patterns
   - Find Claude's acknowledgment in next message
   - Extract insight from correction
   - Calculate confidence
4. Add configuration for custom patterns
5. Add tests with sample transcripts

### Verification

```bash
cargo test -p vibes-groove extraction::patterns::correction
```

---

## FEAT0023: Error Recovery Detector

**Goal:** Detect tool failure → fix sequences.

### Steps

1. Create `extraction/patterns/error_recovery.rs`:
   - `ErrorRecoveryDetector` struct
   - `ErrorRecoveryCandidate` struct
2. Implement failure detection:
   - Bash tool with non-zero exit code
   - Compilation errors in output
   - Test failures in output
3. Implement recovery detection:
   - Subsequent successful tool call
   - Same or related file/command
4. Implement `detect()`:
   - Find failure → success pairs
   - Extract error type and recovery strategy
   - Calculate confidence based on recovery quality
5. Add tests with sample transcripts

### Verification

```bash
cargo test -p vibes-groove extraction::patterns::error_recovery
```

---

## FEAT0024: Extraction Consumer

**Goal:** Iggy consumer that orchestrates extraction pipeline.

### Steps

1. Create `extraction/consumer.rs`:
   - `ExtractionConsumer` struct
   - `ExtractionConfig` struct
2. Implement Iggy consumer setup:
   - Subscribe to `groove.assessment.heavy`
   - Resume from last acknowledged offset
3. Implement `process_heavy_event()`:
   - Collect LLM candidates from event
   - Run pattern detectors on transcript (if available)
   - Filter by minimum confidence
   - Embed and deduplicate each candidate
   - Persist to CozoDB
   - Write `ExtractionEvent` to Iggy
4. Add Iggy topic `groove.extraction`
5. Wire consumer startup into plugin lifecycle
6. Add integration tests

### Verification

```bash
cargo test -p vibes-groove extraction::consumer
```

---

## FEAT0025: CLI Commands

**Goal:** `vibes groove learn` subcommands.

### Steps

1. Add `learn` subcommand to groove CLI:
   - `status` - Show extraction status and counts
   - `list` - List learnings with filters
   - `show <id>` - Full learning details
   - `delete <id>` - Remove a learning
   - `export` - Export learnings as JSON
2. Implement `status`:
   - Query learning counts by scope
   - Show embedder status
   - Show recent extraction activity
3. Implement `list`:
   - `--scope project|user|global`
   - `--category correction|error_recovery|pattern`
   - Table output with ID, category, confidence, description
4. Implement `show`:
   - Full learning details
   - Source information
   - Embedding metadata
5. Implement `delete`:
   - Confirm before deletion
   - Remove from CozoDB
6. Implement `export`:
   - JSON output of all learnings
   - Optional scope filter
7. Add HTTP routes for CLI to query

### Verification

```bash
cargo test -p vibes-groove -- cli::learn
vibes groove learn status
vibes groove learn list --scope project
```

---

## Completion Checklist

- [ ] FEAT0019: Learning types and storage
- [ ] FEAT0020: Local embedder
- [ ] FEAT0021: Semantic deduplication
- [ ] FEAT0022: Correction detector
- [ ] FEAT0023: Error recovery detector
- [ ] FEAT0024: Extraction consumer
- [ ] FEAT0025: CLI commands
- [ ] All tests passing (`just test`)
- [ ] Pre-commit checks passing (`just pre-commit`)
- [ ] Documentation updated
