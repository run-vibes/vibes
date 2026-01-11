//! Integration tests for the open-world adaptation pipeline
//!
//! Tests the full flow from configuration through hook processing to
//! response actions. Validates that the components work together correctly.

use std::sync::Arc;

use vibes_groove::assessment::SessionId;
use vibes_groove::strategy::{
    ContextType, InjectionFormat, InjectionStrategy, OutcomeSource, SessionContext, StrategyOutcome,
};
use vibes_groove::types::{Learning, LearningCategory, LearningContent, LearningSource, Scope};
use vibes_groove::{
    GapsConfig, NoveltyConfig, OpenWorldConfig, OpenWorldHook, ResponseAction, ResponseConfig,
    SolutionsConfig,
};

// =============================================================================
// Test Helpers
// =============================================================================

fn test_learning() -> Learning {
    Learning {
        id: uuid::Uuid::now_v7(),
        scope: Scope::Project("test-project".into()),
        category: LearningCategory::CodePattern,
        content: LearningContent {
            description: "Test learning for integration".into(),
            pattern: Some("test pattern".into()),
            insight: "Test insight".into(),
        },
        confidence: 0.8,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        source: LearningSource::UserCreated,
    }
}

fn test_context() -> SessionContext {
    SessionContext::new(
        SessionId::from("integration-test-session"),
        ContextType::Interactive,
    )
}

fn test_strategy() -> InjectionStrategy {
    InjectionStrategy::MainContext {
        position: vibes_groove::strategy::ContextPosition::Prefix,
        format: InjectionFormat::Plain,
    }
}

fn positive_outcome() -> StrategyOutcome {
    StrategyOutcome::new(0.8, 0.9, OutcomeSource::Attribution)
}

fn negative_outcome() -> StrategyOutcome {
    StrategyOutcome::new(-0.5, 0.8, OutcomeSource::Attribution)
}

fn low_confidence_outcome() -> StrategyOutcome {
    StrategyOutcome::new(0.3, 0.2, OutcomeSource::Direct)
}

// =============================================================================
// Configuration Tests
// =============================================================================

#[test]
fn test_openworld_config_from_default() {
    let config = OpenWorldConfig::default();

    assert!(config.enabled);
    assert!((config.novelty.initial_threshold - 0.85).abs() < f64::EPSILON);
    assert_eq!(config.gaps.min_failures_for_gap, 3);
    assert_eq!(config.response.monitor_threshold, 3);
    assert_eq!(config.solutions.max_solutions, 5);
}

#[test]
fn test_openworld_config_custom_values() {
    let config = OpenWorldConfig {
        enabled: true,
        novelty: NoveltyConfig {
            initial_threshold: 0.9,
            threshold_prior: (9.0, 1.0),
            max_pending_outliers: 100,
            min_cluster_size: 5,
        },
        gaps: GapsConfig {
            min_failures_for_gap: 5,
            negative_attribution_threshold: -0.5,
            low_confidence_threshold: 0.5,
        },
        response: ResponseConfig {
            monitor_threshold: 5,
            cluster_threshold: 15,
            auto_adjust_threshold: 30,
            exploration_adjustment: 0.15,
            max_exploration_bonus: 0.6,
        },
        solutions: SolutionsConfig {
            template_confidence: 0.8,
            pattern_analysis_confidence: 0.7,
            max_solutions: 10,
        },
    };

    let hook = OpenWorldHook::from_openworld_config(&config);

    assert_eq!(hook.config().gap_creation_threshold, 5);
    assert!((hook.config().negative_value_threshold - (-0.5)).abs() < f64::EPSILON);
    assert!((hook.config().exploration_bonus - 0.15).abs() < f64::EPSILON);
}

// =============================================================================
// Hook Integration Tests
// =============================================================================

#[test]
fn test_hook_tracks_positive_outcomes() {
    let hook = OpenWorldHook::from_openworld_config(&OpenWorldConfig::default());

    let learning = test_learning();
    let context = test_context();
    let outcome = positive_outcome();

    let action = hook.determine_response(&outcome, &context, &learning);

    // Positive outcomes should return no action
    assert!(matches!(action, ResponseAction::None));
}

#[test]
fn test_hook_tracks_negative_outcomes() {
    let hook = OpenWorldHook::from_openworld_config(&OpenWorldConfig::default());

    let learning = test_learning();
    let context = test_context();
    let outcome = negative_outcome();

    // First negative outcome - monitor only
    let action = hook.determine_response(&outcome, &context, &learning);
    assert!(matches!(action, ResponseAction::None));

    // Second negative outcome - should adjust exploration
    let action = hook.determine_response(&outcome, &context, &learning);
    assert!(matches!(action, ResponseAction::AdjustExploration(_)));
}

