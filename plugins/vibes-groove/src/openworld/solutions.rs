//! Solution generation for capability gaps
//!
//! Generates actionable suggestions for resolving capability gaps using
//! templates and pattern analysis. Solutions are ranked by confidence.
//!
//! # Overview
//!
//! The SolutionGenerator produces solutions using two approaches:
//! - **Templates** - Predefined solutions for each gap category
//! - **Pattern Analysis** - Solutions derived from similar resolved gaps

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openworld::types::{CapabilityGap, GapCategory, SolutionSource};

    fn test_generator() -> SolutionGenerator {
        SolutionGenerator::new(SolutionsConfig::default())
    }

    fn make_gap(category: GapCategory) -> CapabilityGap {
        CapabilityGap::new(category, "test_context_pattern".to_string())
    }

    // =========================================================================
    // Config tests
    // =========================================================================

    #[test]
    fn test_config_defaults() {
        let config = SolutionsConfig::default();
        assert!(config.template_confidence > 0.0);
        assert!(config.template_confidence <= 1.0);
        assert!(config.pattern_analysis_confidence > 0.0);
        assert!(config.max_solutions > 0);
    }

    // =========================================================================
    // Template tests
    // =========================================================================

    #[test]
    fn test_has_templates_for_all_categories() {
        let generator = test_generator();

        for category in [
            GapCategory::MissingKnowledge,
            GapCategory::IncorrectPattern,
            GapCategory::ContextMismatch,
            GapCategory::ToolGap,
        ] {
            assert!(
                generator.templates.contains_key(&category),
                "Missing templates for {:?}",
                category
            );
            assert!(
                !generator.templates[&category].is_empty(),
                "Empty templates for {:?}",
                category
            );
        }
    }

    #[test]
    fn test_missing_knowledge_templates() {
        let generator = test_generator();
        let templates = &generator.templates[&GapCategory::MissingKnowledge];

        // Should have at least CreateLearning and RequestHumanInput options
        let actions: Vec<_> = templates.iter().map(|t| &t.action).collect();
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, TemplateAction::CreateLearning { .. }))
        );
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, TemplateAction::RequestHumanInput { .. }))
        );
    }

    #[test]
    fn test_incorrect_pattern_templates() {
        let generator = test_generator();
        let templates = &generator.templates[&GapCategory::IncorrectPattern];

        // Should have DisableLearning and ModifyLearning options
        let actions: Vec<_> = templates.iter().map(|t| &t.action).collect();
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, TemplateAction::DisableLearning))
        );
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, TemplateAction::ModifyLearning { .. }))
        );
    }

    // =========================================================================
    // Solution generation tests
    // =========================================================================

    #[test]
    fn test_generate_returns_solutions_for_gap() {
        let generator = test_generator();
        let gap = make_gap(GapCategory::MissingKnowledge);

        let solutions = generator.generate(&gap);

        assert!(!solutions.is_empty());
    }

    #[test]
    fn test_generate_solutions_have_correct_source() {
        let generator = test_generator();
        let gap = make_gap(GapCategory::MissingKnowledge);

        let solutions = generator.generate(&gap);

        // Template-sourced solutions should be marked as such
        assert!(
            solutions
                .iter()
                .any(|s| s.source == SolutionSource::Template)
        );
    }

    #[test]
    fn test_generate_solutions_sorted_by_confidence() {
        let generator = test_generator();
        let gap = make_gap(GapCategory::MissingKnowledge);

        let solutions = generator.generate(&gap);

        // Should be sorted descending by confidence
        for i in 1..solutions.len() {
            assert!(
                solutions[i - 1].confidence >= solutions[i].confidence,
                "Solutions not sorted by confidence: {} < {}",
                solutions[i - 1].confidence,
                solutions[i].confidence
            );
        }
    }

    #[test]
    fn test_generate_respects_max_solutions() {
        let config = SolutionsConfig {
            max_solutions: 2,
            ..Default::default()
        };
        let generator = SolutionGenerator::new(config);
        let gap = make_gap(GapCategory::MissingKnowledge);

        let solutions = generator.generate(&gap);

        assert!(solutions.len() <= 2);
    }

    // =========================================================================
    // Action specialization tests
    // =========================================================================

    #[test]
    fn test_specialize_action_replaces_context_pattern() {
        let generator = test_generator();
        let gap = make_gap(GapCategory::MissingKnowledge);

        let template_action = TemplateAction::CreateLearning {
            content_template: "{context_pattern}".to_string(),
        };

        let specialized = generator.specialize_action(&template_action, &gap);

        if let SolutionAction::CreateLearning { content, .. } = specialized {
            assert!(
                content.contains("test_context_pattern"),
                "Context pattern not substituted: {}",
                content
            );
        } else {
            panic!("Expected CreateLearning action");
        }
    }

    #[test]
    fn test_specialize_action_replaces_failure_count() {
        let generator = test_generator();
        let mut gap = make_gap(GapCategory::ContextMismatch);
        gap.failure_count = 42;

        let template_action = TemplateAction::RequestHumanInput {
            question_template: "Pattern failed {failure_count} times".to_string(),
        };

        let specialized = generator.specialize_action(&template_action, &gap);

        if let SolutionAction::RequestHumanInput { question } = specialized {
            assert!(
                question.contains("42"),
                "Failure count not substituted: {}",
                question
            );
        } else {
            panic!("Expected RequestHumanInput action");
        }
    }

    // =========================================================================
    // Prerequisite tests
    // =========================================================================

    #[test]
    fn test_prerequisites_met_with_empty_prerequisites() {
        let generator = test_generator();
        let gap = make_gap(GapCategory::MissingKnowledge);
        let template = SolutionTemplate {
            action: TemplateAction::RequestHumanInput {
                question_template: "test".to_string(),
            },
            description: "test".to_string(),
            prerequisites: vec![],
            base_confidence: 0.7,
        };

        assert!(generator.prerequisites_met(&template, &gap));
    }

    #[test]
    fn test_prerequisites_met_is_permissive_by_default() {
        let generator = test_generator();
        let gap = make_gap(GapCategory::IncorrectPattern);

        let template_with_prereq = SolutionTemplate {
            action: TemplateAction::DisableLearning,
            description: "test".to_string(),
            prerequisites: vec!["identified_learning".to_string()],
            base_confidence: 0.8,
        };

        // Current implementation is permissive - prerequisites are logged but don't block
        // This allows solutions to be suggested even when prerequisites aren't fully met,
        // which is appropriate for a suggestion system where users make final decisions
        let result = generator.prerequisites_met(&template_with_prereq, &gap);
        assert!(result, "Prerequisites should be permissive by default");
    }

    // =========================================================================
    // PatternAnalyzer tests
    // =========================================================================

    #[test]
    fn test_pattern_analyzer_returns_empty_for_no_similar_gaps() {
        let analyzer = PatternAnalyzer::new();
        let gap = make_gap(GapCategory::MissingKnowledge);

        let solutions = analyzer.find_solutions_from_similar_contexts(&gap);

        // Without any stored resolved gaps, should return empty
        assert!(solutions.is_empty());
    }

    // =========================================================================
    // Integration tests
    // =========================================================================

    #[test]
    fn test_generate_all_categories_produce_solutions() {
        let generator = test_generator();

        for category in [
            GapCategory::MissingKnowledge,
            GapCategory::IncorrectPattern,
            GapCategory::ContextMismatch,
            GapCategory::ToolGap,
        ] {
            let gap = make_gap(category);
            let solutions = generator.generate(&gap);

            assert!(!solutions.is_empty(), "No solutions for {:?}", category);
        }
    }
}

