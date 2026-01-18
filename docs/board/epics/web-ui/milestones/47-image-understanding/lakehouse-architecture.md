# Lakehouse Architecture

> **The lakehouse is Spoke's sensory system.** Data sources are how the vessel perceives the universe.

## Overview

The Spoke lakehouse is a hybrid execution, multi-modal query system built on:

- **Lance format** — The columnar file format (like Parquet but ML-native)
- **LanceDB** — Database layer providing vector indexes, versioning, compaction
- **DataFusion** — The SQL query execution engine (runs locally AND remotely)
- **Arrow Flight** — The nervous system transport (streaming Arrow batches)
- **object_store** — The storage abstraction (object-first, not filesystem-first)

### Terminology Clarification

| Term | What It Is | Role in Spoke |
|------|------------|---------------|
| **Lance format** | Columnar file format (`.lance` files) | The bytes stored in object storage |
| **LanceDB** | Database built on Lance format | Vector indexing (IVF, HNSW), versioning API, compaction |
| **lance crate** | Rust library to read/write Lance | DataFusion integration for SQL queries |
| **DataFusion** | SQL query engine | Relational operations, distributed execution |

**They work together:** DataFusion handles SQL/relational operations, LanceDB handles vector operations, both read/write the same Lance format files in object storage.

---

## The Sensory Metaphor

If Iggy is the nervous system, the lakehouse is the **sensory cortex**:

```
                    ┌─────────────────────────────────────┐
                    │           THE UNIVERSE              │
                    │    (external data, APIs, files)     │
                    └──────────────┬──────────────────────┘
                                   │
                    ╭──────────────▼──────────────╮
                    │      SENSORY PERIPHERY      │
                    │   (data source connections) │
                    ╰──────────────┬──────────────╯
                                   │
                         ══════════╪══════════  ← the rim
                                   │
                    ╭──────────────▼──────────────╮
                    │     SENSORY PROCESSING      │
                    │   (ingestion, transforms)   │
                    ╰──────────────┬──────────────╯
                                   │
                              spokes (axons)
                                   │
                    ╭──────────────▼──────────────╮
                    │       THE HUB (you)         │
                    │   Unified, queryable view   │
                    ╰─────────────────────────────╯
```

The spoke diagram becomes a **living sensory map**:
- Active connections pulse with event flow
- Dormant connections dim
- Errors show as disrupted signals
- Fresh data shows as bright, recent sensation

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           QUERY INTERFACE                                   │
│                 SQL • Vector • Graph • Natural Language                     │
└─────────────────────────────────────┬───────────────────────────────────────┘
                                      │
┌─────────────────────────────────────▼───────────────────────────────────────┐
│                          QUERY PLANNER                                      │
│                                                                             │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│   │ SQL Parser  │  │ Vector Ops  │  │ Graph→SQL   │  │  NL→SQL     │       │
│   │             │  │             │  │ Compiler    │  │ (LLM layer) │       │
│   └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘       │
│          └─────────────────┼─────────────────┼───────────────┘              │
│                            ▼                                                │
│                    Substrait Plan                                           │
│            (cross-system query representation)                              │
└─────────────────────────────────────┬───────────────────────────────────────┘
                                      │
┌─────────────────────────────────────▼───────────────────────────────────────┐
│                        GRAVITY OPTIMIZER                                    │
│                                                                             │
│   • Estimate data sizes at each node                                        │
│   • Model network transfer cost                                             │
│   • Identify push-down opportunities (filters, aggregations)                │
│   • Respect security boundaries (what MUST stay local)                      │
│   • Decide: pull data vs push computation                                   │
│                                                                             │
└─────────────────────────────────────┬───────────────────────────────────────┘
                                      │
┌─────────────────────────────────────▼───────────────────────────────────────┐
│                      DISTRIBUTED EXECUTION                                  │
│                                                                             │
│   ┌───────────────────────┐       ┌───────────────────────┐                │
│   │   LOCAL DATAFUSION    │◄─────►│  REMOTE DATAFUSION    │                │
│   │   (Edge compute)      │ Arrow │  (Spoke server)       │                │
│   │                       │ Flight│                       │                │
│   │   • Your machine      │       │   • Cloud/server      │                │
│   │   • Low latency       │       │   • Scalable          │                │
│   │   • Private data      │       │   • Shared data       │                │
│   └───────────┬───────────┘       └───────────┬───────────┘                │
│               │                               │                             │
│               └───────────────┬───────────────┘                             │
│                               ▼                                             │
│                    RESULT MERGER (Arrow batches)                            │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
┌─────────────────────────────────────▼───────────────────────────────────────┐
│                         STORAGE LAYER                                       │
│                      (object_store crate)                                   │
│                                                                             │
│                    Object-first, not filesystem-first                       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Hybrid Execution Model

