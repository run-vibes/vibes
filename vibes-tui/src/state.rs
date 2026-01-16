//! Application state for the TUI.

use std::collections::HashMap;

use crate::widgets::{
    ConfirmationDialog, ControlBar, DiffModal, MergeDialog, MergeResultsView, OutputBuffer,
    PermissionWidget,
};

/// Unique identifier for a session.
pub type SessionId = String;

/// Unique identifier for an agent.
pub type AgentId = String;

/// Unique identifier for a swarm.
pub type SwarmId = String;

/// State for a single agent including output buffer, permission widget, and diff modal.
#[derive(Debug, Clone, Default)]
pub struct AgentState {
    /// Output buffer for the agent's output stream.
    pub output: OutputBuffer,
    /// Permission widget for handling permission requests.
    pub permission: PermissionWidget,
    /// Diff modal for viewing file changes.
    pub diff_modal: DiffModal,
    /// Control bar for agent actions.
    pub control_bar: ControlBar,
    /// Confirmation dialog for destructive actions.
    pub confirmation: ConfirmationDialog,
}

/// State for a swarm including merge dialog and results view.
#[derive(Debug, Clone, Default)]
pub struct SwarmState {
    /// Merge dialog for confirming result aggregation.
    pub merge_dialog: MergeDialog,
    /// Merged results view for displaying combined output.
    pub merge_results: MergeResultsView,
}

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

    #[test]
    fn agent_state_has_permission_widget() {
        let state = AgentState::default();
        assert!(!state.permission.has_pending());
    }

    #[test]
    fn agent_state_permission_can_store_request() {
        use crate::widgets::{PermissionDetails, PermissionRequest, PermissionType};
        use chrono::Utc;
        use std::path::PathBuf;

        let mut state = AgentState::default();
        state.permission.set_pending(PermissionRequest {
            id: "req-1".to_string(),
            request_type: PermissionType::FileWrite,
            description: "Write to file".to_string(),
            details: PermissionDetails::FileWrite {
                path: PathBuf::from("test.rs"),
                content: "test".to_string(),
                original: None,
            },
            timestamp: Utc::now(),
        });

        assert!(state.permission.has_pending());
    }

    #[test]
    fn agent_state_has_diff_modal() {
        let state = AgentState::default();
        assert!(!state.diff_modal.is_visible());
    }

    #[test]
    fn agent_state_diff_modal_can_be_shown() {
        let mut state = AgentState::default();
        state.diff_modal.show("test.rs", Some("old"), "new");
        assert!(state.diff_modal.is_visible());
    }

    #[test]
    fn agent_state_has_control_bar() {
        use crate::widgets::AgentStatus;
        let state = AgentState::default();
        assert_eq!(state.control_bar.status(), AgentStatus::Running);
    }

    #[test]
    fn agent_state_control_bar_can_be_updated() {
        use crate::widgets::AgentStatus;
        let mut state = AgentState::default();
        state.control_bar.set_status(AgentStatus::Paused);
        assert_eq!(state.control_bar.status(), AgentStatus::Paused);
    }

    #[test]
    fn agent_state_has_confirmation_dialog() {
        let state = AgentState::default();
        assert!(!state.confirmation.is_visible());
    }

    #[test]
    fn agent_state_confirmation_can_be_shown() {
        use crate::widgets::ConfirmationType;
        let mut state = AgentState::default();
        state.confirmation.show(ConfirmationType::Cancel);
        assert!(state.confirmation.is_visible());
    }

    // ==================== SwarmState Tests ====================

    #[test]
    fn swarm_state_defaults_with_hidden_dialogs() {
        let state = SwarmState::default();
        assert!(!state.merge_dialog.is_visible());
        assert!(!state.merge_results.is_visible());
    }

    #[test]
    fn swarm_state_has_merge_dialog() {
        use crate::widgets::CompletedAgent;
        let mut state = SwarmState::default();
        state.merge_dialog.show(
            vec![CompletedAgent {
                agent_id: "a1".into(),
                name: "Test".into(),
                task_summary: "Task".into(),
            }],
            0,
        );
        assert!(state.merge_dialog.is_visible());
    }

    #[test]
    fn swarm_state_has_merge_results() {
        use crate::widgets::ResultSection;
        let mut state = SwarmState::default();
        state.merge_results.show(vec![ResultSection {
            agent_name: "Agent".into(),
            content: "Results".into(),
        }]);
        assert!(state.merge_results.is_visible());
    }

    #[test]
    fn app_state_can_store_swarms() {
        let mut state = AppState::default();
        state
            .swarms
            .insert("swarm-1".to_string(), SwarmState::default());
        assert_eq!(state.swarms.len(), 1);
        assert!(state.swarms.contains_key("swarm-1"));
    }
}
