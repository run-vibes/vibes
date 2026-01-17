# Milestone 32: Adaptive Strategies - Design

## Overview

Adaptive Strategies learns which injection strategies work best for each learning category and context, using Thompson sampling for exploration/exploitation. It connects the Attribution Engine (which measures learning value) to Learning Injection (which applies learnings).

**Core principle**: Hierarchical learning with conservative defaults. Category-level priors provide cold-start behavior; individual learnings specialize over time. Default to deferred injection until evidence supports more aggressive strategies.

## Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Iggy: groove.attribution                         │
│            (AttributionEvent with learning outcomes)                │
└──────────────────────────┬──────────────────────────────────────────┘
                           │ consumed by
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Strategy Consumer                                 │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐ │
│  │ Outcome Router  │  │ Distribution    │  │ Novelty Hook        │ │
│  │ (attr + direct) │─▶│ Updater         │─▶│ (future extension)  │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────────┘ │
└──────────────────────────────────────────────────────────────────────┘
                           │ writes to
           ┌───────────────┴───────────────┐
           ▼                               ▼
┌─────────────────────┐         ┌─────────────────────┐
│ Iggy: strategy      │         │ CozoDB:             │
│ (raw events)        │         │ strategy_distribution│
└─────────────────────┘         └─────────────────────┘
                                          │
                                          ▼
                               ┌─────────────────────┐
                               │ Learning Injector   │
                               │ (lazy strategy      │
                               │  selection + cache) │
                               └─────────────────────┘
