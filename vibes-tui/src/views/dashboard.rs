//! Dashboard view - the default landing view.

use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Paragraph},
};

use super::traits::ViewRenderer;
use crate::App;

/// The main dashboard view showing an overview of activity.
#[derive(Debug, Clone, Default)]
pub struct DashboardView;

impl ViewRenderer for DashboardView {
    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        let block = Block::default()
            .title(" vibes ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border));

        let text = Paragraph::new("Dashboard - Coming in Milestone 42")
            .style(Style::default().fg(app.theme.fg))
            .block(block);

        frame.render_widget(text, area);
    }

    fn title(&self) -> &str {
        "Dashboard"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_view_has_correct_title() {
        let view = DashboardView;
        assert_eq!(view.title(), "Dashboard");
    }
}
