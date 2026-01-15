//! Widgets for the vibes TUI.
//!
//! This module contains reusable widget components for rendering
//! UI elements in the terminal.

mod session_list;
mod stats_bar;

pub use session_list::{SessionInfo, SessionListWidget, SessionStatus};
pub use stats_bar::StatsBarWidget;
