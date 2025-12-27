//! Chat history persistence with SQLite storage

mod builder;
mod error;
mod migrations;
mod query;
mod service;
mod store;
mod types;

pub use builder::MessageBuilder;
pub use error::HistoryError;
pub use query::{
    MessageListResult, MessageQuery, SessionListResult, SessionQuery, SortField, SortOrder,
};
pub use service::HistoryService;
pub use store::{HistoryStore, SqliteHistoryStore};
pub use types::{HistoricalMessage, HistoricalSession, MessageRole, SessionSummary};
