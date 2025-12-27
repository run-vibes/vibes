//! Session management

pub mod manager;
pub mod ownership;
pub mod state;

// Re-export key types for convenience
pub use manager::SessionManager;
pub use ownership::{ClientId, SessionOwnership};
pub use state::{Session, SessionState};
