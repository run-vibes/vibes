# Milestone 34: Open-World Adaptation - Design

## Overview

Open-World Adaptation closes the loop on the learning system by detecting unknown patterns and surfacing capability gaps. It implements M32's NoveltyHook trait to create a feedback loop between novelty detection and strategy selection.

**Core components:**
- **NoveltyDetector** - Detects patterns not matching known fingerprints using embedding similarity + incremental DBSCAN clustering
- **CapabilityGapDetector** - Identifies recurring failures using combined signals from M31 attribution
- **GraduatedResponse** - Progressive handling: monitor → cluster → auto-adjust → surface to user
- **SolutionGenerator** - Templates for common gaps + pattern analysis for novel gaps

## Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Strategy Outcomes (from M32)                      │
│                    via NoveltyHook trait                            │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    NoveltyDetector                                   │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐ │
│  │ Embedding       │  │ Fingerprint     │  │ Incremental         │ │
│  │ (reuse M30)     │─▶│ Matching        │─▶│ DBSCAN Clustering   │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────────┘ │
└──────────────────────────┬──────────────────────────────────────────┘
                           │ novel patterns
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    GraduatedResponse                                 │
│  Monitor ──▶ Cluster ──▶ Auto-adjust thresholds ──▶ Surface gap     │
└──────────────────────────┬──────────────────────────────────────────┘
                           │ persistent gaps
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    CapabilityGapDetector                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐ │
│  │ Failure         │  │ Negative        │  │ Low Confidence      │ │
│  │ Frequency       │ +│ Attribution     │ +│ Extractions         │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────────┘ │
└──────────────────────────┬──────────────────────────────────────────┘
                           │ capability gaps
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Solution Generator                                │
│  ┌─────────────────┐  ┌─────────────────┐                          │
│  │ Template-based  │  │ Pattern         │  → Dashboard + Events    │
│  │ (common gaps)   │ +│ Analysis        │                          │
│  └─────────────────┘  └─────────────────┘                          │
└─────────────────────────────────────────────────────────────────────┘
```

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Scope | Both novelty + gaps | Full open-world adaptation in one milestone |
| Novelty detection | Hybrid embedding + clustering | Fast screening via embeddings, pattern grouping via clustering |
| Capability gaps | Combined signals | Failure frequency + negative attribution + low confidence |
| Novelty response | Graduated | Progressive escalation avoids overreacting to noise |
| M32 integration | Full closed loop | Hook + feedback + distribution adjustment |
| Gap surfacing | Actionable suggestions | Dashboard + events + generated solutions |
| Solution generation | Templates + pattern analysis | Reliable for known gaps, adaptive for novel |
| Persistence | Iggy + CozoDB | Consistent with M30-32 |
| Embeddings | Reuse M30 gte-small | 384-dim, already indexed with HNSW |
| Clustering | Incremental DBSCAN | Online updates, handles outliers naturally |
| CLI | Full management | Status + inspect + manual actions |

---

## Core Types

### Pattern Fingerprint

```rust
/// Fingerprint of a pattern for novelty detection
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PatternFingerprint {
    pub id: Uuid,
    pub embedding: Vec<f32>,        // 384 dims from gte-small
    pub category: LearningCategory,
    pub context_hash: u64,          // Fast pre-filter
    pub created_at: DateTime<Utc>,
}

/// Cluster of similar novel patterns
#[derive(Debug, Clone)]
pub struct AnomalyCluster {
    pub id: Uuid,
    pub centroid: Vec<f32>,         // Cluster center embedding
    pub members: Vec<PatternFingerprint>,
    pub density: f64,               // DBSCAN density metric
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub occurrence_count: u32,
}

/// Novelty detection result
#[derive(Debug, Clone)]
pub enum NoveltyResult {
    /// Pattern matches known fingerprint
    Known { fingerprint_id: Uuid, similarity: f64 },
    /// Pattern is novel, assigned to existing cluster
    NovelClustered { cluster_id: Uuid, distance: f64 },
    /// Pattern is novel outlier (doesn't fit any cluster)
    NovelOutlier { embedding: Vec<f32> },
}
```

### Capability Gap

```rust
/// A gap in system capabilities
#[derive(Debug, Clone)]
pub struct CapabilityGap {
    pub id: Uuid,
    pub description: String,
    pub category: GapCategory,
    pub severity: GapSeverity,

