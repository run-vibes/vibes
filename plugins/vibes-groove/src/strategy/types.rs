//! Strategy types for adaptive injection
//!
//! Defines injection strategies, outcomes, and events for the adaptive
//! strategies system.

use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::assessment::{EventId, SessionId};
use crate::types::LearningId;

/// Strategy for injecting a learning into a session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

impl InjectionStrategy {
    /// Get the variant type of this strategy
    pub fn variant(&self) -> StrategyVariant {
        StrategyVariant::from(self)
    }
}

/// Simplified strategy variant for matching and distribution keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrategyVariant {
    MainContext,
    Subagent,
    BackgroundSubagent,
    Deferred,
}

impl StrategyVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MainContext => "main_context",
            Self::Subagent => "subagent",
            Self::BackgroundSubagent => "background_subagent",
            Self::Deferred => "deferred",
        }
    }

    /// All variants for iteration
    pub fn all() -> &'static [StrategyVariant] {
        &[
            Self::MainContext,
            Self::Subagent,
            Self::BackgroundSubagent,
            Self::Deferred,
        ]
    }
}

/// Error type for parsing StrategyVariant from string
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseStrategyVariantError(String);

impl std::fmt::Display for ParseStrategyVariantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown strategy variant: {}", self.0)
    }
}

impl std::error::Error for ParseStrategyVariantError {}

impl FromStr for StrategyVariant {
    type Err = ParseStrategyVariantError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "main_context" => Ok(Self::MainContext),
            "subagent" => Ok(Self::Subagent),
            "background_subagent" => Ok(Self::BackgroundSubagent),
            "deferred" => Ok(Self::Deferred),
            _ => Err(ParseStrategyVariantError(s.to_string())),
        }
    }
}

impl From<&InjectionStrategy> for StrategyVariant {
    fn from(strategy: &InjectionStrategy) -> Self {
        match strategy {
            InjectionStrategy::MainContext { .. } => Self::MainContext,
            InjectionStrategy::Subagent { .. } => Self::Subagent,
            InjectionStrategy::BackgroundSubagent { .. } => Self::BackgroundSubagent,
            InjectionStrategy::Deferred { .. } => Self::Deferred,
        }
    }
}

/// Where to position the learning in context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextPosition {
    /// At the beginning of context
    Prefix,
    /// At the end of context
    Suffix,
    /// Near relevant content
    Contextual,
}

impl ContextPosition {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Prefix => "prefix",
            Self::Suffix => "suffix",
            Self::Contextual => "contextual",
        }
    }
}

impl FromStr for ContextPosition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "prefix" => Ok(Self::Prefix),
            "suffix" => Ok(Self::Suffix),
            "contextual" => Ok(Self::Contextual),
            _ => Err(format!("unknown context position: {s}")),
        }
    }
}

/// How to format the learning for injection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InjectionFormat {
    /// Plain text
    Plain,
    /// Structured with tags
    Tagged,
    /// As a system instruction
    SystemInstruction,
}

impl InjectionFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Tagged => "tagged",
            Self::SystemInstruction => "system_instruction",
        }
    }
}

impl FromStr for InjectionFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "plain" => Ok(Self::Plain),
            "tagged" => Ok(Self::Tagged),
            "system_instruction" => Ok(Self::SystemInstruction),
            _ => Err(format!("unknown injection format: {s}")),
        }
    }
}

/// Type of subagent to use
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubagentType {
    /// General purpose agent
    General,
    /// Specialized for exploration
    Explorer,
    /// Specialized for planning
    Planner,
    /// Custom agent with specified type
    Custom(String),
}

impl SubagentType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::General => "general",
            Self::Explorer => "explorer",
            Self::Planner => "planner",
            Self::Custom(s) => s,
        }
    }
}

impl FromStr for SubagentType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "general" => Self::General,
            "explorer" => Self::Explorer,
            "planner" => Self::Planner,
            other => Self::Custom(other.to_string()),
        })
    }
}

/// How to receive results from background subagent
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallbackMethod {
    /// Poll for results
    Poll,
    /// Wait for completion
    Wait,
    /// Notify via event
    Event,
}

impl CallbackMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Poll => "poll",
            Self::Wait => "wait",
            Self::Event => "event",
        }
    }
}

impl FromStr for CallbackMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "poll" => Ok(Self::Poll),
            "wait" => Ok(Self::Wait),
            "event" => Ok(Self::Event),
            _ => Err(format!("unknown callback method: {s}")),
        }
    }
}

