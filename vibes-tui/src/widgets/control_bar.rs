//! Control bar widget for the agent detail view.
//!
//! Displays context-sensitive agent controls (pause, resume, cancel, restart)
//! based on the current agent status.

use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::Theme;

/// The execution status of an agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AgentStatus {
    /// Agent is currently executing.
    #[default]
    Running,
    /// Agent execution is paused.
    Paused,
    /// Agent is waiting for permission approval.
    WaitingForPermission,
    /// Agent completed successfully.
    Completed,
    /// Agent execution failed.
    Failed,
    /// Agent was cancelled by user.
    Cancelled,
}

/// Widget for displaying agent control actions.
#[derive(Debug, Clone, Default)]
pub struct ControlBar {
    status: AgentStatus,
}

impl ControlBar {
    /// Creates a new control bar with the given status.
    pub fn new(status: AgentStatus) -> Self {
        Self { status }
    }

    /// Updates the agent status.
    pub fn set_status(&mut self, status: AgentStatus) {
        self.status = status;
    }

    /// Returns the current agent status.
    pub fn status(&self) -> AgentStatus {
        self.status
    }

    /// Converts the widget to a renderable Paragraph with the given theme.
    pub fn to_paragraph(&self, theme: &Theme) -> Paragraph<'_> {
        let spans = match self.status {
            AgentStatus::Running | AgentStatus::WaitingForPermission => {
                // Running: [p] Pause  [c] Cancel  [r] Restart  [Esc] Back
                vec![
                    Span::styled("[p]", Style::default().fg(theme.accent)),
                    Span::styled(" Pause  ", Style::default().fg(theme.fg)),
                    Span::styled("[c]", Style::default().fg(theme.accent)),
                    Span::styled(" Cancel  ", Style::default().fg(theme.fg)),
                    Span::styled("[r]", Style::default().fg(theme.accent)),
                    Span::styled(" Restart  ", Style::default().fg(theme.fg)),
                    Span::styled("[Esc]", Style::default().fg(theme.accent)),
                    Span::styled(" Back", Style::default().fg(theme.fg)),
                ]
            }
            AgentStatus::Paused => {
                // Paused: [p] Resume  [c] Cancel  [r] Restart  [Esc] Back
                vec![
                    Span::styled("[p]", Style::default().fg(theme.accent)),
                    Span::styled(" Resume  ", Style::default().fg(theme.fg)),
                    Span::styled("[c]", Style::default().fg(theme.accent)),
                    Span::styled(" Cancel  ", Style::default().fg(theme.fg)),
                    Span::styled("[r]", Style::default().fg(theme.accent)),
                    Span::styled(" Restart  ", Style::default().fg(theme.fg)),
                    Span::styled("[Esc]", Style::default().fg(theme.accent)),
                    Span::styled(" Back", Style::default().fg(theme.fg)),
                ]
            }
            AgentStatus::Completed | AgentStatus::Failed | AgentStatus::Cancelled => {
                // Terminated states: [r] Restart  [Esc] Back (other actions dimmed)
                vec![
                    Span::styled("[p]", theme.dim),
                    Span::styled(" ---  ", theme.dim),
                    Span::styled("[c]", theme.dim),
                    Span::styled(" ---  ", theme.dim),
                    Span::styled("[r]", Style::default().fg(theme.accent)),
                    Span::styled(" Restart  ", Style::default().fg(theme.fg)),
                    Span::styled("[Esc]", Style::default().fg(theme.accent)),
                    Span::styled(" Back", Style::default().fg(theme.fg)),
                ]
            }
        };

