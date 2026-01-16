//! Output panel widget for the agent detail view.
//!
//! Displays real-time agent output with different styling for output types,
//! scrolling support, and a ring buffer to prevent unbounded memory growth.

use chrono::{DateTime, Utc};
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::Theme;

/// The type of output line, determining visual styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputLineType {
    /// Standard agent text output.
    Text,
    /// Tool call with associated tool name.
    ToolCall,
    /// Agent thinking/reasoning (dimmed).
    Thinking,
    /// Error message (highlighted in red).
    Error,
}

/// A single line of output with metadata.
#[derive(Debug, Clone)]
pub struct OutputLine {
    /// Timestamp when the line was produced.
    pub timestamp: DateTime<Utc>,
    /// The content of the line.
    pub content: String,
    /// The type of output for styling.
    pub line_type: OutputLineType,
    /// Optional tool name (for ToolCall type).
    pub tool_name: Option<String>,
}

impl OutputLine {
    /// Creates a new text output line.
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            content: content.into(),
            line_type: OutputLineType::Text,
            tool_name: None,
        }
    }

    /// Creates a new tool call output line.
    pub fn tool_call(tool_name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            content: content.into(),
            line_type: OutputLineType::ToolCall,
            tool_name: Some(tool_name.into()),
        }
    }

    /// Creates a new thinking output line.
    pub fn thinking(content: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            content: content.into(),
            line_type: OutputLineType::Thinking,
            tool_name: None,
        }
    }

    /// Creates a new error output line.
    pub fn error(content: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            content: content.into(),
            line_type: OutputLineType::Error,
            tool_name: None,
        }
    }

    /// Creates an output line with a specific timestamp (for testing).
    #[cfg(test)]
    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }
}

/// Ring buffer for output lines with bounded memory.
#[derive(Debug, Clone)]
pub struct OutputBuffer {
    /// The output lines (ring buffer).
    lines: Vec<OutputLine>,
    /// Maximum number of lines to keep.
    max_lines: usize,
    /// Current scroll offset from the bottom.
    scroll_offset: usize,
    /// Whether to auto-scroll when new output arrives.
    auto_scroll: bool,
}

impl Default for OutputBuffer {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl OutputBuffer {
    /// Creates a new output buffer with the specified capacity.
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines: Vec::with_capacity(max_lines.min(1000)),
            max_lines,
            scroll_offset: 0,
            auto_scroll: true,
        }
    }

    /// Returns the number of lines in the buffer.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Returns true if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Returns the current scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Returns whether auto-scroll is enabled.
    pub fn auto_scroll(&self) -> bool {
        self.auto_scroll
    }

    /// Pushes a new line to the buffer.
    ///
    /// If the buffer is at capacity, the oldest line is removed.
    /// If auto-scroll is enabled, the scroll offset is reset to show the new line.
    pub fn push(&mut self, line: OutputLine) {
        // If at capacity, remove the oldest line
        if self.lines.len() >= self.max_lines {
            self.lines.remove(0);
            // Adjust scroll offset if we removed a line above the current view
            if self.scroll_offset > 0 {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
        }

        self.lines.push(line);

        // Auto-scroll to bottom
        if self.auto_scroll {
            self.scroll_offset = 0;
        }
    }

    /// Scrolls up by one line.
    ///
    /// Disables auto-scroll when user scrolls manually.
    pub fn scroll_up(&mut self) {
        let max_offset = self.lines.len().saturating_sub(1);
        if self.scroll_offset < max_offset {
            self.scroll_offset += 1;
            self.auto_scroll = false;
        }
    }

    /// Scrolls down by one line.
    ///
    /// Re-enables auto-scroll when reaching the bottom.
    pub fn scroll_down(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
            if self.scroll_offset == 0 {
                self.auto_scroll = true;
            }
        }
    }

    /// Scrolls to the top of the buffer.
    pub fn scroll_to_top(&mut self) {
        if !self.lines.is_empty() {
            self.scroll_offset = self.lines.len().saturating_sub(1);
            self.auto_scroll = false;
        }
    }

    /// Scrolls to the bottom of the buffer.
    ///
    /// Re-enables auto-scroll.
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
        self.auto_scroll = true;
    }

    /// Returns an iterator over visible lines given a viewport height.
    ///
    /// Lines are returned from top to bottom of the viewport.
    pub fn visible_lines(&self, viewport_height: usize) -> impl Iterator<Item = &OutputLine> {
        let total = self.lines.len();
        if total == 0 {
            return self.lines.iter();
        }

        // Calculate the range of lines to show
        // scroll_offset=0 means we're at the bottom (showing newest lines)
        let end = total.saturating_sub(self.scroll_offset);
        let start = end.saturating_sub(viewport_height);

        self.lines[start..end].iter()
    }

    /// Returns the scroll position as a fraction (0.0 = top, 1.0 = bottom).
    pub fn scroll_position(&self) -> f64 {
        if self.lines.is_empty() {
            return 1.0;
        }
        let max_offset = self.lines.len().saturating_sub(1);
        if max_offset == 0 {
            return 1.0;
        }
        1.0 - (self.scroll_offset as f64 / max_offset as f64)
    }

    /// Clears all lines from the buffer.
    pub fn clear(&mut self) {
        self.lines.clear();
        self.scroll_offset = 0;
        self.auto_scroll = true;
    }
}

