//! Traits for view rendering in vibes TUI.

use ratatui::{Frame, layout::Rect};

use crate::App;

/// Trait for views that can render themselves.
///
/// Each view type implements this trait to define its rendering behavior.
pub trait ViewRenderer {
    /// Render the view to the terminal frame.
    fn render(&self, frame: &mut Frame, area: Rect, app: &App);

    /// Get the view's title for display.
    fn title(&self) -> &str;
}
