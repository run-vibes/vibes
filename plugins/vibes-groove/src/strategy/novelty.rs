//! Novelty detection extension point for adaptive strategies
//!
//! This module provides the [`NoveltyHook`] trait for implementing
//! novelty detection algorithms. Novelty detection identifies:
//!
//! - New patterns in user behavior that could become learnings
//! - Strategy combinations that perform unexpectedly well
//! - Context shifts that require strategy adaptation
//!
//! # Future Implementation Ideas
//!
//! - Statistical change detection (CUSUM, ADWIN)
//! - Clustering-based anomaly detection
//! - Reinforcement learning for strategy exploration
//!
//! # Example Implementation
//!
//! ```rust,ignore
//! struct StatisticalNoveltyHook {
//!     change_detector: CusumDetector,
//!     window_size: usize,
//! }
//!
//! #[async_trait]
//! impl NoveltyHook for StatisticalNoveltyHook {
//!     async fn on_strategy_outcome(
//!         &self,
//!         _learning: &Learning,
//!         _context: &SessionContext,
//!         _strategy: &InjectionStrategy,
//!         outcome: &StrategyOutcome,
//!     ) -> Result<()> {
//!         self.change_detector.observe(outcome.value);
//!         if self.change_detector.is_change_point() {
//!             // Trigger strategy re-exploration
//!         }
//!         Ok(())
//!     }
//!
//!     async fn on_session_end(&self, _session_id: SessionId) -> Result<()> {
//!         Ok(())
//!     }
//! }
//! ```

use async_trait::async_trait;

use crate::assessment::SessionId;
use crate::error::Result;
use crate::types::Learning;

use super::learner::SessionContext;
use super::types::{InjectionStrategy, StrategyOutcome};

/// Extension point for future novelty detection
///
/// Implementations can monitor strategy outcomes for patterns that
/// indicate new learning opportunities or strategy innovations.
///
/// # Implementing NoveltyHook
///
/// Implementations should be lightweight and non-blocking. Heavy processing
/// should be offloaded to background tasks. The hook is called synchronously
/// in the strategy processing pipeline.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to allow sharing across async tasks.
#[async_trait]
pub trait NoveltyHook: Send + Sync {
    /// Called after each strategy outcome is processed
    ///
    /// Use this to observe individual outcomes and detect changes or anomalies.
    /// The implementation should be fast - defer heavy computation.
    async fn on_strategy_outcome(
        &self,
        learning: &Learning,
        context: &SessionContext,
        strategy: &InjectionStrategy,
        outcome: &StrategyOutcome,
    ) -> Result<()>;

    /// Called at session boundary
    ///
    /// Use this for session-level aggregation or cleanup. Called when
    /// a session ends or when transitioning between sessions.
    async fn on_session_end(&self, session_id: SessionId) -> Result<()>;
}

/// Default no-op novelty hook
///
/// Does nothing, used when novelty detection is disabled or not configured.
/// This is the default hook used by [`StrategyConsumer`](super::consumer::StrategyConsumer).
pub struct NoOpNoveltyHook;

#[async_trait]
impl NoveltyHook for NoOpNoveltyHook {
    async fn on_strategy_outcome(
        &self,
        _learning: &Learning,
        _context: &SessionContext,
        _strategy: &InjectionStrategy,
        _outcome: &StrategyOutcome,
    ) -> Result<()> {
        Ok(())
    }

    async fn on_session_end(&self, _session_id: SessionId) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::types::{ContextPosition, ContextType, InjectionFormat, OutcomeSource};
    use crate::types::{LearningCategory, LearningContent, LearningSource, Scope};
    use chrono::Utc;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use uuid::Uuid;

    fn test_learning() -> Learning {
        Learning {
            id: Uuid::now_v7(),
            scope: Scope::Project("test".into()),
            category: LearningCategory::CodePattern,
            content: LearningContent {
                description: "Test".into(),
                pattern: None,
                insight: "Test insight".into(),
            },
            confidence: 0.8,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            source: LearningSource::UserCreated,
        }
    }

    fn test_context() -> SessionContext {
        SessionContext::new(SessionId::from("test-session"), ContextType::Interactive)
    }

    fn test_strategy() -> InjectionStrategy {
        InjectionStrategy::MainContext {
            position: ContextPosition::Prefix,
            format: InjectionFormat::Plain,
        }
    }

    fn test_outcome() -> StrategyOutcome {
        StrategyOutcome::new(0.5, 0.8, OutcomeSource::Both)
    }

    #[tokio::test]
    async fn noop_hook_does_nothing() {
        let hook = NoOpNoveltyHook;
        let learning = test_learning();
        let context = test_context();
        let strategy = test_strategy();
        let outcome = test_outcome();

        // Should complete without error
        let result = hook
            .on_strategy_outcome(&learning, &context, &strategy, &outcome)
            .await;
        assert!(result.is_ok());

        let result = hook.on_session_end(SessionId::from("test")).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn trait_can_be_implemented() {
        // Custom implementation that counts calls
        struct CountingHook {
            outcome_count: AtomicU32,
            session_count: AtomicU32,
        }

        #[async_trait]
        impl NoveltyHook for CountingHook {
            async fn on_strategy_outcome(
                &self,
                _learning: &Learning,
                _context: &SessionContext,
                _strategy: &InjectionStrategy,
                _outcome: &StrategyOutcome,
            ) -> Result<()> {
                self.outcome_count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }

            async fn on_session_end(&self, _session_id: SessionId) -> Result<()> {
                self.session_count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let hook = CountingHook {
            outcome_count: AtomicU32::new(0),
            session_count: AtomicU32::new(0),
        };

        let learning = test_learning();
        let context = test_context();
        let strategy = test_strategy();
        let outcome = test_outcome();

        // Call multiple times
        hook.on_strategy_outcome(&learning, &context, &strategy, &outcome)
            .await
            .unwrap();
        hook.on_strategy_outcome(&learning, &context, &strategy, &outcome)
            .await
            .unwrap();
        hook.on_session_end(SessionId::from("test1")).await.unwrap();

        assert_eq!(hook.outcome_count.load(Ordering::SeqCst), 2);
        assert_eq!(hook.session_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn hook_can_be_boxed_and_shared() {
        let hook: Arc<dyn NoveltyHook> = Arc::new(NoOpNoveltyHook);

        let learning = test_learning();
        let context = test_context();
        let strategy = test_strategy();
        let outcome = test_outcome();

        // Should work through Arc<dyn NoveltyHook>
        let result = hook
            .on_strategy_outcome(&learning, &context, &strategy, &outcome)
            .await;
        assert!(result.is_ok());
    }
}
