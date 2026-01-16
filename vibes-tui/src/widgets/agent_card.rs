//! Agent card widget for swarm visualization.
//!
//! Displays an individual agent's progress within a swarm,
//! including task description, progress bar, and status indicator.

use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::Theme;

/// Status of an agent's task within a swarm card.
///
/// This is distinct from `control_bar::AgentStatus` which tracks
/// the agent's control state (paused, waiting for permission, etc).
/// This enum focuses on task progress display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)] // Variants used in tests and future swarm data integration
pub enum AgentCardStatus {
    /// Agent is actively working on its task.
    #[default]
    Running,
    /// Agent has completed its task successfully.
    Completed,
    /// Agent has encountered an error.
    Failed,
    /// Agent is waiting to start or for dependencies.
    Waiting,
}

/// Widget displaying an individual agent's progress and status.
#[derive(Debug, Clone)]
pub struct AgentCard {
    #[allow(dead_code)] // Used for identification in future swarm data integration
    pub agent_id: String,
    pub name: String,
    pub task: String,
    pub progress: f32,
    pub status: AgentCardStatus,
    pub selected: bool,
}

impl AgentCardStatus {
    /// Returns the status indicator character.
    ///
    /// - Completed: checkmark (✓)
    /// - Failed: cross (✗)
    /// - Running/Waiting: empty string
    pub fn indicator(&self) -> &'static str {
        match self {
            Self::Completed => "✓",
            Self::Failed => "✗",
            Self::Running | Self::Waiting => "",
        }
    }
}

impl AgentCard {
    /// Formats progress as a percentage string (e.g., "45%").
    pub fn format_progress_percent(&self) -> String {
        let percent = (self.progress.clamp(0.0, 1.0) * 100.0) as u8;
        format!("{}%", percent)
    }

    /// Renders an ASCII progress bar of the given width.
    ///
    /// Uses `█` for filled portions and `░` for empty portions.
    pub fn render_progress_bar(&self, width: usize) -> String {
        let clamped = self.progress.clamp(0.0, 1.0);
        let filled = (clamped * width as f32).round() as usize;
        let empty = width.saturating_sub(filled);

        "█".repeat(filled) + &"░".repeat(empty)
    }

    /// Returns the color for the current status based on the theme.
    fn status_color(&self, theme: &Theme) -> ratatui::style::Color {
        match self.status {
            AgentCardStatus::Running => theme.running,
            AgentCardStatus::Completed => theme.completed,
            AgentCardStatus::Failed => theme.error,
            AgentCardStatus::Waiting => theme.border, // dim
        }
    }

    /// Renders the agent card into the given area.
    ///
    /// Layout:
    /// ```text
    /// ┌──────────────────────────────────────┐
    /// │ agent-name ──── Task description (45%)│
    /// │ ████████░░░░░░░░░░░░░░░░░░░░░░       │
    /// └──────────────────────────────────────┘
    /// ```
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Build the block with optional selection background
        let mut block_style = Style::default().fg(theme.border);
        if self.selected {
            block_style = block_style.bg(theme.selection);
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(block_style);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Skip rendering content if no space
        if inner.height < 2 || inner.width < 10 {
            return;
        }

        // Calculate progress bar width (inner width minus some padding)
        let progress_bar_width = inner.width.saturating_sub(2) as usize;

        // Row 1: name, task, percentage, status indicator
        let status_indicator = self.status.indicator();
        let status_color = self.status_color(theme);

        let mut row1_spans = vec![
            Span::styled(&self.name, Style::default().fg(theme.accent)),
            Span::styled(" ─── ", Style::default().fg(theme.border)),
            Span::styled(&self.task, Style::default().fg(theme.fg)),
            Span::raw(" "),
            Span::styled(
                format!("({})", self.format_progress_percent()),
                Style::default().fg(status_color),
            ),
        ];

        // Add status indicator if present
        if !status_indicator.is_empty() {
            row1_spans.push(Span::raw(" "));
            row1_spans.push(Span::styled(
                status_indicator,
                Style::default().fg(status_color),
            ));
        }

        let row1 = Line::from(row1_spans);

        // Row 2: progress bar
        let progress_bar_str = self.render_progress_bar(progress_bar_width);
        let row2 = Line::from(Span::styled(
            progress_bar_str,
            Style::default().fg(status_color),
        ));

        // Build paragraph with both rows
        let content = Paragraph::new(vec![row1, row2]);

        // Apply selection background to content if selected
        let content = if self.selected {
            content.style(Style::default().bg(theme.selection))
        } else {
            content
        };

        frame.render_widget(content, inner);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    // === Construction tests ===

    #[test]
    fn agent_card_stores_agent_id() {
        let card = AgentCard {
            agent_id: "agent-123".into(),
            name: "Test Agent".into(),
            task: "Running tests".into(),
            progress: 0.5,
            status: AgentCardStatus::Running,
            selected: false,
        };
        assert_eq!(card.agent_id, "agent-123");
    }

    #[test]
    fn agent_card_stores_name() {
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Code Reviewer".into(),
            task: "Review PR".into(),
            progress: 0.0,
            status: AgentCardStatus::Waiting,
            selected: false,
        };
        assert_eq!(card.name, "Code Reviewer");
    }