        Paragraph::new(Line::from(spans))
    }

    /// Returns whether pause/resume is available for the current status.
    pub fn can_pause_resume(&self) -> bool {
        matches!(
            self.status,
            AgentStatus::Running | AgentStatus::Paused | AgentStatus::WaitingForPermission
        )
    }

    /// Returns whether cancel is available for the current status.
    pub fn can_cancel(&self) -> bool {
        matches!(
            self.status,
            AgentStatus::Running | AgentStatus::Paused | AgentStatus::WaitingForPermission
        )
    }

    /// Returns whether restart is available for the current status.
    pub fn can_restart(&self) -> bool {
        // Always available
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    // ==================== AgentStatus Tests ====================

    #[test]
    fn agent_status_defaults_to_running() {
        let status = AgentStatus::default();
        assert_eq!(status, AgentStatus::Running);
    }

    #[test]
    fn agent_status_variants_are_distinct() {
        assert_ne!(AgentStatus::Running, AgentStatus::Paused);
        assert_ne!(AgentStatus::Paused, AgentStatus::WaitingForPermission);
        assert_ne!(AgentStatus::WaitingForPermission, AgentStatus::Completed);
        assert_ne!(AgentStatus::Completed, AgentStatus::Failed);
        assert_ne!(AgentStatus::Failed, AgentStatus::Cancelled);
    }

    // ==================== ControlBar Tests ====================

    #[test]
    fn control_bar_new_sets_status() {
        let bar = ControlBar::new(AgentStatus::Paused);
        assert_eq!(bar.status(), AgentStatus::Paused);
    }

    #[test]
    fn control_bar_default_is_running() {
        let bar = ControlBar::default();
        assert_eq!(bar.status(), AgentStatus::Running);
    }

    #[test]
    fn control_bar_set_status_updates_status() {
        let mut bar = ControlBar::new(AgentStatus::Running);
        bar.set_status(AgentStatus::Paused);
        assert_eq!(bar.status(), AgentStatus::Paused);
    }

    // ==================== Action Availability Tests ====================

    #[test]
    fn control_bar_running_can_pause() {
        let bar = ControlBar::new(AgentStatus::Running);
        assert!(bar.can_pause_resume());
    }

    #[test]
    fn control_bar_paused_can_resume() {
        let bar = ControlBar::new(AgentStatus::Paused);
        assert!(bar.can_pause_resume());
    }

    #[test]
    fn control_bar_waiting_can_pause() {
        let bar = ControlBar::new(AgentStatus::WaitingForPermission);
        assert!(bar.can_pause_resume());
    }

    #[test]
    fn control_bar_completed_cannot_pause() {
        let bar = ControlBar::new(AgentStatus::Completed);
        assert!(!bar.can_pause_resume());
    }

    #[test]
    fn control_bar_failed_cannot_pause() {
        let bar = ControlBar::new(AgentStatus::Failed);
        assert!(!bar.can_pause_resume());
    }

    #[test]
    fn control_bar_cancelled_cannot_pause() {
        let bar = ControlBar::new(AgentStatus::Cancelled);
        assert!(!bar.can_pause_resume());
    }

    #[test]
    fn control_bar_running_can_cancel() {
        let bar = ControlBar::new(AgentStatus::Running);
        assert!(bar.can_cancel());
    }

    #[test]
    fn control_bar_completed_cannot_cancel() {
        let bar = ControlBar::new(AgentStatus::Completed);
        assert!(!bar.can_cancel());
    }

    #[test]
    fn control_bar_all_states_can_restart() {
        for status in [
            AgentStatus::Running,
            AgentStatus::Paused,
            AgentStatus::WaitingForPermission,
            AgentStatus::Completed,
            AgentStatus::Failed,
            AgentStatus::Cancelled,
        ] {
            let bar = ControlBar::new(status);
            assert!(bar.can_restart(), "Expected can_restart for {:?}", status);
        }
    }

    // ==================== Rendering Tests ====================

    #[test]
    fn control_bar_running_shows_pause_action() {
        let theme = crate::vibes_default();
        let bar = ControlBar::new(AgentStatus::Running);

        let backend = TestBackend::new(60, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(bar.to_paragraph(&theme), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("Pause"),
            "Expected 'Pause' in: {}",
            content
        );
        assert!(
            !content.contains("Resume"),
            "Expected no 'Resume' in: {}",
            content
        );
    }

    #[test]
    fn control_bar_paused_shows_resume_action() {
        let theme = crate::vibes_default();
        let bar = ControlBar::new(AgentStatus::Paused);

        let backend = TestBackend::new(60, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(bar.to_paragraph(&theme), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("Resume"),
            "Expected 'Resume' in: {}",
            content
        );
        assert!(
            !content.contains("Pause"),
            "Expected no 'Pause' in: {}",
            content
        );
    }

    #[test]
    fn control_bar_completed_shows_restart_only() {
        let theme = crate::vibes_default();
        let bar = ControlBar::new(AgentStatus::Completed);

        let backend = TestBackend::new(60, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(bar.to_paragraph(&theme), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("Restart"),
            "Expected 'Restart' in: {}",
            content
        );
        // Pause and Cancel should show as dimmed placeholders
        assert!(
            content.contains("---"),
            "Expected dimmed placeholder in: {}",
            content
        );
    }

    #[test]
    fn control_bar_running_shows_all_controls() {
        let theme = crate::vibes_default();
        let bar = ControlBar::new(AgentStatus::Running);

        let backend = TestBackend::new(70, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(bar.to_paragraph(&theme), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(content.contains("[p]"), "Expected '[p]' in: {}", content);
        assert!(content.contains("[c]"), "Expected '[c]' in: {}", content);
        assert!(content.contains("[r]"), "Expected '[r]' in: {}", content);
        assert!(
            content.contains("[Esc]"),
            "Expected '[Esc]' in: {}",
            content
        );
    }

    #[test]
    fn control_bar_waiting_shows_pause_action() {
        let theme = crate::vibes_default();
        let bar = ControlBar::new(AgentStatus::WaitingForPermission);

        let backend = TestBackend::new(60, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(bar.to_paragraph(&theme), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(
            content.contains("Pause"),
            "Expected 'Pause' in: {}",
            content
        );
    }
}