/// What triggers deferred injection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeferralTrigger {
    /// User explicitly requests
    Explicit,
    /// Related topic comes up
    TopicMatch,
    /// Error occurs that learning addresses
    ErrorMatch,
    /// After N messages
    MessageCount(u32),
}

impl DeferralTrigger {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Explicit => "explicit",
            Self::TopicMatch => "topic_match",
            Self::ErrorMatch => "error_match",
            Self::MessageCount(_) => "message_count",
        }
    }
}

/// Outcome of using a strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategyOutcome {
    /// Value contribution (-1 to +1)
    pub value: f64,
    /// Confidence in the value estimate (0 to 1)
    pub confidence: f64,
    /// Source of the outcome signal
    pub source: OutcomeSource,
}

impl StrategyOutcome {
    /// Create a new outcome
    pub fn new(value: f64, confidence: f64, source: OutcomeSource) -> Self {
        Self {
            value: value.clamp(-1.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
            source,
        }
    }
}

/// Where the outcome signal came from
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutcomeSource {
    /// From M31 attribution engine
    Attribution,
    /// From lightweight direct signals
    Direct,
    /// Combined from both sources
    Both,
}

impl OutcomeSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Attribution => "attribution",
            Self::Direct => "direct",
            Self::Both => "both",
        }
    }
}

impl FromStr for OutcomeSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "attribution" => Ok(Self::Attribution),
            "direct" => Ok(Self::Direct),
            "both" => Ok(Self::Both),
            _ => Err(format!("unknown outcome source: {s}")),
        }
    }
}

/// Event recording a strategy selection and outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyEvent {
    pub event_id: EventId,
    pub learning_id: LearningId,
    pub session_id: SessionId,
    pub strategy: InjectionStrategy,
    pub outcome: StrategyOutcome,
    pub timestamp: DateTime<Utc>,
}

impl StrategyEvent {
    /// Create a new strategy event
    pub fn new(
        learning_id: LearningId,
        session_id: SessionId,
        strategy: InjectionStrategy,
        outcome: StrategyOutcome,
    ) -> Self {
        Self {
            event_id: EventId::new(),
            learning_id,
            session_id,
            strategy,
            outcome,
            timestamp: Utc::now(),
        }
    }
}

/// Context type for strategy distribution keys
///
/// Note: This type implements Copy for convenience in HashMap keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextType {
    /// Interactive chat session
    Interactive,
    /// Automated/batch processing
    Batch,
    /// Code review context
    CodeReview,
    /// Planning/architecture context
    Planning,
}

impl ContextType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Interactive => "interactive",
            Self::Batch => "batch",
            Self::CodeReview => "code_review",
            Self::Planning => "planning",
        }
    }
}

impl FromStr for ContextType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "interactive" => Ok(Self::Interactive),
            "batch" => Ok(Self::Batch),
            "code_review" => Ok(Self::CodeReview),
            "planning" => Ok(Self::Planning),
            _ => Err(format!("unknown context type: {s}")),
        }
    }
}

// Re-export types needed for strategy distribution
pub use crate::types::{AdaptiveParam, LearningCategory};

/// Hierarchical distribution for strategy selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyDistribution {
    pub category: LearningCategory,
    pub context_type: ContextType,

    /// Weights for each strategy variant (serialized for storage)
    pub strategy_weights: std::collections::HashMap<StrategyVariant, AdaptiveParam>,

    /// Parameters within each strategy (also adaptive)
    pub strategy_params: std::collections::HashMap<StrategyVariant, StrategyParams>,

    pub session_count: u32,
    pub updated_at: DateTime<Utc>,
}

impl StrategyDistribution {
    /// Create a new distribution with default weights
    pub fn new(category: LearningCategory, context_type: ContextType) -> Self {
        use std::collections::HashMap;

        let mut strategy_weights = HashMap::new();
        // Default weights from design: Deferred=0.4, MainContext=0.3, Subagent=0.2, BackgroundSubagent=0.1
        strategy_weights.insert(
            StrategyVariant::MainContext,
            AdaptiveParam::new_with_prior(3.0, 7.0),
        );
        strategy_weights.insert(
            StrategyVariant::Subagent,
            AdaptiveParam::new_with_prior(2.0, 8.0),
        );
        strategy_weights.insert(
            StrategyVariant::BackgroundSubagent,
            AdaptiveParam::new_with_prior(1.0, 9.0),
        );
        strategy_weights.insert(
            StrategyVariant::Deferred,
            AdaptiveParam::new_with_prior(4.0, 6.0),
        );

        Self {
            category,
            context_type,
            strategy_weights,
            strategy_params: HashMap::new(),
            session_count: 0,
            updated_at: Utc::now(),
        }
    }

