//! Claude Code backend abstraction

pub mod mock;
pub mod print_mode;
pub mod traits;

// Re-export key types for convenience
pub use mock::MockBackend;
pub use print_mode::{PrintModeBackend, PrintModeBackendFactory, PrintModeConfig};
pub use traits::{BackendFactory, BackendState, ClaudeBackend};
