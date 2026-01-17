//! Settings view for theme configuration.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Paragraph},
};

use super::traits::ViewRenderer;
use crate::App;
use crate::widgets::{ThemePreview, ThemeSelector};

/// Settings view with theme selection and preview.
#[derive(Debug, Clone, Default)]
pub struct SettingsView;

impl ViewRenderer for SettingsView {
    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        // Main layout: title + content
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // Title bar
        let title = Paragraph::new("Settings")
            .style(Style::default().fg(app.theme.fg))
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(app.theme.border)),
            );
        frame.render_widget(title, chunks[0]);

        // Content area: split into selector and preview
        let content_chunks = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(chunks[1]);

        // Theme selector (left side)
        let themes: Vec<String> = app
            .theme_loader
            .list()
            .into_iter()
            .map(String::from)
            .collect();

        let (selected_index, preview_theme_name) = if let Some(ref settings) = app.settings_state {
            (
                settings.selected_index(),
                settings.preview_theme().to_string(),
            )
        } else {
            (0, app.theme.name.clone())
        };

        let selector_block = Block::default()
            .title("Themes")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border));

        let selector_inner = selector_block.inner(content_chunks[0]);
        frame.render_widget(selector_block, content_chunks[0]);

        let selector = ThemeSelector::new(&themes, selected_index, &app.theme.name, &app.theme);
        frame.render_widget(selector, selector_inner);

        // Theme preview (right side)
        let preview_block = Block::default()
            .title("Preview")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border));

        let preview_inner = preview_block.inner(content_chunks[1]);
        frame.render_widget(preview_block, content_chunks[1]);

        // Get the preview theme
        let preview_theme = app
            .theme_loader
            .get(&preview_theme_name)
            .unwrap_or(&app.theme);

        let preview = ThemePreview::new(preview_theme);
        frame.render_widget(preview, preview_inner);
    }

    fn title(&self) -> &str {
        "Settings"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_view_has_correct_title() {
        let view = SettingsView;
        assert_eq!(view.title(), "Settings");
    }

    #[test]
    fn settings_view_default_is_empty_struct() {
        let _view = SettingsView;
    }
}
