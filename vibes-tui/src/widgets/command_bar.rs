//! Command bar widget for command mode input.
//!
//! Displays the command input line at the bottom of the screen
//! when in command mode.

use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::Theme;
use crate::commands::CommandInput;

/// Widget for rendering the command input bar.
pub struct CommandBarWidget;

impl CommandBarWidget {
    /// Render the command bar to the frame.
    ///
    /// Shows the `:` prompt followed by the input buffer.
    /// Positions the cursor correctly within the input.
    pub fn render(frame: &mut Frame, area: Rect, input: &CommandInput, theme: &Theme) {
        // Build the command line: ":{buffer}"
        let prompt_style = Style::default().fg(theme.accent);
        let input_style = Style::default().fg(theme.fg).bg(theme.bg);

        let line = Line::from(vec![
            Span::styled(":", prompt_style),
            Span::styled(&input.buffer, input_style),
        ]);

        let para = Paragraph::new(line).style(Style::default().bg(theme.bg));
        frame.render_widget(para, area);

        // Position cursor after the ':' + cursor position in buffer
        frame.set_cursor_position((area.x + 1 + input.cursor as u16, area.y));

        // If there's a message, show it on the right side
        if let Some((msg, is_error)) = &input.message {
            let msg_color = if *is_error {
                theme.error
            } else {
                theme.success
            };
            let msg_style = Style::default().fg(msg_color);
            let msg_span = Span::styled(msg, msg_style);
            let msg_para = Paragraph::new(Line::from(msg_span));

            // Position message on the right
            let msg_width = msg.len() as u16;
            if area.width > msg_width + 2 {
                let msg_area = Rect {
                    x: area.x + area.width - msg_width - 1,
                    y: area.y,
                    width: msg_width + 1,
                    height: 1,
                };
                frame.render_widget(msg_para, msg_area);
            }
        }

        // If completions are visible, render completion popup
        if !input.completions.is_empty() {
            Self::render_completions(frame, area, input, theme);
        }
    }

    /// Render the completion popup above the command bar.
    fn render_completions(frame: &mut Frame, area: Rect, input: &CommandInput, theme: &Theme) {
        let completions = &input.completions;
        let selected_idx = input.completion_idx;

        // Calculate popup dimensions
        let max_width = completions.iter().map(|s| s.len()).max().unwrap_or(0) as u16 + 2;
        let height = completions.len().min(5) as u16;

        // Position popup above the command bar
        if area.y >= height {
            let popup_area = Rect {
                x: area.x + 1, // Align with input, after ':'
                y: area.y - height,
                width: max_width.min(area.width - 1),
                height,
            };

            // Build completion lines
            let lines: Vec<Line> = completions
                .iter()
                .enumerate()
                .take(5)
                .map(|(idx, comp)| {
                    let style = if Some(idx) == selected_idx {
                        Style::default().fg(theme.bg).bg(theme.accent)
                    } else {
                        Style::default().fg(theme.fg).bg(theme.selection)
                    };
                    Line::from(Span::styled(format!(" {} ", comp), style))
                })
                .collect();

            let popup = Paragraph::new(lines);
            frame.render_widget(popup, popup_area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_bar_widget_exists() {
        // Widget exists and can be referenced
        let _ = CommandBarWidget;
    }
}
