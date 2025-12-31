//! Iggy server lifecycle management.

/// State of the Iggy server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IggyState {
    /// Server is stopped.
    Stopped,
    /// Server is starting.
    Starting,
    /// Server is running.
    Running,
}

/// Manages the Iggy server subprocess lifecycle.
#[derive(Debug)]
pub struct IggyManager {
    // Will be populated in subsequent tasks
    _placeholder: (),
}
