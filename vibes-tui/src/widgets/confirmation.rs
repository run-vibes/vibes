//! Confirmation dialog widget for destructive actions.
//!
//! Displays a confirmation prompt with yes/no options for actions
//! like cancel or restart that have significant consequences.

use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::Theme;

/// Type of confirmation action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationType {
    /// Confirm cancel agent execution.
    Cancel,
    /// Confirm restart agent (loses progress).
    Restart,
}

impl ConfirmationType {
    /// Returns the dialog title for this confirmation type.
    pub fn title(&self) -> &'static str {
        match self {
            ConfirmationType::Cancel => "Cancel Agent",
            ConfirmationType::Restart => "Restart Agent",
        }
    }

    /// Returns the dialog message for this confirmation type.
    pub fn message(&self) -> &'static str {
        match self {
            ConfirmationType::Cancel => "Cancel agent execution?",
            ConfirmationType::Restart => "Restart agent? Current progress will be lost.",
        }
    }
}

/// Modal dialog for confirming destructive actions.
#[derive(Debug, Clone, Default)]
pub struct ConfirmationDialog {
    /// Whether the dialog is currently visible.
    visible: bool,
    /// Type of confirmation being requested.
    confirmation_type: Option<ConfirmationType>,
}

impl ConfirmationDialog {
    /// Creates a new confirmation dialog.
    pub fn new() -> Self {
        Self::default()
    }

    /// Shows the dialog for the given confirmation type.
    pub fn show(&mut self, confirmation_type: ConfirmationType) {
        self.confirmation_type = Some(confirmation_type);
        self.visible = true;
    }

    /// Hides the dialog.
    pub fn hide(&mut self) {
        self.visible = false;
        self.confirmation_type = None;
    }

    /// Returns true if the dialog is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Returns the current confirmation type, if visible.
    pub fn confirmation_type(&self) -> Option<ConfirmationType> {
        self.confirmation_type
    }

    /// Renders the dialog centered on the screen.
    pub fn render(&self, frame: &mut Frame, theme: &Theme) {
        if !self.visible {
            return;
        }

        let Some(confirmation_type) = self.confirmation_type else {
            return;
        };

        let area = frame.area();
        // Small centered dialog box
        let dialog_width = 50u16.min(area.width.saturating_sub(4));
        let dialog_height = 5u16;
        let x = (area.width.saturating_sub(dialog_width)) / 2;
        let y = (area.height.saturating_sub(dialog_height)) / 2;
        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        // Clear the background
        frame.render_widget(Clear, dialog_area);

        // Dialog block with title
        let title = format!(" {} ", confirmation_type.title());
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.warning));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Message and controls
        let message = confirmation_type.message();
        let lines = vec![
            Line::from(Span::styled(message, Style::default().fg(theme.fg))),
            Line::default(),
            Line::from(vec![
                Span::styled("[y]", Style::default().fg(theme.accent)),
                Span::styled(" Yes  ", Style::default().fg(theme.fg)),
                Span::styled("[n]", Style::default().fg(theme.accent)),
                Span::styled(" No", Style::default().fg(theme.fg)),
            ]),
        ];

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, inner);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== ConfirmationType Tests ====================

    #[test]
    fn confirmation_type_cancel_has_correct_title() {
        assert_eq!(ConfirmationType::Cancel.title(), "Cancel Agent");
    }

    #[test]
    fn confirmation_type_restart_has_correct_title() {
        assert_eq!(ConfirmationType::Restart.title(), "Restart Agent");
    }

    #[test]
    fn confirmation_type_cancel_has_correct_message() {
        assert_eq!(
            ConfirmationType::Cancel.message(),
            "Cancel agent execution?"
        );
    }

    #[test]
    fn confirmation_type_restart_has_correct_message() {
        assert_eq!(
            ConfirmationType::Restart.message(),
            "Restart agent? Current progress will be lost."
        );
    }

    #[test]
    fn confirmation_types_are_distinct() {
        assert_ne!(ConfirmationType::Cancel, ConfirmationType::Restart);
    }

    // ==================== ConfirmationDialog Tests ====================

    #[test]
    fn confirmation_dialog_new_is_hidden() {
        let dialog = ConfirmationDialog::new();
        assert!(!dialog.is_visible());
    }

    #[test]
    fn confirmation_dialog_default_is_hidden() {
        let dialog = ConfirmationDialog::default();
        assert!(!dialog.is_visible());
    }

    #[test]
    fn confirmation_dialog_show_makes_visible() {
        let mut dialog = ConfirmationDialog::new();
        dialog.show(ConfirmationType::Cancel);
        assert!(dialog.is_visible());
    }

    #[test]
    fn confirmation_dialog_show_sets_type() {
        let mut dialog = ConfirmationDialog::new();
        dialog.show(ConfirmationType::Restart);
        assert_eq!(dialog.confirmation_type(), Some(ConfirmationType::Restart));
    }

    #[test]
    fn confirmation_dialog_hide_makes_invisible() {
        let mut dialog = ConfirmationDialog::new();
        dialog.show(ConfirmationType::Cancel);
        dialog.hide();
        assert!(!dialog.is_visible());
    }

    #[test]
    fn confirmation_dialog_hide_clears_type() {
        let mut dialog = ConfirmationDialog::new();
        dialog.show(ConfirmationType::Cancel);
        dialog.hide();
        assert_eq!(dialog.confirmation_type(), None);
    }

    #[test]
    fn confirmation_dialog_new_has_no_type() {
        let dialog = ConfirmationDialog::new();
        assert_eq!(dialog.confirmation_type(), None);
    }

    // ==================== Rendering Tests ====================

    #[test]
    fn confirmation_dialog_does_not_render_when_hidden() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let dialog = ConfirmationDialog::new();

        let backend = TestBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                dialog.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        // Should not contain any dialog content
        assert!(
            !content.contains("Cancel Agent"),
            "Hidden dialog should not render"
        );
    }

    #[test]
    fn confirmation_dialog_renders_cancel_title() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let mut dialog = ConfirmationDialog::new();
        dialog.show(ConfirmationType::Cancel);

        let backend = TestBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                dialog.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("Cancel Agent"),
            "Expected 'Cancel Agent' in: {}",
            content
        );
    }

    #[test]
    fn confirmation_dialog_renders_restart_message() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let mut dialog = ConfirmationDialog::new();
        dialog.show(ConfirmationType::Restart);

        let backend = TestBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                dialog.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("progress will be lost"),
            "Expected restart message in: {}",
            content
        );
    }

    #[test]
    fn confirmation_dialog_renders_yes_no_controls() {
        use ratatui::{Terminal, backend::TestBackend};

        let theme = crate::vibes_default();
        let mut dialog = ConfirmationDialog::new();
        dialog.show(ConfirmationType::Cancel);

        let backend = TestBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                dialog.render(f, &theme);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(content.contains("[y]"), "Expected '[y]' in: {}", content);
        assert!(content.contains("[n]"), "Expected '[n]' in: {}", content);
    }
}