/// Widget for displaying the output panel.
#[derive(Debug, Clone, Default)]
pub struct OutputPanelWidget {
    /// The output buffer.
    pub buffer: OutputBuffer,
}

impl OutputPanelWidget {
    /// Creates a new output panel widget.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an output panel widget with a specific buffer capacity.
    #[allow(dead_code)] // Used when initializing agents with custom capacity
    pub fn with_capacity(max_lines: usize) -> Self {
        Self {
            buffer: OutputBuffer::new(max_lines),
        }
    }

    /// Pushes a new line to the output buffer.
    #[allow(dead_code)] // Called when processing agent output events
    pub fn push(&mut self, line: OutputLine) {
        self.buffer.push(line);
    }

    /// Scrolls up (older content).
    #[allow(dead_code)] // Called by keyboard handler (j/k keys)
    pub fn scroll_up(&mut self) {
        self.buffer.scroll_up();
    }

    /// Scrolls down (newer content).
    #[allow(dead_code)] // Called by keyboard handler (j/k keys)
    pub fn scroll_down(&mut self) {
        self.buffer.scroll_down();
    }

    /// Scrolls to top (oldest content).
    #[allow(dead_code)] // Called by keyboard handler (g key)
    pub fn scroll_to_top(&mut self) {
        self.buffer.scroll_to_top();
    }

    /// Scrolls to bottom (newest content).
    #[allow(dead_code)] // Called by keyboard handler (G key)
    pub fn scroll_to_bottom(&mut self) {
        self.buffer.scroll_to_bottom();
    }

