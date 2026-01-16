//! Swarm view - displays a swarm's agents in a grid layout with controls.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::traits::ViewRenderer;
use crate::App;
use crate::state::{AgentId, SwarmId};

/// The swarm view showing agents in a grid with header and footer controls.
#[derive(Debug, Clone)]
pub struct SwarmView {
    swarm_id: SwarmId,
    agents: Vec<AgentId>,
    #[allow(dead_code)] // Used in tests and future key handling
    selected_index: usize,
}

impl SwarmView {
    /// Creates a new SwarmView for the given swarm.
    pub fn new(swarm_id: SwarmId) -> Self {
        Self {
            swarm_id,
            agents: Vec::new(),
            selected_index: 0,
        }
    }

    /// Returns the swarm ID (used in tests).
    #[cfg(test)]
    pub fn swarm_id(&self) -> &SwarmId {
        &self.swarm_id
    }

    /// Returns the list of agents in this swarm (used in tests).
    #[cfg(test)]
    pub fn agents(&self) -> &[AgentId] {
        &self.agents
    }

    /// Returns the currently selected index (used in tests).
    #[cfg(test)]
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Returns the currently selected agent, if any (used in tests).
    #[cfg(test)]
    pub fn selected_agent(&self) -> Option<&AgentId> {
        self.agents.get(self.selected_index)
    }

    /// Moves selection to the next agent (used in tests).
    #[cfg(test)]
    pub fn select_next(&mut self) {
        if !self.agents.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.agents.len();
        }
    }

    /// Moves selection to the previous agent (used in tests).
    #[cfg(test)]
    pub fn select_prev(&mut self) {
        if !self.agents.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.agents.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    /// Sets the list of agents for this swarm (used in tests).
    #[cfg(test)]
    pub fn set_agents(&mut self, agents: Vec<AgentId>) {
        self.agents = agents;
        // Reset selection if out of bounds
        if self.selected_index >= self.agents.len() && !self.agents.is_empty() {
            self.selected_index = 0;
        }
    }

    /// Renders the header section with swarm metadata.
    fn render_header(&self, frame: &mut Frame, area: Rect, app: &App) {
        // Row 1: Swarm name and status
        let row1 = Line::from(vec![
            Span::styled("Strategy: ", Style::default().fg(app.theme.fg)),
            Span::styled("parallel", app.theme.dim),
            Span::raw("   "),
            Span::styled("Status: ", Style::default().fg(app.theme.fg)),
            Span::styled("Running", Style::default().fg(app.theme.running)),
        ]);

        // Row 2: Task description
        let row2 = Line::from(vec![
            Span::styled("Task: ", Style::default().fg(app.theme.fg)),
            Span::styled("implement feature across multiple files", app.theme.dim),
        ]);

        let header = Paragraph::new(vec![row1, row2]);
        frame.render_widget(header, area);
    }

    /// Renders the agent grid area (placeholder for now).
    fn render_agent_area(&self, frame: &mut Frame, area: Rect, app: &App) {
        let block = Block::default()
            .borders(Borders::TOP | Borders::BOTTOM)
            .border_style(Style::default().fg(app.theme.border));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Show agent count or empty state
        let content = if self.agents.is_empty() {
            Line::from(Span::styled("No agents in swarm", app.theme.dim))
        } else {
            Line::from(vec![
                Span::styled(
                    format!("{} agents", self.agents.len()),
                    Style::default().fg(app.theme.fg),
                ),
                Span::raw(" - "),
                Span::styled("(agent cards rendered in later story)", app.theme.dim),
            ])
        };

        let paragraph = Paragraph::new(content);
        frame.render_widget(paragraph, inner);
    }

    /// Renders the footer with keybinding hints.
    fn render_footer(&self, frame: &mut Frame, area: Rect, app: &App) {
        let hints = Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(app.theme.accent)),
            Span::raw(" Agent detail  "),
            Span::styled("[m]", Style::default().fg(app.theme.accent)),
            Span::raw(" Merge results  "),
            Span::styled("[Esc]", Style::default().fg(app.theme.accent)),
            Span::raw(" Back"),
        ]);

        let footer = Paragraph::new(hints);
        frame.render_widget(footer, area);
    }
}