#[test]
fn test_hook_creates_gap_after_threshold() {
    let config = OpenWorldConfig {
        gaps: GapsConfig {
            min_failures_for_gap: 3, // Lower threshold for test
            ..Default::default()
        },
        ..Default::default()
    };
    let hook = OpenWorldHook::from_openworld_config(&config);

    let learning = test_learning();
    let context = test_context();
    let outcome = negative_outcome();

    // Process enough negative outcomes to trigger gap creation
    for _ in 0..2 {
        hook.determine_response(&outcome, &context, &learning);
    }

    // Third should create gap
    let action = hook.determine_response(&outcome, &context, &learning);
    assert!(
        matches!(action, ResponseAction::CreateGap(_)),
        "Expected CreateGap action after threshold"
    );
}

#[test]
fn test_hook_low_confidence_tracking() {
    let hook = OpenWorldHook::from_openworld_config(&OpenWorldConfig::default());

    let learning = test_learning();
    let context = test_context();
    let outcome = low_confidence_outcome();

    // First low confidence - monitor
    let action = hook.determine_response(&outcome, &context, &learning);
    assert!(matches!(action, ResponseAction::None));

    // Second - should adjust exploration
    let action = hook.determine_response(&outcome, &context, &learning);
    assert!(matches!(action, ResponseAction::AdjustExploration(_)));
}

#[test]
fn test_hook_stats_tracking() {
    let hook = OpenWorldHook::from_openworld_config(&OpenWorldConfig::default());

    let learning = test_learning();
    let context = test_context();

    // Initial stats should be zero
    let stats = hook.stats();
    assert_eq!(stats.outcomes_processed, 0);
    assert_eq!(stats.negative_outcomes, 0);

    // Process a negative outcome through determine_response
    // (Note: stats are updated in on_strategy_outcome, not determine_response)
    let _action = hook.determine_response(&negative_outcome(), &context, &learning);

    // determine_response doesn't update atomic counters, only on_strategy_outcome does
    // This test documents the current behavior
    let stats = hook.stats();
    assert_eq!(stats.outcomes_processed, 0);
}

// =============================================================================
// Async Hook Tests (using tokio runtime)
// =============================================================================

#[tokio::test]
async fn test_hook_on_strategy_outcome() {
    use vibes_groove::strategy::NoveltyHook;

    let hook = Arc::new(OpenWorldHook::from_openworld_config(
        &OpenWorldConfig::default(),
    ));

    let learning = test_learning();
    let context = test_context();
    let strategy = test_strategy();
    let outcome = negative_outcome();

    // Process outcome through async method
    hook.on_strategy_outcome(&learning, &context, &strategy, &outcome)
        .await
        .unwrap();

    // Stats should be updated
    let stats = hook.stats();
    assert_eq!(stats.outcomes_processed, 1);
    assert_eq!(stats.negative_outcomes, 1);
}

#[tokio::test]
async fn test_hook_disabled_skips_processing() {
    use vibes_groove::strategy::NoveltyHook;

    let config = OpenWorldConfig {
        enabled: false,
        ..Default::default()
    };
    let hook = Arc::new(OpenWorldHook::from_openworld_config(&config));

    let learning = test_learning();
    let context = test_context();
    let strategy = test_strategy();
    let outcome = negative_outcome();

    // Process outcome - should be skipped
    hook.on_strategy_outcome(&learning, &context, &strategy, &outcome)
        .await
        .unwrap();

    // Stats should remain zero since processing is disabled
    let stats = hook.stats();
    assert_eq!(stats.outcomes_processed, 0);
}

#[tokio::test]
async fn test_full_pipeline_negative_to_gap() {
    use vibes_groove::strategy::NoveltyHook;

    let config = OpenWorldConfig {
        gaps: GapsConfig {
            min_failures_for_gap: 3,
            ..Default::default()
        },
        ..Default::default()
    };
    let hook = Arc::new(OpenWorldHook::from_openworld_config(&config));

    let learning = test_learning();
    let context = test_context();
    let strategy = test_strategy();
    let outcome = negative_outcome();

    // Process enough negative outcomes to trigger gap creation
    for _ in 0..4 {
        hook.on_strategy_outcome(&learning, &context, &strategy, &outcome)
            .await
            .unwrap();
    }

    // Check stats reflect processing
    let stats = hook.stats();
    assert_eq!(stats.outcomes_processed, 4);
    assert_eq!(stats.negative_outcomes, 4);
    // Gap should have been created after threshold
    assert!(stats.gaps_created >= 1, "Expected at least one gap created");
}