    // Evidence
    pub failure_count: u32,
    pub negative_attribution_sum: f64,
    pub low_confidence_count: u32,
    pub example_contexts: Vec<String>,  // Up to 5 examples

    // Lifecycle
    pub status: GapStatus,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,

    // Solutions
    pub potential_solutions: Vec<SuggestedSolution>,
}

#[derive(Debug, Clone, Copy)]
pub enum GapCategory {
    ExtractionFailure,    // Can't extract learnings from this pattern
    InjectionFailure,     // Learnings don't help in this context
    NegativeImpact,       // Learnings actively hurt
    NovelDomain,          // Entirely new problem space
}

#[derive(Debug, Clone, Copy)]
pub enum GapSeverity {
    Low,      // < 5 occurrences, monitor
    Medium,   // 5-20 occurrences, adjust thresholds
    High,     // > 20 occurrences, surface to user
}

#[derive(Debug, Clone, Copy)]
pub enum GapStatus {
    Monitoring,     // Watching, not yet actionable
    Confirmed,      // Persistent gap, surfaced to user
    Dismissed,      // User marked as not a real gap
    Resolved,       // Gap addressed (manually or auto)
}
```

### Failure Record

```rust
/// Record of a failure for gap analysis
#[derive(Debug, Clone)]
pub struct FailureRecord {
    pub context_hash: u64,
    pub learning_id: Option<LearningId>,
    pub failure_type: FailureType,
    pub attribution_value: Option<f64>,
    pub extraction_confidence: Option<f64>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub enum FailureType {
    ExtractionFailed,      // Couldn't extract a learning
    InjectionIgnored,      // Learning injected but not used
    NegativeOutcome,       // Learning made things worse
    LowConfidence,         // Extraction confidence stayed low
}
```

### Solution Types

```rust
#[derive(Debug, Clone)]
pub struct SuggestedSolution {
    pub id: Uuid,
    pub description: String,
    pub action: SolutionAction,
    pub confidence: f64,
    pub source: SolutionSource,
}

#[derive(Debug, Clone)]
pub enum SolutionAction {
    /// Add explicit examples to learning store
    AddExamples { suggested_count: u32 },

    /// Adjust extraction confidence threshold
    AdjustConfidence { suggested_threshold: f64 },

    /// Mark context as out-of-scope
    MarkOutOfScope { reason: String },

    /// Suggest manual learning creation
    CreateManualLearning { template: String },

    /// Disable problematic learning
    DisableLearning { learning_id: LearningId },

    /// Adjust injection strategy
    AdjustStrategy { suggested_variant: StrategyVariant },
}

#[derive(Debug, Clone)]
pub enum SolutionSource {
    Template,           // From predefined templates
    PatternAnalysis,    // Discovered from similar contexts
    UserFeedback,       // Learned from past user actions
}
```

---

## NoveltyDetector

```rust
/// Detects unknown patterns using embedding similarity + clustering
pub struct NoveltyDetector {
    /// Distance threshold for "known" (cosine similarity)
    similarity_threshold: AdaptiveParam,  // Default 0.85

    /// Known pattern fingerprints (loaded from store)
    known_fingerprints: HashSet<PatternFingerprint>,

    /// Anomaly clusters (incremental DBSCAN)
    anomaly_clusters: Vec<AnomalyCluster>,

    /// Pending outliers awaiting clustering
    pending_outliers: Vec<Vec<f32>>,

    /// DBSCAN parameters
    dbscan_eps: f64,        // Neighborhood radius (default 0.3)
    dbscan_min_pts: usize,  // Min points for cluster (default 3)

    /// Embedder (reuses M30's gte-small)
    embedder: Arc<dyn Embedder>,

