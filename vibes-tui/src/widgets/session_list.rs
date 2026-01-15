//! Session list widget for the dashboard.
//!
//! Displays active sessions with status indicators, agent counts,
//! and branch information. Supports keyboard navigation.

use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::Theme;

/// Session status for display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SessionStatus {
    #[default]
    Running,
    Paused,
    Completed,
    Failed,
}

/// Information about a session for display.
#[derive(Debug, Clone, Default)]
pub struct SessionInfo {
    pub id: String,
    pub status: SessionStatus,
    pub agent_count: usize,
    pub branch: Option<String>,
    pub name: Option<String>,
}

/// Widget displaying a list of sessions.
#[derive(Debug, Clone, Default)]
pub struct SessionListWidget {
    pub sessions: Vec<SessionInfo>,
    pub selected: usize,
    pub scroll_offset: usize,
}

impl SessionListWidget {
    /// Creates a new empty session list widget.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a session list widget with the given sessions.
    pub fn with_sessions(sessions: Vec<SessionInfo>) -> Self {
        Self {
            sessions,
            selected: 0,
            scroll_offset: 0,
        }
    }

    /// Moves selection to the next item, wrapping at the end.
    pub fn select_next(&mut self) {
        if self.sessions.is_empty() {
            return;
        }
        self.selected = (self.selected + 1) % self.sessions.len();
    }

    /// Moves selection to the previous item, wrapping at the start.
    pub fn select_prev(&mut self) {
        if self.sessions.is_empty() {
            return;
        }
        if self.selected == 0 {
            self.selected = self.sessions.len() - 1;
        } else {
            self.selected -= 1;
        }
    }

    /// Updates the scroll offset to ensure the selected item is visible.
    ///
    /// Call this after changing selection and before rendering.
    pub fn ensure_visible(&mut self, visible_height: usize) {
        if visible_height == 0 || self.sessions.is_empty() {
            return;
        }
        // Scroll down if selected is below visible area
        if self.selected >= self.scroll_offset + visible_height {
            self.scroll_offset = self.selected - visible_height + 1;
        }
        // Scroll up if selected is above visible area
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
    }

    /// Returns the currently selected session, if any.
    pub fn selected_session(&self) -> Option<&SessionInfo> {
        self.sessions.get(self.selected)
    }

    /// Converts the widget to a renderable List with the given theme.
    ///
    /// The list respects the scroll_offset when there are many items.
    pub fn to_list(&self, theme: &Theme) -> List<'_> {
        let items: Vec<ListItem> = if self.sessions.is_empty() {
            vec![ListItem::new(Line::from(vec![Span::styled(
                "No sessions",
                Style::default()
                    .fg(theme.fg)
                    .add_modifier(ratatui::style::Modifier::DIM),
            )]))]
        } else {
            self.sessions
                .iter()
                .enumerate()
                .skip(self.scroll_offset)
                .map(|(i, session)| self.session_to_item(session, i == self.selected, theme))
                .collect()
        };

        let block = Block::default()
            .title(" Sessions ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border));

        List::new(items).block(block)
    }

    /// Converts a session to a list item.
    fn session_to_item(
        &self,
        session: &SessionInfo,
        selected: bool,
        theme: &Theme,
    ) -> ListItem<'_> {
        let status_color = match session.status {
            SessionStatus::Running => theme.running,
            SessionStatus::Paused => theme.paused,
            SessionStatus::Completed => theme.completed,
            SessionStatus::Failed => theme.failed,
        };

        let status_text = match session.status {
            SessionStatus::Running => "Running",
            SessionStatus::Paused => "Paused",
            SessionStatus::Completed => "Completed",
            SessionStatus::Failed => "Failed",
        };

        let bullet = if selected { "●" } else { " " };
        let agent_text = if session.agent_count == 1 {
            "1 agent".to_string()
        } else {
            format!("{} agents", session.agent_count)
        };

        let branch = session.branch.as_deref().unwrap_or("-");

        // Truncate session ID to 12 chars
        let id_display = if session.id.len() > 12 {
            &session.id[..12]
        } else {
            &session.id
        };

