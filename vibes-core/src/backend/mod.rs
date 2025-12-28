//! Claude Code backend abstraction

pub mod mock;
pub mod print_mode;
pub mod slow_mock;
pub mod traits;

// Re-export key types for convenience
pub use mock::MockBackend;
pub use print_mode::{PrintModeBackend, PrintModeBackendFactory, PrintModeConfig};
pub use slow_mock::SlowMockBackend;
pub use traits::{BackendFactory, BackendState, ClaudeBackend};