    /// Persistence
    store: Arc<dyn OpenWorldStore>,
}

impl NoveltyDetector {
    /// Check if a pattern is novel
    pub async fn detect(&mut self, context: &SessionContext) -> Result<NoveltyResult> {
        // Step 1: Generate embedding
        let embedding = self.embedder.embed(&context.to_embedding_input()).await?;

        // Step 2: Fast pre-filter with context hash
        let context_hash = self.hash_context(context);
        let candidates = self.known_fingerprints
            .iter()
            .filter(|fp| fp.context_hash == context_hash || fp.context_hash == 0)
            .collect::<Vec<_>>();

        // Step 3: Check similarity against known fingerprints
        for fingerprint in candidates {
            let similarity = cosine_similarity(&embedding, &fingerprint.embedding);
            if similarity >= self.similarity_threshold.value() {
                return Ok(NoveltyResult::Known {
                    fingerprint_id: fingerprint.id,
                    similarity,
                });
            }
        }

        // Step 4: Pattern is novel - try to assign to cluster
        if let Some((cluster_id, distance)) = self.find_nearest_cluster(&embedding) {
            if distance <= self.dbscan_eps {
                self.add_to_cluster(cluster_id, &embedding).await?;
                return Ok(NoveltyResult::NovelClustered { cluster_id, distance });
            }
        }

        // Step 5: Novel outlier - may form new cluster later
        self.pending_outliers.push(embedding.clone());
        self.maybe_recluster().await?;

        Ok(NoveltyResult::NovelOutlier { embedding })
    }

    /// Incremental DBSCAN: periodically merge outliers into clusters
    async fn maybe_recluster(&mut self) -> Result<()> {
        if self.pending_outliers.len() >= self.dbscan_min_pts {
            let new_clusters = incremental_dbscan(
                &self.pending_outliers,
                &self.anomaly_clusters,
                self.dbscan_eps,
                self.dbscan_min_pts,
            );

            self.anomaly_clusters.extend(new_clusters);
            self.pending_outliers.clear();
        }
        Ok(())
    }

    /// Mark a pattern as known (after learning succeeds)
    pub async fn mark_known(&mut self, context: &SessionContext) -> Result<()> {
        let embedding = self.embedder.embed(&context.to_embedding_input()).await?;
        let fingerprint = PatternFingerprint {
            id: Uuid::new_v4(),
            embedding,
            category: context.inferred_category(),
            context_hash: self.hash_context(context),
            created_at: Utc::now(),
        };
        self.known_fingerprints.insert(fingerprint.clone());
        self.store.save_fingerprint(&fingerprint).await?;
        Ok(())
    }
}
```

---

## CapabilityGapDetector

```rust
/// Identifies recurring failures using combined signals
pub struct CapabilityGapDetector {
    /// Thresholds for gap detection
    failure_threshold: u32,              // Default 3 failures
    negative_attribution_threshold: f64, // Default -0.5 cumulative
    low_confidence_threshold: u32,       // Default 5 low-confidence extractions

    /// Severity escalation thresholds
    medium_severity_count: u32,  // Default 5
    high_severity_count: u32,    // Default 20

    /// Active gaps being monitored
    gaps: HashMap<Uuid, CapabilityGap>,

    /// Pattern matcher for grouping similar failures
    failure_clusters: HashMap<u64, Vec<FailureRecord>>,

    /// Persistence
    store: Arc<dyn OpenWorldStore>,
}

impl CapabilityGapDetector {
    /// Process an outcome and check for capability gaps
    pub async fn process_outcome(
        &mut self,
        context: &SessionContext,
        attribution: Option<&AttributionRecord>,
        extraction: Option<&ExtractionResult>,
    ) -> Result<Option<CapabilityGap>> {
        let context_hash = hash_context(context);

        // Record failure if applicable
        let failure = self.detect_failure(attribution, extraction);
        if let Some(failure_type) = failure {
            self.record_failure(FailureRecord {
                context_hash,
                learning_id: attribution.map(|a| a.learning_id),
                failure_type,
                attribution_value: attribution.map(|a| a.attributed_value),
                extraction_confidence: extraction.map(|e| e.confidence),
                timestamp: Utc::now(),
            }).await?;
        }

        // Check if failures have accumulated into a gap
        self.check_for_gap(context_hash, context).await
    }

    fn detect_failure(
        &self,
        attribution: Option<&AttributionRecord>,
        extraction: Option<&ExtractionResult>,
    ) -> Option<FailureType> {
        // Negative attribution = learning hurt
        if let Some(attr) = attribution {
            if attr.attributed_value < -0.1 {
                return Some(FailureType::NegativeOutcome);
            }
            if !attr.was_activated {
                return Some(FailureType::InjectionIgnored);
            }
        }

        // Low confidence extraction
        if let Some(ext) = extraction {
            if ext.confidence < 0.3 {
                return Some(FailureType::LowConfidence);
            }
            if ext.failed {
                return Some(FailureType::ExtractionFailed);
            }
        }

        None
    }