// =============================================================================
// Implementation
// =============================================================================

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use super::types::{CapabilityGap, GapCategory, SolutionAction, SolutionSource, SuggestedSolution};
use crate::types::LearningCategory;

/// Configuration for solution generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionsConfig {
    /// Base confidence for template-sourced solutions
    pub template_confidence: f64,
    /// Base confidence for pattern analysis solutions
    pub pattern_analysis_confidence: f64,
    /// Maximum number of solutions to return
    pub max_solutions: usize,
}

impl Default for SolutionsConfig {
    fn default() -> Self {
        Self {
            template_confidence: 0.7,
            pattern_analysis_confidence: 0.6,
            max_solutions: 5,
        }
    }
}

/// Template action types (before specialization)
#[derive(Debug, Clone)]
pub enum TemplateAction {
    /// Create a new learning with template content
    CreateLearning { content_template: String },
    /// Modify an existing learning
    ModifyLearning { change_template: String },
    /// Disable a problematic learning
    DisableLearning,
    /// Adjust strategy parameters
    AdjustStrategy { exploration_delta: f64 },
    /// Request human input with template question
    RequestHumanInput { question_template: String },
}

/// A solution template for a gap category
#[derive(Debug, Clone)]
pub struct SolutionTemplate {
    /// The template action to take
    pub action: TemplateAction,
    /// Description of this solution (used for user-facing explanations)
    #[allow(dead_code)]
    pub description: String,
    /// Prerequisites that must be met
    pub prerequisites: Vec<String>,
    /// Base confidence for this template
    pub base_confidence: f64,
}

