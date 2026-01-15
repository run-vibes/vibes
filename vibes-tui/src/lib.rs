//! Terminal UI for vibes.
//!
//! This crate provides a CRT-inspired terminal interface for vibes,
//! built on ratatui and crossterm.

mod app;
mod client;
mod keybindings;
mod state;
mod terminal;
mod theme;
mod views;
mod widgets;

pub use app::App;
pub use client::TuiClient;
pub use keybindings::{Action, KeyBindings};
pub use state::{AgentId, AgentState, AppState, Mode, Selection, SessionId, SwarmId, SwarmState};
pub use terminal::{VibesTerminal, install_panic_hook, restore_terminal, setup_terminal};
pub use theme::{Theme, vibes_default};
pub use views::{DashboardView, View, ViewRenderer, ViewStack};
pub use widgets::{
    ActivityEvent, ActivityFeedWidget, SessionInfo, SessionListWidget, SessionStatus,
};
