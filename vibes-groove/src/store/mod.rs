//! Storage traits and implementations for vibes-groove

mod cozo;
mod schema;
mod traits;

pub use cozo::CozoStore;
pub use schema::{Migration, CURRENT_SCHEMA_VERSION, INITIAL_SCHEMA, MIGRATIONS};
pub use traits::{LearningStore, ParamStore};