    async fn check_for_gap(
        &mut self,
        context_hash: u64,
        context: &SessionContext,
    ) -> Result<Option<CapabilityGap>> {
        let failures = self.failure_clusters.get(&context_hash);
        let failures = match failures {
            Some(f) if f.len() >= self.failure_threshold as usize => f,
            _ => return Ok(None),
        };

        // Aggregate signals
        let failure_count = failures.len() as u32;
        let negative_sum: f64 = failures.iter()
            .filter_map(|f| f.attribution_value)
            .filter(|v| *v < 0.0)
            .sum();
        let low_conf_count = failures.iter()
            .filter(|f| matches!(f.failure_type, FailureType::LowConfidence))
            .count() as u32;

        // Determine if this is a gap
        let is_gap = failure_count >= self.failure_threshold
            || negative_sum <= self.negative_attribution_threshold
            || low_conf_count >= self.low_confidence_threshold;

        if !is_gap {
            return Ok(None);
        }

        // Create or update gap
        let mut gap = self.get_or_create_gap(context_hash, context, failures).await?;
        self.update_severity(&mut gap);

        Ok(Some(gap))
    }

    fn update_severity(&self, gap: &mut CapabilityGap) {
        gap.severity = if gap.failure_count >= self.high_severity_count {
            GapSeverity::High
        } else if gap.failure_count >= self.medium_severity_count {
            GapSeverity::Medium
        } else {
            GapSeverity::Low
        };
    }
}
```

---

## GraduatedResponse

```rust
/// Progressive response to novelty and capability gaps
pub struct GraduatedResponse {
    /// Response stages with thresholds
    stages: ResponseStages,

    /// Reference to strategy learner (for threshold adjustments)
    strategy_learner: Arc<RwLock<StrategyLearner>>,

    /// Event emitter for notifications
    event_emitter: Arc<dyn EventEmitter>,
}

#[derive(Debug, Clone)]
pub struct ResponseStages {
    monitor_threshold: u32,      // Default 3
    cluster_threshold: u32,      // Default 3
    adjust_threshold: u32,       // Default 5
    surface_threshold: u32,      // Default 10
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseStage {
    Monitor,
    Cluster,
    AdjustThresholds,
    SurfaceToUser,
}

#[derive(Debug, Clone)]
pub enum ResponseAction {
    None,
    Monitor,
    LogCluster { cluster_id: Uuid, size: usize },
    AdjustedExploration { cluster_id: Uuid, new_exploration_bonus: f64 },
    SurfacedGap { gap_id: Uuid },
}

impl GraduatedResponse {
    /// Determine appropriate response for a novelty result
    pub async fn respond(
        &self,
        novelty: &NoveltyResult,
        detector: &mut NoveltyDetector,
        gap_detector: &mut CapabilityGapDetector,
    ) -> Result<ResponseAction> {
        match novelty {
            NoveltyResult::Known { .. } => Ok(ResponseAction::None),

            NoveltyResult::NovelClustered { cluster_id, .. } => {
                let cluster = detector.get_cluster(*cluster_id)?;
                self.respond_to_cluster(cluster).await
            }

            NoveltyResult::NovelOutlier { .. } => Ok(ResponseAction::Monitor),
        }
    }

    async fn respond_to_cluster(&self, cluster: &AnomalyCluster) -> Result<ResponseAction> {
        let stage = self.determine_stage(cluster);

        match stage {
            ResponseStage::Monitor => Ok(ResponseAction::Monitor),

            ResponseStage::Cluster => Ok(ResponseAction::LogCluster {
                cluster_id: cluster.id,
                size: cluster.members.len(),
            }),

            ResponseStage::AdjustThresholds => {
                self.adjust_exploration(cluster).await?;
                Ok(ResponseAction::AdjustedExploration {
                    cluster_id: cluster.id,
                    new_exploration_bonus: 0.2,
                })
            }

            ResponseStage::SurfaceToUser => {
                let gap = self.create_gap_from_cluster(cluster).await?;
                self.emit_gap_event(&gap).await?;
                Ok(ResponseAction::SurfacedGap { gap_id: gap.id })
            }
        }
    }

