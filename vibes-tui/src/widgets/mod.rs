//! Widgets for the vibes TUI.
//!
//! This module contains reusable widget components for rendering
//! UI elements in the terminal.

mod activity_feed;
mod agent_card;
mod confirmation;
mod control_bar;
mod diff_modal;
mod merge_dialog;
mod merge_results;
mod output_panel;
mod permission;
mod session_list;
mod stats_bar;

pub use activity_feed::{ActivityEvent, ActivityFeedWidget};
#[allow(unused_imports)] // AgentCardStatus used in tests and future swarm integration
pub use agent_card::{AgentCard, AgentCardStatus};
pub use confirmation::{ConfirmationDialog, ConfirmationType};
pub use control_bar::{AgentStatus, ControlBar};
pub use diff_modal::DiffModal;
#[allow(unused_imports)] // Public API for merge dialog types
pub use merge_dialog::{CompletedAgent, MergeDialog, MergeStrategy};
#[allow(unused_imports)] // Public API for merge results types
pub use merge_results::{MergeResultsView, ResultSection};
#[allow(unused_imports)] // Public API for creating output lines
pub use output_panel::{OutputBuffer, OutputLine, OutputLineType, OutputPanelWidget};
#[allow(unused_imports)] // Public API for permission request IDs
pub use permission::{
    PermissionDetails, PermissionId, PermissionRequest, PermissionType, PermissionWidget,
};
pub use session_list::{SessionInfo, SessionListWidget, SessionStatus};
pub use stats_bar::{ConnectionStatus, StatsBarWidget};
