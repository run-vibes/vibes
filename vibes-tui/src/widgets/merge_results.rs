//! Merged results view widget for displaying combined agent output.
//!
//! Shows the merged results from multiple agents with scrolling,
//! copy to clipboard, and save to file functionality.

use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::Theme;

/// Section of merged results from a single agent.
#[derive(Debug, Clone)]
pub struct ResultSection {
    /// Agent name/title for the section.
    pub agent_name: String,
    /// The agent's result content.
    pub content: String,
}

/// Modal view for displaying merged results from swarm agents.
#[derive(Debug, Clone, Default)]
pub struct MergeResultsView {
    /// Whether the view is visible.
    visible: bool,
    /// Sections of merged results.
    sections: Vec<ResultSection>,
    /// Scroll offset for viewing long content.
    scroll_offset: u16,
    /// Total number of content lines (for scroll bounds).
    total_lines: usize,
}

impl MergeResultsView {
    /// Creates a new merge results view.
    pub fn new() -> Self {
        Self::default()
    }

    /// Shows the view with merged results.
    pub fn show(&mut self, sections: Vec<ResultSection>) {
        // Calculate total lines
        let mut total = 0;
        for section in &sections {
            total += 2; // header + blank line
            total += section.content.lines().count().max(1);
            total += 1; // spacing between sections
        }
        self.total_lines = total;
        self.sections = sections;
        self.scroll_offset = 0;
        self.visible = true;
    }

    /// Hides the view.
    pub fn hide(&mut self) {
        self.visible = false;
        self.sections.clear();
        self.total_lines = 0;
    }

    /// Returns true if the view is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Returns the result sections.
    pub fn sections(&self) -> &[ResultSection] {
        &self.sections
    }

    /// Returns the current scroll offset.
    pub fn scroll_offset(&self) -> u16 {
        self.scroll_offset
    }

    /// Scrolls up by one line.
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scrolls down by one line, respecting content bounds.
    pub fn scroll_down(&mut self, viewport_height: u16) {
        let max_scroll = self.total_lines.saturating_sub(viewport_height as usize) as u16;
        if self.scroll_offset < max_scroll {
            self.scroll_offset = self.scroll_offset.saturating_add(1);
        }
    }

    /// Scrolls up by a page (viewport height).
    pub fn page_up(&mut self, viewport_height: u16) {
        self.scroll_offset = self.scroll_offset.saturating_sub(viewport_height);
    }

    /// Scrolls down by a page (viewport height).
    pub fn page_down(&mut self, viewport_height: u16) {
        let max_scroll = self.total_lines.saturating_sub(viewport_height as usize) as u16;
        self.scroll_offset = self
            .scroll_offset
            .saturating_add(viewport_height)
            .min(max_scroll);
    }

    /// Returns the full merged content as markdown (for copy/save).
    pub fn as_markdown(&self) -> String {
        let mut output = String::new();
        for (i, section) in self.sections.iter().enumerate() {
            if i > 0 {
                output.push_str("\n\n");
            }
            output.push_str(&format!("## {} \n\n", section.agent_name));
            output.push_str(&section.content);
        }
        output
    }

    /// Returns true if there is content to display.
    pub fn has_content(&self) -> bool {
        !self.sections.is_empty()
    }