/// Analyzes patterns to find solutions from similar contexts
pub struct PatternAnalyzer {
    // In a full implementation, this would have:
    // store: Arc<dyn OpenWorldStore>,
    // embedder: Arc<dyn Embedder>,
}

impl PatternAnalyzer {
    /// Create a new pattern analyzer
    pub fn new() -> Self {
        Self {}
    }

    /// Find solutions from similar resolved gaps
    #[instrument(skip(self, _gap))]
    pub fn find_solutions_from_similar_contexts(
        &self,
        _gap: &CapabilityGap,
    ) -> Vec<SuggestedSolution> {
        // In a full implementation, this would:
        // 1. Find resolved gaps with similar context patterns
        // 2. Extract what solutions worked for them
        // 3. Adapt those solutions to the current gap

        // For now, return empty - pattern analysis requires a store
        debug!("Pattern analysis not yet implemented with store");
        Vec::new()
    }
}

impl Default for PatternAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Generates solutions for capability gaps
pub struct SolutionGenerator {
    /// Templates organized by gap category
    templates: HashMap<GapCategory, Vec<SolutionTemplate>>,
    /// Pattern analyzer for similar context solutions
    pattern_analyzer: PatternAnalyzer,
    /// Configuration
    config: SolutionsConfig,
}

impl SolutionGenerator {
    /// Create a new solution generator with config
    pub fn new(config: SolutionsConfig) -> Self {
        Self {
            templates: Self::default_templates(),
            pattern_analyzer: PatternAnalyzer::new(),
            config,
        }
    }

    /// Create default templates for all gap categories
    fn default_templates() -> HashMap<GapCategory, Vec<SolutionTemplate>> {
        let mut templates = HashMap::new();

        // MissingKnowledge templates
        templates.insert(
            GapCategory::MissingKnowledge,
            vec![
                SolutionTemplate {
                    action: TemplateAction::CreateLearning {
                        content_template: "Pattern for: {context_pattern}".to_string(),
                    },
                    description: "Create a new learning for this pattern".to_string(),
                    prerequisites: vec![],
                    base_confidence: 0.75,
                },
                SolutionTemplate {
                    action: TemplateAction::RequestHumanInput {
                        question_template:
                            "What should happen when encountering: {context_pattern}?".to_string(),
                    },
                    description: "Ask user for guidance on this pattern".to_string(),
                    prerequisites: vec![],
                    base_confidence: 0.65,
                },
            ],
        );

        // IncorrectPattern templates
        templates.insert(
            GapCategory::IncorrectPattern,
            vec![
                SolutionTemplate {
                    action: TemplateAction::DisableLearning,
                    description: "Disable the problematic learning".to_string(),
                    prerequisites: vec!["identified_learning".to_string()],
                    base_confidence: 0.8,
                },
                SolutionTemplate {
                    action: TemplateAction::ModifyLearning {
                        change_template: "Adjust for: {context_pattern}".to_string(),
                    },
                    description: "Modify the learning to correct the pattern".to_string(),
                    prerequisites: vec!["identified_learning".to_string()],
                    base_confidence: 0.7,
                },
                SolutionTemplate {
                    action: TemplateAction::RequestHumanInput {
                        question_template:
                            "The pattern '{context_pattern}' appears incorrect. How should it be corrected?"
                                .to_string(),
                    },
                    description: "Ask user how to correct this pattern".to_string(),
                    prerequisites: vec![],
                    base_confidence: 0.6,
                },
            ],
        );

        // ContextMismatch templates
        templates.insert(
            GapCategory::ContextMismatch,
            vec![
                SolutionTemplate {
                    action: TemplateAction::AdjustStrategy {
                        exploration_delta: 0.15,
                    },
                    description: "Increase exploration to find better context matches".to_string(),
                    prerequisites: vec![],
                    base_confidence: 0.7,
                },
                SolutionTemplate {
                    action: TemplateAction::CreateLearning {
                        content_template: "Context-specific pattern: {context_pattern}".to_string(),
                    },
                    description: "Create a context-specific learning".to_string(),
                    prerequisites: vec![],
                    base_confidence: 0.65,
                },
                SolutionTemplate {
                    action: TemplateAction::RequestHumanInput {
                        question_template:
                            "Context '{context_pattern}' doesn't match known patterns (failed {failure_count} times). What context should match?"
                                .to_string(),
                    },
                    description: "Ask user to clarify context".to_string(),
                    prerequisites: vec![],
                    base_confidence: 0.55,
                },
            ],
        );

        // ToolGap templates
        templates.insert(
            GapCategory::ToolGap,
            vec![
                SolutionTemplate {
                    action: TemplateAction::RequestHumanInput {
                        question_template:
                            "A capability gap exists for '{context_pattern}'. Is there a tool or plugin that could help?"
                                .to_string(),
                    },
                    description: "Ask user about available tools".to_string(),
                    prerequisites: vec![],
                    base_confidence: 0.7,
                },
                SolutionTemplate {
                    action: TemplateAction::CreateLearning {
                        content_template: "Workaround for missing tool: {context_pattern}"
                            .to_string(),
                    },
                    description: "Create a workaround learning".to_string(),
                    prerequisites: vec![],
                    base_confidence: 0.5,
                },
            ],
        );

        templates
    }

