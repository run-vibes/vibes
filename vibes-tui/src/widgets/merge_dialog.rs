//! Merge dialog widget for swarm result aggregation.
//!
//! Displays a confirmation dialog for merging results from completed agents,
//! showing which agents are ready and the selected merge strategy.

use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::Theme;

/// Strategy for merging agent results.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum MergeStrategy {
    /// Concatenate all agent results in order.
    #[default]
    Concatenate,
    /// Ask orchestrator to summarize results.
    Summarize,
    /// User-provided custom merge prompt.
    Custom(String),
}

impl MergeStrategy {
    /// Returns display name for the strategy.
    pub fn display_name(&self) -> &str {
        match self {
            Self::Concatenate => "Concatenate",
            Self::Summarize => "Summarize",
            Self::Custom(_) => "Custom",
        }
    }
}

/// Information about a completed agent for the merge dialog.
#[derive(Debug, Clone)]
pub struct CompletedAgent {
    /// Agent identifier.
    pub agent_id: String,
    /// Human-readable agent name.
    pub name: String,
    /// Summary of the agent's task.
    pub task_summary: String,
}

/// Modal dialog for confirming result merge from swarm agents.
#[derive(Debug, Clone, Default)]
pub struct MergeDialog {
    /// Whether the dialog is visible.
    visible: bool,
    /// Completed agents ready to merge.
    completed_agents: Vec<CompletedAgent>,
    /// Agents still in progress (for warning display).
    incomplete_count: usize,
    /// Selected merge strategy.
    strategy: MergeStrategy,
}

impl MergeDialog {
    /// Creates a new merge dialog.
    pub fn new() -> Self {
        Self::default()
    }

    /// Shows the dialog with completed agents.
    pub fn show(&mut self, completed: Vec<CompletedAgent>, incomplete_count: usize) {
        self.completed_agents = completed;
        self.incomplete_count = incomplete_count;
        self.strategy = MergeStrategy::default();
        self.visible = true;
    }

    /// Hides the dialog.
    pub fn hide(&mut self) {
        self.visible = false;
        self.completed_agents.clear();
        self.incomplete_count = 0;
    }

    /// Returns true if the dialog is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Returns the completed agents.
    pub fn completed_agents(&self) -> &[CompletedAgent] {
        &self.completed_agents
    }

    /// Returns the count of incomplete agents.
    pub fn incomplete_count(&self) -> usize {
        self.incomplete_count
    }

    /// Returns the current merge strategy.
    pub fn strategy(&self) -> &MergeStrategy {
        &self.strategy
    }

    /// Sets the merge strategy.
    pub fn set_strategy(&mut self, strategy: MergeStrategy) {
        self.strategy = strategy;
    }

    /// Returns true if there are agents to merge.
    pub fn can_merge(&self) -> bool {
        !self.completed_agents.is_empty()
    }

    /// Returns true if partial merge warning should be shown.
    pub fn has_incomplete(&self) -> bool {
        self.incomplete_count > 0
    }

