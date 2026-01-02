# Milestone 4.2 Storage Foundation: Design Decisions

> Decisions made during brainstorming session 2025-12-28

## Table of Contents

1. [Integration Architecture](#integration-architecture)
2. [Storage Backend](#storage-backend)
3. [Database Schema](#database-schema)
4. [Tiered Storage Architecture](#tiered-storage-architecture)
5. [Learning Flow Model](#learning-flow-model)
6. [Enterprise Governance](#enterprise-governance-future)

---

## Integration Architecture

### Decision 1: CozoDB Location

**Decision:** CozoDB lives in `vibes-groove` plugin (self-contained).

**Options Considered:**

| Option | Description | Verdict |
|--------|-------------|---------|
| A: In vibes-groove | groove owns its storage | ✅ Chosen |
| B: In vibes-core | Shared storage infrastructure | ❌ Premature abstraction |
| C: Separate vibes-storage crate | New crate for CozoDB wrapper | ❌ Over-engineering |

**Rationale:** groove is *the* learning system, not one of many plugins that need shared storage. Keeping CozoDB in groove avoids premature abstraction and keeps all learning logic in one place.

---

### Decision 2: Harness Access

**Decision:** Extend `PluginContext` with `harness()` and `capabilities()` methods.

**Options Considered:**

| Option | Description | Verdict |
|--------|-------------|---------|
| A: Direct dependency | groove imports vibes-introspection directly | ❌ Plugin doesn't know command being proxied |
| B: Injected trait object | groove receives `Arc<dyn Harness>` at construction | ❌ Conflicts with `Default` plugin creation |
| C: Event-driven | Capabilities sent via events | ❌ Requires Plugin trait changes |
| D: Extend PluginContext | Add harness/capabilities to context | ✅ Chosen |

**Rationale:** The plugin system creates plugins via `Default::default()`, then calls `on_load(ctx)`. We can't inject constructor parameters, but we can extend what `PluginContext` provides. This is non-breaking for existing plugins.

**Implementation:**

```rust
pub struct PluginContext {
    // existing fields...
    harness: Option<Arc<dyn Harness>>,
    capabilities: Option<HarnessCapabilities>,
    project_root: Option<PathBuf>,
}

impl PluginContext {
    pub fn harness(&self) -> Option<&dyn Harness> { ... }
    pub fn capabilities(&self) -> Option<&HarnessCapabilities> { ... }
    pub fn project_root(&self) -> Option<&Path> { ... }
}
```

---

### Decision 3: Capability Updates

**Decision:** Live updates — groove subscribes to `CapabilityWatcher` and reacts mid-session.

**Options Considered:**

| Option | Description | Verdict |
|--------|-------------|---------|
| A: Snapshot at session start | Read once, ignore changes | ❌ Less responsive |
| B: Live updates | Subscribe to watcher, react mid-session | ✅ Chosen |
| C: Live with session boundary | Watch changes, apply next session | ❌ Unnecessary delay |

**Rationale:** Users should be able to modify their hooks or injection targets and see immediate effect. groove holds a `CapabilityWatcher` subscription and updates its injection strategy when targets change.

---

## Storage Backend

### Decision 4: CozoDB Backend

**Decision:** Use RocksDB backend for all tiers.

**Options Considered:**

| Backend | Pros | Cons | Verdict |
|---------|------|------|---------|
| Memory | Fastest | No persistence | ❌ Not suitable |
| SQLite | Simple, portable, single file | Single writer, write lock contention | ❌ Doesn't scale to swarm |
| RocksDB | Concurrent writes, LSM tree, scales | Directory-based, higher memory | ✅ Chosen |

**Rationale:** Designing for future swarm scenario where many agents write learnings concurrently. RocksDB's LSM tree handles concurrent writes without blocking, and background compaction keeps performance smooth under load.

**Swarm Scenario:**

```
Orchestrator (vibes server)
    ├── Agent 1 ──┐
    ├── Agent 2 ──┼── All writing learnings concurrently
    ├── Agent 3 ──┤
    └── Agent N ──┘
            │
            ▼
      groove.db (RocksDB) ← Handles concurrent writes natively
```

**Performance Characteristics:**

| Factor | SQLite | RocksDB |
|--------|--------|---------|
| 10 agents, 1 write/min each | Write queue, latency spikes | Smooth concurrent writes |
| Burst of 100 learnings | Lock contention, timeouts | LSM handles burst |
| Reads during writes | WAL helps but writes block | Reads never blocked |

---

## Database Schema

### Decision 5: Scope Representation

**Decision:** Composite scope string with `type:value` format.

**Options Considered:**

| Option | Description | Verdict |
|--------|-------------|---------|
| Split columns | `scope_type: String, scope_value: String?` | ❌ Two columns to manage |
| Composite | Single `scope: String` with pattern `type:value` | ✅ Chosen |

**Format:**

- `"global"` — no colon, global scope
- `"user:alex"` — user-scoped
- `"project:/home/alex/myrepo"` — project-scoped

**Query Performance:** CozoDB's `starts_with()` uses index prefix scan, so `starts_with(scope, "user:")` is efficient.

**Rust Implementation:**

```rust
impl Scope {
    pub fn to_db_string(&self) -> String {
        match self {
            Scope::Global => "global".to_string(),
            Scope::User(id) => format!("user:{id}"),
            Scope::Project(path) => format!("project:{path}"),
        }
    }

    pub fn from_db_string(s: &str) -> Result<Self> {
        if s == "global" { return Ok(Scope::Global) }
        if let Some(id) = s.strip_prefix("user:") {
            return Ok(Scope::User(id.into()))
        }
        if let Some(path) = s.strip_prefix("project:") {
            return Ok(Scope::Project(path.into()))
        }
        Err(GrooveError::InvalidScope(s.into()))
    }
}
```

---

### Decision 6: Embedding Dimensions

**Decision:** Start with 384-dim local embeddings (GteSmall). Add 1536-dim API embeddings later if search quality proves insufficient.

**Options Considered:**

| Option | Description | Verdict |
|--------|-------------|---------|
| A: Fixed 1536, pad smaller | Normalize all to largest dimension | ❌ Semantically incorrect, wastes space |
| B: Multiple tables | Separate HNSW index per dimension | ⚠️ Complex, can't compare across models |
| C: Realtime + background split | 384 for hot path, 1536 for analysis | ✅ Future state if needed |
| D: Single model (384) | GteSmall everywhere | ✅ **Chosen for MVP** |

**Trade-offs Accepted:**

| Trade-off | Mitigation |
|-----------|------------|
| Lower embedding quality than API models | GteSmall is "good enough" for semantic search; groove's value comes from *having* learnings, not perfect retrieval |
| Migration cost if we switch | Re-embedding is a background job; 10K learnings × 5ms = 50 seconds |
| No offline/online quality split | Can add 1536-dim table later without schema break |

**Storage Estimates (384-dim):**

| Learnings | Embedding Storage | HNSW Index |
|-----------|------------------|------------|
| 1,000 | 1.5 MB | ~1.7 MB |
| 10,000 | 15 MB | ~17 MB |
| 100,000 | 150 MB | ~170 MB |

**Migration Path:**

1. Add `learning_embeddings_1536` table with HNSW index
2. Background job re-embeds existing learnings via API
3. Query logic: use 1536 when available, fallback to 384
4. Eventually deprecate 384 table if API is always available

**Schema:**

```datalog
:create learning_embeddings {
    learning_id: String =>
    embedding: <F32; 384>
}

::hnsw create learning_embeddings:semantic_idx {
    dim: 384,
    m: 16,
    ef_construction: 200,
    fields: [embedding]
}
```

---

## Tiered Storage Architecture

### Decision 7: Three-Tier Database Model

**Decision:** Separate RocksDB database per tier (user, project, enterprise).

**Architecture:**

```
USER TIER (RocksDB)
~/.local/share/vibes/groove/user/
├── learnings (personal patterns)
├── promotion_queue (pending proposals to enterprise)
└── preferences (copied from enterprise)

PROJECT TIER (RocksDB)
/project/.vibes/groove/
└── learnings (project-specific)

ENTERPRISE TIER (RocksDB) - Optional
/shared/org/vibes/groove/  OR  ~/.vibes/enterprise/acme/
├── learnings (curated org patterns)
├── pending_reviews (awaiting curator approval)
└── rejection_log (audit trail)
```

**Context-Aware Reading:**

| Project Context | Tiers Read |
|-----------------|------------|
| Personal project | User + Project |
| Enterprise project | User + Project + Enterprise |

**Implementation:**

```rust
enum ProjectContext {
    Personal,
    Enterprise { org_id: String },
}

impl GrooveStorage {
    fn get_read_tiers(&self, ctx: &ProjectContext) -> Vec<&dyn LearningStore> {
        let mut tiers = vec![&self.user_db, &self.project_db];

        if let ProjectContext::Enterprise { org_id } = ctx {
            if let Some(enterprise_db) = self.enterprise_dbs.get(org_id) {
                tiers.push(enterprise_db);
            }
        }

        tiers
    }
}
```

**Configuration:**

```toml
# ~/.config/vibes/groove.toml (user-level)
[user]
db_path = "~/.local/share/vibes/groove/user"

[enterprises.acme-corp]
db_path = "/shared/acme/vibes/groove"

[enterprises.personal-llc]
db_path = "~/.local/share/vibes/groove/enterprises/personal-llc"
```

```toml
# /work-project/.vibes/config.toml (project-level)
[groove]
enterprise = "acme-corp"  # Links to enterprise config
```

```toml
# /personal-project/.vibes/config.toml
[groove]
# No enterprise key = personal context
```

---

## Learning Flow Model

### Decision 8: Start Local, Promote Up

**Decision:** All learnings start at project scope, then get promoted based on demonstrated value.

**Promotion Flow:**

```
PROJECT TIER (all learnings start here)
       │
       │  User: "promote to personal"
       ▼
USER TIER (immediate, no review)
       │
       │  User: "propose for enterprise"
       ▼
ENTERPRISE TIER (LLM + curator approval)
```

**Handoff Processes:**

| Transition | Trigger | Review |
|------------|---------|--------|
| Project → User | User command "promote" | None (user's personal tier) |
| Project → Enterprise | User command "propose" | LLM pre-filter + curator approval |
| User → Enterprise | User command "propose" | LLM pre-filter + curator approval |
| Enterprise → User | User command "copy to personal" | None (user's personal copy) |

**LLM Evaluation for Enterprise Proposals:**

```rust
struct LLMEvaluation {
    generalizability: f64,      // Is this broadly applicable?
    quality: f64,               // Is it clearly described?
    conflicts: Vec<Conflict>,   // Contradicts existing learnings?
    risk_flags: Vec<RiskFlag>,  // Security/compliance concerns?
    recommendation: Recommendation,
    confidence: f64,
}

enum RiskFlag {
    ContainsSecrets,
    SecurityAntiPattern,
    LicenseConflict,
    ComplianceRisk,
}
```

---

## Enterprise Governance (Future)

### Decision 9: Dedicated Governance Milestone

**Decision:** Enterprise learning governance policies are out of scope for 4.2. Create dedicated milestone (4.X Governance).

**Policy Model (Future):**

```toml
# /shared/acme/vibes/groove/policy.toml
[governance]
outbound_policy = "require_approval"
promotion_from_enterprise_context = "block_all"
personal_learning_during_enterprise = "block"
```

**Policy Options:**

| Policy | Values | Effect |
|--------|--------|--------|
| `outbound_policy` | block_all, require_approval, allow_generic, allow_all | Controls copying FROM enterprise TO user |
| `promotion_from_enterprise_context` | block_all, require_approval, allow | Controls promoting learnings discovered ON enterprise projects |
| `personal_learning_during_enterprise` | block, allow, allow_non_code | Controls writing to user tier during enterprise sessions |

**Default Behaviors:**

| Context | Default | Rationale |
|---------|---------|-----------|
| No enterprise config | All allowed | Personal/indie use |
| Enterprise, no explicit policy | require_approval | Safe default |
| Enterprise with explicit policy | Policy enforced | Enterprise controls |

**Future Considerations:**

- Audit logging of policy enforcement
- Admin dashboard for policy configuration
- DLP integration (scan for secrets/PII)
- Learning "anonymization" (strip code, keep pattern)

---

## Schema Operations

### Decision 10: Schema Migrations

**Decision:** Schema versioning table with explicit migration scripts.

**Options Considered:**

| Option | Description | Verdict |
|--------|-------------|---------|
| A: Schema versioning table | Track version, run migrations on startup | ✅ Chosen |
| B: Append-only evolution | Never modify, only add new relations | ❌ Accumulates cruft |
| C: CozoDB `:replace` | In-place schema updates | ❌ Risky for type changes |

**Implementation:**

```datalog
:create schema_version {
    version: Int =>
    applied_at: Int,
    description: String
}
```

```rust
impl GrooveStorage {
    async fn ensure_schema(&self) -> Result<()> {
        let current = self.get_schema_version().await?;

        for migration in MIGRATIONS.iter().filter(|m| m.version > current) {
            self.db.run(&migration.script)?;
            self.record_migration(migration).await?;
        }

        Ok(())
    }
}
```

---

### Decision 11: Adaptive Parameters — Two-Level Hierarchy

**Decision:** Both system-wide parameters AND per-learning parameters (Option C).

**Rationale:** System params and per-learning params answer different questions:
- **System:** "How should groove behave in general?"
- **Per-learning:** "How reliable is this specific piece of knowledge?"

#### System-Wide Parameters

Control groove's global behavior. Stored in `adaptive_params` table.

| Parameter | What It Controls | Why Adaptive |
|-----------|------------------|--------------|
| `injection_budget` | Max learnings to inject per session | Too many = noise, too few = missed value |
| `confidence_threshold` | Min confidence to inject a learning | Starts conservative, adjusts based on outcomes |
| `semantic_similarity_threshold` | How similar must context be? | Too strict = never triggers, too loose = irrelevant |
| `assessment_sample_rate` | % of sessions with heavy assessment | Balance cost vs learning speed |
| `ablation_frequency` | How often to run ablation tests | Need data without being disruptive |

**Example Evolution:**

```
Session 1:  injection_budget = 5.0 → user frustrated (noise)
Session 10: injection_budget = 4.2 → better
Session 50: injection_budget = 3.2 → system learned optimal for this user
```

#### Per-Learning Parameters

Control individual learning behavior. Stored in `learning` (confidence) and `usage_stats` tables.

| Parameter | What It Controls | Why Per-Learning |
|-----------|------------------|------------------|
| `confidence` | Base confidence in this learning | Updated when learning helps/hurts |
| `times_helpful` | Count of positive attributions | Track proven value |
| `times_ignored` | Count of neutral outcomes | Track relevance |
| `times_contradicted` | Count of negative attributions | Track harm potential |
| `confidence_alpha/beta` | Bayesian priors for this learning | Per-learning uncertainty |

**Example Evolution:**

```
Learning: "Use Result types for error handling"

Usage 1: Injected → user followed → commit succeeded
         confidence: 0.50 → 0.65, times_helpful: 1

Usage 5: Injected → user corrected it
         confidence: 0.72 → 0.58, times_contradicted: 1

Usage 10: confidence = 0.82 (proven valuable)
          times_helpful: 7, times_ignored: 2, times_contradicted: 1
```

#### How They Work Together

```rust
async fn should_inject(&self, learning: &Learning, context: &SessionContext) -> bool {
    // 1. System-level: check confidence threshold
    let threshold = self.system_params.get("confidence_threshold").sample();
    if learning.confidence < threshold {
        return false;
    }

    // 2. System-level: check semantic similarity threshold
    let similarity_threshold = self.system_params.get("similarity_threshold").sample();
    let similarity = self.compute_similarity(learning, context).await;
    if similarity < similarity_threshold {
        return false;
    }

    // 3. System-level: check injection budget
    let budget = self.system_params.get("injection_budget").sample() as usize;
    if self.injected_count >= budget {
        return false;
    }

    // 4. Per-learning: has this learning caused more harm than good?
    if learning.usage_stats.times_contradicted > learning.usage_stats.times_helpful {
        return false;
    }

    true
}

async fn update_from_outcome(&mut self, session: &Session, outcome: &Outcome) {
    // Update SYSTEM parameters based on overall session
    if outcome.was_successful() {
        self.system_params.get_mut("injection_budget")
            .update(outcome: 0.8, weight: 1.0);
    }

    // Update PER-LEARNING parameters based on attribution
    for (learning_id, attribution) in outcome.attributions {
        let learning = self.get_learning(learning_id).await?;

        match attribution {
            Attribution::Helpful => {
                learning.usage_stats.times_helpful += 1;
                learning.confidence.update(outcome: 0.9, weight: 1.0);
            }
            Attribution::Ignored => {
                learning.usage_stats.times_ignored += 1;
            }
            Attribution::Contradicted => {
                learning.usage_stats.times_contradicted += 1;
                learning.confidence.update(outcome: 0.1, weight: 1.5);
            }
        }

        self.save_learning(learning).await?;
    }
}
```

#### Schema

```datalog
# System-wide adaptive parameters
:create adaptive_params {
    param_name: String =>
    value: Float,
    uncertainty: Float,
    observations: Int,
    prior_alpha: Float,
    prior_beta: Float,
    updated_at: Int
}

# Core learning entity (includes base confidence)
:create learning {
    id: String =>
    scope: String,
    category: String,
    description: String,
    pattern_json: String,
    insight: String,
    confidence: Float,
    created_at: Int,
    updated_at: Int,
    source_type: String,
    source_json: String
}

# Per-learning usage statistics (updated frequently)
:create usage_stats {
    learning_id: String =>
    times_injected: Int,
    times_helpful: Int,
    times_ignored: Int,
    times_contradicted: Int,
    last_used: Int?,
    confidence_alpha: Float,
    confidence_beta: Float
}
```

---

### Decision 12: Learning Relationships — Schema Now, Logic Later

**Decision:** Include `learning_relations` table in 4.2 schema, but defer relationship detection logic to later milestones.

**Rationale:** The schema is cheap (just a table definition), avoids migration later, and prepares the foundation for conflict detection, provenance tracking, and clustering without adding complexity to 4.2.

#### Relationship Types

| Relation | Meaning | Concrete Example |
|----------|---------|------------------|
| `supersedes` | B replaces outdated A | "Use async/await" supersedes "Use callbacks" after codebase migration |
| `contradicts` | A and B conflict | "Use snake_case" contradicts "Use camelCase" (different project conventions) |
| `derived_from` | B was generalized from A | "Check token validity before expensive ops" derived from specific "Check token expiry in UserService.authenticate()" |
| `related_to` | Similar topic, complementary | Multiple error handling learnings that work together |
| `specializes` | B is specific case of general A | "Use strong parameters in Rails" specializes "Validate inputs at boundaries" |

#### Deep Dive: Each Relationship Type

##### `supersedes` — Deprecation & Evolution

**Scenario:**
```
Day 1:  Learning A: "Use callbacks for async operations"
        confidence: 0.8, scope: project

Day 30: Codebase migrates to async/await

Day 31: Learning B: "Use async/await for async operations"
        confidence: 0.7, scope: project

Problem: Both exist. groove might inject outdated callback pattern.
```

**With relation:**
```
B --supersedes--> A

Injection behavior:
  - groove sees B supersedes A
  - Only injects B
  - A is effectively deprecated (auto-lower confidence)
```

**Detection methods (future):**
- Manual: User says "this replaces that"
- Automatic: LLM detects contradiction + temporal order
- Heuristic: Same category + same scope + high similarity + newer

##### `contradicts` — Conflict Detection

**Scenario:**
```
Learning A (user tier): "Always use snake_case for functions"
Learning B (project tier): "Use camelCase for functions"

User works on multiple projects with different conventions.
```

**With relation:**
```
A --contradicts--> B (bidirectional)

Injection behavior:
  - groove sees conflict
  - Project scope takes precedence in project context
  - Only injects B for this project
  - Could notify: "Your user preference differs from project convention"
```

**Detection methods (future):**
- LLM analysis: Semantic similarity + opposite recommendations
- User feedback: "These conflict"
- Automatic: User contradicted A while B was injected

##### `derived_from` — Provenance Tracking

**Scenario:**
```
Day 1: Learning A (specific):
  "In UserService.authenticate(), always check token expiry before database lookup"
  scope: project, category: CodePattern

Day 10: User promotes generalized version

Learning B (general):
  "Check token validity before expensive operations"
  scope: user, category: Preference

B --derived_from--> A
```

**Why it matters:**
- **Provenance:** Where did this knowledge come from?
- **Confidence inheritance:** If A is proven, B gets credibility boost
- **Debugging:** "Why does groove think this?" → trace to source

##### `related_to` — Clustering & Discovery

**Scenario:**
```
Learning A: "Use Result<T, E> for error handling"
Learning B: "Map errors to domain-specific types at boundaries"
Learning C: "Log errors with context before propagating"

All about error handling, complementary, not conflicting.
```

**With relations:**
```
A --related_to--> B
A --related_to--> C
B --related_to--> C

Use cases:
  - Query: "Show me all learnings about error handling"
  - Injection: When injecting A, consider B and C too
  - Dashboard: "Error Handling (3 learnings)" cluster
```

**Detection methods (future):**
- Semantic similarity above threshold
- Same category + similar embeddings
- User manually groups them

##### `specializes` — Hierarchy

**Scenario:**
```
Learning A (general):
  "Validate inputs at system boundaries"
  scope: user, category: Preference

Learning B (specific):
  "In Rails controllers, use strong parameters for input validation"
  scope: project, category: CodePattern

B --specializes--> A
```

**Why it matters:**
- **Injection priority:** Rails context → prefer B over A
- **Confidence flow:** High-confidence A → B inherits credibility
- **Generalization:** Extract B into A for broader applicability

#### Implementation Timeline

| Milestone | Relationship Logic Added |
|-----------|-------------------------|
| 4.2 Storage | Schema only (table empty) |
| 4.3 Capture & Inject | `contradicts` detection for conflict avoidance |
| 4.5 Learning Extraction | `derived_from` for provenance, `supersedes` for evolution |
| 4.8 Dashboard | `related_to` for clustering and visualization |
| 4.9 Open-World | `specializes` for hierarchy detection |

#### Schema

```datalog
:create learning_relations {
    from_id: String,
    relation_type: String,
    to_id: String =>
    weight: Float,
    created_at: Int
}

::index create learning_relations:by_from { from_id }
::index create learning_relations:by_to { to_id }
```

#### Query Examples (for future milestones)

```datalog
# Find all learnings that supersede a given learning
?[superseding_id, description] :=
    *learning_relations{from_id: superseding_id, relation_type: "supersedes", to_id: $target_id},
    *learning{id: superseding_id, description}

# Find learnings in conflict with any learning we're about to inject
?[conflicting_id, description] :=
    *learning_relations{from_id: $candidate_id, relation_type: "contradicts", to_id: conflicting_id},
    *learning{id: conflicting_id, description}

# Get all related learnings (for clustering)
?[related_id, description] :=
    *learning_relations{from_id: $learning_id, relation_type: "related_to", to_id: related_id},
    *learning{id: related_id, description}

# Trace provenance chain
?[ancestor_id, depth] :=
    *learning_relations{from_id: $learning_id, relation_type: "derived_from", to_id: ancestor_id},
    depth = 1
?[ancestor_id, depth] :=
    *learning_relations{from_id: intermediate, relation_type: "derived_from", to_id: ancestor_id},
    ?[intermediate, prev_depth],
    depth = prev_depth + 1
```

---

### Decision 13: Backup & Recovery — Minimal Viable for MVP

**Decision:** Implement minimal backup (snapshots before migration + manual JSON export/import). Defer automatic snapshots and team sync features.

#### MVP Scope

| Feature | In 4.2 | Deferred |
|---------|--------|----------|
| RocksDB snapshot before migration | ✅ | |
| Manual `vibes groove export` command | ✅ | |
| Manual `vibes groove import` command | ✅ | |
| Re-embed on import (exclude embeddings from JSON) | ✅ | |
| Automatic periodic snapshots | | Future |
| Auto-generated project export.json | | Future |
| Snapshot pruning policies | | Future |
| Team sync via git | | Future |

#### Export Format

Embeddings excluded from JSON exports (too large). Re-generated on import.

```rust
struct GrooveExport {
    version: u32,
    exported_at: DateTime<Utc>,
    learnings: Vec<LearningExport>,  // No embeddings field
    params: Vec<AdaptiveParam>,
    relations: Vec<Relation>,
}

impl GrooveStorage {
    async fn export(&self, path: &Path) -> Result<()> {
        let export = GrooveExport {
            version: EXPORT_VERSION,
            exported_at: Utc::now(),
            learnings: self.all_learnings_without_embeddings().await?,
            params: self.all_params().await?,
            relations: self.all_relations().await?,
        };
        std::fs::write(path, serde_json::to_string_pretty(&export)?)?;
        Ok(())
    }

    async fn import(&self, path: &Path) -> Result<ImportStats> {
        let export: GrooveExport = serde_json::from_str(&std::fs::read_to_string(path)?)?;

        for learning in export.learnings {
            self.store_learning(&learning).await?;
            // Queue for background re-embedding
            self.embedding_queue.push(learning.id);
        }

        // Background job processes embedding queue
        Ok(ImportStats { ... })
    }
}
```

#### Recovery Scenarios

| Scenario | Recovery Path |
|----------|---------------|
| Corruption after migration | Restore from pre-migration snapshot |
| Moving to new machine | Export → transfer → import |
| Accidental deletion | Restore from external backup (Time Machine, etc.) |

---

### Decision 14: Connection Management — Eager User, Lazy Others

**Decision:** Open user database eagerly at startup. Open project and enterprise databases lazily on first access.

#### Rationale

- **User DB:** Always needed, fail fast if corrupted
- **Project DB:** Only needed if in project context
- **Enterprise DB:** Only needed if enterprise config exists and context matches

#### Implementation

```rust
pub struct GrooveStorage {
    // Always opened at startup
    user_db: Arc<CozoDb>,

    // Opened lazily on first access
    project_db: OnceCell<Arc<CozoDb>>,
    enterprise_db: OnceCell<Arc<CozoDb>>,

    // Config for lazy opening
    project_path: Option<PathBuf>,
    enterprise_config: Option<EnterpriseConfig>,
}

impl GrooveStorage {
    pub fn new(user_db_path: &Path) -> Result<Self> {
        Ok(Self {
            user_db: Arc::new(open_groove_db(user_db_path)?),
            project_db: OnceCell::new(),
            enterprise_db: OnceCell::new(),
            project_path: None,
            enterprise_config: None,
        })
    }

    pub fn with_project(mut self, path: PathBuf) -> Self {
        self.project_path = Some(path);
        self
    }

    pub fn with_enterprise(mut self, config: EnterpriseConfig) -> Self {
        self.enterprise_config = Some(config);
        self
    }

    pub fn project(&self) -> Result<&CozoDb> {
        self.project_db.get_or_try_init(|| {
            let path = self.project_path.as_ref()
                .ok_or(GrooveError::NoProjectContext)?;
            Ok(Arc::new(open_groove_db(&project_db_path(path))?))
        }).map(|arc| arc.as_ref())
    }

    pub fn enterprise(&self) -> Result<&CozoDb> {
        self.enterprise_db.get_or_try_init(|| {
            let config = self.enterprise_config.as_ref()
                .ok_or(GrooveError::NoEnterpriseContext)?;
            Ok(Arc::new(open_groove_db(&config.db_path)?))
        }).map(|arc| arc.as_ref())
    }
}
```

#### RocksDB Memory Tuning

Smaller buffer sizes for groove's modest dataset:

```rust
fn open_groove_db(path: &Path) -> Result<CozoDb> {
    let opts = json!({
        "rocksdb": {
            "block_cache_size": 4 * 1024 * 1024,      // 4 MB (default 8 MB)
            "write_buffer_size": 8 * 1024 * 1024,     // 8 MB (default 64 MB)
            "max_write_buffer_number": 2,
        }
    });

    DbInstance::new("rocksdb", path.to_str().unwrap(), &opts.to_string())
}
```

**Memory per tier:** ~15-20 MB
**Total with all 3 tiers:** ~50-60 MB

#### Thread Safety

CozoDB's `DbInstance` is `Send + Sync`. RocksDB handles concurrent access internally.

```rust
// Safe to share across async tasks
let db: Arc<CozoDb> = storage.user_db.clone();

tokio::spawn(async move {
    db.run_script("?[id] := *learning{id}", ...)?;
});
```

---

---

## Rust Types for 4.2

### Decision 15: Complete Type Definitions

**Decision:** Define all types needed for 4.2 Storage Foundation. Defer types for 4.3+ (CaptureAdapter, InjectionAdapter, etc.).

#### Identity Types

```rust
use uuid::Uuid;

/// UUIDv7 provides time-ordered unique identifiers
/// Use uuid::Uuid::now_v7() to generate
pub type LearningId = Uuid;
```

#### Scope Types

```rust
use serde::{Deserialize, Serialize};

/// Hierarchical scope for learning isolation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Scope {
    Global,
    User(String),
    Project(String),
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid scope string: {0}")]
pub struct ScopeParseError(String);

impl Scope {
    /// Convert to database string format
    pub fn to_db_string(&self) -> String {
        match self {
            Scope::Global => "global".to_string(),
            Scope::User(id) => format!("user:{id}"),
            Scope::Project(path) => format!("project:{path}"),
        }
    }

    /// Parse from database string format
    pub fn from_db_string(s: &str) -> Result<Self, ScopeParseError> {
        if s == "global" {
            return Ok(Scope::Global);
        }
        if let Some(id) = s.strip_prefix("user:") {
            return Ok(Scope::User(id.to_string()));
        }
        if let Some(path) = s.strip_prefix("project:") {
            return Ok(Scope::Project(path.to_string()));
        }
        Err(ScopeParseError(s.to_string()))
    }
}
```

#### Learning Types

```rust
use chrono::{DateTime, Utc};

/// A captured piece of knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Learning {
    pub id: LearningId,
    pub scope: Scope,
    pub category: LearningCategory,
    pub content: LearningContent,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source: LearningSource,
}

/// Category of learning for filtering and organization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LearningCategory {
    /// Code patterns and idioms
    CodePattern,
    /// User preferences and style
    Preference,
    /// Solutions to specific problems
    Solution,
    /// How to recover from errors
    ErrorRecovery,
    /// Tool and CLI usage patterns
    ToolUsage,
    /// Knowledge about the harness itself
    HarnessKnowledge,
}

impl LearningCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CodePattern => "code_pattern",
            Self::Preference => "preference",
            Self::Solution => "solution",
            Self::ErrorRecovery => "error_recovery",
            Self::ToolUsage => "tool_usage",
            Self::HarnessKnowledge => "harness_knowledge",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "code_pattern" => Some(Self::CodePattern),
            "preference" => Some(Self::Preference),
            "solution" => Some(Self::Solution),
            "error_recovery" => Some(Self::ErrorRecovery),
            "tool_usage" => Some(Self::ToolUsage),
            "harness_knowledge" => Some(Self::HarnessKnowledge),
            _ => None,
        }
    }
}

/// The actual content of a learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningContent {
    /// Human-readable description of what was learned
    pub description: String,

    /// Structured pattern data (flexible JSON for different pattern types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<serde_json::Value>,

    /// Actionable insight for injection into sessions
    pub insight: String,
}

/// Where this learning originated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningSource {
    /// Extracted from a session transcript (4.5+)
    Transcript {
        session_id: String,
        message_index: usize,
    },

    /// User explicitly created this learning
    UserCreated,

    /// Promoted from a narrower scope
    Promoted {
        from_scope: Scope,
        original_id: LearningId,
    },

    /// Imported from an export file
    Imported {
        source_file: String,
        imported_at: DateTime<Utc>,
    },

    /// Curated by enterprise administrator
    EnterpriseCurated {
        curator: String,
    },
}

impl LearningSource {
    pub fn source_type(&self) -> &'static str {
        match self {
            Self::Transcript { .. } => "transcript",
            Self::UserCreated => "user_created",
            Self::Promoted { .. } => "promoted",
            Self::Imported { .. } => "imported",
            Self::EnterpriseCurated { .. } => "enterprise_curated",
        }
    }
}
```

#### Usage Statistics

```rust
/// Per-learning usage statistics (updated frequently)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    /// Number of times this learning was injected
    pub times_injected: u32,

    /// Number of times injection led to positive outcome
    pub times_helpful: u32,

    /// Number of times user ignored the injected learning
    pub times_ignored: u32,

    /// Number of times user explicitly contradicted the learning
    pub times_contradicted: u32,

    /// When this learning was last used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<DateTime<Utc>>,

    /// Bayesian prior alpha (successes + 1)
    pub confidence_alpha: f64,

    /// Bayesian prior beta (failures + 1)
    pub confidence_beta: f64,
}

impl Default for UsageStats {
    fn default() -> Self {
        Self {
            times_injected: 0,
            times_helpful: 0,
            times_ignored: 0,
            times_contradicted: 0,
            last_used: None,
            confidence_alpha: 1.0,  // Uniform prior
            confidence_beta: 1.0,
        }
    }
}

impl UsageStats {
    /// Calculate current confidence from Bayesian priors
    pub fn confidence(&self) -> f64 {
        self.confidence_alpha / (self.confidence_alpha + self.confidence_beta)
    }

    /// Update confidence based on outcome
    pub fn record_outcome(&mut self, outcome: Outcome) {
        self.times_injected += 1;
        self.last_used = Some(Utc::now());

        match outcome {
            Outcome::Helpful => {
                self.times_helpful += 1;
                self.confidence_alpha += 1.0;
            }
            Outcome::Ignored => {
                self.times_ignored += 1;
                // Neutral - slight decay
                self.confidence_beta += 0.1;
            }
            Outcome::Contradicted => {
                self.times_contradicted += 1;
                self.confidence_beta += 1.5;  // Strong negative signal
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Outcome {
    Helpful,
    Ignored,
    Contradicted,
}
```

#### Adaptive Parameters

```rust
/// A parameter that learns via Bayesian updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveParam {
    /// Current estimated value
    pub value: f64,

    /// Uncertainty in the estimate (decreases with observations)
    pub uncertainty: f64,

    /// Number of observations
    pub observations: u64,

    /// Beta distribution alpha (successes)
    pub prior_alpha: f64,

    /// Beta distribution beta (failures)
    pub prior_beta: f64,
}

impl Default for AdaptiveParam {
    fn default() -> Self {
        Self::new_uninformed()
    }
}

impl AdaptiveParam {
    /// Create with uninformed (uniform) prior
    pub fn new_uninformed() -> Self {
        Self {
            value: 0.5,
            uncertainty: 1.0,
            observations: 0,
            prior_alpha: 1.0,
            prior_beta: 1.0,
        }
    }

    /// Create with informed prior
    pub fn new_with_prior(alpha: f64, beta: f64) -> Self {
        let value = alpha / (alpha + beta);
        Self {
            value,
            uncertainty: 1.0,
            observations: 0,
            prior_alpha: alpha,
            prior_beta: beta,
        }
    }

    /// Bayesian update based on outcome
    pub fn update(&mut self, outcome: f64, weight: f64) {
        self.observations += 1;
        let effective_weight = weight / (1.0 + self.uncertainty);
        self.prior_alpha += outcome * effective_weight;
        self.prior_beta += (1.0 - outcome) * effective_weight;
        self.value = self.prior_alpha / (self.prior_alpha + self.prior_beta);
        self.uncertainty = 1.0 / (1.0 + (self.observations as f64).sqrt());
    }

    /// Thompson sampling for exploration
    pub fn sample(&self) -> f64 {
        use rand_distr::{Beta, Distribution};
        let beta = Beta::new(self.prior_alpha, self.prior_beta)
            .unwrap_or_else(|_| Beta::new(1.0, 1.0).unwrap());
        beta.sample(&mut rand::thread_rng())
    }
}

/// Named system-wide parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemParam {
    /// Parameter name (e.g., "injection_budget")
    pub name: String,

    /// The adaptive parameter
    pub param: AdaptiveParam,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl SystemParam {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            param: AdaptiveParam::new_uninformed(),
            updated_at: Utc::now(),
        }
    }
}
```

#### Learning Relations

```rust
/// Relationship between two learnings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningRelation {
    pub from_id: LearningId,
    pub relation_type: RelationType,
    pub to_id: LearningId,
    pub weight: f64,
    pub created_at: DateTime<Utc>,
}

/// Types of relationships between learnings
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationType {
    /// from_id replaces/deprecates to_id
    Supersedes,
    /// from_id conflicts with to_id
    Contradicts,
    /// from_id was derived/generalized from to_id
    DerivedFrom,
    /// from_id is related to to_id (same topic)
    RelatedTo,
    /// from_id is a specific case of general to_id
    Specializes,
}

impl RelationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Supersedes => "supersedes",
            Self::Contradicts => "contradicts",
            Self::DerivedFrom => "derived_from",
            Self::RelatedTo => "related_to",
            Self::Specializes => "specializes",
        }
    }
}
```

#### Storage Traits

```rust
use async_trait::async_trait;

/// Storage operations for a single learning database
#[async_trait]
pub trait LearningStore: Send + Sync {
    /// Store a new learning, returns its ID
    async fn store(&self, learning: &Learning) -> Result<LearningId, GrooveError>;

    /// Retrieve a learning by ID
    async fn get(&self, id: LearningId) -> Result<Option<Learning>, GrooveError>;

    /// Find all learnings in a scope
    async fn find_by_scope(&self, scope: &Scope) -> Result<Vec<Learning>, GrooveError>;

    /// Find learnings by category
    async fn find_by_category(&self, category: &LearningCategory) -> Result<Vec<Learning>, GrooveError>;

    /// Semantic search using embedding vector
    async fn semantic_search(
        &self,
        embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(Learning, f64)>, GrooveError>;  // Returns (learning, similarity_score)

    /// Update usage statistics for a learning
    async fn update_usage(&self, id: LearningId, stats: &UsageStats) -> Result<(), GrooveError>;

    /// Find related learnings by relation type
    async fn find_related(
        &self,
        id: LearningId,
        relation_type: Option<&RelationType>,
    ) -> Result<Vec<Learning>, GrooveError>;

    /// Store a relation between learnings
    async fn store_relation(&self, relation: &LearningRelation) -> Result<(), GrooveError>;

    /// Delete a learning
    async fn delete(&self, id: LearningId) -> Result<bool, GrooveError>;

    /// Count learnings (for stats)
    async fn count(&self) -> Result<u64, GrooveError>;
}

/// System parameter storage
#[async_trait]
pub trait ParamStore: Send + Sync {
    /// Get a system parameter by name
    async fn get_param(&self, name: &str) -> Result<Option<SystemParam>, GrooveError>;

    /// Store/update a system parameter
    async fn store_param(&self, param: &SystemParam) -> Result<(), GrooveError>;

    /// Get all system parameters
    async fn all_params(&self) -> Result<Vec<SystemParam>, GrooveError>;
}
```

#### Configuration Types

```rust
use std::path::PathBuf;

/// groove plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrooveConfig {
    /// Path to user tier database
    pub user_db_path: PathBuf,

    /// Configured enterprises (org_id -> config)
    #[serde(default)]
    pub enterprises: std::collections::HashMap<String, EnterpriseConfig>,
}

impl Default for GrooveConfig {
    fn default() -> Self {
        Self {
            user_db_path: default_user_db_path(),
            enterprises: std::collections::HashMap::new(),
        }
    }
}

fn default_user_db_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vibes")
        .join("groove")
        .join("user")
}

/// Enterprise configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseConfig {
    /// Organization identifier
    pub org_id: String,

    /// Path to enterprise database
    pub db_path: PathBuf,

    /// Optional: remote sync endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_url: Option<String>,
}

/// Current project context
#[derive(Debug, Clone)]
pub enum ProjectContext {
    /// Personal project (no enterprise)
    Personal,

    /// Enterprise project
    Enterprise {
        org_id: String,
    },
}
```

#### Backup/Export Types

```rust
/// Export format for backup/portability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrooveExport {
    /// Export format version
    pub version: u32,

    /// When export was created
    pub exported_at: DateTime<Utc>,

    /// Learnings (without embeddings)
    pub learnings: Vec<LearningExport>,

    /// System parameters
    pub params: Vec<SystemParam>,

    /// Relations
    pub relations: Vec<LearningRelation>,
}

/// Learning without embedding (for export)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningExport {
    pub id: LearningId,
    pub scope: Scope,
    pub category: LearningCategory,
    pub content: LearningContent,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source: LearningSource,
    pub usage_stats: UsageStats,
}

/// Import statistics
#[derive(Debug, Clone, Default)]
pub struct ImportStats {
    pub learnings_imported: u32,
    pub learnings_skipped: u32,  // Already existed
    pub params_imported: u32,
    pub relations_imported: u32,
    pub embeddings_queued: u32,
}

pub const EXPORT_VERSION: u32 = 1;
```

#### Error Types

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GrooveError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("No project context available")]
    NoProjectContext,

    #[error("No enterprise context for org: {0}")]
    NoEnterpriseContext(String),

    #[error("Invalid scope: {0}")]
    InvalidScope(#[from] ScopeParseError),

    #[error("Learning not found: {0}")]
    NotFound(LearningId),

    #[error("Schema migration failed: {0}")]
    Migration(String),

    #[error("Export/import error: {0}")]
    Export(String),

    #[error("Embedding error: {0}")]
    Embedding(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
```

#### Schema Version

```rust
/// Database schema version tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaVersion {
    pub version: u32,
    pub applied_at: DateTime<Utc>,
    pub description: String,
}

pub const CURRENT_SCHEMA_VERSION: u32 = 1;
```

---

## Summary Table

| # | Decision | Choice |
|---|----------|--------|
| 1 | CozoDB location | vibes-groove plugin |
| 2 | Harness access | Extend PluginContext |
| 3 | Capability updates | Live (subscribe to watcher) |
| 4 | Storage backend | RocksDB |
| 5 | Scope representation | Composite string `type:value` |
| 6 | Embedding dimensions | 384-dim (GteSmall) for MVP |
| 7 | Database tiers | Separate RocksDB per tier |
| 8 | Learning flow | Start project, promote up |
| 9 | Enterprise governance | Future milestone |
| 10 | Schema migrations | Versioning table + scripts |
| 11 | Adaptive parameters | Two-level: system + per-learning |
| 12 | Learning relationships | Schema now, logic later |
| 13 | Backup & recovery | Minimal (snapshot + export/import) |
| 14 | Connection management | Eager user, lazy others |
| 15 | Rust types | Complete definitions for 4.2 |