**Data has gravity.** We can't pull petabytes to a laptop, but we can push computation to the data.

The gravity optimizer decides for each query node:

```rust
// Pseudocode
fn plan_execution(node: &QueryNode) -> Strategy {
    let local_cost = estimate_local_execution(node);
    let remote_cost = estimate_remote_execution(node);
    let transfer_cost = estimate_data_transfer(node.input_size());

    if local_cost + transfer_cost < remote_cost {
        Strategy::PullAndExecute  // Small data, fetch it
    } else if can_push_down(node) {
        Strategy::PushDown        // Filter/aggregate at source
    } else {
        Strategy::Hybrid          // Split execution
    }
}
```

**Key heuristics:**
- Small local + large remote → Upload local data, join at remote
- Large local + small remote → Download remote data, join locally
- Both large → Push filters/aggregations, stream results
- Security policies → Constrain what operations are possible at each location

### Security-Aware Execution

Security applies to **both** local and remote execution. Policies define what operations are permitted:

```
┌─────────────────────────────────────────────────────────────────┐
│                    SECURITY POLICIES                            │
│                                                                 │
│   LOCAL POLICIES (your data):                                   │
│   ├── Some data never leaves local machine                     │
│   ├── Some data can be uploaded for joins                      │
│   └── Some data requires encryption in transit                 │
│                                                                 │
│   REMOTE POLICIES (shared/external data):                       │
│   ├── Join allowed, but only project aggregates                │
│   ├── Join allowed, but columns are masked/redacted            │
│   ├── Raw rows allowed for some roles, aggregates for others   │
│   └── Some datasets: read-only, no joins with external data    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Policy examples:**

| Policy | Effect on Query Planning |
|--------|-------------------------|
| `remote.customers: aggregate_only` | Can `COUNT(*)`, `AVG(spend)` but not `SELECT *` |
| `remote.pii: mask(email, phone)` | Joins work, but PII columns return `***@***.com` |
| `remote.financial: role_based` | Executives see raw data, analysts see aggregates |
| `local.secrets: never_upload` | Must execute locally, cannot push to remote join |
| `remote.audit: append_only_read` | Can query but gravity optimizer can't push writes |

**The gravity optimizer becomes security-aware:**

```rust
// Pseudocode
fn plan_execution(node: &QueryNode, policies: &SecurityPolicies) -> Strategy {
    // Check what's even allowed before optimizing for cost
    let local_allowed = policies.can_execute_locally(node);
    let remote_allowed = policies.can_execute_remotely(node);
    let projections_allowed = policies.allowed_projections(node);

    // Rewrite query if needed (mask columns, force aggregation)
    let node = policies.rewrite_for_compliance(node);

    // Now optimize within security constraints
    if !remote_allowed {
        Strategy::LocalOnly
    } else if !local_allowed {
        Strategy::RemoteOnly
    } else {
        optimize_for_cost(node)  // Normal gravity optimization
    }
}
```

### Arrow Flight as the Transport

DataFusion instances communicate via Arrow Flight:
- Streaming Arrow record batches
- Efficient serialization (zero-copy when possible)
- gRPC-based, works across networks
- The "axons" of the nervous system

---

## Object-First Storage

### The Principle

Instead of hierarchical filesystem paths:
```
/home/user/data/sales/2024/q1/data.lance
```

We use object keys:
```
spoke://tenant/sales/v42/fragment-0017
```

The [`object_store`](https://docs.rs/object_store/latest/object_store/) crate provides a unified async API across:
- AWS S3
- Google Cloud Storage
- Azure Blob Storage
- Local filesystem (for dev/edge)
- In-memory (for testing)

**Same code, different backends via runtime config.**

### Why Object-First Matters

| Filesystem Model | Object Model |
|------------------|--------------|
| Hierarchical paths | Flat namespace with prefixes |
| Directories exist | No directories, just key prefixes |
| Rename is atomic | Rename = copy + delete |
| Local-first, cloud-adapted | Cloud-first, local-compatible |
| Cache keys derived from paths | Keys ARE cache keys |
| Tiering is complex | Tiering is storage class metadata |

**The object model is more honest about what cloud storage actually is.**

### Key Construction = Cache Keys

Object keys become universal identifiers across all cache layers:

```
┌─────────────────────────────────────────────────────────────────┐
│                        OBJECT KEY                               │
│                                                                 │
│   spoke://{tenant}/{dataset}/{version}/{fragment}               │
│                                                                 │
│   Examples:                                                     │
│   spoke://acme/sales/v42/frag-0017                             │
│   spoke://acme/embeddings/v3/index-main                        │
│   spoke://acme/customers/v108/manifest                         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      CACHE LAYERS                               │
│                                                                 │
│   L1: In-memory (Arrow RecordBatch)                            │
│       Key: spoke://acme/sales/v42/frag-0017                    │
│       Value: Deserialized Arrow data                           │
│                                                                 │
│   L2: Local disk (NVMe)                                        │
│       Key: spoke://acme/sales/v42/frag-0017                    │
│       Value: Lance fragment bytes                              │
│                                                                 │
│   L3: Object storage (S3/GCS)                                  │
│       Key: spoke://acme/sales/v42/frag-0017                    │
│       Value: Canonical source                                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

