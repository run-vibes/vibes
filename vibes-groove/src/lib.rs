//! vibes-groove - Continual learning storage
//!
//! This crate provides CozoDB-based storage for the groove continual learning system.
//! It implements three-tier storage (user/project/enterprise) with learning storage,
//! adaptive parameters, and semantic search capabilities.

// Module declarations - commented out until implemented
pub mod error;
pub mod store;
pub mod types;
// pub mod storage;
// pub mod config;
// pub mod export;

// Re-exports - commented out until modules are implemented
pub use error::{GrooveError, Result};
pub use store::{LearningStore, ParamStore};
pub use types::*;
// pub use storage::GrooveStorage;
// pub use config::GrooveConfig;
// pub use export::{GrooveExport, ImportStats};