    /// Get the weight for a strategy variant
    pub fn get_weight(&self, variant: StrategyVariant) -> Option<&AdaptiveParam> {
        self.strategy_weights.get(&variant)
    }

    /// Update the weight for a strategy variant based on an outcome
    ///
    /// The value is the outcome signal (-1 to +1), and confidence weights the update.
    pub fn update_weight(&mut self, variant: StrategyVariant, value: f64, confidence: f64) {
        if let Some(param) = self.strategy_weights.get_mut(&variant) {
            // Convert outcome value from [-1, +1] to [0, 1] for AdaptiveParam
            // +1.0 -> 1.0 (success), -1.0 -> 0.0 (failure), 0.0 -> 0.5 (neutral)
            let outcome = (value.clamp(-1.0, 1.0) + 1.0) / 2.0;
            let weight = confidence.clamp(0.0, 1.0);
            param.update(outcome, weight);
            self.updated_at = Utc::now();
        }
    }
}

/// Per-learning specialization (inherits from category distribution)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStrategyOverride {
    pub learning_id: LearningId,
    pub base_category: LearningCategory,

    /// Only populated once learning has enough data
    pub specialized_weights: Option<std::collections::HashMap<StrategyVariant, AdaptiveParam>>,
    pub specialization_threshold: u32,

    pub session_count: u32,
    pub updated_at: DateTime<Utc>,
}

impl LearningStrategyOverride {
    /// Create a new override with no specialization yet
    pub fn new(learning_id: LearningId, base_category: LearningCategory) -> Self {
        Self {
            learning_id,
            base_category,
            specialized_weights: None,
            specialization_threshold: 20, // Default from design
            session_count: 0,
            updated_at: Utc::now(),
        }
    }

    /// Specialize by copying weights from category distribution
    pub fn specialize_from(&mut self, dist: &StrategyDistribution) {
        self.specialized_weights = Some(dist.strategy_weights.clone());
        self.updated_at = Utc::now();
    }

    /// Check if this override has specialized weights
    pub fn is_specialized(&self) -> bool {
        self.specialized_weights.is_some()
    }
}

/// Get effective weights for a learning, resolving the hierarchy
///
/// If the learning has specialized weights, use those. Otherwise, fall back
/// to the category distribution weights.
pub fn get_effective_weights<'a>(
    override_: Option<&'a LearningStrategyOverride>,
    category_dist: &'a StrategyDistribution,
) -> &'a std::collections::HashMap<StrategyVariant, AdaptiveParam> {
    if let Some(override_) = override_
        && let Some(ref specialized) = override_.specialized_weights
    {
        return specialized;
    }
    &category_dist.strategy_weights
}