Cache invalidation becomes simple: version changes → new keys → old cache entries naturally expire.

### Content-Addressable Storage

For deduplication and geo-distribution, we use content-addressable keys where beneficial:

```
┌─────────────────────────────────────────────────────────────────┐
│                   KEY SCHEMA OPTIONS                            │
│                                                                 │
│   Path-based (for navigation):                                  │
│   spoke://{tenant}/{dataset}/{version}/manifest                │
│                                                                 │
│   Content-addressed (for deduplication):                        │
│   spoke://{tenant}/blobs/{content-hash}                        │
│                                                                 │
│   The manifest points to content-addressed fragments:           │
│   manifest.json:                                                │
│   {                                                             │
│     "version": 42,                                              │
│     "fragments": [                                              │
│       "spoke://acme/blobs/sha256-abc123...",                   │
│       "spoke://acme/blobs/sha256-def456..."                    │
│     ]                                                           │
│   }                                                             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Benefits:**
- **Deduplication across versions** — unchanged data reuses same content hash
- **Deduplication across datasets** — shared data recognized automatically
- **Geo-distribution friendly** — replicate by hash, location independent
- **Cache-friendly** — same content = same key = cache hit

**Content-Defined Chunking (CDC):**

[Parquet CDC](https://huggingface.co/blog/parquet-cdc) (PyArrow 21+) enables intelligent chunking at the data page level. When Lance/LanceDB supports this:
- Row insertions/deletions: 7-15x reduction in transfer
- Schema changes: only new columns transferred
- Cross-repository deduplication works automatically

This is a key capability to track and adopt when available.

---

## Lance Format + LanceDB

### Lance Format (The Files)

Lance is the columnar file format — like Parquet but designed for ML/AI workloads:

| Capability | How It Works |
|------------|--------------|
| **Columnar storage** | Efficient for analytical queries, column pruning |
| **Random access** | Fast point lookups, not just sequential scans |
| **Compression** | Lance 2.1 adds cascading encoding without sacrificing random access |
| **Append-friendly** | New data creates new fragments, old data untouched |

### LanceDB (The Database Layer)

LanceDB provides higher-level capabilities on top of Lance format:

| Capability | How It Works |
|------------|--------------|
| **Versioning** | Each insert creates a new version. Old versions retained for concurrent readers. |
| **Compaction** | Merges fragments to reduce metadata overhead. Keep fragments under ~100. |
| **Streaming ingestion** | Automatic index updates during writes. Non-blocking. |
| **Vector indexes** | Native IVF, HNSW indexes for similarity search |
| **Hybrid search** | Vector + full-text + SQL in single queries |

### Fragment Model

```
Dataset (v42)
├── Manifest
│   └── Points to all fragments for this version
├── Fragment 0001  →  spoke://acme/sales/v42/frag-0001
├── Fragment 0002  →  spoke://acme/sales/v42/frag-0002
├── ...
└── Index          →  spoke://acme/sales/v42/index-ivf
```

Each fragment is an independent object. The manifest tracks which objects comprise a version.

### Streaming + Batch Unification

```
             STREAMING WRITES
                   │
                   ▼
          ┌─────────────────┐
          │  DELTA LOG      │  ← Small, frequent fragments
          │  (Recent writes)│     Append-only, versioned
          └────────┬────────┘
                   │
                   │  Background compaction
                   ▼
          ┌─────────────────┐
          │  BASE FILES     │  ← Large, optimized fragments
          │  (Compacted)    │     Columnar, indexed
          └─────────────────┘

READ PATH:
  Query → Scan base files + merge delta log → Unified result