impl ViewRenderer for SwarmView {
    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        // Main container block with swarm ID in title
        let title = format!(" Swarm: {} ", self.swarm_id);
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split into: header (2 lines), agent area (flex), footer (1 line)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Header
                Constraint::Min(4),    // Agent grid area
                Constraint::Length(1), // Footer
            ])
            .split(inner);

        // Render sections
        self.render_header(frame, chunks[0], app);
        self.render_agent_area(frame, chunks[1], app);
        self.render_footer(frame, chunks[2], app);
    }

    fn title(&self) -> &str {
        "Swarm"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn swarm_view_stores_swarm_id() {
        let view = SwarmView::new("swarm-123".into());
        assert_eq!(view.swarm_id(), "swarm-123");
    }

    #[test]
    fn swarm_view_has_correct_title() {
        let view = SwarmView::new("swarm-1".into());
        assert_eq!(view.title(), "Swarm");
    }

    #[test]
    fn swarm_view_clone_preserves_id() {
        let view = SwarmView::new("swarm-clone-test".into());
        let cloned = view.clone();
        assert_eq!(view.swarm_id(), cloned.swarm_id());
    }

    #[test]
    fn swarm_view_starts_with_empty_agents() {
        let view = SwarmView::new("swarm-1".into());
        assert!(view.agents().is_empty());
        assert_eq!(view.selected_index(), 0);
    }

    #[test]
    fn swarm_view_selected_agent_returns_none_when_empty() {
        let view = SwarmView::new("swarm-1".into());
        assert!(view.selected_agent().is_none());
    }

    #[test]
    fn swarm_view_set_agents_updates_list() {
        let mut view = SwarmView::new("swarm-1".into());
        view.set_agents(vec!["agent-1".into(), "agent-2".into()]);

        assert_eq!(view.agents().len(), 2);
        assert_eq!(view.agents()[0], "agent-1");
        assert_eq!(view.agents()[1], "agent-2");
    }

    #[test]
    fn swarm_view_selected_agent_returns_first_after_set() {
        let mut view = SwarmView::new("swarm-1".into());
        view.set_agents(vec!["agent-1".into(), "agent-2".into()]);

        assert_eq!(view.selected_agent(), Some(&"agent-1".to_string()));
    }

    #[test]
    fn swarm_view_select_next_cycles_through_agents() {
        let mut view = SwarmView::new("swarm-1".into());
        view.set_agents(vec!["agent-1".into(), "agent-2".into(), "agent-3".into()]);

        assert_eq!(view.selected_index(), 0);

        view.select_next();
        assert_eq!(view.selected_index(), 1);

        view.select_next();
        assert_eq!(view.selected_index(), 2);

        // Wraps around
        view.select_next();
        assert_eq!(view.selected_index(), 0);
    }

    #[test]
    fn swarm_view_select_prev_cycles_backwards() {
        let mut view = SwarmView::new("swarm-1".into());
        view.set_agents(vec!["agent-1".into(), "agent-2".into(), "agent-3".into()]);

        assert_eq!(view.selected_index(), 0);

        // Wraps to end
        view.select_prev();
        assert_eq!(view.selected_index(), 2);

        view.select_prev();
        assert_eq!(view.selected_index(), 1);

        view.select_prev();
        assert_eq!(view.selected_index(), 0);
    }

    #[test]
    fn swarm_view_select_next_noop_when_empty() {
        let mut view = SwarmView::new("swarm-1".into());
        view.select_next();
        assert_eq!(view.selected_index(), 0);
    }

    #[test]
    fn swarm_view_select_prev_noop_when_empty() {
        let mut view = SwarmView::new("swarm-1".into());
        view.select_prev();
        assert_eq!(view.selected_index(), 0);
    }

    #[test]
    fn swarm_view_set_agents_resets_selection_if_out_of_bounds() {
        let mut view = SwarmView::new("swarm-1".into());
        view.set_agents(vec!["agent-1".into(), "agent-2".into(), "agent-3".into()]);
        view.select_next();
        view.select_next();
        assert_eq!(view.selected_index(), 2);

        // Now set fewer agents
        view.set_agents(vec!["agent-a".into()]);
        assert_eq!(view.selected_index(), 0);
    }

    #[test]
    fn swarm_view_renders_without_panic() {
        let app = App::default();
        let view = SwarmView::new("swarm-1".into());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                view.render(f, f.area(), &app);
            })
            .unwrap();
    }

    #[test]
    fn swarm_view_renders_title_with_swarm_id() {
        let app = App::default();
        let view = SwarmView::new("test-swarm".into());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                view.render(f, f.area(), &app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();

        assert!(
            content.contains("Swarm: test-swarm"),
            "Expected 'Swarm: test-swarm' in rendered view"
        );
    }

    #[test]
    fn swarm_view_renders_header_with_strategy() {
        let app = App::default();
        let view = SwarmView::new("swarm-1".into());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                view.render(f, f.area(), &app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();

        assert!(
            content.contains("Strategy:"),
            "Expected 'Strategy:' in header"
        );
        assert!(content.contains("Status:"), "Expected 'Status:' in header");
        assert!(content.contains("Task:"), "Expected 'Task:' in header");
    }

    #[test]
    fn swarm_view_renders_footer_keybindings() {
        let app = App::default();
        let view = SwarmView::new("swarm-1".into());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                view.render(f, f.area(), &app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();

        assert!(
            content.contains("[Enter]"),
            "Expected '[Enter]' in footer keybindings"
        );
        assert!(
            content.contains("[Esc]"),
            "Expected '[Esc]' in footer keybindings"
        );
        assert!(
            content.contains("Agent detail"),
            "Expected 'Agent detail' in footer"
        );
        assert!(content.contains("Back"), "Expected 'Back' in footer");
    }

    #[test]
    fn swarm_view_renders_empty_state_when_no_agents() {
        let app = App::default();
        let view = SwarmView::new("swarm-1".into());
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                view.render(f, f.area(), &app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();

        assert!(
            content.contains("No agents in swarm"),
            "Expected 'No agents in swarm' when empty"
        );
    }

    #[test]
    fn swarm_view_renders_agent_count_when_has_agents() {
        let app = App::default();
        let mut view = SwarmView::new("swarm-1".into());
        view.set_agents(vec!["agent-1".into(), "agent-2".into()]);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                view.render(f, f.area(), &app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();

        assert!(
            content.contains("2 agents"),
            "Expected '2 agents' in agent area"
        );
    }
}