```

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Scope | Strategy + parameters | Full adaptability without novelty detection complexity |
| Novelty detection | Hooks for future | Clean extension point via `NoveltyHook` trait |
| Distribution hierarchy | Category → learning | Solves cold-start, allows specialization over time |
| Parameters learned | All (type + params) | Maximum adaptability for each strategy variant |
| Outcome sources | Attribution + direct | Attributed value preferred, direct signals as fallback |
| Persistence | Iggy + CozoDB | Consistent with M30/M31, enables replay |
| Integration | Lazy + cached | Compute on demand, consistent within session |
| Default strategy | Deferred (0.4 weight) | Conservative until evidence accumulates |
| Specialization | 20 sessions threshold | Balance between learning speed and stability |

---

## Core Types

### Injection Strategy

```rust
/// Strategy for injecting a learning into a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InjectionStrategy {
    /// Inject into main Claude context
    MainContext {
        position: ContextPosition,
        format: InjectionFormat,
    },

    /// Delegate to a subagent
    Subagent {
        agent_type: SubagentType,
        blocking: bool,
        prompt_template: Option<String>,
    },

    /// Run in background, surface later
    BackgroundSubagent {
        agent_type: SubagentType,
        callback: CallbackMethod,
        timeout_ms: u64,
    },

    /// Don't inject now, wait for trigger
    Deferred {
        trigger: DeferralTrigger,
        max_wait_ms: Option<u64>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StrategyVariant {
    MainContext,
    Subagent,
    BackgroundSubagent,
    Deferred,
}
```

### Strategy Distribution

```rust
/// Hierarchical distribution for strategy selection
pub struct StrategyDistribution {
    pub category: LearningCategory,
    pub context_type: ContextType,

    /// Weights for each strategy variant
    pub strategy_weights: HashMap<StrategyVariant, AdaptiveParam>,

    /// Parameters within each strategy (also adaptive)
    pub strategy_params: HashMap<StrategyVariant, StrategyParams>,

    pub session_count: u32,
    pub updated_at: DateTime<Utc>,
}

/// Per-learning specialization (inherits from category distribution)
pub struct LearningStrategyOverride {
    pub learning_id: LearningId,
    pub base_category: LearningCategory,

    /// Only populated once learning has enough data
    pub specialized_weights: Option<HashMap<StrategyVariant, AdaptiveParam>>,
    pub specialization_threshold: u32,  // Sessions before specializing

    pub session_count: u32,
}
```

### Outcome Types

```rust
pub struct StrategyOutcome {
    pub value: f64,
    pub confidence: f64,
    pub source: OutcomeSource,
}

pub enum OutcomeSource {
    Attribution,  // From M31 attribution engine
    Direct,       // From lightweight signals
    Both,         // Combined
}

pub struct StrategyEvent {
    pub event_id: EventId,
    pub learning_id: LearningId,
    pub session_id: SessionId,
    pub strategy: InjectionStrategy,
    pub outcome: StrategyOutcome,
    pub timestamp: DateTime<Utc>,
}
```

---

## Strategy Learner

### Thompson Sampling Selection

```rust
/// Learns which strategies work via Thompson sampling
pub struct StrategyLearner {
    /// Category-level distributions (the priors)
    category_distributions: HashMap<(LearningCategory, ContextType), StrategyDistribution>,

    /// Per-learning overrides (specializations)
    learning_overrides: HashMap<LearningId, LearningStrategyOverride>,

    /// Session cache for lazy + consistent selection
    session_cache: HashMap<(SessionId, LearningId), InjectionStrategy>,
}

#[async_trait]
impl StrategyLearner {
    /// Select strategy for a learning in context (lazy + cached)
    pub async fn select_strategy(
        &mut self,
        learning: &Learning,
        context: &SessionContext,
    ) -> InjectionStrategy {
        let cache_key = (context.session_id, learning.id);

        if let Some(cached) = self.session_cache.get(&cache_key) {
            return cached.clone();
        }

        let strategy = self.sample_strategy(learning, context);
        self.session_cache.insert(cache_key, strategy.clone());
        strategy
    }

    fn sample_strategy(&self, learning: &Learning, context: &SessionContext) -> InjectionStrategy {
        // Check for learning-specific override first
        let weights = self.get_effective_weights(learning, context);

        // Thompson sampling: sample from each Beta posterior, pick highest
        let selected_variant = weights
            .iter()
            .map(|(variant, param)| (variant.clone(), param.sample()))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(v, _)| v)
            .unwrap_or(StrategyVariant::Deferred);

        // Sample parameters for the selected strategy
        self.sample_params(selected_variant, learning, context)
    }

    fn get_effective_weights(&self, learning: &Learning, context: &SessionContext)
        -> &HashMap<StrategyVariant, AdaptiveParam>
    {
        // Use learning override if specialized, otherwise category distribution
        if let Some(override_) = self.learning_overrides.get(&learning.id) {
            if let Some(ref specialized) = override_.specialized_weights {
                return specialized;
            }
        }

        let key = (learning.category, context.context_type);
        &self.category_distributions.get(&key).unwrap().strategy_weights
    }

    /// Clear session cache (call at session end)
    pub fn clear_session(&mut self, session_id: SessionId) {
        self.session_cache.retain(|(sid, _), _| *sid != session_id);
    }
}
```

---

## Outcome Routing

### Combining Attribution and Direct Signals

```rust
/// Routes outcomes from multiple sources to distribution updates
pub struct OutcomeRouter {
    /// Weight for attributed value vs direct signals
    attribution_weight: f64,  // 0.7
    direct_weight: f64,       // 0.3
}

