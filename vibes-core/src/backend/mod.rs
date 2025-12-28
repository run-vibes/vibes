//! Claude Code backend abstraction
//!
//! With PTY mode, the backend abstraction is primarily used for testing.
//! Real Claude interaction happens through PtyManager.

pub mod mock;
pub mod traits;

// Re-export key types for convenience
pub use mock::{MockBackend, MockBackendFactory};
pub use traits::{BackendFactory, BackendState, ClaudeBackend};
