//! Activity feed widget for the dashboard.
//!
//! Displays recent activity events with timestamps, agent IDs,
//! and action descriptions in a scrollable list.

use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

use crate::Theme;

/// An activity event for display in the feed.
#[derive(Debug, Clone)]
pub struct ActivityEvent {
    /// Time of the event (e.g., "14:32").
    pub time: String,
    /// Agent or source identifier (e.g., "agent-1").
    pub source: String,
    /// Description of the activity (e.g., "completed task \"implement login\"").
    pub description: String,
}

/// Widget displaying a scrollable list of recent activity events.
#[derive(Debug, Clone, Default)]
pub struct ActivityFeedWidget {
    pub events: Vec<ActivityEvent>,
    pub scroll_offset: usize,
}

impl ActivityFeedWidget {
    /// Creates a new empty activity feed widget.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an activity feed widget with the given events.
    pub fn with_events(events: Vec<ActivityEvent>) -> Self {
        Self {
            events,
            scroll_offset: 0,
        }
    }

    /// Adds an event to the beginning of the feed.
    ///
    /// New events appear at the top.
    pub fn push_event(&mut self, event: ActivityEvent) {
        self.events.insert(0, event);
    }

    /// Updates the scroll offset to show more recent or older events.
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Scrolls down to show older events.
    pub fn scroll_down(&mut self) {
        if !self.events.is_empty() && self.scroll_offset < self.events.len().saturating_sub(1) {
            self.scroll_offset += 1;
        }
    }

    /// Converts the widget to a renderable List with the given theme.
    pub fn to_list(&self, theme: &Theme) -> List<'_> {
        let items: Vec<ListItem> = if self.events.is_empty() {
            vec![ListItem::new(Line::from(vec![Span::styled(
                "No recent activity",
                Style::default()
                    .fg(theme.fg)
                    .add_modifier(ratatui::style::Modifier::DIM),
            )]))]
        } else {
            self.events
                .iter()
                .skip(self.scroll_offset)
                .map(|event| self.event_to_item(event, theme))
                .collect()
        };

        let block = Block::default()
            .title(" Recent Activity ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border));

        List::new(items).block(block)
    }

