//! View system for vibes TUI.
//!
//! This module provides:
//! - `View` enum for all available views
//! - `ViewStack` for stack-based navigation
//! - `ViewRenderer` trait for custom view rendering
//! - Built-in view implementations (Dashboard, Agent, etc.)

mod agent;
mod dashboard;
mod stack;
mod swarm;
mod traits;

pub use agent::AgentView;
pub use dashboard::DashboardView;
pub use stack::{View, ViewStack};
pub use swarm::SwarmView;
pub use traits::ViewRenderer;