impl OutcomeRouter {
    /// Compute effective outcome from available signals
    pub fn compute_outcome(
        &self,
        attribution: Option<&AttributionRecord>,
        direct_signals: &[LightweightEvent],
        strategy_used: &InjectionStrategy,
    ) -> Option<StrategyOutcome> {
        let attributed_value = attribution
            .filter(|a| a.was_activated)
            .map(|a| a.attributed_value);

        let direct_value = Self::aggregate_direct_signals(direct_signals);

        match (attributed_value, direct_value) {
            (Some(av), Some(dv)) => Some(StrategyOutcome {
                value: self.attribution_weight * av + self.direct_weight * dv,
                confidence: 0.9,  // Both sources available
                source: OutcomeSource::Both,
            }),
            (Some(av), None) => Some(StrategyOutcome {
                value: av,
                confidence: 0.8,
                source: OutcomeSource::Attribution,
            }),
            (None, Some(dv)) => Some(StrategyOutcome {
                value: dv,
                confidence: 0.5,  // Lower confidence for direct only
                source: OutcomeSource::Direct,
            }),
            (None, None) => None,  // No signal, skip update
        }
    }
}
```

### Distribution Updates

```rust
/// Updates strategy distributions based on outcomes
pub struct DistributionUpdater {
    specialization_threshold: u32,  // Sessions before learning specializes (default: 20)
}

impl DistributionUpdater {
    pub fn update(
        &self,
        distributions: &mut HashMap<(LearningCategory, ContextType), StrategyDistribution>,
        overrides: &mut HashMap<LearningId, LearningStrategyOverride>,
        learning: &Learning,
        context: &SessionContext,
        strategy: &InjectionStrategy,
        outcome: &StrategyOutcome,
    ) {
        let variant = StrategyVariant::from(strategy);
        let key = (learning.category, context.context_type);

        // Always update category distribution
        if let Some(dist) = distributions.get_mut(&key) {
            if let Some(param) = dist.strategy_weights.get_mut(&variant) {
                param.update(outcome.value, outcome.confidence);
            }
            dist.session_count += 1;
        }

        // Update or create learning override
        let override_ = overrides.entry(learning.id).or_insert_with(|| {
            LearningStrategyOverride::new(learning.id, learning.category)
        });
        override_.session_count += 1;

        // Specialize if threshold reached
        if override_.session_count >= self.specialization_threshold
            && override_.specialized_weights.is_none()
        {
            override_.specialize_from(&distributions[&key]);
        }

        // Update specialized weights if they exist
        if let Some(ref mut specialized) = override_.specialized_weights {
            if let Some(param) = specialized.get_mut(&variant) {
                param.update(outcome.value, outcome.confidence);
            }
        }
    }
}
```

---

## Strategy Consumer

```rust
/// Iggy consumer that updates strategy distributions from outcomes
pub struct StrategyConsumer {
    outcome_router: OutcomeRouter,
    distribution_updater: DistributionUpdater,
    store: Arc<dyn StrategyStore>,

    /// Hook for future novelty detection
    novelty_hook: Option<Box<dyn NoveltyHook>>,
}

impl StrategyConsumer {
    pub async fn new(store: Arc<dyn StrategyStore>) -> Result<Self> {
        Ok(Self {
            outcome_router: OutcomeRouter::default(),
            distribution_updater: DistributionUpdater::default(),
            store,
            novelty_hook: None,
        })
    }

    /// Register novelty detection hook (future extension)
    pub fn with_novelty_hook(mut self, hook: Box<dyn NoveltyHook>) -> Self {
        self.novelty_hook = Some(hook);
        self
    }

    /// Process attribution event from Iggy
    pub async fn process_attribution_event(&self, event: &AttributionEvent) -> Result<()> {
        // Load current distributions
        let mut distributions = self.store.load_distributions().await?;
        let mut overrides = self.store.load_overrides().await?;

        for record in &event.attribution_records {
            let learning = self.store.get_learning(record.learning_id).await?;
            let context = self.store.get_session_context(record.session_id).await?;
            let strategy = self.store.get_used_strategy(record.session_id, record.learning_id).await?;

            // Route outcome from available signals
            let outcome = self.outcome_router.compute_outcome(
                Some(record),
                &event.lightweight_events,
                &strategy,
            );

            if let Some(outcome) = outcome {
                // Update distributions
                self.distribution_updater.update(
                    &mut distributions,
                    &mut overrides,
                    &learning,
                    &context,
                    &strategy,
                    &outcome,
                );

                // Write strategy event to Iggy
                self.write_strategy_event(&learning, &strategy, &outcome).await?;

                // Invoke novelty hook if present
                if let Some(ref hook) = self.novelty_hook {
                    hook.on_strategy_outcome(&learning, &context, &strategy, &outcome).await?;
                }
            }
        }

        // Persist updated distributions
        self.store.save_distributions(&distributions).await?;
        self.store.save_overrides(&overrides).await?;

        Ok(())
    }
}

