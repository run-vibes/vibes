//! Chat history persistence with SQLite storage

mod error;
mod types;
mod query;
mod migrations;
mod store;
mod builder;
mod service;

// Re-exports will be added as types are implemented
