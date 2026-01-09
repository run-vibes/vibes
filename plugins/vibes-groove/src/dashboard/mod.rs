//! Dashboard module for groove WebSocket API
//!
//! Provides real-time dashboard updates via WebSocket with topic-based subscriptions.

mod handler;
mod types;

pub use handler::DashboardHandler;
pub use types::*;