    /// Adjust strategy learner to explore more in novel contexts
    async fn adjust_exploration(&self, cluster: &AnomalyCluster) -> Result<()> {
        let mut learner = self.strategy_learner.write().await;
        learner.increase_exploration_for_context(
            cluster.inferred_context_type(),
            0.2,  // Temporary exploration bonus
        );
        Ok(())
    }
}
```

---

## SolutionGenerator

```rust
/// Generates potential solutions for capability gaps
pub struct SolutionGenerator {
    /// Template solutions for common gap types
    templates: HashMap<GapCategory, Vec<SolutionTemplate>>,

    /// Pattern analyzer for discovering solutions from adjacent contexts
    pattern_analyzer: PatternAnalyzer,
}

#[derive(Debug, Clone)]
pub struct SolutionTemplate {
    pub id: &'static str,
    pub description: String,
    pub applicability: GapCategory,
    pub action_type: SolutionAction,
    pub confidence: f64,
}

impl SolutionGenerator {
    pub fn new() -> Self {
        Self {
            templates: Self::default_templates(),
            pattern_analyzer: PatternAnalyzer::new(),
        }
    }

    fn default_templates() -> HashMap<GapCategory, Vec<SolutionTemplate>> {
        let mut templates = HashMap::new();

        templates.insert(GapCategory::ExtractionFailure, vec![
            SolutionTemplate {
                id: "add_examples",
                description: "Add explicit examples for this pattern".into(),
                applicability: GapCategory::ExtractionFailure,
                action_type: SolutionAction::AddExamples { suggested_count: 3 },
                confidence: 0.7,
            },
            SolutionTemplate {
                id: "lower_confidence",
                description: "Lower extraction confidence threshold".into(),
                applicability: GapCategory::ExtractionFailure,
                action_type: SolutionAction::AdjustConfidence { suggested_threshold: 0.5 },
                confidence: 0.5,
            },
        ]);

        templates.insert(GapCategory::NegativeImpact, vec![
            SolutionTemplate {
                id: "disable_learning",
                description: "Disable the problematic learning".into(),
                applicability: GapCategory::NegativeImpact,
                action_type: SolutionAction::DisableLearning { learning_id: LearningId::default() },
                confidence: 0.8,
            },
            SolutionTemplate {
                id: "adjust_strategy",
                description: "Try a different injection strategy".into(),
                applicability: GapCategory::NegativeImpact,
                action_type: SolutionAction::AdjustStrategy { suggested_variant: StrategyVariant::Deferred },
                confidence: 0.6,
            },
        ]);

        templates.insert(GapCategory::InjectionFailure, vec![
            SolutionTemplate {
                id: "change_strategy",
                description: "Switch to a different injection strategy".into(),
                applicability: GapCategory::InjectionFailure,
                action_type: SolutionAction::AdjustStrategy { suggested_variant: StrategyVariant::MainContext },
                confidence: 0.6,
            },
        ]);

        templates.insert(GapCategory::NovelDomain, vec![
            SolutionTemplate {
                id: "mark_out_of_scope",
                description: "Mark this domain as out of scope".into(),
                applicability: GapCategory::NovelDomain,
                action_type: SolutionAction::MarkOutOfScope { reason: "Novel domain".into() },
                confidence: 0.5,
            },
            SolutionTemplate {
                id: "create_manual",
                description: "Create a manual learning for this pattern".into(),
                applicability: GapCategory::NovelDomain,
                action_type: SolutionAction::CreateManualLearning { template: String::new() },
                confidence: 0.6,
            },
        ]);

        templates
    }

