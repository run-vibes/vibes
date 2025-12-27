//! Chat history persistence with SQLite storage

mod error;
mod types;
mod query;
mod migrations;
mod store;
mod builder;
mod service;

pub use error::HistoryError;
pub use types::{HistoricalSession, HistoricalMessage, MessageRole, SessionSummary};
pub use query::{SessionQuery, MessageQuery, SessionListResult, MessageListResult, SortField, SortOrder};
