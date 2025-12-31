//! Iggy-backed event log for vibes.
//!
//! This crate provides persistent event storage using Iggy as the backing store.
//! It implements a producer/consumer model with independent offset tracking
//! per consumer group.
//!
//! # Key Types
//!
//! - [`EventLog`] - Trait for appending events and creating consumers
//! - [`EventConsumer`] - Trait for polling events with offset tracking
//! - [`IggyEventLog`] - Iggy-backed implementation of EventLog
//! - [`IggyManager`] - Manages Iggy server subprocess lifecycle

pub mod config;
pub mod error;
pub mod manager;
pub mod memory;
pub mod traits;

// Re-exports
pub use config::IggyConfig;
pub use error::{Error, Result};
pub use manager::{IggyManager, IggyState};
pub use memory::InMemoryEventLog;
pub use traits::{EventBatch, EventConsumer, EventLog, Offset, SeekPosition};