    /// Renders the view centered on the screen.
    pub fn render(&self, frame: &mut Frame, theme: &Theme) {
        if !self.visible {
            return;
        }

        let area = frame.area();
        // Modal takes 80% of screen, centered
        let modal_width = ((area.width as f32 * 0.8) as u16).max(40);
        let modal_height = ((area.height as f32 * 0.8) as u16).max(10);
        let x = (area.width.saturating_sub(modal_width)) / 2;
        let y = (area.height.saturating_sub(modal_height)) / 2;
        let modal_area = Rect::new(x, y, modal_width, modal_height);

        // Clear background
        frame.render_widget(Clear, modal_area);

        // Main block
        let title = " Merged Results ";
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent));

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        // Reserve space for footer controls
        let content_height = inner.height.saturating_sub(2);
        let content_area = Rect::new(inner.x, inner.y, inner.width, content_height);
        let footer_area = Rect::new(inner.x, inner.y + content_height + 1, inner.width, 1);

        // Build content lines
        let lines = self.build_content_lines(theme);

        let paragraph = Paragraph::new(lines).scroll((self.scroll_offset, 0));

        frame.render_widget(paragraph, content_area);

        // Footer controls
        let footer = Line::from(vec![
            Span::styled("[c]", Style::default().fg(theme.accent)),
            Span::styled(" Copy  ", Style::default().fg(theme.fg)),
            Span::styled("[s]", Style::default().fg(theme.accent)),
            Span::styled(" Save  ", Style::default().fg(theme.fg)),
            Span::styled("[Esc]", Style::default().fg(theme.accent)),
            Span::styled(" Close", Style::default().fg(theme.fg)),
        ]);
        let footer_para = Paragraph::new(footer);
        frame.render_widget(footer_para, footer_area);
    }

    /// Builds the content lines for display.
    fn build_content_lines(&self, theme: &Theme) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        for (i, section) in self.sections.iter().enumerate() {
            if i > 0 {
                lines.push(Line::default());
            }

            // Section header
            let header = format!("## {} ", section.agent_name);
            lines.push(Line::from(Span::styled(
                header,
                Style::default().fg(theme.accent),
            )));
            lines.push(Line::default());

            // Section content
            for content_line in section.content.lines() {
                lines.push(Line::from(Span::styled(
                    content_line.to_string(),
                    Style::default().fg(theme.fg),
                )));
            }

            // Handle empty content
            if section.content.is_empty() {
                lines.push(Line::from(Span::styled(
                    "(no output)",
                    Style::default().fg(theme.border),
                )));
            }
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== ResultSection Tests ====================

    #[test]
    fn result_section_stores_agent_name() {
        let section = ResultSection {
            agent_name: "Security Review".into(),
            content: "All secure".into(),
        };
        assert_eq!(section.agent_name, "Security Review");
    }

    #[test]
    fn result_section_stores_content() {
        let section = ResultSection {
            agent_name: "Agent".into(),
            content: "Line 1\nLine 2".into(),
        };
        assert_eq!(section.content, "Line 1\nLine 2");
    }

    // ==================== MergeResultsView State Tests ====================

    #[test]
    fn merge_results_view_new_is_hidden() {
        let view = MergeResultsView::new();
        assert!(!view.is_visible());
    }

    #[test]
    fn merge_results_view_default_is_hidden() {
        let view = MergeResultsView::default();
        assert!(!view.is_visible());
    }

    #[test]
    fn merge_results_view_show_makes_visible() {
        let mut view = MergeResultsView::new();
        view.show(vec![]);
        assert!(view.is_visible());
    }

    #[test]
    fn merge_results_view_show_stores_sections() {
        let mut view = MergeResultsView::new();
        view.show(vec![
            ResultSection {
                agent_name: "A".into(),
                content: "Content A".into(),
            },
            ResultSection {
                agent_name: "B".into(),
                content: "Content B".into(),
            },
        ]);
        assert_eq!(view.sections().len(), 2);
    }

    #[test]
    fn merge_results_view_show_resets_scroll() {
        let mut view = MergeResultsView::new();
        view.scroll_offset = 10;
        view.show(vec![]);
        assert_eq!(view.scroll_offset(), 0);
    }

    #[test]
    fn merge_results_view_hide_makes_invisible() {
        let mut view = MergeResultsView::new();
        view.show(vec![]);
        view.hide();
        assert!(!view.is_visible());
    }

    #[test]
    fn merge_results_view_hide_clears_sections() {
        let mut view = MergeResultsView::new();
        view.show(vec![ResultSection {
            agent_name: "A".into(),
            content: "Test".into(),
        }]);
        view.hide();
        assert!(view.sections().is_empty());
    }

    #[test]
    fn merge_results_view_has_content_true() {
        let mut view = MergeResultsView::new();
        view.show(vec![ResultSection {
            agent_name: "A".into(),
            content: "Test".into(),
        }]);
        assert!(view.has_content());
    }

    #[test]
    fn merge_results_view_has_content_false() {
        let mut view = MergeResultsView::new();
        view.show(vec![]);
        assert!(!view.has_content());
    }

    // ==================== Scrolling Tests ====================

    #[test]
    fn merge_results_view_scroll_up_decrements() {
        let mut view = MergeResultsView::new();
        view.scroll_offset = 5;
        view.scroll_up();
        assert_eq!(view.scroll_offset(), 4);
    }

    #[test]
    fn merge_results_view_scroll_up_stops_at_zero() {
        let mut view = MergeResultsView::new();
        view.scroll_offset = 0;
        view.scroll_up();
        assert_eq!(view.scroll_offset(), 0);
    }

    #[test]
    fn merge_results_view_scroll_down_increments() {
        let mut view = MergeResultsView::new();
        view.total_lines = 100;
        view.scroll_down(10);
        assert_eq!(view.scroll_offset(), 1);
    }

    #[test]
    fn merge_results_view_scroll_down_respects_bounds() {
        let mut view = MergeResultsView::new();
        view.total_lines = 15;
        view.scroll_offset = 5; // max = 15 - 10 = 5
        view.scroll_down(10);
        assert_eq!(view.scroll_offset(), 5); // should not increase
    }

    #[test]
    fn merge_results_view_page_up() {
        let mut view = MergeResultsView::new();
        view.scroll_offset = 20;
        view.page_up(10);
        assert_eq!(view.scroll_offset(), 10);
    }

    #[test]
    fn merge_results_view_page_up_stops_at_zero() {
        let mut view = MergeResultsView::new();
        view.scroll_offset = 5;
        view.page_up(10);
        assert_eq!(view.scroll_offset(), 0);
    }

    #[test]
    fn merge_results_view_page_down() {
        let mut view = MergeResultsView::new();
        view.total_lines = 100;
        view.scroll_offset = 0;
        view.page_down(10);
        assert_eq!(view.scroll_offset(), 10);
    }

    #[test]
    fn merge_results_view_page_down_respects_bounds() {
        let mut view = MergeResultsView::new();
        view.total_lines = 20;
        view.scroll_offset = 5;
        view.page_down(10); // max = 20 - 10 = 10
        assert_eq!(view.scroll_offset(), 10);
    }

    // ==================== to_string Tests ====================

    #[test]
    fn merge_results_view_to_string_single_section() {
        let mut view = MergeResultsView::new();
        view.show(vec![ResultSection {
            agent_name: "Security".into(),
            content: "All secure".into(),
        }]);
        let output = view.as_markdown();
        assert!(output.contains("## Security"));
        assert!(output.contains("All secure"));
    }

    #[test]
    fn merge_results_view_to_string_multiple_sections() {
        let mut view = MergeResultsView::new();
        view.show(vec![
            ResultSection {
                agent_name: "Security".into(),
                content: "Secure".into(),
            },
            ResultSection {
                agent_name: "Performance".into(),
                content: "Fast".into(),
            },
        ]);
        let output = view.as_markdown();
        assert!(output.contains("## Security"));
        assert!(output.contains("Secure"));
        assert!(output.contains("## Performance"));
        assert!(output.contains("Fast"));
    }

    #[test]
    fn merge_results_view_to_string_empty() {
        let view = MergeResultsView::new();
        let output = view.as_markdown();
        assert!(output.is_empty());
    }

    // ==================== Rendering Tests ====================

    #[test]
    fn merge_results_view_does_not_render_when_hidden() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let view = MergeResultsView::new();

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                view.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            !content.contains("Merged Results"),
            "Hidden view should not render"
        );
    }

    #[test]
    fn merge_results_view_renders_title() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let mut view = MergeResultsView::new();
        view.show(vec![ResultSection {
            agent_name: "Agent".into(),
            content: "Test".into(),
        }]);

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                view.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("Merged Results"),
            "Expected 'Merged Results' in: {}",
            content
        );
    }

    #[test]
    fn merge_results_view_renders_section_headers() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let mut view = MergeResultsView::new();
        view.show(vec![ResultSection {
            agent_name: "Security Review".into(),
            content: "Findings".into(),
        }]);

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                view.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("Security Review"),
            "Expected section header in: {}",
            content
        );
    }

    #[test]
    fn merge_results_view_renders_section_content() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let mut view = MergeResultsView::new();
        view.show(vec![ResultSection {
            agent_name: "Agent".into(),
            content: "No vulnerabilities found".into(),
        }]);

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                view.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("No vulnerabilities"),
            "Expected content in: {}",
            content
        );
    }

    #[test]
    fn merge_results_view_renders_controls() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let mut view = MergeResultsView::new();
        view.show(vec![ResultSection {
            agent_name: "Agent".into(),
            content: "Test".into(),
        }]);

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                view.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(content.contains("[c]"), "Expected '[c]' in: {}", content);
        assert!(content.contains("[s]"), "Expected '[s]' in: {}", content);
        assert!(
            content.contains("[Esc]"),
            "Expected '[Esc]' in: {}",
            content
        );
    }

    #[test]
    fn merge_results_view_renders_multiple_sections() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let mut view = MergeResultsView::new();
        view.show(vec![
            ResultSection {
                agent_name: "First".into(),
                content: "Content 1".into(),
            },
            ResultSection {
                agent_name: "Second".into(),
                content: "Content 2".into(),
            },
        ]);

        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                view.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("First"),
            "Expected 'First' in: {}",
            content
        );
        assert!(
            content.contains("Second"),
            "Expected 'Second' in: {}",
            content
        );
    }
}