    /// Renders the dialog centered on the screen.
    pub fn render(&self, frame: &mut Frame, theme: &Theme) {
        if !self.visible {
            return;
        }

        let area = frame.area();
        // Calculate dialog size based on content
        // Lines: header + empty + agents + empty + strategy + (warning?) + empty + controls + borders
        let agent_lines = self.completed_agents.len().min(5);
        let warning_line = if self.has_incomplete() { 1 } else { 0 };
        let content_height = 8 + agent_lines + warning_line;
        let dialog_height = content_height as u16;
        let dialog_width = 55u16.min(area.width.saturating_sub(4));

        let x = (area.width.saturating_sub(dialog_width)) / 2;
        let y = (area.height.saturating_sub(dialog_height)) / 2;
        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        // Clear background
        frame.render_widget(Clear, dialog_area);

        // Dialog block
        let title = " Merge Results ";
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Build content lines
        let mut lines = Vec::new();

        // Header
        let header_text = format!(
            "Ready to merge results from {} agent{}:",
            self.completed_agents.len(),
            if self.completed_agents.len() == 1 {
                ""
            } else {
                "s"
            }
        );
        lines.push(Line::from(Span::styled(
            header_text,
            Style::default().fg(theme.fg),
        )));
        lines.push(Line::default());

        // Agent list (max 5)
        for agent in self.completed_agents.iter().take(5) {
            let agent_line = format!("  ✓ {}: {}", agent.name, agent.task_summary);
            lines.push(Line::from(Span::styled(
                agent_line,
                Style::default().fg(theme.success),
            )));
        }

        if self.completed_agents.len() > 5 {
            let more = format!("  ... and {} more", self.completed_agents.len() - 5);
            lines.push(Line::from(Span::styled(
                more,
                Style::default().fg(theme.fg),
            )));
        }

        lines.push(Line::default());

        // Strategy
        let strategy_line = format!("Merge strategy: {}", self.strategy.display_name());
        lines.push(Line::from(Span::styled(
            strategy_line,
            Style::default().fg(theme.fg),
        )));

        // Incomplete warning
        if self.has_incomplete() {
            lines.push(Line::from(Span::styled(
                format!(
                    "⚠ {} agent{} still running",
                    self.incomplete_count,
                    if self.incomplete_count == 1 { "" } else { "s" }
                ),
                Style::default().fg(theme.warning),
            )));
        }

        // Controls
        lines.push(Line::default());
        lines.push(Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(theme.accent)),
            Span::styled(" Confirm    ", Style::default().fg(theme.fg)),
            Span::styled("[Esc]", Style::default().fg(theme.accent)),
            Span::styled(" Cancel", Style::default().fg(theme.fg)),
        ]));

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== MergeStrategy Tests ====================

    #[test]
    fn merge_strategy_default_is_concatenate() {
        assert_eq!(MergeStrategy::default(), MergeStrategy::Concatenate);
    }

    #[test]
    fn merge_strategy_concatenate_display_name() {
        assert_eq!(MergeStrategy::Concatenate.display_name(), "Concatenate");
    }

    #[test]
    fn merge_strategy_summarize_display_name() {
        assert_eq!(MergeStrategy::Summarize.display_name(), "Summarize");
    }

    #[test]
    fn merge_strategy_custom_display_name() {
        let strategy = MergeStrategy::Custom("Extract key points".into());
        assert_eq!(strategy.display_name(), "Custom");
    }

    #[test]
    fn merge_strategy_equality() {
        assert_eq!(MergeStrategy::Concatenate, MergeStrategy::Concatenate);
        assert_ne!(MergeStrategy::Concatenate, MergeStrategy::Summarize);
    }

    #[test]
    fn merge_strategy_custom_stores_prompt() {
        if let MergeStrategy::Custom(prompt) = MergeStrategy::Custom("my prompt".into()) {
            assert_eq!(prompt, "my prompt");
        } else {
            panic!("Expected Custom variant");
        }
    }

    // ==================== CompletedAgent Tests ====================

    #[test]
    fn completed_agent_stores_agent_id() {
        let agent = CompletedAgent {
            agent_id: "agent-123".into(),
            name: "Security".into(),
            task_summary: "Review auth".into(),
        };
        assert_eq!(agent.agent_id, "agent-123");
    }

    #[test]
    fn completed_agent_stores_name() {
        let agent = CompletedAgent {
            agent_id: "agent-1".into(),
            name: "Performance".into(),
            task_summary: "Profile DB".into(),
        };
        assert_eq!(agent.name, "Performance");
    }

    #[test]
    fn completed_agent_stores_task_summary() {
        let agent = CompletedAgent {
            agent_id: "agent-1".into(),
            name: "Style".into(),
            task_summary: "Check formatting".into(),
        };
        assert_eq!(agent.task_summary, "Check formatting");
    }

    // ==================== MergeDialog State Tests ====================

    #[test]
    fn merge_dialog_new_is_hidden() {
        let dialog = MergeDialog::new();
        assert!(!dialog.is_visible());
    }

    #[test]
    fn merge_dialog_default_is_hidden() {
        let dialog = MergeDialog::default();
        assert!(!dialog.is_visible());
    }

    #[test]
    fn merge_dialog_show_makes_visible() {
        let mut dialog = MergeDialog::new();
        dialog.show(vec![], 0);
        assert!(dialog.is_visible());
    }

    #[test]
    fn merge_dialog_show_stores_agents() {
        let mut dialog = MergeDialog::new();
        let agents = vec![
            CompletedAgent {
                agent_id: "a1".into(),
                name: "Agent 1".into(),
                task_summary: "Task 1".into(),
            },
            CompletedAgent {
                agent_id: "a2".into(),
                name: "Agent 2".into(),
                task_summary: "Task 2".into(),
            },
        ];
        dialog.show(agents, 0);
        assert_eq!(dialog.completed_agents().len(), 2);
    }

    #[test]
    fn merge_dialog_show_stores_incomplete_count() {
        let mut dialog = MergeDialog::new();
        dialog.show(vec![], 3);
        assert_eq!(dialog.incomplete_count(), 3);
    }

    #[test]
    fn merge_dialog_show_resets_strategy_to_default() {
        let mut dialog = MergeDialog::new();
        dialog.set_strategy(MergeStrategy::Summarize);
        dialog.show(vec![], 0);
        assert_eq!(dialog.strategy(), &MergeStrategy::Concatenate);
    }

    #[test]
    fn merge_dialog_hide_makes_invisible() {
        let mut dialog = MergeDialog::new();
        dialog.show(vec![], 0);
        dialog.hide();
        assert!(!dialog.is_visible());
    }

    #[test]
    fn merge_dialog_hide_clears_agents() {
        let mut dialog = MergeDialog::new();
        dialog.show(
            vec![CompletedAgent {
                agent_id: "a1".into(),
                name: "Agent".into(),
                task_summary: "Task".into(),
            }],
            0,
        );
        dialog.hide();
        assert!(dialog.completed_agents().is_empty());
    }

    #[test]
    fn merge_dialog_hide_clears_incomplete_count() {
        let mut dialog = MergeDialog::new();
        dialog.show(vec![], 5);
        dialog.hide();
        assert_eq!(dialog.incomplete_count(), 0);
    }

    #[test]
    fn merge_dialog_set_strategy() {
        let mut dialog = MergeDialog::new();
        dialog.set_strategy(MergeStrategy::Summarize);
        assert_eq!(dialog.strategy(), &MergeStrategy::Summarize);
    }

    #[test]
    fn merge_dialog_can_merge_true_with_agents() {
        let mut dialog = MergeDialog::new();
        dialog.show(
            vec![CompletedAgent {
                agent_id: "a1".into(),
                name: "Agent".into(),
                task_summary: "Task".into(),
            }],
            0,
        );
        assert!(dialog.can_merge());
    }

    #[test]
    fn merge_dialog_can_merge_false_without_agents() {
        let mut dialog = MergeDialog::new();
        dialog.show(vec![], 0);
        assert!(!dialog.can_merge());
    }

    #[test]
    fn merge_dialog_has_incomplete_true_when_count_positive() {
        let mut dialog = MergeDialog::new();
        dialog.show(vec![], 2);
        assert!(dialog.has_incomplete());
    }

    #[test]
    fn merge_dialog_has_incomplete_false_when_count_zero() {
        let mut dialog = MergeDialog::new();
        dialog.show(vec![], 0);
        assert!(!dialog.has_incomplete());
    }

    // ==================== Rendering Tests ====================

    #[test]
    fn merge_dialog_does_not_render_when_hidden() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let dialog = MergeDialog::new();

        let backend = TestBackend::new(60, 15);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                dialog.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            !content.contains("Merge Results"),
            "Hidden dialog should not render"
        );
    }

    #[test]
    fn merge_dialog_renders_title() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let mut dialog = MergeDialog::new();
        dialog.show(
            vec![CompletedAgent {
                agent_id: "a1".into(),
                name: "Security".into(),
                task_summary: "Review".into(),
            }],
            0,
        );

        let backend = TestBackend::new(60, 15);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                dialog.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("Merge Results"),
            "Expected 'Merge Results' in: {}",
            content
        );
    }

    #[test]
    fn merge_dialog_renders_agent_count() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let mut dialog = MergeDialog::new();
        dialog.show(
            vec![
                CompletedAgent {
                    agent_id: "a1".into(),
                    name: "A".into(),
                    task_summary: "T1".into(),
                },
                CompletedAgent {
                    agent_id: "a2".into(),
                    name: "B".into(),
                    task_summary: "T2".into(),
                },
                CompletedAgent {
                    agent_id: "a3".into(),
                    name: "C".into(),
                    task_summary: "T3".into(),
                },
            ],
            0,
        );

        let backend = TestBackend::new(60, 15);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                dialog.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("3 agents"),
            "Expected '3 agents' in: {}",
            content
        );
    }

    #[test]
    fn merge_dialog_renders_single_agent_grammar() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let mut dialog = MergeDialog::new();
        dialog.show(
            vec![CompletedAgent {
                agent_id: "a1".into(),
                name: "Solo".into(),
                task_summary: "Task".into(),
            }],
            0,
        );

        let backend = TestBackend::new(60, 15);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                dialog.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("1 agent:"),
            "Expected singular '1 agent:' in: {}",
            content
        );
    }

    #[test]
    fn merge_dialog_renders_agent_names() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let mut dialog = MergeDialog::new();
        dialog.show(
            vec![CompletedAgent {
                agent_id: "a1".into(),
                name: "Security".into(),
                task_summary: "Review auth".into(),
            }],
            0,
        );

        let backend = TestBackend::new(60, 15);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                dialog.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("Security"),
            "Expected 'Security' in: {}",
            content
        );
    }

    #[test]
    fn merge_dialog_renders_checkmarks() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let mut dialog = MergeDialog::new();
        dialog.show(
            vec![CompletedAgent {
                agent_id: "a1".into(),
                name: "Agent".into(),
                task_summary: "Task".into(),
            }],
            0,
        );

        let backend = TestBackend::new(60, 15);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                dialog.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(content.contains('✓'), "Expected checkmark in: {}", content);
    }

    #[test]
    fn merge_dialog_renders_strategy() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let mut dialog = MergeDialog::new();
        dialog.show(
            vec![CompletedAgent {
                agent_id: "a1".into(),
                name: "Agent".into(),
                task_summary: "Task".into(),
            }],
            0,
        );

        let backend = TestBackend::new(60, 15);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                dialog.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("Concatenate"),
            "Expected default strategy 'Concatenate' in: {}",
            content
        );
    }

    #[test]
    fn merge_dialog_renders_controls() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let mut dialog = MergeDialog::new();
        dialog.show(
            vec![CompletedAgent {
                agent_id: "a1".into(),
                name: "Agent".into(),
                task_summary: "Task".into(),
            }],
            0,
        );

        let backend = TestBackend::new(60, 15);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                dialog.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("[Enter]"),
            "Expected '[Enter]' in: {}",
            content
        );
        assert!(
            content.contains("[Esc]"),
            "Expected '[Esc]' in: {}",
            content
        );
    }

    #[test]
    fn merge_dialog_renders_incomplete_warning() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let mut dialog = MergeDialog::new();
        dialog.show(
            vec![CompletedAgent {
                agent_id: "a1".into(),
                name: "Done".into(),
                task_summary: "Finished".into(),
            }],
            2,
        );

        let backend = TestBackend::new(60, 15);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                dialog.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("2 agents still running"),
            "Expected incomplete warning in: {}",
            content
        );
    }

    #[test]
    fn merge_dialog_renders_single_incomplete_grammar() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let mut dialog = MergeDialog::new();
        dialog.show(
            vec![CompletedAgent {
                agent_id: "a1".into(),
                name: "Done".into(),
                task_summary: "Finished".into(),
            }],
            1,
        );

        let backend = TestBackend::new(60, 15);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                dialog.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("1 agent still running"),
            "Expected singular incomplete warning in: {}",
            content
        );
    }

    #[test]
    fn merge_dialog_no_warning_when_all_complete() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let mut dialog = MergeDialog::new();
        dialog.show(
            vec![CompletedAgent {
                agent_id: "a1".into(),
                name: "Done".into(),
                task_summary: "Finished".into(),
            }],
            0,
        );

        let backend = TestBackend::new(60, 15);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                dialog.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            !content.contains("still running"),
            "Should not show warning when all complete in: {}",
            content
        );
    }
}
