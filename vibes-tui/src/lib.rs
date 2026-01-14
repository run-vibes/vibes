//! Terminal UI for vibes.
//!
//! This crate provides a CRT-inspired terminal interface for vibes,
//! built on ratatui and crossterm.

mod app;
mod state;
mod terminal;
mod theme;

pub use app::{App, KeyBindings, VibesClient, ViewStack};
pub use state::{AgentId, AgentState, AppState, Mode, Selection, SessionId, SwarmId, SwarmState};
pub use terminal::{VibesTerminal, install_panic_hook, restore_terminal, setup_terminal};
pub use theme::{Theme, vibes_default};