```

---

## Tiered Storage

```
┌─────────────────────────────────────────────────────────────────┐
│                       STORAGE TIERS                             │
│                                                                 │
│   ┌─────────────┐      ┌─────────────┐      ┌─────────────┐    │
│   │     HOT     │      │    WARM     │      │    COLD     │    │
│   │             │      │             │      │             │    │
│   │ Local NVMe  │      │ S3 Standard │      │  Glacier    │    │
│   │ In-memory   │      │ GCS Standard│      │  Archive    │    │
│   │             │      │             │      │             │    │
│   │ < 1ms       │      │ 10-100ms    │      │ minutes-hrs │    │
│   │ $$$         │      │ $$          │      │ $           │    │
│   └─────────────┘      └─────────────┘      └─────────────┘    │
│                                                                 │
│   Automatic tiering based on access patterns + cost             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

```rust
// Pseudocode for tiering policy
enum StorageClass {
    Hot,        // Local NVMe, in-memory cache
    Standard,   // S3 Standard, GCS Standard
    Infrequent, // S3 IA, GCS Nearline
    Archive,    // S3 Glacier, GCS Archive
}

fn determine_tier(fragment: &Fragment) -> StorageClass {
    let age = now() - fragment.last_accessed;
    let frequency = fragment.access_count / age.as_days();

    match (age, frequency) {
        (_, f) if f > 10.0 => StorageClass::Hot,
        (d, _) if d < 7 => StorageClass::Standard,
        (d, _) if d < 30 => StorageClass::Infrequent,
        _ => StorageClass::Archive,
    }
}
```

---

## Multi-Modal Query Interface

The query interface supports multiple modalities, all compiled to a unified execution plan:

### SQL (Relational)
```sql
SELECT * FROM customers WHERE region = 'APAC';
```

### Vector Similarity
```sql
SELECT name, embedding <-> $query_vec AS distance
FROM documents
ORDER BY distance LIMIT 10;
```

### Full-Text Search
```sql
SELECT * FROM articles WHERE text @@ 'machine learning';
```

### Graph Queries
```cypher
MATCH (p:Person)-[:KNOWS]->(f:Person)
WHERE p.name = 'Alice'
RETURN f.name
```

### Hybrid Queries
```sql
SELECT
    d.title,
    d.embedding <-> $query_vec AS semantic_distance,
    d.text @@ $keywords AS keyword_match
FROM documents d
JOIN authors a ON d.author_id = a.id
WHERE a.department = 'Research'
  AND d.created_at > '2024-01-01'
ORDER BY semantic_distance
LIMIT 20;
```

### Natural Language (Layer on Top)
```
"Find documents similar to this about machine learning from the research team"
    ↓ (LLM-powered query planning)
    ↓
Compiled to hybrid SQL query above
```

---

## Graph Query Strategy

Graph queries are supported via a phased approach that keeps options open:

```
┌─────────────────────────────────────────────────────────────────┐
│                   GRAPH QUERY EVOLUTION                         │
│                                                                 │
│   Phase 1 (Now): Store as relational tables                    │
│   ├── nodes: (id, type, properties...)                         │
│   └── edges: (src, dst, type, properties...)                   │
│                                                                 │
│   Phase 2 (Later): Add Cypher-to-SQL compiler                  │
│   └── MATCH (a)-[:KNOWS]->(b) → JOIN operations                │
│                                                                 │
│   Phase 3 (If needed): Native graph operators                  │
│   └── DataFusion UDAFs for iterative algorithms                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Why this approach:**

Per [research on DataFusion graph processing](https://semyonsinchenko.github.io/ssinchenko/post/datafusion-graphs-cc/):
- Graph algorithms (connected components, etc.) work via relational operations
- 4-5x faster than Spark GraphFrames, half the memory of NetworkX
- Limitation: single-node, no disk spill for iterative algorithms

**The key insight:** Store edges/nodes in Lance format, compile graph patterns to DataFusion operations. Add native graph operators only if we hit performance walls on specific patterns.

---

## Federation Strategy

Federation lets us query data where it lives without forced ingestion:

```
┌─────────────────────────────────────────────────────────────────┐
│                   FEDERATION MODEL                              │
│                                                                 │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│   │   Lance     │  │  Postgres   │  │  Snowflake  │            │
│   │  (Primary)  │  │ (Federated) │  │ (Federated) │            │
│   └──────┬──────┘  └──────┬──────┘  └──────┬──────┘            │
│          │                │                │                    │
│          └────────────────┼────────────────┘                    │
│                           ▼                                     │
│              DataFusion TableProviders                          │
│                           │                                     │
│                           ▼                                     │
│              Unified query across all sources                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Push-down where supported:**
- Filters → pushed to Postgres/Snowflake
- Aggregations → pushed where possible
- Joins → gravity optimizer decides (small side moves to large side)

