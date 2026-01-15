//! Widgets for the vibes TUI.
//!
//! This module contains reusable widget components for rendering
//! UI elements in the terminal.

mod activity_feed;
mod session_list;
mod stats_bar;

pub use activity_feed::{ActivityEvent, ActivityFeedWidget};
pub use session_list::{SessionInfo, SessionListWidget, SessionStatus};
pub use stats_bar::{ConnectionStatus, StatsBarWidget};
