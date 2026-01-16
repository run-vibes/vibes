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
use crate::widgets::AgentCard;

/// The swarm view showing agents in a grid with header and footer controls.
#[derive(Debug, Clone)]
pub struct SwarmView {
    swarm_id: SwarmId,
    agents: Vec<AgentId>,
    agent_cards: Vec<AgentCard>,
    #[allow(dead_code)] // Used in tests and future key handling
    selected_index: usize,
}

impl SwarmView {
    /// Creates a new SwarmView for the given swarm.
    pub fn new(swarm_id: SwarmId) -> Self {
        Self {
            swarm_id,
            agents: Vec::new(),
            agent_cards: Vec::new(),
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

    /// Sets the agent cards for this swarm.
    #[cfg(test)]
    pub fn set_agent_cards(&mut self, cards: Vec<AgentCard>) {
        self.agent_cards = cards;
        // Reset selection if out of bounds
        if self.selected_index >= self.agent_cards.len() && !self.agent_cards.is_empty() {
            self.selected_index = 0;
        }
    }

    /// Returns the agent cards (used in tests).
    #[cfg(test)]
    pub fn agent_cards(&self) -> &[AgentCard] {
        &self.agent_cards
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

    /// Renders the agent grid area with AgentCards.
    fn render_agent_area(&self, frame: &mut Frame, area: Rect, app: &App) {
        let block = Block::default()
            .borders(Borders::TOP | Borders::BOTTOM)
            .border_style(Style::default().fg(app.theme.border));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // If we have agent cards, render them in a grid
        if !self.agent_cards.is_empty() {
            self.render_agent_cards(frame, inner, app);
            return;
        }

        // Fallback: Show agent count or empty state (for backwards compatibility)
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

    /// Renders agent cards in a vertical list within the given area.
    fn render_agent_cards(&self, frame: &mut Frame, area: Rect, app: &App) {
        // Each card needs 4 rows (border + 2 content lines + border)
        let card_height = 4u16;
        let card_count = self.agent_cards.len();

        // Calculate how many cards can fit
        let max_cards = (area.height / card_height) as usize;
        let cards_to_render = card_count.min(max_cards);

        if cards_to_render == 0 {
            return;
        }

        // Create constraints for the cards
        let constraints: Vec<Constraint> = (0..cards_to_render)
            .map(|_| Constraint::Length(card_height))
            .chain(std::iter::once(Constraint::Min(0))) // Remaining space
            .collect();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        // Render each card
        for (i, card) in self.agent_cards.iter().take(cards_to_render).enumerate() {
            let mut card_to_render = card.clone();
            // Mark as selected if this is the selected index
            card_to_render.selected = i == self.selected_index;

            card_to_render.render(frame, chunks[i], &app.theme);
        }
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

        // Render merge dialog/results as modal overlays
        if let Some(swarm_state) = app.state.swarms.get(&self.swarm_id) {
            if swarm_state.merge_results.is_visible() {
                swarm_state.merge_results.render(frame, &app.theme);
            } else if swarm_state.merge_dialog.is_visible() {
                swarm_state.merge_dialog.render(frame, &app.theme);
            }
        }
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

    // === AgentCard integration tests ===

    #[test]
    fn swarm_view_set_agent_cards_stores_cards() {
        use crate::widgets::{AgentCard, AgentCardStatus};

        let mut view = SwarmView::new("swarm-1".into());
        let cards = vec![
            AgentCard {
                agent_id: "agent-1".into(),
                name: "Code Reviewer".into(),
                task: "Review PR".into(),
                progress: 0.5,
                status: AgentCardStatus::Running,
                selected: false,
            },
            AgentCard {
                agent_id: "agent-2".into(),
                name: "Security Auditor".into(),
                task: "Security audit".into(),
                progress: 0.8,
                status: AgentCardStatus::Running,
                selected: false,
            },
        ];

        view.set_agent_cards(cards);

        assert_eq!(view.agent_cards().len(), 2);
        assert_eq!(view.agent_cards()[0].name, "Code Reviewer");
        assert_eq!(view.agent_cards()[1].name, "Security Auditor");
    }

    #[test]
    fn swarm_view_renders_agent_cards_with_names() {
        use crate::widgets::{AgentCard, AgentCardStatus};

        let app = App::default();
        let mut view = SwarmView::new("swarm-1".into());
        view.set_agent_cards(vec![AgentCard {
            agent_id: "agent-1".into(),
            name: "Code Reviewer".into(),
            task: "Review PR".into(),
            progress: 0.5,
            status: AgentCardStatus::Running,
            selected: false,
        }]);

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
            content.contains("Code Reviewer"),
            "Expected 'Code Reviewer' in rendered view: {}",
            content
        );
    }

    #[test]
    fn swarm_view_renders_agent_cards_with_progress_bars() {
        use crate::widgets::{AgentCard, AgentCardStatus};

        let app = App::default();
        let mut view = SwarmView::new("swarm-1".into());
        view.set_agent_cards(vec![AgentCard {
            agent_id: "agent-1".into(),
            name: "Agent".into(),
            task: "Task".into(),
            progress: 0.5,
            status: AgentCardStatus::Running,
            selected: false,
        }]);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                view.render(f, f.area(), &app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();

        // Should contain progress bar characters
        assert!(
            content.contains('█') && content.contains('░'),
            "Expected progress bar characters in: {}",
            content
        );
    }

    #[test]
    fn swarm_view_selected_agent_card_has_highlight() {
        use crate::widgets::{AgentCard, AgentCardStatus};

        let app = App::default();
        let mut view = SwarmView::new("swarm-1".into());
        view.set_agent_cards(vec![
            AgentCard {
                agent_id: "agent-1".into(),
                name: "Agent 1".into(),
                task: "Task 1".into(),
                progress: 0.3,
                status: AgentCardStatus::Running,
                selected: false,
            },
            AgentCard {
                agent_id: "agent-2".into(),
                name: "Agent 2".into(),
                task: "Task 2".into(),
                progress: 0.7,
                status: AgentCardStatus::Running,
                selected: false,
            },
        ]);

        // Select first agent (index 0 is selected by default)
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                view.render(f, f.area(), &app);
            })
            .unwrap();

        // Check that selection background color is present
        let buffer = terminal.backend().buffer();
        let has_selection_bg = buffer.content().iter().any(|c| c.bg == app.theme.selection);
        assert!(
            has_selection_bg,
            "Selected agent card should have selection background"
        );
    }

    #[test]
    fn swarm_view_multiple_cards_rendered_in_grid() {
        use crate::widgets::{AgentCard, AgentCardStatus};

        let app = App::default();
        let mut view = SwarmView::new("swarm-1".into());
        view.set_agent_cards(vec![
            AgentCard {
                agent_id: "agent-1".into(),
                name: "Agent Alpha".into(),
                task: "Task A".into(),
                progress: 0.25,
                status: AgentCardStatus::Running,
                selected: false,
            },
            AgentCard {
                agent_id: "agent-2".into(),
                name: "Agent Beta".into(),
                task: "Task B".into(),
                progress: 0.75,
                status: AgentCardStatus::Completed,
                selected: false,
            },
        ]);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                view.render(f, f.area(), &app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();

        // Both agent names should be visible
        assert!(
            content.contains("Agent Alpha"),
            "Expected 'Agent Alpha' in: {}",
            content
        );
        assert!(
            content.contains("Agent Beta"),
            "Expected 'Agent Beta' in: {}",
            content
        );
    }

    // ==================== Merge Dialog Integration Tests ====================

    #[test]
    fn swarm_view_renders_merge_dialog_when_visible() {
        use crate::state::SwarmState;
        use crate::widgets::CompletedAgent;

        let mut app = App::default();
        let view = SwarmView::new("swarm-test".into());

        // Set up swarm state with visible merge dialog
        let mut swarm_state = SwarmState::default();
        swarm_state.merge_dialog.show(
            vec![CompletedAgent {
                agent_id: "agent-1".into(),
                name: "Security Agent".into(),
                task_summary: "Security review".into(),
            }],
            0,
        );
        app.state.swarms.insert("swarm-test".into(), swarm_state);

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
            content.contains("Merge Results"),
            "Expected merge dialog title 'Merge Results' in: {}",
            content
        );
        assert!(
            content.contains("Security Agent"),
            "Expected agent name in merge dialog: {}",
            content
        );
    }

    #[test]
    fn swarm_view_renders_merge_results_when_visible() {
        use crate::state::SwarmState;
        use crate::widgets::ResultSection;

        let mut app = App::default();
        let view = SwarmView::new("swarm-results".into());

        // Set up swarm state with visible merge results
        let mut swarm_state = SwarmState::default();
        swarm_state.merge_results.show(vec![ResultSection {
            agent_name: "Code Review Agent".into(),
            content: "No issues found.".into(),
        }]);
        app.state.swarms.insert("swarm-results".into(), swarm_state);

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
            content.contains("Merged Results"),
            "Expected results view title 'Merged Results' in: {}",
            content
        );
        assert!(
            content.contains("Code Review Agent"),
            "Expected agent section in results: {}",
            content
        );
    }

    #[test]
    fn swarm_view_results_take_priority_over_dialog() {
        use crate::state::SwarmState;
        use crate::widgets::{CompletedAgent, ResultSection};

        let mut app = App::default();
        let view = SwarmView::new("swarm-priority".into());

        // Set up swarm state with BOTH dialog and results visible
        let mut swarm_state = SwarmState::default();
        swarm_state.merge_dialog.show(
            vec![CompletedAgent {
                agent_id: "agent-1".into(),
                name: "Dialog Agent".into(),
                task_summary: "Task".into(),
            }],
            0,
        );
        swarm_state.merge_results.show(vec![ResultSection {
            agent_name: "Results Agent".into(),
            content: "Output".into(),
        }]);
        app.state
            .swarms
            .insert("swarm-priority".into(), swarm_state);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                view.render(f, f.area(), &app);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|cell| cell.symbol()).collect();

        // Results should be shown, not dialog
        assert!(
            content.contains("Merged Results"),
            "Results view should take priority"
        );
        assert!(
            content.contains("Results Agent"),
            "Results content should be shown"
        );
    }

    #[test]
    fn swarm_view_footer_shows_merge_keybinding() {
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
            content.contains("[m]"),
            "Expected '[m]' keybinding hint in footer: {}",
            content
        );
        assert!(
            content.contains("Merge"),
            "Expected 'Merge' label in footer: {}",
            content
        );
    }
}
