//! Stats summary bar widget for the dashboard.
//!
//! Displays aggregate metrics at the top of the dashboard:
//! sessions, agents, and cost.

use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::Theme;

/// Cost threshold in USD above which the cost is displayed in warning color.
pub const COST_WARNING_THRESHOLD: f64 = 10.0;

/// Widget displaying aggregate stats in a single line.
#[derive(Debug, Clone, Default)]
pub struct StatsBarWidget {
    pub session_count: u32,
    pub agent_count: u32,
    pub total_cost: f64,
}

impl StatsBarWidget {
    /// Creates a new stats bar widget with the given values.
    pub fn new(session_count: u32, agent_count: u32, total_cost: f64) -> Self {
        Self {
            session_count,
            agent_count,
            total_cost,
        }
    }

    /// Formats the cost as "$X.XX".
    pub fn format_cost(&self) -> String {
        format!("${:.2}", self.total_cost)
    }

    /// Converts the widget to a renderable Paragraph with the given theme.
    ///
    /// Layout: "Sessions: N active   Agents: N running   Cost: $X.XX"
    pub fn to_paragraph(&self, theme: &Theme) -> Paragraph<'_> {
        let cost_color = if self.total_cost > COST_WARNING_THRESHOLD {
            theme.warning
        } else {
            theme.fg
        };

        let line = Line::from(vec![
            Span::styled("Sessions: ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{} active", self.session_count),
                Style::default().fg(theme.accent),
            ),
            Span::raw("   "),
            Span::styled("Agents: ", Style::default().fg(theme.fg)),
            Span::styled(
                format!("{} running", self.agent_count),
                Style::default().fg(theme.running),
            ),
            Span::raw("   "),
            Span::styled("Cost: ", Style::default().fg(theme.fg)),
            Span::styled(self.format_cost(), Style::default().fg(cost_color)),
        ]);

        Paragraph::new(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    // Construction tests
    #[test]
    fn widget_default_has_zero_values() {
        let widget = StatsBarWidget::default();
        assert_eq!(widget.session_count, 0);
        assert_eq!(widget.agent_count, 0);
        assert_eq!(widget.total_cost, 0.0);
    }

    #[test]
    fn widget_new_stores_values() {
        let widget = StatsBarWidget::new(3, 7, 12.50);
        assert_eq!(widget.session_count, 3);
        assert_eq!(widget.agent_count, 7);
        assert_eq!(widget.total_cost, 12.50);
    }

    // Cost formatting tests
    #[test]
    fn format_cost_with_zero() {
        let widget = StatsBarWidget::new(0, 0, 0.0);
        assert_eq!(widget.format_cost(), "$0.00");
    }

    #[test]
    fn format_cost_with_cents() {
        let widget = StatsBarWidget::new(0, 0, 12.5);
        assert_eq!(widget.format_cost(), "$12.50");
    }

    #[test]
    fn format_cost_rounds_to_two_decimals() {
        let widget = StatsBarWidget::new(0, 0, 1.999);
        assert_eq!(widget.format_cost(), "$2.00");
    }

    #[test]
    fn format_cost_large_value() {
        let widget = StatsBarWidget::new(0, 0, 1234.56);
        assert_eq!(widget.format_cost(), "$1234.56");
    }

    // Rendering tests
    #[test]
    fn to_paragraph_creates_paragraph() {
        let theme = crate::vibes_default();
        let widget = StatsBarWidget::new(3, 7, 12.50);
        let _paragraph = widget.to_paragraph(&theme);
        // Should compile and not panic
    }

    #[test]
    fn renders_session_count_with_active_label() {
        let theme = crate::vibes_default();
        let widget = StatsBarWidget::new(3, 7, 12.50);

        let backend = TestBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(widget.to_paragraph(&theme), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("3 active"),
            "Expected '3 active' in: {}",
            content
        );
    }

    #[test]
    fn renders_agent_count_with_running_label() {
        let theme = crate::vibes_default();
        let widget = StatsBarWidget::new(3, 7, 12.50);

        let backend = TestBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(widget.to_paragraph(&theme), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("7 running"),
            "Expected '7 running' in: {}",
            content
        );
    }

    #[test]
    fn renders_cost_with_dollar_format() {
        let theme = crate::vibes_default();
        let widget = StatsBarWidget::new(3, 7, 12.50);

        let backend = TestBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(widget.to_paragraph(&theme), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("$12.50"),
            "Expected '$12.50' in: {}",
            content
        );
    }

    #[test]
    fn renders_sessions_label() {
        let theme = crate::vibes_default();
        let widget = StatsBarWidget::new(1, 1, 1.0);

        let backend = TestBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(widget.to_paragraph(&theme), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("Sessions:"),
            "Expected 'Sessions:' in: {}",
            content
        );
    }

    #[test]
    fn renders_agents_label() {
        let theme = crate::vibes_default();
        let widget = StatsBarWidget::new(1, 1, 1.0);

        let backend = TestBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(widget.to_paragraph(&theme), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("Agents:"),
            "Expected 'Agents:' in: {}",
            content
        );
    }

    #[test]
    fn renders_cost_label() {
        let theme = crate::vibes_default();
        let widget = StatsBarWidget::new(1, 1, 1.0);

        let backend = TestBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(widget.to_paragraph(&theme), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("Cost:"),
            "Expected 'Cost:' in: {}",
            content
        );
    }

    // Color tests
    #[test]
    fn cost_below_threshold_uses_normal_color() {
        let theme = crate::vibes_default();
        let widget = StatsBarWidget::new(0, 0, 5.0); // Below threshold

        let backend = TestBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(widget.to_paragraph(&theme), area);
            })
            .unwrap();

        // Find the cell containing '$' and check its foreground color
        let buffer = terminal.backend().buffer();
        let dollar_cell = buffer.content().iter().find(|c| c.symbol() == "$").unwrap();
        assert_eq!(
            dollar_cell.fg, theme.fg,
            "Cost below threshold should use normal fg color"
        );
    }

    #[test]
    fn cost_above_threshold_uses_warning_color() {
        let theme = crate::vibes_default();
        let widget = StatsBarWidget::new(0, 0, 15.0); // Above threshold

        let backend = TestBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(widget.to_paragraph(&theme), area);
            })
            .unwrap();

        // Find the cell containing '$' and check its foreground color
        let buffer = terminal.backend().buffer();
        let dollar_cell = buffer.content().iter().find(|c| c.symbol() == "$").unwrap();
        assert_eq!(
            dollar_cell.fg, theme.warning,
            "Cost above threshold should use warning color"
        );
    }
}
