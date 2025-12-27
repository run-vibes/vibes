//! Query parameter types for history search

use serde::{Deserialize, Serialize};
use crate::session::SessionState;
use super::types::{SessionSummary, HistoricalMessage};

/// Sort field for session queries
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    #[default]
    CreatedAt,
    LastAccessedAt,
    MessageCount,
    TotalTokens,
}

impl SortField {
    pub fn as_column(&self) -> &'static str {
        match self {
            Self::CreatedAt => "created_at",
            Self::LastAccessedAt => "last_accessed_at",
            Self::MessageCount => "message_count",
            Self::TotalTokens => "(total_input_tokens + total_output_tokens)",
        }
    }
}

/// Sort order
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

impl SortOrder {
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

/// Query parameters for listing sessions
#[derive(Debug, Clone, Default)]
pub struct SessionQuery {
    /// Full-text search across message content
    pub search: Option<String>,
    /// Filter by session name (LIKE pattern)
    pub name: Option<String>,
    /// Filter by session state
    pub state: Option<SessionState>,
    /// Filter sessions that used this tool
    pub tool: Option<String>,
    /// Minimum total tokens
    pub min_tokens: Option<u32>,
    /// Created after this timestamp
    pub after: Option<i64>,
    /// Created before this timestamp
    pub before: Option<i64>,
    /// Max results (default 20, max 100)
    pub limit: u32,
    /// Offset for pagination
    pub offset: u32,
    /// Sort field
    pub sort: SortField,
    /// Sort order
    pub order: SortOrder,
}

impl SessionQuery {
    pub fn new() -> Self {
        Self {
            limit: 20,
            ..Default::default()
        }
    }

    /// Clamp limit to valid range
    pub fn effective_limit(&self) -> u32 {
        self.limit.clamp(1, 100)
    }
}

/// Query parameters for listing messages
#[derive(Debug, Clone, Default)]
pub struct MessageQuery {
    /// Max results (default 50)
    pub limit: u32,
    /// Offset for pagination
    pub offset: u32,
    /// Filter by role
    pub role: Option<super::types::MessageRole>,
}

impl MessageQuery {
    pub fn new() -> Self {
        Self {
            limit: 50,
            ..Default::default()
        }
    }

    pub fn effective_limit(&self) -> u32 {
        self.limit.clamp(1, 500)
    }
}

/// Paginated session list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionListResult {
    pub sessions: Vec<SessionSummary>,
    pub total: u32,
    pub limit: u32,
    pub offset: u32,
}

/// Paginated message list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageListResult {
    pub messages: Vec<HistoricalMessage>,
    pub total: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_query_defaults() {
        let query = SessionQuery::new();
        assert_eq!(query.limit, 20);
        assert_eq!(query.offset, 0);
        assert_eq!(query.sort, SortField::CreatedAt);
        assert_eq!(query.order, SortOrder::Desc);
    }

    #[test]
    fn test_effective_limit_clamping() {
        let mut query = SessionQuery::new();
        query.limit = 0;
        assert_eq!(query.effective_limit(), 1);

        query.limit = 500;
        assert_eq!(query.effective_limit(), 100);
    }

    #[test]
    fn test_sort_field_column() {
        assert_eq!(SortField::CreatedAt.as_column(), "created_at");
        assert_eq!(SortField::TotalTokens.as_column(), "(total_input_tokens + total_output_tokens)");
    }

    #[test]
    fn test_sort_order_sql() {
        assert_eq!(SortOrder::Asc.as_sql(), "ASC");
        assert_eq!(SortOrder::Desc.as_sql(), "DESC");
    }
}
