//! Theme preview widget for displaying color swatches and sample elements.

use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use crate::Theme;

/// Widget that displays a visual preview of a theme's colors.
///
/// Shows color swatches, semantic indicators, status colors, and sample text
/// to help users understand how a theme will look before applying it.
pub struct ThemePreview<'a> {
    theme: &'a Theme,
}

impl<'a> ThemePreview<'a> {
    /// Create a new theme preview for the given theme.
    pub fn new(theme: &'a Theme) -> Self {
        Self { theme }
    }

    /// Get the theme being previewed.
    #[allow(dead_code)] // Used in tests, will be used for future settings interaction
    pub fn theme(&self) -> &Theme {
        self.theme
    }
}

impl Widget for ThemePreview<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 || area.width < 10 {
            return;
        }

        let theme = self.theme;
        let mut y = area.y;

        // Row 1: Theme name
        if y < area.y + area.height {
            let name = format!("Theme: {}", theme.name);
            buf.set_string(area.x, y, &name, Style::default().fg(theme.fg));
            y += 1;
        }

        // Row 2: Color swatches (bg, fg, accent)
        if y < area.y + area.height {
            let mut x = area.x;
            // bg swatch
            buf.set_string(x, y, "██", Style::default().fg(theme.bg));
            x += 3;
            buf.set_string(x, y, "bg", Style::default().fg(theme.fg));
            x += 4;
            // fg swatch
            buf.set_string(x, y, "██", Style::default().fg(theme.fg));
            x += 3;
            buf.set_string(x, y, "fg", Style::default().fg(theme.fg));
            x += 4;
            // accent swatch
            buf.set_string(x, y, "██", Style::default().fg(theme.accent));
            x += 3;
            buf.set_string(x, y, "accent", Style::default().fg(theme.fg));
            let _ = x; // suppress unused warning
            y += 1;
        }

        // Row 3: Semantic colors (success, warning, error)
        if y < area.y + area.height {
            let mut x = area.x;
            buf.set_string(x, y, "●", Style::default().fg(theme.success));
            x += 2;
            buf.set_string(x, y, "Success", Style::default().fg(theme.fg));
            x += 9;
            buf.set_string(x, y, "●", Style::default().fg(theme.warning));
            x += 2;
            buf.set_string(x, y, "Warning", Style::default().fg(theme.fg));
            x += 9;
            buf.set_string(x, y, "●", Style::default().fg(theme.error));
            x += 2;
            buf.set_string(x, y, "Error", Style::default().fg(theme.fg));
            let _ = x;
            y += 1;
        }

        // Row 4: Status colors (running, paused, completed, failed)
        if y < area.y + area.height {
            let mut x = area.x;
            buf.set_string(x, y, "●", Style::default().fg(theme.running));
            x += 2;
            buf.set_string(x, y, "Running", Style::default().fg(theme.fg));
            x += 9;
            buf.set_string(x, y, "●", Style::default().fg(theme.paused));
            x += 2;
            buf.set_string(x, y, "Paused", Style::default().fg(theme.fg));
            x += 8;
            buf.set_string(x, y, "●", Style::default().fg(theme.completed));
            x += 2;
            buf.set_string(x, y, "Done", Style::default().fg(theme.fg));
            let _ = x;
            y += 1;
        }

        // Row 5: Sample text with styles
        if y < area.y + area.height {
            let mut x = area.x;
            buf.set_string(x, y, "Sample", Style::default().fg(theme.fg));
            x += 7;
            buf.set_string(x, y, "Text", theme.bold);
            x += 5;
            buf.set_string(x, y, "dim", theme.dim);
            let _ = x;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vibes_default;

    #[test]
    fn theme_preview_holds_reference_to_theme() {
        let theme = vibes_default();
        let preview = ThemePreview::new(&theme);
        assert_eq!(preview.theme().name, "vibes");
    }

    #[test]
    fn theme_preview_can_be_created_for_any_theme() {
        let theme = crate::theme::dark();
        let preview = ThemePreview::new(&theme);
        assert_eq!(preview.theme().name, "dark");
    }

    #[test]
    fn theme_preview_renders_theme_name() {
        let theme = vibes_default();
        let preview = ThemePreview::new(&theme);
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 8));
        preview.render(buf.area, &mut buf);

        // The theme name should appear in the preview
        let content = buffer_to_string(&buf);
        assert!(
            content.contains("vibes"),
            "Expected theme name in: {}",
            content
        );
    }

    #[test]
    fn theme_preview_renders_color_swatches() {
        let theme = vibes_default();
        let preview = ThemePreview::new(&theme);
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 8));
        preview.render(buf.area, &mut buf);

        let content = buffer_to_string(&buf);
        // Should have labels for the main colors
        assert!(
            content.contains("bg") || content.contains("fg") || content.contains("accent"),
            "Expected color labels in: {}",
            content
        );
    }

    #[test]
    fn theme_preview_renders_status_indicators() {
        let theme = vibes_default();
        let preview = ThemePreview::new(&theme);
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 8));
        preview.render(buf.area, &mut buf);

        let content = buffer_to_string(&buf);
        // Should show status indicators
        assert!(
            content.contains("Success") || content.contains("Warning") || content.contains("Error"),
            "Expected semantic colors in: {}",
            content
        );
    }

    #[test]
    fn theme_preview_renders_sample_text() {
        let theme = vibes_default();
        let preview = ThemePreview::new(&theme);
        let mut buf = Buffer::empty(Rect::new(0, 0, 60, 8));
        preview.render(buf.area, &mut buf);

        let content = buffer_to_string(&buf);
        // Should have sample text
        assert!(
            content.contains("Sample") || content.contains("Text"),
            "Expected sample text in: {}",
            content
        );
    }

    /// Helper to convert buffer content to a string for assertions.
    fn buffer_to_string(buf: &Buffer) -> String {
        let mut result = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                result.push(buf[(x, y)].symbol().chars().next().unwrap_or(' '));
            }
            result.push('\n');
        }
        result
    }
}
