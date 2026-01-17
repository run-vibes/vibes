//! Theme selector widget for displaying available themes in a list.

use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use crate::Theme;

/// Widget that displays a list of available themes for selection.
///
/// Shows theme names with the current theme marked and the selected theme highlighted.
pub struct ThemeSelector<'a> {
    themes: &'a [String],
    selected: usize,
    current_theme: &'a str,
    theme: &'a Theme,
}

impl<'a> ThemeSelector<'a> {
    /// Create a new theme selector.
    ///
    /// - `themes`: List of available theme names
    /// - `selected`: Index of the currently selected theme
    /// - `current_theme`: Name of the currently active theme (marked with ●)
    /// - `theme`: Theme to use for styling the widget
    pub fn new(
        themes: &'a [String],
        selected: usize,
        current_theme: &'a str,
        theme: &'a Theme,
    ) -> Self {
        Self {
            themes,
            selected,
            current_theme,
            theme,
        }
    }
}

impl Widget for ThemeSelector<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 || area.width < 5 || self.themes.is_empty() {
            return;
        }

        for (i, name) in self.themes.iter().enumerate() {
            let y = area.y + i as u16;
            if y >= area.y + area.height {
                break;
            }

            // Build the line: marker + name
            let marker = if name == self.current_theme {
                "● "
            } else {
                "  "
            };
            let line = format!("{}{}", marker, name);

            // Style: selected gets highlight, current gets accent
            let style = if i == self.selected {
                Style::default().fg(self.theme.fg).bg(self.theme.selection)
            } else if name == self.current_theme {
                Style::default().fg(self.theme.accent)
            } else {
                Style::default().fg(self.theme.fg)
            };

            buf.set_string(area.x, y, &line, style);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vibes_default;

    #[test]
    fn theme_selector_stores_themes() {
        let themes = vec!["vibes".to_string(), "dark".to_string()];
        let theme = vibes_default();
        let selector = ThemeSelector::new(&themes, 0, "vibes", &theme);
        assert_eq!(selector.themes.len(), 2);
    }

    #[test]
    fn theme_selector_stores_selected_index() {
        let themes = vec!["vibes".to_string(), "dark".to_string()];
        let theme = vibes_default();
        let selector = ThemeSelector::new(&themes, 1, "vibes", &theme);
        assert_eq!(selector.selected, 1);
    }

    #[test]
    fn theme_selector_stores_current_theme() {
        let themes = vec!["vibes".to_string(), "dark".to_string()];
        let theme = vibes_default();
        let selector = ThemeSelector::new(&themes, 0, "dark", &theme);
        assert_eq!(selector.current_theme, "dark");
    }

    #[test]
    fn theme_selector_renders_theme_names() {
        let themes = vec!["vibes".to_string(), "dark".to_string()];
        let theme = vibes_default();
        let selector = ThemeSelector::new(&themes, 0, "vibes", &theme);
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 5));
        selector.render(buf.area, &mut buf);

        let content = buffer_to_string(&buf);
        assert!(content.contains("vibes"), "Expected vibes in: {}", content);
        assert!(content.contains("dark"), "Expected dark in: {}", content);
    }

    #[test]
    fn theme_selector_marks_current_theme() {
        let themes = vec!["vibes".to_string(), "dark".to_string()];
        let theme = vibes_default();
        let selector = ThemeSelector::new(&themes, 0, "vibes", &theme);
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 5));
        selector.render(buf.area, &mut buf);

        let content = buffer_to_string(&buf);
        // Current theme should have a marker
        assert!(content.contains("●"), "Expected marker in: {}", content);
    }

    #[test]
    fn theme_selector_handles_empty_list() {
        let themes: Vec<String> = vec![];
        let theme = vibes_default();
        let selector = ThemeSelector::new(&themes, 0, "vibes", &theme);
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 5));
        // Should not panic
        selector.render(buf.area, &mut buf);
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
