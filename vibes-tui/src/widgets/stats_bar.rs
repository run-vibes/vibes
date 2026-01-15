//! Stats summary bar widget for the dashboard.
//!
//! Displays aggregate metrics at the top of the dashboard:
//! sessions, agents, cost, and connection status.

use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::Theme;

/// Cost threshold in USD above which the cost is displayed in warning color.
pub const COST_WARNING_THRESHOLD: f64 = 10.0;

/// Connection status for the WebSocket connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionStatus {
    /// Connected to the server.
    #[default]
    Connected,
    /// Currently attempting to connect.
    Connecting,
    /// Disconnected from the server.
    Disconnected,
    /// Reconnecting after a disconnect.
    Reconnecting,
}

impl ConnectionStatus {
    /// Returns the display text for this status.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Connected => "connected",
            Self::Connecting => "connecting...",
            Self::Disconnected => "disconnected",
            Self::Reconnecting => "reconnecting...",
        }
    }
}

/// Widget displaying aggregate stats in a single line.
#[derive(Debug, Clone, Default)]
pub struct StatsBarWidget {
    pub session_count: u32,
    pub agent_count: u32,
    pub total_cost: f64,
    pub connection_status: ConnectionStatus,
}

impl StatsBarWidget {
    /// Creates a new stats bar widget with the given values.
    pub fn new(session_count: u32, agent_count: u32, total_cost: f64) -> Self {
        Self {
            session_count,
            agent_count,
            total_cost,
            connection_status: ConnectionStatus::default(),
        }
    }

    /// Formats the cost as "$X.XX".
    pub fn format_cost(&self) -> String {
        format!("${:.2}", self.total_cost)
    }

    /// Converts the widget to a renderable Paragraph with the given theme.
    ///
    /// Layout: "Sessions: N active   Agents: N running   Cost: $X.XX   Status: connected"
    pub fn to_paragraph(&self, theme: &Theme) -> Paragraph<'_> {
        let cost_color = if self.total_cost > COST_WARNING_THRESHOLD {
            theme.warning
        } else {
            theme.fg
        };

        let status_color = match self.connection_status {
            ConnectionStatus::Connected => theme.running,
            ConnectionStatus::Connecting | ConnectionStatus::Reconnecting => theme.warning,
            ConnectionStatus::Disconnected => theme.failed,
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
            Span::raw("   "),
            Span::styled("Status: ", Style::default().fg(theme.fg)),
            Span::styled(
                self.connection_status.as_str(),
                Style::default().fg(status_color),
            ),
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

    // Connection status tests
    #[test]
    fn connection_status_default_is_connected() {
        let status = ConnectionStatus::default();
        assert_eq!(status, ConnectionStatus::Connected);
    }

    #[test]
    fn connection_status_as_str_returns_correct_text() {
        assert_eq!(ConnectionStatus::Connected.as_str(), "connected");
        assert_eq!(ConnectionStatus::Connecting.as_str(), "connecting...");
        assert_eq!(ConnectionStatus::Disconnected.as_str(), "disconnected");
        assert_eq!(ConnectionStatus::Reconnecting.as_str(), "reconnecting...");
    }

    #[test]
    fn renders_connection_status() {
        let theme = crate::vibes_default();
        let mut widget = StatsBarWidget::new(1, 1, 1.0);
        widget.connection_status = ConnectionStatus::Connected;

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
            content.contains("connected"),
            "Expected 'connected' in: {}",
            content
        );
    }

    #[test]
    fn connected_status_uses_running_color() {
        let theme = crate::vibes_default();
        let mut widget = StatsBarWidget::new(0, 0, 0.0);
        widget.connection_status = ConnectionStatus::Connected;

        let backend = TestBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(widget.to_paragraph(&theme), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        // Find "connected" text and check the first character's color
        let content: Vec<_> = buffer.content().iter().collect();
        let status_start = content
            .iter()
            .position(|c| {
                c.symbol() == "c"
                    && content[content
                        .iter()
                        .position(|x| std::ptr::eq(*x, *c))
                        .unwrap()
                        .saturating_add(1)]
                    .symbol()
                        == "o"
            })
            .unwrap();
        assert_eq!(
            content[status_start].fg, theme.running,
            "Connected status should use running color"
        );
    }

    #[test]
    fn disconnected_status_uses_failed_color() {
        let theme = crate::vibes_default();
        let mut widget = StatsBarWidget::new(0, 0, 0.0);
        widget.connection_status = ConnectionStatus::Disconnected;

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
            content.contains("disconnected"),
            "Expected 'disconnected' in: {}",
            content
        );
    }
}