**Lance is the primary store** for data we manage. Federation enables:
- Query data where it lives (no forced ingestion)
- Join managed data with external sources
- Migrate gradually (federate first, ingest later if beneficial)

---

## Tiering & Caching Principles

We don't need to implement tiering now, but the architecture must support it:

| Principle | How We Preserve It |
|-----------|-------------------|
| **Observable access patterns** | Iggy events track every read — we have the data |
| **Movable fragments** | Object keys are abstract — same key resolves to different tiers |
| **Cache coherency** | Immutable fragments + version manifests = no coherency problem |
| **Cost awareness** | Tiering metadata (storage class, retrieval cost) in object_store |

**Why this works:**
- Fragments are immutable → cache forever until evicted
- Versions are append-only → no cache invalidation race conditions
- Object keys abstract location → hot/warm/cold is just metadata
- Content-addressed keys → same content = same cache entry everywhere

---

## Biological Mapping

| System Layer | Biological Equivalent |
|--------------|----------------------|
| Query Interface | Sensory input (what you're perceiving/asking) |
| Query Planner | Primary sensory cortex (pattern recognition) |
| Gravity Optimizer | Prefrontal cortex (cost-benefit decisions) |
| Security Policies | Blood-brain barrier + immune system (what crosses boundaries) |
| Arrow Flight | Axons (long-distance signal transmission) |
| Local DataFusion | Reflexes (fast, local processing) |
| Remote DataFusion | Higher cognition (powerful but latent) |
| Result Merger | Binding problem (unifying perception) |
| Lance Storage | Long-term memory (versioned, consolidated) |
| Hot/Warm/Cold Tiers | Memory hierarchy (working → episodic → archival) |
| Compaction | Memory consolidation |

**Security as immune system:** Just as the body has multiple defense layers (skin, blood-brain barrier, immune cells), the lakehouse has security policies at each boundary. Some data never crosses certain barriers. Some is transformed (masked) when crossing. The system protects itself from unauthorized access patterns.

---

## Open Questions

### Resolved

| Question | Decision |
|----------|----------|
| **Graph query model** | Phased approach: relational storage → Cypher compiler → native ops if needed |
| **Content addressing** | Yes, use content hashes for fragments, manifests point to hashes |
| **Federation** | Yes, use DataFusion TableProviders, push down where possible |
| **Tiering architecture** | Immutable fragments + abstract keys = tiering-ready without implementation now |

### Still Open

**Security Policy Model:**
- How are policies defined? (SQL GRANT-style? YAML config? UI?)
- Policy inheritance (dataset → table → column)?
- How do policies compose across federated sources?
- Audit logging for policy enforcement?

**Key Schema Details:**
- Exact format for geo-distribution (region prefix? separate namespace?)
- When does Lance/LanceDB support content-defined chunking?

**Compaction Policy:**
- Continuous background vs scheduled vs threshold-triggered?
- LanceDB Cloud has auto-compaction — can we leverage similar?

**Graph Query Specifics:**
- Full Cypher or subset? (Recommend: subset covering common patterns)
- Property graph vs simple labeled edges?

**Cache Warming:**
- Can gravity optimizer predict needed fragments?
- Prefetch based on query patterns?

---

## Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Storage Format | Lance format | Columnar files, random access, compression |
| Database Layer | LanceDB | Vector indexes, versioning, compaction, hybrid search |
| Storage Abstraction | object_store | Unified S3/GCS/Azure/Local API |
| Query Engine | DataFusion | SQL execution, distributed via Arrow Flight |
| Query Plan Format | Substrait | Cross-system query representation |
| Transport | Arrow Flight | Streaming Arrow batches between DataFusion instances |
| Event Backbone | Iggy | Event sourcing, streaming, the nervous system |

---

## References

### Internal Docs
- [Visual Depth System](visual-depth-system.md) — Aesthetic hierarchy
- [Biological Layer](biological-layer.md) — Iggy as nervous system
- [Command Modes](command-modes-system.md) — Survey/Command/Deep Dive

### External Resources
- [Lance Format Docs](https://lancedb.com/docs/overview/lance/) — Format specification
- [LanceDB Docs](https://lancedb.com/docs/) — Database layer
- [DataFusion Docs](https://datafusion.apache.org/) — Query engine
- [Arrow Flight](https://arrow.apache.org/docs/format/Flight.html) — Transport protocol
- [object_store crate](https://docs.rs/object_store/latest/object_store/) — Storage abstraction
- [DataFusion Graph Processing](https://semyonsinchenko.github.io/ssinchenko/post/datafusion-graphs-cc/) — Graph query approach
- [Parquet CDC](https://huggingface.co/blog/parquet-cdc) — Content-defined chunking