/// Extension point for future novelty detection
#[async_trait]
pub trait NoveltyHook: Send + Sync {
    async fn on_strategy_outcome(
        &self,
        learning: &Learning,
        context: &SessionContext,
        strategy: &InjectionStrategy,
        outcome: &StrategyOutcome,
    ) -> Result<()>;
}
```

---

## Storage

### CozoDB Schema

```datalog
:create strategy_distribution {
    category: String,
    context_type: String =>
    strategy_weights_json: String,    -- HashMap<StrategyVariant, AdaptiveParam> serialized
    strategy_params_json: String,     -- HashMap<StrategyVariant, StrategyParams> serialized
    session_count: Int,
    updated_at: Int
}

:create learning_strategy_override {
    learning_id: String =>
    base_category: String,
    specialized_weights_json: String?,  -- None until threshold reached
    specialization_threshold: Int,
    session_count: Int,
    updated_at: Int
}

:create strategy_event {
    event_id: String =>
    learning_id: String,
    session_id: String,
    strategy_variant: String,
    strategy_params_json: String,
    outcome_value: Float,
    outcome_confidence: Float,
    outcome_source: String,
    timestamp: Int
}

:create strategy_session_cache {
    session_id: String,
    learning_id: String =>
    strategy_json: String,
    selected_at: Int
}

-- Indexes for common queries
::index create strategy_distribution:by_category { category }
::index create learning_strategy_override:by_category { base_category }
::index create strategy_event:by_learning { learning_id }
::index create strategy_event:by_session { session_id }
::index create strategy_event:by_time { timestamp }
```

### Iggy Topics

- `groove.strategy` - Raw strategy selection and outcome events (audit trail)
- Consumed from: `groove.attribution` - Attribution events with learning outcomes

---

## Configuration

```toml
[plugins.groove.strategy]
enabled = true

[plugins.groove.strategy.sampling]
# Initial prior for new strategy variants (Beta distribution)
initial_alpha = 1.0
initial_beta = 1.0

# Exploration bonus for under-sampled strategies
exploration_bonus = 0.1
min_samples_for_confidence = 5

[plugins.groove.strategy.hierarchy]
# Sessions before a learning gets specialized weights
specialization_threshold = 20

# Minimum confidence to use specialized over category
specialization_confidence = 0.6

[plugins.groove.strategy.outcome]
# Weights for combining attribution vs direct signals
attribution_weight = 0.7
direct_weight = 0.3

# Minimum confidence to update distributions
min_outcome_confidence = 0.3

[plugins.groove.strategy.cache]
# Session cache TTL (cleared on session end anyway)
ttl_seconds = 3600

[plugins.groove.strategy.defaults]
# Default strategy when no data available
default_strategy = "Deferred"
default_trigger = "Explicit"

# Strategy variant weights for cold start (sums to 1.0)
[plugins.groove.strategy.defaults.initial_weights]
MainContext = 0.3
Subagent = 0.2
BackgroundSubagent = 0.1
Deferred = 0.4
```

---

## CLI Commands

```
vibes groove strategy status              # Show strategy learner status
vibes groove strategy distributions       # List category distributions
vibes groove strategy show <category>     # Detailed distribution breakdown
vibes groove strategy learning <id>       # Show learning's strategy override
vibes groove strategy history <learning>  # Strategy selection history
vibes groove strategy reset <category>    # Reset category to default priors
vibes groove strategy reset-learning <id> # Clear learning specialization
```