    #[test]
    fn agent_card_stores_task() {
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Security review of authentication module".into(),
            progress: 0.75,
            status: AgentCardStatus::Running,
            selected: false,
        };
        assert_eq!(card.task, "Security review of authentication module");
    }

    #[test]
    fn agent_card_stores_progress() {
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Task".into(),
            progress: 0.45,
            status: AgentCardStatus::Running,
            selected: false,
        };
        assert!((card.progress - 0.45).abs() < f32::EPSILON);
    }

    #[test]
    fn agent_card_stores_status() {
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Task".into(),
            progress: 1.0,
            status: AgentCardStatus::Completed,
            selected: false,
        };
        assert_eq!(card.status, AgentCardStatus::Completed);
    }

    #[test]
    fn agent_card_stores_selected() {
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Task".into(),
            progress: 0.0,
            status: AgentCardStatus::Running,
            selected: true,
        };
        assert!(card.selected);
    }

    // === AgentStatus tests ===

    #[test]
    fn agent_card_status_default_is_running() {
        assert_eq!(AgentCardStatus::default(), AgentCardStatus::Running);
    }

    #[test]
    fn agent_card_status_equality() {
        assert_eq!(AgentCardStatus::Running, AgentCardStatus::Running);
        assert_ne!(AgentCardStatus::Running, AgentCardStatus::Completed);
        assert_ne!(AgentCardStatus::Failed, AgentCardStatus::Waiting);
    }

    #[test]
    fn agent_card_status_is_copy() {
        let status = AgentCardStatus::Running;
        let copied = status;
        assert_eq!(status, copied);
    }

    // === Progress percentage formatting ===

    #[test]
    fn format_progress_percent_at_zero() {
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Task".into(),
            progress: 0.0,
            status: AgentCardStatus::Running,
            selected: false,
        };
        assert_eq!(card.format_progress_percent(), "0%");
    }

    #[test]
    fn format_progress_percent_at_fifty() {
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Task".into(),
            progress: 0.5,
            status: AgentCardStatus::Running,
            selected: false,
        };
        assert_eq!(card.format_progress_percent(), "50%");
    }

    #[test]
    fn format_progress_percent_at_hundred() {
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Task".into(),
            progress: 1.0,
            status: AgentCardStatus::Completed,
            selected: false,
        };
        assert_eq!(card.format_progress_percent(), "100%");
    }

    #[test]
    fn format_progress_percent_rounds_down() {
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Task".into(),
            progress: 0.456,
            status: AgentCardStatus::Running,
            selected: false,
        };
        assert_eq!(card.format_progress_percent(), "45%");
    }

    // === Progress bar rendering ===

    #[test]
    fn render_progress_bar_at_zero() {
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Task".into(),
            progress: 0.0,
            status: AgentCardStatus::Running,
            selected: false,
        };
        assert_eq!(card.render_progress_bar(8), "░░░░░░░░");
    }

    #[test]
    fn render_progress_bar_at_fifty() {
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Task".into(),
            progress: 0.5,
            status: AgentCardStatus::Running,
            selected: false,
        };
        assert_eq!(card.render_progress_bar(8), "████░░░░");
    }

    #[test]
    fn render_progress_bar_at_hundred() {
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Task".into(),
            progress: 1.0,
            status: AgentCardStatus::Completed,
            selected: false,
        };
        assert_eq!(card.render_progress_bar(8), "████████");
    }

    #[test]
    fn render_progress_bar_clamps_over_one() {
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Task".into(),
            progress: 1.5,
            status: AgentCardStatus::Running,
            selected: false,
        };
        assert_eq!(card.render_progress_bar(8), "████████");
    }

    #[test]
    fn render_progress_bar_clamps_negative() {
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Task".into(),
            progress: -0.5,
            status: AgentCardStatus::Running,
            selected: false,
        };
        assert_eq!(card.render_progress_bar(8), "░░░░░░░░");
    }

    // === Status indicator ===

    #[test]
    fn status_indicator_completed_is_checkmark() {
        assert_eq!(AgentCardStatus::Completed.indicator(), "✓");
    }

    #[test]
    fn status_indicator_failed_is_x() {
        assert_eq!(AgentCardStatus::Failed.indicator(), "✗");
    }

    #[test]
    fn status_indicator_running_is_empty() {
        assert_eq!(AgentCardStatus::Running.indicator(), "");
    }

    #[test]
    fn status_indicator_waiting_is_empty() {
        assert_eq!(AgentCardStatus::Waiting.indicator(), "");
    }

    // === Rendering tests ===

    #[test]
    fn render_renders_without_panic() {
        let theme = crate::vibes_default();
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Test Agent".into(),
            task: "Security review".into(),
            progress: 0.45,
            status: AgentCardStatus::Running,
            selected: false,
        };

        let backend = TestBackend::new(40, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                card.render(f, area, &theme);
            })
            .unwrap();
    }

    #[test]
    fn render_displays_agent_name() {
        let theme = crate::vibes_default();
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Code Reviewer".into(),
            task: "Review PR".into(),
            progress: 0.5,
            status: AgentCardStatus::Running,
            selected: false,
        };

        let backend = TestBackend::new(40, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                card.render(f, area, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("Code Reviewer"),
            "Expected 'Code Reviewer' in: {}",
            content
        );
    }

    #[test]
    fn render_displays_task_with_percent() {
        let theme = crate::vibes_default();
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Security review".into(),
            progress: 0.45,
            status: AgentCardStatus::Running,
            selected: false,
        };

        let backend = TestBackend::new(50, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                card.render(f, area, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("Security review"),
            "Expected 'Security review' in: {}",
            content
        );
        assert!(content.contains("45%"), "Expected '45%' in: {}", content);
    }

    #[test]
    fn render_displays_progress_bar() {
        let theme = crate::vibes_default();
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Task".into(),
            progress: 0.5,
            status: AgentCardStatus::Running,
            selected: false,
        };

        let backend = TestBackend::new(40, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                card.render(f, area, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        // Should contain filled and empty progress characters
        assert!(
            content.contains('█') && content.contains('░'),
            "Expected progress bar characters in: {}",
            content
        );
    }

    #[test]
    fn render_completed_shows_checkmark() {
        let theme = crate::vibes_default();
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Done task".into(),
            progress: 1.0,
            status: AgentCardStatus::Completed,
            selected: false,
        };

        let backend = TestBackend::new(50, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                card.render(f, area, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains('✓'),
            "Expected checkmark for completed status in: {}",
            content
        );
    }

    #[test]
    fn render_failed_shows_x() {
        let theme = crate::vibes_default();
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Failed task".into(),
            progress: 0.3,
            status: AgentCardStatus::Failed,
            selected: false,
        };

        let backend = TestBackend::new(50, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                card.render(f, area, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains('✗'),
            "Expected X for failed status in: {}",
            content
        );
    }

    #[test]
    fn render_running_uses_running_color() {
        let theme = crate::vibes_default();
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Task".into(),
            progress: 0.5,
            status: AgentCardStatus::Running,
            selected: false,
        };

        let backend = TestBackend::new(40, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                card.render(f, area, &theme);
            })
            .unwrap();

        // Find a filled progress bar cell and check color
        let buffer = terminal.backend().buffer();
        let filled_cell = buffer.content().iter().find(|c| c.symbol() == "█");
        assert!(filled_cell.is_some(), "Expected filled progress cell");
        assert_eq!(
            filled_cell.unwrap().fg,
            theme.running,
            "Running status progress bar should use running color"
        );
    }

    #[test]
    fn render_failed_uses_error_color() {
        let theme = crate::vibes_default();
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Task".into(),
            progress: 0.5,
            status: AgentCardStatus::Failed,
            selected: false,
        };

        let backend = TestBackend::new(40, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                card.render(f, area, &theme);
            })
            .unwrap();

        // Find the X indicator and check color
        let buffer = terminal.backend().buffer();
        let x_cell = buffer.content().iter().find(|c| c.symbol() == "✗");
        assert!(x_cell.is_some(), "Expected X indicator for failed status");
        assert_eq!(
            x_cell.unwrap().fg,
            theme.error,
            "Failed status should use error color"
        );
    }

    #[test]
    fn render_selected_has_highlight_background() {
        let theme = crate::vibes_default();
        let card = AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Task".into(),
            progress: 0.5,
            status: AgentCardStatus::Running,
            selected: true,
        };

        let backend = TestBackend::new(40, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                card.render(f, area, &theme);
            })
            .unwrap();

        // Check that at least some cells have the selection background
        let buffer = terminal.backend().buffer();
        let has_selection_bg = buffer.content().iter().any(|c| c.bg == theme.selection);
        assert!(
            has_selection_bg,
            "Selected card should have selection background color"
        );
    }
}