    /// Generate solutions for a capability gap
    pub async fn generate(&self, gap: &CapabilityGap) -> Result<Vec<SuggestedSolution>> {
        let mut solutions = Vec::new();

        // Step 1: Apply templates for this gap category
        if let Some(templates) = self.templates.get(&gap.category) {
            for template in templates {
                solutions.push(SuggestedSolution {
                    id: Uuid::new_v4(),
                    description: template.description.clone(),
                    action: self.specialize_action(&template.action_type, gap),
                    confidence: template.confidence,
                    source: SolutionSource::Template,
                });
            }
        }

        // Step 2: Analyze similar successful contexts for hints
        let pattern_solutions = self.pattern_analyzer
            .find_solutions_from_similar_contexts(gap)
            .await?;
        solutions.extend(pattern_solutions);

        // Sort by confidence, return top 3
        solutions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        Ok(solutions.into_iter().take(3).collect())
    }
}
```

---

## M32 NoveltyHook Integration

```rust
/// Implementation of M32's NoveltyHook trait for full integration
pub struct OpenWorldHook {
    novelty_detector: Arc<RwLock<NoveltyDetector>>,
    gap_detector: Arc<RwLock<CapabilityGapDetector>>,
    graduated_response: Arc<GraduatedResponse>,
    solution_generator: Arc<SolutionGenerator>,
    store: Arc<dyn OpenWorldStore>,
}

#[async_trait]
impl NoveltyHook for OpenWorldHook {
    /// Called by M32's StrategyConsumer after each strategy outcome
    async fn on_strategy_outcome(
        &self,
        learning: &Learning,
        context: &SessionContext,
        strategy: &InjectionStrategy,
        outcome: &StrategyOutcome,
    ) -> Result<()> {
        // Step 1: Check for novelty
        let novelty_result = {
            let mut detector = self.novelty_detector.write().await;
            detector.detect(context).await?
        };

        // Step 2: Process through graduated response
        let action = {
            let mut novelty = self.novelty_detector.write().await;
            let mut gaps = self.gap_detector.write().await;
            self.graduated_response.respond(&novelty_result, &mut novelty, &mut gaps).await?
        };

        // Step 3: Check for capability gaps
        let gap = {
            let mut detector = self.gap_detector.write().await;
            detector.process_outcome(
                context,
                Some(&AttributionRecord::from_outcome(learning, outcome)),
                None,
            ).await?
        };

        // Step 4: Generate solutions if gap surfaced
        if let Some(gap) = gap {
            if gap.status == GapStatus::Confirmed {
                let solutions = self.solution_generator.generate(&gap).await?;
                self.store.update_gap_solutions(gap.id, solutions).await?;
                self.emit_gap_confirmed(&gap).await?;
            }
        }

        // Step 5: If pattern succeeded, mark as known
        if outcome.value > 0.5 && outcome.confidence > 0.7 {
            let mut detector = self.novelty_detector.write().await;
            detector.mark_known(context).await?;
        }

        // Step 6: Persist novelty event
        self.store.save_novelty_event(NoveltyEvent {
            event_id: Uuid::new_v4(),
            context_hash: hash_context(context),
            novelty_result: novelty_result.clone(),
            response_action: action,
            timestamp: Utc::now(),
        }).await?;

        Ok(())
    }
}

impl OpenWorldHook {
    /// Feedback to M32's strategy learner for novel contexts
    pub async fn feedback_to_strategy_learner(
        &self,
        cluster: &AnomalyCluster,
        strategy_learner: &mut StrategyLearner,
    ) -> Result<()> {
        strategy_learner.set_exploration_bonus(
            cluster.inferred_context_type(),
            0.2,  // 20% exploration bonus
            Duration::hours(24),  // Decays over 24 hours
        );
        Ok(())
    }

    /// Adjust category priors when capability gaps are confirmed
    pub async fn adjust_category_priors(
        &self,
        gap: &CapabilityGap,
        strategy_learner: &mut StrategyLearner,
    ) -> Result<()> {
        if gap.category == GapCategory::NegativeImpact {
            strategy_learner.reduce_category_confidence(
                gap.inferred_learning_category(),
                0.1,  // Reduce by 10%
            );
        }
        Ok(())
    }
}
```

---

## Storage

### CozoDB Schema

```datalog
:create pattern_fingerprint {
    id: String =>
    embedding: <F32; 384>,
    category: String,
    context_hash: Int,
    created_at: Int
}

:create anomaly_cluster {
    id: String =>
    centroid: <F32; 384>,
    density: Float,
    first_seen: Int,
    last_seen: Int,
    occurrence_count: Int
}

:create cluster_member {
    cluster_id: String,
    fingerprint_id: String =>
    joined_at: Int
}

:create capability_gap {
    id: String =>
    description: String,
    category: String,
    severity: String,
    status: String,
    failure_count: Int,
    negative_attribution_sum: Float,
    low_confidence_count: Int,
    example_contexts_json: String,
    first_seen: Int,
    last_seen: Int
}

:create gap_solution {
    gap_id: String,
    solution_id: String =>
    description: String,
    action_json: String,
    confidence: Float,
    source: String,
    created_at: Int
}

:create failure_record {
    id: String =>
    context_hash: Int,
    learning_id: String?,
    failure_type: String,
    attribution_value: Float?,
    extraction_confidence: Float?,
    timestamp: Int
}

:create novelty_event {
    event_id: String =>
    context_hash: Int,
    result_type: String,
    result_data_json: String,
    response_action: String,
    timestamp: Int
}

-- Indexes
::hnsw create pattern_fingerprint:semantic_idx {
    dim: 384,
    m: 16,
    ef_construction: 200,
    fields: [embedding]
}

::index create failure_record:by_context { context_hash }
::index create failure_record:by_time { timestamp }
::index create capability_gap:by_status { status }
::index create capability_gap:by_severity { severity }
::index create novelty_event:by_time { timestamp }
```

### Iggy Topics

```rust
/// Iggy stream: groove.openworld
/// Topics: novelty, gaps, feedback

pub enum OpenWorldEvent {
    /// Pattern detected as novel or known
    NoveltyDetected {
        event_id: Uuid,
        context_hash: u64,
        result: NoveltyResult,
        response: ResponseAction,
        timestamp: DateTime<Utc>,
    },

