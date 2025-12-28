//! Event system for vibes

pub mod bus;
pub mod memory;
pub mod types;

// Re-export key types for convenience
pub use bus::{EventBus, EventSeq};
pub use memory::MemoryEventBus;
pub use types::{ClaudeEvent, InputSource, Usage, VibesEvent};
