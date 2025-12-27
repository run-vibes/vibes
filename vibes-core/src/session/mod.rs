//! Session management

pub mod lifecycle;
pub mod manager;
pub mod ownership;
pub mod state;

// Re-export key types for convenience
pub use lifecycle::{DisconnectResult, SessionLifecycleManager};
pub use manager::SessionManager;
pub use ownership::{ClientId, SessionOwnership};
pub use state::{Session, SessionState};