    /// Cluster formed or updated
    ClusterUpdated {
        cluster_id: Uuid,
        member_count: u32,
        density: f64,
        timestamp: DateTime<Utc>,
    },

    /// Gap status changed
    GapStatusChanged {
        gap_id: Uuid,
        old_status: GapStatus,
        new_status: GapStatus,
        severity: GapSeverity,
        timestamp: DateTime<Utc>,
    },

    /// Solutions generated for gap
    SolutionsGenerated {
        gap_id: Uuid,
        solutions: Vec<SuggestedSolution>,
        timestamp: DateTime<Utc>,
    },

    /// Feedback sent to strategy learner
    StrategyFeedback {
        context_type: ContextType,
        exploration_bonus: f64,
        reason: String,
        timestamp: DateTime<Utc>,
    },
}
```

---

## Configuration

```toml
[plugins.groove.openworld]
enabled = true

[plugins.groove.openworld.novelty]
# Similarity threshold for "known" pattern (cosine similarity)
similarity_threshold = 0.85

# DBSCAN clustering parameters
dbscan_eps = 0.3              # Neighborhood radius
dbscan_min_points = 3         # Minimum points for cluster

# How often to attempt reclustering (seconds)
recluster_interval = 300

[plugins.groove.openworld.gaps]
# Thresholds for gap detection
failure_threshold = 3
negative_attribution_threshold = -0.5
low_confidence_threshold = 5

# Severity escalation
medium_severity_count = 5
high_severity_count = 20

[plugins.groove.openworld.response]
# Graduated response thresholds
monitor_threshold = 3
cluster_threshold = 3
adjust_threshold = 5
surface_threshold = 10

# Exploration bonus when adjusting strategy learner
exploration_bonus = 0.2
exploration_decay_hours = 24

[plugins.groove.openworld.solutions]
# Maximum solutions to generate per gap
max_solutions = 3

# Enable pattern analysis (slower but finds novel solutions)
pattern_analysis_enabled = true

[plugins.groove.openworld.embedding]
# Reuse M30's embedder configuration
# Falls back to [plugins.groove.extraction.embedding]
```

---

## CLI Commands

```
# Novelty Detection
vibes groove novelty status              # Show detector status, threshold, cluster count
vibes groove novelty clusters            # List anomaly clusters with sizes
vibes groove novelty cluster <id>        # Show cluster details and members
vibes groove novelty fingerprints        # List known pattern fingerprints
vibes groove novelty mark-known <hash>   # Manually mark a pattern as known
vibes groove novelty reset               # Reset detector (clear clusters, keep fingerprints)

# Capability Gaps
vibes groove gaps status                 # Show gap detector status, active gaps count
vibes groove gaps list                   # List all gaps by severity
vibes groove gaps show <id>              # Show gap details, evidence, solutions
vibes groove gaps dismiss <id>           # Mark gap as dismissed (not a real gap)
vibes groove gaps resolve <id>           # Mark gap as resolved
vibes groove gaps apply <gap> <solution> # Apply a suggested solution

# Combined
vibes groove openworld status            # Overview of both novelty + gaps
vibes groove openworld history           # Recent novelty events and gap changes
```