/// Parameters for a specific strategy variant
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StrategyParams {
    /// Position preference for MainContext
    pub position_weights: Option<std::collections::HashMap<String, AdaptiveParam>>,

    /// Format preference for MainContext
    pub format_weights: Option<std::collections::HashMap<String, AdaptiveParam>>,

    /// Agent type preferences for Subagent variants
    pub agent_type_weights: Option<std::collections::HashMap<String, AdaptiveParam>>,

    /// Blocking preference for Subagent
    pub blocking_weight: Option<AdaptiveParam>,

    /// Trigger preferences for Deferred
    pub trigger_weights: Option<std::collections::HashMap<String, AdaptiveParam>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_strategy_variant_roundtrip() {
        for variant in StrategyVariant::all() {
            let s = variant.as_str();
            let parsed = StrategyVariant::from_str(s).unwrap();
            assert_eq!(*variant, parsed);
        }
    }

    #[test]
    fn test_injection_strategy_to_variant() {
        let strategy = InjectionStrategy::MainContext {
            position: ContextPosition::Prefix,
            format: InjectionFormat::Plain,
        };
        assert_eq!(strategy.variant(), StrategyVariant::MainContext);

        let strategy = InjectionStrategy::Subagent {
            agent_type: SubagentType::General,
            blocking: true,
            prompt_template: None,
        };
        assert_eq!(strategy.variant(), StrategyVariant::Subagent);

        let strategy = InjectionStrategy::BackgroundSubagent {
            agent_type: SubagentType::Explorer,
            callback: CallbackMethod::Poll,
            timeout_ms: 5000,
        };
        assert_eq!(strategy.variant(), StrategyVariant::BackgroundSubagent);

        let strategy = InjectionStrategy::Deferred {
            trigger: DeferralTrigger::Explicit,
            max_wait_ms: Some(10000),
        };
        assert_eq!(strategy.variant(), StrategyVariant::Deferred);
    }

    #[test]
    fn test_injection_strategy_serialization() {
        let strategy = InjectionStrategy::MainContext {
            position: ContextPosition::Suffix,
            format: InjectionFormat::Tagged,
        };
        let json = serde_json::to_string(&strategy).unwrap();
        let parsed: InjectionStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, strategy);

        let strategy = InjectionStrategy::Subagent {
            agent_type: SubagentType::Custom("my-agent".into()),
            blocking: false,
            prompt_template: Some("Do {{action}}".into()),
        };
        let json = serde_json::to_string(&strategy).unwrap();
        let parsed: InjectionStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, strategy);
    }

    #[test]
    fn test_strategy_outcome_clamping() {
        let outcome = StrategyOutcome::new(1.5, 2.0, OutcomeSource::Attribution);
        assert_eq!(outcome.value, 1.0);
        assert_eq!(outcome.confidence, 1.0);

        let outcome = StrategyOutcome::new(-1.5, -0.5, OutcomeSource::Direct);
        assert_eq!(outcome.value, -1.0);
        assert_eq!(outcome.confidence, 0.0);
    }

    #[test]
    fn test_outcome_source_roundtrip() {
        for source in [
            OutcomeSource::Attribution,
            OutcomeSource::Direct,
            OutcomeSource::Both,
        ] {
            let s = source.as_str();
            let parsed = OutcomeSource::from_str(s).unwrap();
            assert_eq!(source, parsed);
        }
    }

    #[test]
    fn test_strategy_event_creation() {
        let event = StrategyEvent::new(
            Uuid::now_v7(),
            SessionId::from("test-session"),
            InjectionStrategy::Deferred {
                trigger: DeferralTrigger::TopicMatch,
                max_wait_ms: None,
            },
            StrategyOutcome::new(0.5, 0.8, OutcomeSource::Both),
        );

        assert!(event.timestamp <= Utc::now());
        assert_eq!(event.strategy.variant(), StrategyVariant::Deferred);
    }

    #[test]
    fn test_strategy_event_serialization() {
        let event = StrategyEvent::new(
            Uuid::now_v7(),
            SessionId::from("sess-123"),
            InjectionStrategy::MainContext {
                position: ContextPosition::Contextual,
                format: InjectionFormat::SystemInstruction,
            },
            StrategyOutcome::new(0.7, 0.9, OutcomeSource::Attribution),
        );

        let json = serde_json::to_string(&event).unwrap();
        let parsed: StrategyEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.learning_id, event.learning_id);
        assert_eq!(parsed.session_id, event.session_id);
        assert_eq!(parsed.strategy, event.strategy);
    }

    #[test]
    fn test_context_type_roundtrip() {
        for ctx in [
            ContextType::Interactive,
            ContextType::Batch,
            ContextType::CodeReview,
            ContextType::Planning,
        ] {
            let s = ctx.as_str();
            let parsed = ContextType::from_str(s).unwrap();
            assert_eq!(ctx, parsed);
        }
    }

    #[test]
    fn test_context_position_roundtrip() {
        for pos in [
            ContextPosition::Prefix,
            ContextPosition::Suffix,
            ContextPosition::Contextual,
        ] {
            let s = pos.as_str();
            let parsed = ContextPosition::from_str(s).unwrap();
            assert_eq!(pos, parsed);
        }
    }

    #[test]
    fn test_injection_format_roundtrip() {
        for fmt in [
            InjectionFormat::Plain,
            InjectionFormat::Tagged,
            InjectionFormat::SystemInstruction,
        ] {
            let s = fmt.as_str();
            let parsed = InjectionFormat::from_str(s).unwrap();
            assert_eq!(fmt, parsed);
        }
    }

    #[test]
    fn test_subagent_type_parsing() {
        assert_eq!(
            SubagentType::from_str("general").unwrap(),
            SubagentType::General
        );
        assert_eq!(
            SubagentType::from_str("explorer").unwrap(),
            SubagentType::Explorer
        );
        assert_eq!(
            SubagentType::from_str("planner").unwrap(),
            SubagentType::Planner
        );
        assert_eq!(
            SubagentType::from_str("custom-type").unwrap(),
            SubagentType::Custom("custom-type".into())
        );
    }

    #[test]
    fn test_callback_method_roundtrip() {
        for method in [
            CallbackMethod::Poll,
            CallbackMethod::Wait,
            CallbackMethod::Event,
        ] {
            let s = method.as_str();
            let parsed = CallbackMethod::from_str(s).unwrap();
            assert_eq!(method, parsed);
        }
    }

    #[test]
    fn test_deferral_trigger_types() {
        assert_eq!(DeferralTrigger::Explicit.as_str(), "explicit");
        assert_eq!(DeferralTrigger::TopicMatch.as_str(), "topic_match");
        assert_eq!(DeferralTrigger::ErrorMatch.as_str(), "error_match");
        assert_eq!(DeferralTrigger::MessageCount(5).as_str(), "message_count");
    }

    // Distribution hierarchy tests (FEAT0035)

    #[test]
    fn test_distribution_get_weight() {
        let dist =
            StrategyDistribution::new(LearningCategory::CodePattern, ContextType::Interactive);

        let weight = dist.get_weight(StrategyVariant::MainContext);
        assert!(weight.is_some());
        // Default prior is 3.0/10.0 = ~0.3
        assert!((weight.unwrap().value - 0.3).abs() < 0.01);

        let weight = dist.get_weight(StrategyVariant::Deferred);
        assert!(weight.is_some());
        // Default prior is 4.0/10.0 = ~0.4
        assert!((weight.unwrap().value - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_distribution_update_weight() {
        let mut dist =
            StrategyDistribution::new(LearningCategory::CodePattern, ContextType::Interactive);

        let before = dist.get_weight(StrategyVariant::MainContext).unwrap().value;

        // Positive outcome should increase the weight
        dist.update_weight(StrategyVariant::MainContext, 1.0, 0.8);

        let after = dist.get_weight(StrategyVariant::MainContext).unwrap().value;
        assert!(after > before);
    }

    #[test]
    fn test_override_is_specialized() {
        let mut override_ =
            LearningStrategyOverride::new(Uuid::now_v7(), LearningCategory::CodePattern);

        assert!(!override_.is_specialized());

        let dist =
            StrategyDistribution::new(LearningCategory::CodePattern, ContextType::Interactive);
        override_.specialize_from(&dist);

        assert!(override_.is_specialized());
    }

    #[test]
    fn test_get_effective_weights_uses_override_when_specialized() {
        let dist =
            StrategyDistribution::new(LearningCategory::CodePattern, ContextType::Interactive);
        let mut override_ =
            LearningStrategyOverride::new(Uuid::now_v7(), LearningCategory::CodePattern);

        // Before specialization, should use category distribution
        let weights = get_effective_weights(Some(&override_), &dist);
        assert!(std::ptr::eq(weights, &dist.strategy_weights));

        // Specialize and modify the override weights
        override_.specialize_from(&dist);

        // After specialization, should use override weights
        let weights = get_effective_weights(Some(&override_), &dist);
        assert!(std::ptr::eq(
            weights,
            override_.specialized_weights.as_ref().unwrap()
        ));
    }

    #[test]
    fn test_get_effective_weights_fallback_to_category() {
        let dist =
            StrategyDistribution::new(LearningCategory::CodePattern, ContextType::Interactive);

        // No override at all
        let weights = get_effective_weights(None, &dist);
        assert!(std::ptr::eq(weights, &dist.strategy_weights));
    }

    #[test]
    fn test_distribution_serialization_roundtrip() {
        let dist =
            StrategyDistribution::new(LearningCategory::ErrorRecovery, ContextType::CodeReview);

        let json = serde_json::to_string(&dist).unwrap();
        let parsed: StrategyDistribution = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.category, dist.category);
        assert_eq!(parsed.context_type, dist.context_type);
        assert_eq!(parsed.session_count, dist.session_count);
    }

    #[test]
    fn test_override_serialization_roundtrip() {
        let mut override_ =
            LearningStrategyOverride::new(Uuid::now_v7(), LearningCategory::Preference);
        let dist = StrategyDistribution::new(LearningCategory::Preference, ContextType::Planning);
        override_.specialize_from(&dist);

        let json = serde_json::to_string(&override_).unwrap();
        let parsed: LearningStrategyOverride = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.learning_id, override_.learning_id);
        assert_eq!(parsed.base_category, override_.base_category);
        assert!(parsed.is_specialized());
    }
}