    /// Generate solutions for a capability gap
    #[instrument(skip(self), fields(gap_id = %gap.id, category = ?gap.category))]
    pub fn generate(&self, gap: &CapabilityGap) -> Vec<SuggestedSolution> {
        let mut solutions = Vec::new();

        // Get templates for this category
        if let Some(category_templates) = self.templates.get(&gap.category) {
            for template in category_templates {
                if self.prerequisites_met(template, gap) {
                    let action = self.specialize_action(&template.action, gap);
                    let confidence = self.calculate_confidence(template, gap);

                    solutions.push(SuggestedSolution::new(
                        action,
                        SolutionSource::Template,
                        confidence,
                    ));
                }
            }
        }

        // Find solutions from similar contexts
        let pattern_solutions = self
            .pattern_analyzer
            .find_solutions_from_similar_contexts(gap);
        solutions.extend(pattern_solutions);

        // Sort by confidence (descending)
        solutions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit to max solutions
        solutions.truncate(self.config.max_solutions);

        debug!(count = solutions.len(), "Generated solutions for gap");
        solutions
    }

    /// Check if prerequisites are met for a template
    fn prerequisites_met(&self, template: &SolutionTemplate, gap: &CapabilityGap) -> bool {
        for prereq in &template.prerequisites {
            match prereq.as_str() {
                "identified_learning" => {
                    // In a full implementation, check if gap has associated learning IDs
                    // For now, check if context pattern suggests a learning reference
                    if !gap.context_pattern.contains("learning:") {
                        // Be permissive for now - don't block solutions
                        debug!(prereq, "Prerequisite not fully met, allowing anyway");
                    }
                }
                _ => {
                    debug!(prereq, "Unknown prerequisite, skipping");
                }
            }
        }
        true
    }

    /// Specialize a template action with gap-specific details
    fn specialize_action(
        &self,
        template_action: &TemplateAction,
        gap: &CapabilityGap,
    ) -> SolutionAction {
        match template_action {
            TemplateAction::CreateLearning { content_template } => {
                let content = self.substitute_placeholders(content_template, gap);
                SolutionAction::CreateLearning {
                    content,
                    category: LearningCategory::CodePattern,
                }
            }

            TemplateAction::ModifyLearning { change_template } => {
                let change = self.substitute_placeholders(change_template, gap);
                SolutionAction::ModifyLearning {
                    id: uuid::Uuid::nil(), // Would be filled from gap's associated learnings
                    change,
                }
            }

            TemplateAction::DisableLearning => SolutionAction::DisableLearning {
                id: uuid::Uuid::nil(), // Would be filled from gap's associated learnings
            },

            TemplateAction::AdjustStrategy { exploration_delta } => {
                SolutionAction::AdjustStrategy {
                    category: LearningCategory::CodePattern,
                    change: super::types::StrategyChange::new(None, 0.0, *exploration_delta),
                }
            }

            TemplateAction::RequestHumanInput { question_template } => {
                let question = self.substitute_placeholders(question_template, gap);
                SolutionAction::RequestHumanInput { question }
            }
        }
    }

    /// Substitute placeholders in a template string
    fn substitute_placeholders(&self, template: &str, gap: &CapabilityGap) -> String {
        template
            .replace("{context_pattern}", &gap.context_pattern)
            .replace("{failure_count}", &gap.failure_count.to_string())
            .replace("{category}", gap.category.as_str())
            .replace("{severity}", gap.severity.as_str())
    }

    /// Calculate confidence for a solution based on template and gap
    fn calculate_confidence(&self, template: &SolutionTemplate, gap: &CapabilityGap) -> f64 {
        let mut confidence = template.base_confidence;

        // Adjust based on gap severity - higher severity = higher confidence in solutions
        confidence += match gap.severity {
            super::types::GapSeverity::Low => 0.0,
            super::types::GapSeverity::Medium => 0.05,
            super::types::GapSeverity::High => 0.1,
            super::types::GapSeverity::Critical => 0.15,
        };

        // Cap at 1.0
        confidence.min(1.0)
    }

    /// Get the current config
    pub fn config(&self) -> &SolutionsConfig {
        &self.config
    }
}