        let style = if selected {
            Style::default().bg(theme.selection).fg(theme.fg)
        } else {
            Style::default().fg(theme.fg)
        };

        ListItem::new(Line::from(vec![
            Span::styled(format!("{} ", bullet), Style::default().fg(status_color)),
            Span::styled(format!("{:<12}  ", id_display), style),
            Span::styled(format!("{:<10}  ", agent_text), style),
            Span::styled(format!("{:<15}  ", branch), style),
            Span::styled(status_text, Style::default().fg(status_color)),
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    // SessionStatus tests
    #[test]
    fn session_status_default_is_running() {
        let status = SessionStatus::default();
        assert_eq!(status, SessionStatus::Running);
    }

    #[test]
    fn session_status_has_all_variants() {
        let _running = SessionStatus::Running;
        let _paused = SessionStatus::Paused;
        let _completed = SessionStatus::Completed;
        let _failed = SessionStatus::Failed;
    }

    // SessionInfo tests
    #[test]
    fn session_info_can_be_created() {
        let info = SessionInfo {
            id: "session-abc".into(),
            status: SessionStatus::Running,
            agent_count: 2,
            branch: Some("feature/auth".into()),
            name: None,
        };
        assert_eq!(info.id, "session-abc");
        assert_eq!(info.agent_count, 2);
    }

    #[test]
    fn session_info_default_has_no_agents() {
        let info = SessionInfo::default();
        assert_eq!(info.agent_count, 0);
        assert!(info.branch.is_none());
        assert!(info.name.is_none());
    }

    // SessionListWidget tests
    #[test]
    fn widget_new_creates_empty_list() {
        let widget = SessionListWidget::new();
        assert!(widget.sessions.is_empty());
        assert_eq!(widget.selected, 0);
    }

    #[test]
    fn widget_with_sessions_stores_them() {
        let sessions = vec![
            SessionInfo {
                id: "s1".into(),
                status: SessionStatus::Running,
                agent_count: 1,
                branch: None,
                name: None,
            },
            SessionInfo {
                id: "s2".into(),
                status: SessionStatus::Paused,
                agent_count: 2,
                branch: Some("main".into()),
                name: None,
            },
        ];
        let widget = SessionListWidget::with_sessions(sessions.clone());
        assert_eq!(widget.sessions.len(), 2);
    }

    // Navigation tests
    #[test]
    fn select_next_moves_down() {
        let mut widget = SessionListWidget::with_sessions(vec![
            SessionInfo {
                id: "s1".into(),
                ..Default::default()
            },
            SessionInfo {
                id: "s2".into(),
                ..Default::default()
            },
            SessionInfo {
                id: "s3".into(),
                ..Default::default()
            },
        ]);
        assert_eq!(widget.selected, 0);
        widget.select_next();
        assert_eq!(widget.selected, 1);
        widget.select_next();
        assert_eq!(widget.selected, 2);
    }

    #[test]
    fn select_next_wraps_at_end() {
        let mut widget = SessionListWidget::with_sessions(vec![
            SessionInfo {
                id: "s1".into(),
                ..Default::default()
            },
            SessionInfo {
                id: "s2".into(),
                ..Default::default()
            },
        ]);
        widget.selected = 1;
        widget.select_next();
        assert_eq!(widget.selected, 0);
    }

    #[test]
    fn select_prev_moves_up() {
        let mut widget = SessionListWidget::with_sessions(vec![
            SessionInfo {
                id: "s1".into(),
                ..Default::default()
            },
            SessionInfo {
                id: "s2".into(),
                ..Default::default()
            },
        ]);
        widget.selected = 1;
        widget.select_prev();
        assert_eq!(widget.selected, 0);
    }

    #[test]
    fn select_prev_wraps_at_start() {
        let mut widget = SessionListWidget::with_sessions(vec![
            SessionInfo {
                id: "s1".into(),
                ..Default::default()
            },
            SessionInfo {
                id: "s2".into(),
                ..Default::default()
            },
        ]);
        widget.selected = 0;
        widget.select_prev();
        assert_eq!(widget.selected, 1);
    }

    #[test]
    fn select_next_does_nothing_on_empty() {
        let mut widget = SessionListWidget::new();
        widget.select_next();
        assert_eq!(widget.selected, 0);
    }

    #[test]
    fn selected_session_returns_current() {
        let widget = SessionListWidget::with_sessions(vec![
            SessionInfo {
                id: "s1".into(),
                ..Default::default()
            },
            SessionInfo {
                id: "s2".into(),
                ..Default::default()
            },
        ]);
        let selected = widget.selected_session();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "s1");
    }

    #[test]
    fn selected_session_returns_none_on_empty() {
        let widget = SessionListWidget::new();
        assert!(widget.selected_session().is_none());
    }

    // Rendering tests
    #[test]
    fn to_list_creates_list_widget() {
        let theme = crate::vibes_default();
        let widget = SessionListWidget::with_sessions(vec![SessionInfo {
            id: "session-abc".into(),
            status: SessionStatus::Running,
            agent_count: 2,
            branch: Some("feature/auth".into()),
            name: None,
        }]);
        let _list = widget.to_list(&theme);
        // The list should exist - rendering test confirms it compiles
    }

    #[test]
    fn renders_session_rows() {
        let theme = crate::vibes_default();
        let widget = SessionListWidget::with_sessions(vec![SessionInfo {
            id: "session-abc".into(),
            status: SessionStatus::Running,
            agent_count: 2,
            branch: Some("feature/auth".into()),
            name: None,
        }]);

        let backend = TestBackend::new(60, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(widget.to_list(&theme), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        // Check that the session ID appears in the output
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(content.contains("session-abc"));
        assert!(content.contains("feature/auth"));
    }

    #[test]
    fn renders_empty_state_message() {
        let theme = crate::vibes_default();
        let widget = SessionListWidget::new();

        let backend = TestBackend::new(60, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(widget.to_list(&theme), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        assert!(content.contains("No sessions"));
    }

    #[test]
    fn status_indicator_uses_correct_symbol() {
        // The bullet character should appear for running sessions
        let theme = crate::vibes_default();
        let widget = SessionListWidget::with_sessions(vec![SessionInfo {
            id: "s1".into(),
            status: SessionStatus::Running,
            agent_count: 1,
            branch: None,
            name: None,
        }]);

        let backend = TestBackend::new(60, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(widget.to_list(&theme), area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();
        // Check bullet character exists
        assert!(content.contains('●') || content.contains("Running"));
    }

    // Scroll tests
    #[test]
    fn ensure_visible_scrolls_down_when_selection_below() {
        let mut widget = SessionListWidget::with_sessions(
            (0..10)
                .map(|i| SessionInfo {
                    id: format!("s{}", i),
                    ..Default::default()
                })
                .collect(),
        );
        widget.selected = 7;
        widget.ensure_visible(5);
        // With visible height 5 and selected 7, scroll_offset should be 3
        assert_eq!(widget.scroll_offset, 3);
    }

    #[test]
    fn ensure_visible_scrolls_up_when_selection_above() {
        let mut widget = SessionListWidget::with_sessions(
            (0..10)
                .map(|i| SessionInfo {
                    id: format!("s{}", i),
                    ..Default::default()
                })
                .collect(),
        );
        widget.scroll_offset = 5;
        widget.selected = 2;
        widget.ensure_visible(5);
        // Selected 2 is above scroll_offset 5, so should scroll to 2
        assert_eq!(widget.scroll_offset, 2);
    }

    #[test]
    fn ensure_visible_does_nothing_when_already_visible() {
        let mut widget = SessionListWidget::with_sessions(
            (0..10)
                .map(|i| SessionInfo {
                    id: format!("s{}", i),
                    ..Default::default()
                })
                .collect(),
        );
        widget.scroll_offset = 2;
        widget.selected = 4;
        widget.ensure_visible(5);
        // Selected 4 is within visible range [2, 7), scroll should stay at 2
        assert_eq!(widget.scroll_offset, 2);
    }

    #[test]
    fn ensure_visible_handles_empty_list() {
        let mut widget = SessionListWidget::new();
        widget.ensure_visible(5);
        assert_eq!(widget.scroll_offset, 0);
    }
}
