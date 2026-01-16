//! Diff view modal for displaying file changes before/after.
//!
//! Used to show proposed file modifications when a permission request
//! is pending. Shows original content vs proposed content.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::Theme;

/// Modal for displaying file diffs.
#[derive(Debug, Clone, Default)]
pub struct DiffModal {
    /// Whether the modal is currently visible.
    visible: bool,
    /// Path of the file being modified.
    file_path: String,
    /// Original file content (if file exists).
    original: Option<String>,
    /// Proposed new content.
    proposed: String,
    /// Scroll offset for viewing long content.
    scroll_offset: u16,
}

impl DiffModal {
    /// Creates a new diff modal.
    pub fn new() -> Self {
        Self::default()
    }

    /// Shows the modal with the given file diff.
    pub fn show(&mut self, file_path: &str, original: Option<&str>, proposed: &str) {
        self.file_path = file_path.to_string();
        self.original = original.map(|s| s.to_string());
        self.proposed = proposed.to_string();
        self.scroll_offset = 0;
        self.visible = true;
    }

    /// Hides the modal.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Returns true if the modal is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Scrolls up by one line.
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scrolls down by one line.
    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    /// Renders the modal centered on the screen.
    pub fn render(&self, frame: &mut Frame, theme: &Theme) {
        if !self.visible {
            return;
        }

        let area = frame.area();
        // Modal takes 80% of screen, centered
        let modal_width = (area.width as f32 * 0.8) as u16;
        let modal_height = (area.height as f32 * 0.8) as u16;
        let x = (area.width - modal_width) / 2;
        let y = (area.height - modal_height) / 2;
        let modal_area = Rect::new(x, y, modal_width, modal_height);

        // Clear the background
        frame.render_widget(Clear, modal_area);

        // Main modal block
        let title = format!(" Diff: {} ", self.file_path);
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent));

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        // Split into two columns: Original | Proposed
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner);

        // Render original content (left)
        self.render_original(frame, columns[0], theme);

        // Render proposed content (right)
        self.render_proposed(frame, columns[1], theme);
    }

    /// Renders the original file content.
    fn render_original(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let block = Block::default()
            .title(" Original ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border));

        let content = match &self.original {
            Some(text) => self.content_to_lines(text, theme.fg),
            None => vec![Line::from(Span::styled(
                "(new file)",
                Style::default().fg(theme.dim.fg.unwrap_or(theme.fg)),
            ))],
        };

        let paragraph = Paragraph::new(content)
            .block(block)
            .scroll((self.scroll_offset, 0));

        frame.render_widget(paragraph, area);
    }

    /// Renders the proposed file content.
    fn render_proposed(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let block = Block::default()
            .title(" Proposed ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border));

        let content = self.content_to_lines(&self.proposed, theme.fg);

        let paragraph = Paragraph::new(content)
            .block(block)
            .scroll((self.scroll_offset, 0));

        frame.render_widget(paragraph, area);
    }

    /// Converts text content to styled lines with line numbers.
    fn content_to_lines(&self, text: &str, fg: ratatui::style::Color) -> Vec<Line<'static>> {
        text.lines()
            .enumerate()
            .map(|(i, line)| {
                Line::from(vec![
                    Span::styled(
                        format!("{:4} ", i + 1),
                        Style::default().fg(ratatui::style::Color::DarkGray),
                    ),
                    Span::styled(line.to_string(), Style::default().fg(fg)),
                ])
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_modal_new_is_hidden() {
        let modal = DiffModal::new();
        assert!(!modal.is_visible());
    }

    #[test]
    fn diff_modal_show_makes_visible() {
        let mut modal = DiffModal::new();
        modal.show("test.rs", Some("old"), "new");
        assert!(modal.is_visible());
    }

    #[test]
    fn diff_modal_hide_makes_invisible() {
        let mut modal = DiffModal::new();
        modal.show("test.rs", None, "new");
        modal.hide();
        assert!(!modal.is_visible());
    }

    #[test]
    fn diff_modal_show_stores_content() {
        let mut modal = DiffModal::new();
        modal.show(
            "src/main.rs",
            Some("fn main() {}"),
            "fn main() { println!(\"hello\"); }",
        );

        assert_eq!(modal.file_path, "src/main.rs");
        assert_eq!(modal.original, Some("fn main() {}".to_string()));
        assert_eq!(modal.proposed, "fn main() { println!(\"hello\"); }");
    }

    #[test]
    fn diff_modal_show_handles_new_file() {
        let mut modal = DiffModal::new();
        modal.show("new_file.rs", None, "// new content");

        assert!(modal.original.is_none());
        assert_eq!(modal.proposed, "// new content");
    }

    #[test]
    fn diff_modal_scroll_up_decrements() {
        let mut modal = DiffModal::new();
        modal.scroll_offset = 5;
        modal.scroll_up();
        assert_eq!(modal.scroll_offset, 4);
    }

    #[test]
    fn diff_modal_scroll_up_stops_at_zero() {
        let mut modal = DiffModal::new();
        modal.scroll_offset = 0;
        modal.scroll_up();
        assert_eq!(modal.scroll_offset, 0);
    }

    #[test]
    fn diff_modal_scroll_down_increments() {
        let mut modal = DiffModal::new();
        modal.scroll_down();
        assert_eq!(modal.scroll_offset, 1);
    }

    #[test]
    fn diff_modal_show_resets_scroll() {
        let mut modal = DiffModal::new();
        modal.scroll_offset = 10;
        modal.show("file.rs", None, "content");
        assert_eq!(modal.scroll_offset, 0);
    }

    #[test]
    fn diff_modal_content_to_lines_adds_line_numbers() {
        let modal = DiffModal::new();
        let lines = modal.content_to_lines("line1\nline2\nline3", ratatui::style::Color::White);
        assert_eq!(lines.len(), 3);
    }
}
