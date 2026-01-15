//! Dashboard view - the default landing view.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
};

use super::traits::ViewRenderer;
use crate::App;

/// The main dashboard view showing an overview of activity.
#[derive(Debug, Clone, Default)]
pub struct DashboardView;

impl ViewRenderer for DashboardView {
    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        // Split area into stats bar (1 line) and session list (rest)
        let chunks = Layout::default()
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area);

        // Render the stats bar at the top
        let stats_bar = app.stats_widget.to_paragraph(&app.theme);
        frame.render_widget(stats_bar, chunks[0]);

        // Render the session list below
        let session_list = app.session_widget.to_list(&app.theme);
        frame.render_widget(session_list, chunks[1]);
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
