//! vibes-groove - Continual learning storage
//!
//! This crate provides CozoDB-based storage for the groove continual learning system.
//! It implements three-tier storage (user/project/enterprise) with learning storage,
//! adaptive parameters, and semantic search capabilities.

// Module declarations - commented out until implemented
pub mod assessment;
pub mod attribution;
pub mod capture;
pub mod config;
pub mod dashboard;
pub mod error;
pub mod export;
pub mod extraction;
pub mod inject;
pub mod openworld;
pub mod paths;
pub mod plugin;
pub mod security;
pub mod storage;
pub mod store;
pub mod strategy;
pub mod types;

// Re-exports - commented out until modules are implemented
pub use config::{
    DeduplicationConfig, EnterpriseConfig, GrooveConfig, OpenWorldConfig, ProjectContext,
};
pub use error::{GrooveError, Result};
pub use export::{EXPORT_VERSION, GrooveExport, ImportStats, LearningExport};
pub use extraction::patterns::{CorrectionConfig, CorrectionDetector};
pub use paths::GroovePaths;
pub use storage::GrooveStorage;
pub use store::{
    CURRENT_SCHEMA_VERSION, CozoStore, INITIAL_SCHEMA, LearningStore, MIGRATIONS, Migration,
    ParamStore,
};
pub use types::*;

// Assessment re-exports
pub use assessment::{
    AssessmentContext, AssessmentProcessor, EventId, HarnessType, InjectionMethod, ProjectId,
    SessionId, UserId,
};

// Assessment API type re-exports (for HTTP/CLI consumers)
pub use assessment::{
    ActivityStatus, AssessmentHistoryResponse, AssessmentStatsResponse, AssessmentStatusResponse,
    CircuitBreakerStatus, SamplingStatus, SessionHistoryItem, SessionStats, TierDistribution,
};

// Extraction re-exports
pub use extraction::{
    ConsumerResult as ExtractionConsumerResult, ExtractionConfig, ExtractionConsumer,
    ExtractionEvent, ExtractionMethod, ExtractionResult, ExtractionSource, LearningCandidate,
    PatternType, StartConsumerError as ExtractionStartError, TranscriptFetcher,
    extraction_consumer_loop, start_extraction_consumer,
};

// Security re-exports
pub use security::{
    ContentHash, ContentScanner, CreationEvent, CustodyEvent, CustodyEventType, DlpScanner,
    InjectionDetector, NoOpDlpScanner, NoOpInjectionDetector, Operation, OrgRole, Permissions,
    Provenance, QuarantineReason, QuarantineStatus, ReviewOutcome, ScanFinding, ScanResult,
    SecurityError, SecurityResult, Severity, TrustContext, TrustLevel, TrustSource, Verification,
    VerifiedBy,
};

// Plugin re-export
pub use plugin::GroovePlugin;

// Attribution re-exports
pub use attribution::{
    ATTRIBUTION_SCHEMA, AblationConfig, AblationExperiment, AblationManager, AblationResult,
    AblationStrategy, ActivationConfig, ActivationDetector, ActivationResult, ActivationSignal,
    AggregationConfig, AttributionConfig, AttributionConsumer, AttributionConsumerLoopResult,
    AttributionConsumerResult, AttributionRecord, AttributionStartConsumerError, AttributionStore,
    AttributionTranscriptFetcher, ConservativeAblation, CozoAttributionStore, DeprecationEvent,
    ExponentialDecayCorrelator, HybridActivationDetector, LearningLoader, LearningStatus,
    LearningValue, LightweightEventFetcher, SessionOutcome, TemporalConfig, TemporalCorrelator,
    TemporalResult, ValueAggregator, attribution_consumer_loop, start_attribution_consumer,
};

// Strategy re-exports
pub use strategy::{
    CallbackMethod, ContextPosition, ContextType, CozoStrategyStore, DeferralTrigger,
    DistributionUpdater, InjectionFormat, InjectionStrategy, LearningStrategyOverride,
    NoOpNoveltyHook, NoveltyHook, OutcomeRouter, OutcomeRouterConfig, OutcomeSource,
    STRATEGY_SCHEMA, SessionContext, SessionContextProvider, StartStrategyConsumerError,
    StrategyConsumer, StrategyConsumerConfig, StrategyConsumerLoopResult, StrategyConsumerResult,
    StrategyDistribution, StrategyEvent, StrategyInput, StrategyLearner, StrategyLearnerConfig,
    StrategyLearningLoader, StrategyOutcome, StrategyParams, StrategyStore, StrategyVariant,
    SubagentType, UpdaterConfig, UsedStrategyProvider, get_effective_weights,
    start_strategy_consumer, strategy_consumer_loop,
};

// Dashboard re-exports
pub use dashboard::{
    AblationCoverage, ActivityEntry, ActivityType, AdaptiveParamStatus, AttributionData,
    AttributionEntry, AttributionSummary, CategoryDistribution, ComponentHealth, ContributorBrief,
    DashboardData, DashboardHandler, DashboardMessage, DashboardRequest, DashboardTopic,
    HealthData, HealthSummary, LearningBrief, LearningDetailData, LearningOverrideEntry,
    LearningSummary, LearningsData, LearningsFilter, OverviewData, Period, SessionContribution,
    SessionTimelineData, SessionTimelineEntry, StrategyDistributionsData, StrategyOverridesData,
    StrategyWeight, SystemStatus, TrendDirection, TrendSummary,
};

// Open-world adaptation re-exports
pub use openworld::{
    AnomalyCluster, CapabilityGap, CapabilityGapDetector, ClusterId, DbscanConfig, DbscanResult,
    DistanceMetric, FailureId, FailureRecord, FailureType, GapCategory, GapId, GapSeverity,
    GapStatus, GapsConfig, GraduatedResponse, HookStats, NoOpOpenWorldStore, NoveltyConfig,
    NoveltyContext, NoveltyDetector, NoveltyResult, OPENWORLD_STREAM, OpenWorldEvent,
    OpenWorldHook, OpenWorldHookConfig, OpenWorldProducer, OpenWorldProducerConfig, OpenWorldStore,
    PatternFingerprint, ProducerStats, ResponseAction, ResponseConfig, ResponseStage,
    SolutionAction, SolutionGenerator, SolutionSource, SolutionsConfig, StrategyChange,
    SuggestedSolution, topics,
};