    /// Converts the widget to a renderable Paragraph with the given theme and height.
    pub fn to_paragraph(&self, theme: &Theme, viewport_height: usize) -> Paragraph<'_> {
        let block = Block::default()
            .title(self.title())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border));

        // Calculate inner height (accounting for borders)
        let inner_height = viewport_height.saturating_sub(2);

        let lines: Vec<Line> = if self.buffer.is_empty() {
            vec![Line::from(Span::styled(
                "No output yet",
                Style::default().fg(theme.fg).add_modifier(Modifier::DIM),
            ))]
        } else {
            self.buffer
                .visible_lines(inner_height)
                .map(|line| self.line_to_spans(line, theme))
                .collect()
        };

        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
    }

    /// Returns the title including scroll position indicator.
    fn title(&self) -> String {
        if self.buffer.is_empty() {
            " Output ".to_string()
        } else {
            let pos = (self.buffer.scroll_position() * 100.0) as u8;
            format!(" Output [{}%] ", pos)
        }
    }

    /// Converts an output line to styled spans.
    fn line_to_spans<'a>(&self, line: &'a OutputLine, theme: &Theme) -> Line<'a> {
        match line.line_type {
            OutputLineType::Text => Line::from(Span::styled(
                line.content.clone(),
                Style::default().fg(theme.fg),
            )),
            OutputLineType::ToolCall => {
                let tool_name = line.tool_name.as_deref().unwrap_or("tool");
                Line::from(vec![
                    Span::styled(
                        format!("[{}] ", tool_name),
                        Style::default().fg(theme.accent),
                    ),
                    Span::styled(line.content.clone(), Style::default().fg(theme.fg)),
                ])
            }
            OutputLineType::Thinking => Line::from(Span::styled(
                line.content.clone(),
                Style::default()
                    .fg(theme.fg)
                    .add_modifier(Modifier::DIM | Modifier::ITALIC),
            )),
            OutputLineType::Error => Line::from(Span::styled(
                line.content.clone(),
                Style::default().fg(theme.error),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    // ==================== OutputLineType Tests ====================

    #[test]
    fn output_line_type_variants_are_distinct() {
        assert_ne!(OutputLineType::Text, OutputLineType::ToolCall);
        assert_ne!(OutputLineType::ToolCall, OutputLineType::Thinking);
        assert_ne!(OutputLineType::Thinking, OutputLineType::Error);
    }

    // ==================== OutputLine Tests ====================

    #[test]
    fn output_line_text_creates_text_type() {
        let line = OutputLine::text("Hello world");
        assert_eq!(line.content, "Hello world");
        assert_eq!(line.line_type, OutputLineType::Text);
        assert!(line.tool_name.is_none());
    }

    #[test]
    fn output_line_tool_call_creates_tool_type() {
        let line = OutputLine::tool_call("Read", "Reading file.rs");
        assert_eq!(line.content, "Reading file.rs");
        assert_eq!(line.line_type, OutputLineType::ToolCall);
        assert_eq!(line.tool_name, Some("Read".to_string()));
    }

    #[test]
    fn output_line_thinking_creates_thinking_type() {
        let line = OutputLine::thinking("Analyzing the codebase...");
        assert_eq!(line.content, "Analyzing the codebase...");
        assert_eq!(line.line_type, OutputLineType::Thinking);
        assert!(line.tool_name.is_none());
    }

    #[test]
    fn output_line_error_creates_error_type() {
        let line = OutputLine::error("Failed to read file");
        assert_eq!(line.content, "Failed to read file");
        assert_eq!(line.line_type, OutputLineType::Error);
        assert!(line.tool_name.is_none());
    }

    // ==================== OutputBuffer Tests ====================

    #[test]
    fn output_buffer_new_creates_empty_buffer() {
        let buffer = OutputBuffer::new(100);
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.scroll_offset(), 0);
        assert!(buffer.auto_scroll());
    }

    #[test]
    fn output_buffer_default_has_reasonable_capacity() {
        let buffer = OutputBuffer::default();
        assert!(buffer.is_empty());
    }

    #[test]
    fn output_buffer_push_adds_lines() {
        let mut buffer = OutputBuffer::new(100);
        buffer.push(OutputLine::text("Line 1"));
        buffer.push(OutputLine::text("Line 2"));
        assert_eq!(buffer.len(), 2);
    }

    #[test]
    fn output_buffer_ring_buffer_removes_oldest() {
        let mut buffer = OutputBuffer::new(3);
        buffer.push(OutputLine::text("Line 1"));
        buffer.push(OutputLine::text("Line 2"));
        buffer.push(OutputLine::text("Line 3"));
        buffer.push(OutputLine::text("Line 4"));

        assert_eq!(buffer.len(), 3);
        // Line 1 should be gone, newest lines remain
        let lines: Vec<_> = buffer.visible_lines(10).collect();
        assert_eq!(lines[0].content, "Line 2");
        assert_eq!(lines[2].content, "Line 4");
    }

    #[test]
    fn output_buffer_scroll_up_increases_offset() {
        let mut buffer = OutputBuffer::new(100);
        for i in 0..10 {
            buffer.push(OutputLine::text(format!("Line {}", i)));
        }

        assert_eq!(buffer.scroll_offset(), 0);
        buffer.scroll_up();
        assert_eq!(buffer.scroll_offset(), 1);
        assert!(!buffer.auto_scroll());
    }

    #[test]
    fn output_buffer_scroll_up_stops_at_top() {
        let mut buffer = OutputBuffer::new(100);
        buffer.push(OutputLine::text("Line 1"));
        buffer.push(OutputLine::text("Line 2"));

        // Scroll up past the limit
        for _ in 0..10 {
            buffer.scroll_up();
        }

        assert_eq!(buffer.scroll_offset(), 1); // max is len - 1
    }

    #[test]
    fn output_buffer_scroll_down_decreases_offset() {
        let mut buffer = OutputBuffer::new(100);
        for i in 0..10 {
            buffer.push(OutputLine::text(format!("Line {}", i)));
        }
        buffer.scroll_offset = 5;
        buffer.auto_scroll = false;

        buffer.scroll_down();
        assert_eq!(buffer.scroll_offset(), 4);
    }

    #[test]
    fn output_buffer_scroll_down_reenables_autoscroll_at_bottom() {
        let mut buffer = OutputBuffer::new(100);
        buffer.push(OutputLine::text("Line 1"));
        buffer.scroll_offset = 1;
        buffer.auto_scroll = false;

        buffer.scroll_down();
        assert_eq!(buffer.scroll_offset(), 0);
        assert!(buffer.auto_scroll());
    }

    #[test]
    fn output_buffer_scroll_to_top() {
        let mut buffer = OutputBuffer::new(100);
        for i in 0..10 {
            buffer.push(OutputLine::text(format!("Line {}", i)));
        }

        buffer.scroll_to_top();
        assert_eq!(buffer.scroll_offset(), 9);
        assert!(!buffer.auto_scroll());
    }

    #[test]
    fn output_buffer_scroll_to_bottom() {
        let mut buffer = OutputBuffer::new(100);
        for i in 0..10 {
            buffer.push(OutputLine::text(format!("Line {}", i)));
        }
        buffer.scroll_offset = 5;
        buffer.auto_scroll = false;

        buffer.scroll_to_bottom();
        assert_eq!(buffer.scroll_offset(), 0);
        assert!(buffer.auto_scroll());
    }

    #[test]
    fn output_buffer_visible_lines_returns_correct_slice() {
        let mut buffer = OutputBuffer::new(100);
        for i in 0..10 {
            buffer.push(OutputLine::text(format!("Line {}", i)));
        }

        // At bottom (scroll_offset=0), viewport of 3 shows last 3 lines
        let visible: Vec<_> = buffer.visible_lines(3).collect();
        assert_eq!(visible.len(), 3);
        assert_eq!(visible[0].content, "Line 7");
        assert_eq!(visible[2].content, "Line 9");
    }

    #[test]
    fn output_buffer_visible_lines_with_scroll_offset() {
        let mut buffer = OutputBuffer::new(100);
        for i in 0..10 {
            buffer.push(OutputLine::text(format!("Line {}", i)));
        }
        buffer.scroll_offset = 3; // Scroll up 3 lines

        let visible: Vec<_> = buffer.visible_lines(3).collect();
        assert_eq!(visible.len(), 3);
        assert_eq!(visible[0].content, "Line 4");
        assert_eq!(visible[2].content, "Line 6");
    }

    #[test]
    fn output_buffer_scroll_position_at_bottom_is_one() {
        let mut buffer = OutputBuffer::new(100);
        for i in 0..10 {
            buffer.push(OutputLine::text(format!("Line {}", i)));
        }

        assert!((buffer.scroll_position() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn output_buffer_scroll_position_at_top_is_zero() {
        let mut buffer = OutputBuffer::new(100);
        for i in 0..10 {
            buffer.push(OutputLine::text(format!("Line {}", i)));
        }
        buffer.scroll_to_top();

        assert!(buffer.scroll_position().abs() < f64::EPSILON);
    }

    #[test]
    fn output_buffer_clear_resets_state() {
        let mut buffer = OutputBuffer::new(100);
        for i in 0..10 {
            buffer.push(OutputLine::text(format!("Line {}", i)));
        }
        buffer.scroll_offset = 5;
        buffer.auto_scroll = false;

        buffer.clear();
        assert!(buffer.is_empty());
        assert_eq!(buffer.scroll_offset(), 0);
        assert!(buffer.auto_scroll());
    }

    #[test]
    fn output_buffer_push_with_autoscroll_keeps_offset_zero() {
        let mut buffer = OutputBuffer::new(100);
        buffer.push(OutputLine::text("Line 1"));
        buffer.push(OutputLine::text("Line 2"));

        assert_eq!(buffer.scroll_offset(), 0);
        assert!(buffer.auto_scroll());
    }

    #[test]
    fn output_buffer_ring_buffer_adjusts_scroll_offset() {
        let mut buffer = OutputBuffer::new(3);
        buffer.push(OutputLine::text("Line 1"));
        buffer.push(OutputLine::text("Line 2"));
        buffer.push(OutputLine::text("Line 3"));

        // Scroll up
        buffer.scroll_up();
        buffer.scroll_up();
        assert_eq!(buffer.scroll_offset(), 2);

        // Push new line - this removes oldest and should adjust scroll
        buffer.push(OutputLine::text("Line 4"));
        assert_eq!(buffer.scroll_offset(), 1); // Adjusted down by 1
    }

    // ==================== OutputPanelWidget Tests ====================

    #[test]
    fn output_panel_widget_new_creates_empty_panel() {
        let widget = OutputPanelWidget::new();
        assert!(widget.buffer.is_empty());
    }

    #[test]
    fn output_panel_widget_with_capacity() {
        let widget = OutputPanelWidget::with_capacity(50);
        assert!(widget.buffer.is_empty());
    }

    #[test]
    fn output_panel_widget_push_delegates_to_buffer() {
        let mut widget = OutputPanelWidget::new();
        widget.push(OutputLine::text("Hello"));
        assert_eq!(widget.buffer.len(), 1);
    }

    #[test]
    fn output_panel_widget_scroll_methods_delegate() {
        let mut widget = OutputPanelWidget::new();
        for i in 0..10 {
            widget.push(OutputLine::text(format!("Line {}", i)));
        }

        widget.scroll_up();
        assert_eq!(widget.buffer.scroll_offset(), 1);

        widget.scroll_down();
        assert_eq!(widget.buffer.scroll_offset(), 0);

        widget.scroll_to_top();
        assert_eq!(widget.buffer.scroll_offset(), 9);

        widget.scroll_to_bottom();
        assert_eq!(widget.buffer.scroll_offset(), 0);
    }

    #[test]
    fn output_panel_renders_empty_state() {
        let theme = crate::vibes_default();
        let widget = OutputPanelWidget::new();

        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(widget.to_paragraph(&theme, area.height as usize), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("No output yet"),
            "Expected 'No output yet' in: {}",
            content
        );
    }

    #[test]
    fn output_panel_renders_text_lines() {
        let theme = crate::vibes_default();
        let mut widget = OutputPanelWidget::new();
        widget.push(OutputLine::text("Hello world"));

        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(widget.to_paragraph(&theme, area.height as usize), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("Hello world"),
            "Expected 'Hello world' in: {}",
            content
        );
    }

    #[test]
    fn output_panel_renders_tool_calls_with_prefix() {
        let theme = crate::vibes_default();
        let mut widget = OutputPanelWidget::new();
        widget.push(OutputLine::tool_call("Read", "file.rs"));

        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(widget.to_paragraph(&theme, area.height as usize), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("[Read]"),
            "Expected '[Read]' in: {}",
            content
        );
        assert!(
            content.contains("file.rs"),
            "Expected 'file.rs' in: {}",
            content
        );
    }

    #[test]
    fn output_panel_title_shows_scroll_position() {
        let mut widget = OutputPanelWidget::new();
        for i in 0..10 {
            widget.push(OutputLine::text(format!("Line {}", i)));
        }

        // At bottom should show 100%
        let title = widget.title();
        assert!(
            title.contains("100%"),
            "Expected '100%' in title: {}",
            title
        );

        // At top should show 0%
        widget.scroll_to_top();
        let title = widget.title();
        assert!(title.contains("0%"), "Expected '0%' in title: {}", title);
    }

    #[test]
    fn output_panel_empty_shows_output_title() {
        let widget = OutputPanelWidget::new();
        let title = widget.title();
        assert!(
            title.contains("Output"),
            "Expected 'Output' in title: {}",
            title
        );
    }
}
