//! Session management

pub mod manager;
mod ownership;
pub mod state;

// Re-export key types for convenience
pub use manager::SessionManager;
pub use state::{Session, SessionState};
