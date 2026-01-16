//! Application state for the TUI.

use std::collections::HashMap;

use crate::widgets::OutputBuffer;

/// Unique identifier for a session.
pub type SessionId = String;

/// Unique identifier for an agent.
pub type AgentId = String;

/// Unique identifier for a swarm.
pub type SwarmId = String;

/// State for a single agent including output buffer.
#[derive(Debug, Clone, Default)]
pub struct AgentState {
    /// Output buffer for the agent's output stream.
    pub output: OutputBuffer,
}

/// Placeholder for swarm state (expanded in later stories).
#[derive(Debug, Clone, Default)]
pub struct SwarmState;

/// The current UI mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Normal,
    Command,
    Search,
    Help,
}

/// Currently selected item in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Selection {
    #[default]
    None,
    Session(usize),
    Agent(usize),
    Swarm(usize),
}

/// Core application state.
#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub session: Option<SessionId>,
    pub agents: HashMap<AgentId, AgentState>,
    pub swarms: HashMap<SwarmId, SwarmState>,
    pub selected: Selection,
    pub mode: Mode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_defaults_to_normal() {
        let mode = Mode::default();
        assert_eq!(mode, Mode::Normal);
    }

    #[test]
    fn selection_defaults_to_none() {
        let selection = Selection::default();
        assert_eq!(selection, Selection::None);
    }

    #[test]
    fn app_state_defaults_to_empty() {
        let state = AppState::default();
        assert!(state.session.is_none());
        assert!(state.agents.is_empty());
        assert!(state.swarms.is_empty());
        assert_eq!(state.selected, Selection::None);
        assert_eq!(state.mode, Mode::Normal);
    }

    #[test]
    fn mode_variants_are_distinct() {
        assert_ne!(Mode::Normal, Mode::Command);
        assert_ne!(Mode::Command, Mode::Search);
        assert_ne!(Mode::Search, Mode::Help);
    }

    #[test]
    fn selection_variants_hold_indices() {
        let session = Selection::Session(0);
        let agent = Selection::Agent(5);
        let swarm = Selection::Swarm(10);

        assert_eq!(session, Selection::Session(0));
        assert_eq!(agent, Selection::Agent(5));
        assert_eq!(swarm, Selection::Swarm(10));
    }

    #[test]
    fn app_state_can_store_session() {
        let state = AppState {
            session: Some("test-session".to_string()),
            ..Default::default()
        };
        assert_eq!(state.session, Some("test-session".to_string()));
    }

    #[test]
    fn app_state_can_store_agents() {
        let mut state = AppState::default();
        state
            .agents
            .insert("agent-1".to_string(), AgentState::default());
        assert_eq!(state.agents.len(), 1);
        assert!(state.agents.contains_key("agent-1"));
    }

    #[test]
    fn agent_state_has_output_buffer() {
        let state = AgentState::default();
        assert!(state.output.is_empty());
    }
}