    /// Converts an event to a list item.
    fn event_to_item(&self, event: &ActivityEvent, theme: &Theme) -> ListItem<'static> {
        ListItem::new(Line::from(vec![
            Span::styled(
                format!("{} ", event.time),
                Style::default()
                    .fg(theme.fg)
                    .add_modifier(ratatui::style::Modifier::DIM),
            ),
            Span::styled(
                format!("{} ", event.source),
                Style::default().fg(theme.accent),
            ),
            Span::styled(event.description.clone(), Style::default().fg(theme.fg)),
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    // ActivityEvent tests
    #[test]
    fn activity_event_can_be_created() {
        let event = ActivityEvent {
            time: "14:32".into(),
            source: "agent-1".into(),
            description: "completed task \"implement login\"".into(),
        };
        assert_eq!(event.time, "14:32");
        assert_eq!(event.source, "agent-1");
        assert!(event.description.contains("completed task"));
    }

    // ActivityFeedWidget construction tests
    #[test]
    fn widget_new_creates_empty_feed() {
        let widget = ActivityFeedWidget::new();
        assert!(widget.events.is_empty());
        assert_eq!(widget.scroll_offset, 0);
    }

    #[test]
    fn widget_with_events_stores_them() {
        let events = vec![
            ActivityEvent {
                time: "14:32".into(),
                source: "agent-1".into(),
                description: "completed task".into(),
            },
            ActivityEvent {
                time: "14:31".into(),
                source: "agent-2".into(),
                description: "waiting for permission".into(),
            },
        ];
        let widget = ActivityFeedWidget::with_events(events);
        assert_eq!(widget.events.len(), 2);
    }

    // push_event tests
    #[test]
    fn push_event_adds_to_beginning() {
        let mut widget = ActivityFeedWidget::with_events(vec![ActivityEvent {
            time: "14:31".into(),
            source: "agent-1".into(),
            description: "old event".into(),
        }]);

        widget.push_event(ActivityEvent {
            time: "14:32".into(),
            source: "agent-2".into(),
            description: "new event".into(),
        });

        assert_eq!(widget.events.len(), 2);
        assert_eq!(widget.events[0].time, "14:32");
        assert_eq!(widget.events[1].time, "14:31");
    }

    // Scroll tests
    #[test]
    fn scroll_up_decreases_offset() {
        let mut widget = ActivityFeedWidget::with_events(
            (0..10)
                .map(|i| ActivityEvent {
                    time: format!("14:{:02}", i),
                    source: "agent".into(),
                    description: "event".into(),
                })
                .collect(),
        );
        widget.scroll_offset = 5;
        widget.scroll_up();
        assert_eq!(widget.scroll_offset, 4);
    }

    #[test]
    fn scroll_up_stops_at_zero() {
        let mut widget = ActivityFeedWidget::with_events(vec![ActivityEvent {
            time: "14:32".into(),
            source: "agent".into(),
            description: "event".into(),
        }]);
        widget.scroll_offset = 0;
        widget.scroll_up();
        assert_eq!(widget.scroll_offset, 0);
    }

    #[test]
    fn scroll_down_increases_offset() {
        let mut widget = ActivityFeedWidget::with_events(
            (0..10)
                .map(|i| ActivityEvent {
                    time: format!("14:{:02}", i),
                    source: "agent".into(),
                    description: "event".into(),
                })
                .collect(),
        );
        widget.scroll_offset = 0;
        widget.scroll_down();
        assert_eq!(widget.scroll_offset, 1);
    }

    #[test]
    fn scroll_down_stops_at_end() {
        let mut widget = ActivityFeedWidget::with_events(vec![ActivityEvent {
            time: "14:32".into(),
            source: "agent".into(),
            description: "event".into(),
        }]);
        widget.scroll_offset = 0;
        widget.scroll_down();
        assert_eq!(widget.scroll_offset, 0);
    }

    #[test]
    fn scroll_down_on_empty_does_nothing() {
        let mut widget = ActivityFeedWidget::new();
        widget.scroll_down();
        assert_eq!(widget.scroll_offset, 0);
    }

    // Rendering tests
    #[test]
    fn to_list_creates_list_widget() {
        let theme = crate::vibes_default();
        let widget = ActivityFeedWidget::with_events(vec![ActivityEvent {
            time: "14:32".into(),
            source: "agent-1".into(),
            description: "completed task".into(),
        }]);
        let _list = widget.to_list(&theme);
        // Should compile and not panic
    }

    #[test]
    fn renders_event_rows() {
        let theme = crate::vibes_default();
        let widget = ActivityFeedWidget::with_events(vec![ActivityEvent {
            time: "14:32".into(),
            source: "agent-1".into(),
            description: "completed task".into(),
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
        assert!(
            content.contains("14:32"),
            "Expected '14:32' in: {}",
            content
        );
        assert!(
            content.contains("agent-1"),
            "Expected 'agent-1' in: {}",
            content
        );
        assert!(
            content.contains("completed task"),
            "Expected 'completed task' in: {}",
            content
        );
    }

    #[test]
    fn renders_empty_state_message() {
        let theme = crate::vibes_default();
        let widget = ActivityFeedWidget::new();

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
        assert!(
            content.contains("No recent activity"),
            "Expected 'No recent activity' in: {}",
            content
        );
    }

    #[test]
    fn renders_title_recent_activity() {
        let theme = crate::vibes_default();
        let widget = ActivityFeedWidget::new();

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
        assert!(
            content.contains("Recent Activity"),
            "Expected 'Recent Activity' in: {}",
            content
        );
    }

    #[test]
    fn scroll_affects_rendered_items() {
        let theme = crate::vibes_default();
        let mut widget = ActivityFeedWidget::with_events(vec![
            ActivityEvent {
                time: "14:32".into(),
                source: "first".into(),
                description: "first event".into(),
            },
            ActivityEvent {
                time: "14:31".into(),
                source: "second".into(),
                description: "second event".into(),
            },
        ]);
        widget.scroll_offset = 1;

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
        // First event should be skipped
        assert!(
            !content.contains("first event"),
            "Should not contain 'first event' in: {}",
            content
        );
        assert!(
            content.contains("second event"),
            "Expected 'second event' in: {}",
            content
        );
    }
}
