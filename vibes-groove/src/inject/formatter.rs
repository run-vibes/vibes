//! Learning formatter for injection
//!
//! Formats extracted learnings as markdown for injection into
//! CLAUDE.md files or hook responses.

use crate::capture::{ExtractedLearning, LearningCategory};

/// A formatted section ready for injection
#[derive(Debug, Clone)]
pub struct FormattedSection {
    /// Section title
    pub title: String,
    /// Section content (markdown formatted)
    pub content: String,
    /// Priority for ordering (lower = first)
    pub priority: u8,
}

impl FormattedSection {
    /// Create a new formatted section
    pub fn new(title: String, content: String, priority: u8) -> Self {
        Self {
            title,
            content,
            priority,
        }
    }

    /// Check if section has any content
    pub fn is_empty(&self) -> bool {
        self.content.trim().is_empty()
    }
}

/// Formats learnings for injection into Claude Code
pub struct LearningFormatter {
    /// Section header level (number of # characters)
    header_level: u8,
}

impl Default for LearningFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl LearningFormatter {
    /// Create a new learning formatter
    pub fn new() -> Self {
        Self { header_level: 2 }
    }

    /// Create a formatter with custom header level
    pub fn with_header_level(level: u8) -> Self {
        Self {
            header_level: level.clamp(1, 6),
        }
    }

    /// Format all learnings into a single markdown string
    pub fn format_for_injection(&self, learnings: &[ExtractedLearning]) -> String {
        if learnings.is_empty() {
            return String::new();
        }

        let mut sections: Vec<FormattedSection> = [
            LearningCategory::Context,
            LearningCategory::Pattern,
            LearningCategory::Technique,
            LearningCategory::Preference,
        ]
        .iter()
        .filter_map(|&category| {
            let category_learnings: Vec<_> = learnings
                .iter()
                .filter(|l| l.category == category)
                .collect();

            if category_learnings.is_empty() {
                None
            } else {
                Some(self.format_section(category, &category_learnings))
            }
        })
        .collect();

        // Sort by priority
        sections.sort_by_key(|s| s.priority);

        sections
            .iter()
            .map(|s| s.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Format learnings of a specific category into a section
    pub fn format_section(
        &self,
        category: LearningCategory,
        learnings: &[&ExtractedLearning],
    ) -> FormattedSection {
        let (title, priority) = match category {
            LearningCategory::Context => ("Project Context", 0),
            LearningCategory::Pattern => ("Learned Patterns", 1),
            LearningCategory::Technique => ("Preferred Techniques", 2),
            LearningCategory::Preference => ("User Preferences", 3),
        };

        let header = "#".repeat(self.header_level as usize);
        let mut content = format!("{} {}\n", header, title);

        for learning in learnings {
            content.push_str(&format!("- {}\n", learning.content));
        }

        FormattedSection::new(title.to_string(), content, priority)
    }

    /// Get the category title
    pub fn category_title(category: LearningCategory) -> &'static str {
        match category {
            LearningCategory::Context => "Project Context",
            LearningCategory::Pattern => "Learned Patterns",
            LearningCategory::Technique => "Preferred Techniques",
            LearningCategory::Preference => "User Preferences",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_learning(content: &str, category: LearningCategory) -> ExtractedLearning {
        ExtractedLearning::new(content.to_string(), category, 0.8, None)
    }

    #[test]
    fn test_formats_empty_learnings() {
        let formatter = LearningFormatter::new();
        let result = formatter.format_for_injection(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_formats_single_learning() {
        let formatter = LearningFormatter::new();
        let learnings = vec![make_learning("Use TDD", LearningCategory::Technique)];

        let result = formatter.format_for_injection(&learnings);

        assert!(result.contains("## Preferred Techniques"));
        assert!(result.contains("- Use TDD"));
    }

    #[test]
    fn test_formats_by_category() {
        let formatter = LearningFormatter::new();
        let learnings = vec![
            make_learning("Pattern A", LearningCategory::Pattern),
            make_learning("Technique B", LearningCategory::Technique),
            make_learning("Preference C", LearningCategory::Preference),
        ];

        let result = formatter.format_for_injection(&learnings);

        assert!(result.contains("## Learned Patterns"));
        assert!(result.contains("- Pattern A"));
        assert!(result.contains("## Preferred Techniques"));
        assert!(result.contains("- Technique B"));
        assert!(result.contains("## User Preferences"));
        assert!(result.contains("- Preference C"));
    }

    #[test]
    fn test_priority_ordering() {
        let formatter = LearningFormatter::new();
        let learnings = vec![
            make_learning("Pref", LearningCategory::Preference),
            make_learning("Context", LearningCategory::Context),
            make_learning("Pattern", LearningCategory::Pattern),
        ];

        let result = formatter.format_for_injection(&learnings);

        // Context should come before Pattern which should come before Preference
        let context_pos = result.find("Project Context").unwrap();
        let pattern_pos = result.find("Learned Patterns").unwrap();
        let pref_pos = result.find("User Preferences").unwrap();

        assert!(
            context_pos < pattern_pos,
            "Context should come before Pattern"
        );
        assert!(
            pattern_pos < pref_pos,
            "Pattern should come before Preference"
        );
    }

    #[test]
    fn test_custom_header_level() {
        let formatter = LearningFormatter::with_header_level(3);
        let learnings = vec![make_learning("Test", LearningCategory::Pattern)];

        let result = formatter.format_for_injection(&learnings);

        assert!(result.contains("### Learned Patterns"));
    }

    #[test]
    fn test_header_level_clamped() {
        let formatter = LearningFormatter::with_header_level(10);
        let learnings = vec![make_learning("Test", LearningCategory::Pattern)];

        let result = formatter.format_for_injection(&learnings);

        // Should be clamped to h6
        assert!(result.contains("###### Learned Patterns"));
    }

    #[test]
    fn test_formatted_section_is_empty() {
        let empty = FormattedSection::new("Title".to_string(), "   ".to_string(), 0);
        assert!(empty.is_empty());

        let non_empty = FormattedSection::new("Title".to_string(), "Content".to_string(), 0);
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_multiple_learnings_per_category() {
        let formatter = LearningFormatter::new();
        let learnings = vec![
            make_learning("Pattern 1", LearningCategory::Pattern),
            make_learning("Pattern 2", LearningCategory::Pattern),
            make_learning("Pattern 3", LearningCategory::Pattern),
        ];

        let result = formatter.format_for_injection(&learnings);

        assert!(result.contains("- Pattern 1"));
        assert!(result.contains("- Pattern 2"));
        assert!(result.contains("- Pattern 3"));
        // Should only have one Patterns header
        assert_eq!(result.matches("Learned Patterns").count(), 1);
    }

    #[test]
    fn test_category_title() {
        assert_eq!(
            LearningFormatter::category_title(LearningCategory::Context),
            "Project Context"
        );
        assert_eq!(
            LearningFormatter::category_title(LearningCategory::Pattern),
            "Learned Patterns"
        );
        assert_eq!(
            LearningFormatter::category_title(LearningCategory::Technique),
            "Preferred Techniques"
        );
        assert_eq!(
            LearningFormatter::category_title(LearningCategory::Preference),
            "User Preferences"
        );
    }
}
