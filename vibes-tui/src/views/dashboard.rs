//! Dashboard view - the default landing view.

use ratatui::{Frame, layout::Rect};

use super::traits::ViewRenderer;
use crate::App;

/// The main dashboard view showing an overview of activity.
#[derive(Debug, Clone, Default)]
pub struct DashboardView;

impl ViewRenderer for DashboardView {
    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        // Render the session list widget
        let session_list = app.session_widget.to_list(&app.theme);
        frame.render_widget(session_list, area);
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
