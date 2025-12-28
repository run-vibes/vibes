//! History storage trait and SQLite implementation

use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

use super::error::HistoryError;
use super::migrations::Migrator;
use super::query::{MessageListResult, MessageQuery, SessionListResult, SessionQuery};
use super::types::{HistoricalMessage, HistoricalSession, MessageRole, SessionSummary};
use crate::events::InputSource;
use crate::session::SessionState;

/// History storage trait
pub trait HistoryStore: Send + Sync {
    fn save_session(&self, session: &HistoricalSession) -> Result<(), HistoryError>;
    fn get_session(&self, id: &str) -> Result<Option<HistoricalSession>, HistoryError>;
    fn update_session(&self, session: &HistoricalSession) -> Result<(), HistoryError>;
    fn delete_session(&self, id: &str) -> Result<(), HistoryError>;
    fn list_sessions(&self, query: &SessionQuery) -> Result<SessionListResult, HistoryError>;

    fn save_message(&self, message: &HistoricalMessage) -> Result<i64, HistoryError>;
    fn get_messages(
        &self,
        session_id: &str,
        query: &MessageQuery,
    ) -> Result<MessageListResult, HistoryError>;

    fn update_session_stats(
        &self,
        session_id: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<(), HistoryError>;
}

/// SQLite-backed history store
pub struct SqliteHistoryStore {
    conn: Mutex<Connection>,
}

impl SqliteHistoryStore {
    /// Open or create database at path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, HistoryError> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.init()?;
        Ok(store)
    }

    /// Open in-memory database (for testing)
    pub fn open_in_memory() -> Result<Self, HistoryError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.init()?;
        Ok(store)
    }

    /// Run migrations
    fn init(&self) -> Result<(), HistoryError> {
        let conn = self.conn.lock().unwrap();
        let migrator = Migrator::new(&conn);
        migrator.migrate()
    }

    fn state_to_str(state: &SessionState) -> &'static str {
        match state {
            SessionState::Idle => "Idle",
            SessionState::Processing => "Processing",
            SessionState::WaitingPermission { .. } => "WaitingPermission",
            SessionState::Failed { .. } => "Failed",
            SessionState::Finished => "Finished",
        }
    }

    /// Convert database state string back to SessionState enum.
    ///
    /// WaitingPermission and Failed use placeholder values because the database
    /// only stores the variant name (e.g., "Failed"), not the inner fields.
    /// This is intentional - the database stores error details separately in
    /// error_message column, and for history display purposes the state
    /// category is sufficient.
    fn str_to_state(s: &str) -> SessionState {
        match s {
            "Processing" => SessionState::Processing,
            "WaitingPermission" => SessionState::WaitingPermission {
                // Placeholder - DB only stores variant name
                request_id: String::new(),
                tool: String::new(),
            },
            "Failed" => SessionState::Failed {
                // Placeholder - error details in session.error_message
                message: String::new(),
                recoverable: false,
            },
            "Finished" => SessionState::Finished,
            _ => SessionState::Idle,
        }
    }

    fn row_to_message(&self, row: &rusqlite::Row) -> Result<HistoricalMessage, rusqlite::Error> {
        let role_str: String = row.get(2)?;
        let source_str: String = row.get(9)?;
        Ok(HistoricalMessage {
            id: row.get(0)?,
            session_id: row.get(1)?,
            role: MessageRole::parse(&role_str).unwrap_or(MessageRole::User),
            content: row.get(3)?,
            tool_name: row.get(4)?,
            tool_id: row.get(5)?,
            created_at: row.get(6)?,
            input_tokens: row.get(7)?,
            output_tokens: row.get(8)?,
            source: InputSource::parse(&source_str).unwrap_or(InputSource::Unknown),
        })
    }
}

impl HistoryStore for SqliteHistoryStore {
    fn save_session(&self, session: &HistoricalSession) -> Result<(), HistoryError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO sessions (id, name, claude_session_id, state, created_at, last_accessed_at, total_input_tokens, total_output_tokens, message_count, error_message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                session.id,
                session.name,
                session.claude_session_id,
                Self::state_to_str(&session.state),
                session.created_at,
                session.last_accessed_at,
                session.total_input_tokens,
                session.total_output_tokens,
                session.message_count,
                session.error_message,
            ],
        )?;
        Ok(())
    }

    fn get_session(&self, id: &str) -> Result<Option<HistoricalSession>, HistoryError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, claude_session_id, state, created_at, last_accessed_at,
                    total_input_tokens, total_output_tokens, message_count, error_message
             FROM sessions WHERE id = ?1",
        )?;

        let mut rows = stmt.query([id])?;
        match rows.next()? {
            Some(row) => {
                let state_str: String = row.get(3)?;
                Ok(Some(HistoricalSession {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    claude_session_id: row.get(2)?,
                    state: Self::str_to_state(&state_str),
                    created_at: row.get(4)?,
                    last_accessed_at: row.get(5)?,
                    total_input_tokens: row.get(6)?,
                    total_output_tokens: row.get(7)?,
                    message_count: row.get(8)?,
                    error_message: row.get(9)?,
                }))
            }
            None => Ok(None),
        }
    }

    fn update_session(&self, session: &HistoricalSession) -> Result<(), HistoryError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE sessions SET
                name = ?2, claude_session_id = ?3, state = ?4,
                last_accessed_at = ?5, total_input_tokens = ?6,
                total_output_tokens = ?7, message_count = ?8, error_message = ?9
             WHERE id = ?1",
            rusqlite::params![
                session.id,
                session.name,
                session.claude_session_id,
                Self::state_to_str(&session.state),
                session.last_accessed_at,
                session.total_input_tokens,
                session.total_output_tokens,
                session.message_count,
                session.error_message,
            ],
        )?;
        Ok(())
    }

    fn delete_session(&self, id: &str) -> Result<(), HistoryError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sessions WHERE id = ?1", [id])?;
        Ok(())
    }

    fn list_sessions(&self, query: &SessionQuery) -> Result<SessionListResult, HistoryError> {
        let conn = self.conn.lock().unwrap();

        // Build WHERE clauses
        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref name) = query.name {
            conditions.push(format!("s.name LIKE ?{}", params.len() + 1));
            params.push(Box::new(format!("%{}%", name)));
        }

        if let Some(ref state) = query.state {
            conditions.push(format!("s.state = ?{}", params.len() + 1));
            params.push(Box::new(Self::state_to_str(state).to_string()));
        }

        if let Some(min_tokens) = query.min_tokens {
            conditions.push(format!(
                "(s.total_input_tokens + s.total_output_tokens) >= ?{}",
                params.len() + 1
            ));
            params.push(Box::new(min_tokens as i64));
        }

        if let Some(after) = query.after {
            conditions.push(format!("s.created_at >= ?{}", params.len() + 1));
            params.push(Box::new(after));
        }

        if let Some(before) = query.before {
            conditions.push(format!("s.created_at <= ?{}", params.len() + 1));
            params.push(Box::new(before));
        }

        // Tool filter requires join
        let tool_join = if let Some(ref tool) = query.tool {
            conditions.push(format!("m.tool_name = ?{}", params.len() + 1));
            params.push(Box::new(tool.clone()));
            "INNER JOIN messages m ON s.id = m.session_id"
        } else {
            ""
        };

        // FTS search requires subquery
        let _fts_join = if let Some(ref search) = query.search {
            conditions.push(format!(
                "s.id IN (SELECT DISTINCT m2.session_id FROM messages m2
                 INNER JOIN messages_fts ON m2.id = messages_fts.rowid
                 WHERE messages_fts MATCH ?{})",
                params.len() + 1
            ));
            params.push(Box::new(search.clone()));
            ""
        } else {
            ""
        };

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // Count query
        let count_sql = format!(
            "SELECT COUNT(DISTINCT s.id) FROM sessions s {} {}",
            tool_join, where_clause
        );

        let total: u32 = {
            let mut stmt = conn.prepare(&count_sql)?;
            let params_refs: Vec<&dyn rusqlite::ToSql> =
                params.iter().map(|p| p.as_ref()).collect();
            stmt.query_row(params_refs.as_slice(), |row| row.get(0))?
        };

        // Main query with pagination
        let order = format!("{} {}", query.sort.as_column(), query.order.as_sql());
        let select_sql = format!(
            "SELECT DISTINCT s.id, s.name, s.state, s.created_at, s.last_accessed_at,
                    s.message_count, s.total_input_tokens + s.total_output_tokens as total_tokens,
                    COALESCE((SELECT SUBSTR(content, 1, 100) FROM messages WHERE session_id = s.id ORDER BY created_at LIMIT 1), '') as preview
             FROM sessions s {} {}
             ORDER BY {} LIMIT ?{} OFFSET ?{}",
            tool_join, where_clause, order, params.len() + 1, params.len() + 2
        );

        params.push(Box::new(query.effective_limit() as i64));
        params.push(Box::new(query.offset as i64));

        let sessions = {
            let mut stmt = conn.prepare(&select_sql)?;
            let params_refs: Vec<&dyn rusqlite::ToSql> =
                params.iter().map(|p| p.as_ref()).collect();
            let rows = stmt.query_map(params_refs.as_slice(), |row| {
                let state_str: String = row.get(2)?;
                Ok(SessionSummary {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    state: Self::str_to_state(&state_str),
                    created_at: row.get(3)?,
                    last_accessed_at: row.get(4)?,
                    message_count: row.get(5)?,
                    total_tokens: row.get(6)?,
                    preview: row.get(7)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        Ok(SessionListResult {
            sessions,
            total,
            limit: query.effective_limit(),
            offset: query.offset,
        })
    }

    fn save_message(&self, message: &HistoricalMessage) -> Result<i64, HistoryError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO messages (session_id, role, content, tool_name, tool_id, created_at, input_tokens, output_tokens, source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                message.session_id,
                message.role.as_str(),
                message.content,
                message.tool_name,
                message.tool_id,
                message.created_at,
                message.input_tokens,
                message.output_tokens,
                message.source.as_str(),
            ],
        )?;

        // Update session message count
        conn.execute(
            "UPDATE sessions SET message_count = message_count + 1, last_accessed_at = ?2 WHERE id = ?1",
            rusqlite::params![message.session_id, message.created_at],
        )?;

        Ok(conn.last_insert_rowid())
    }

    fn get_messages(
        &self,
        session_id: &str,
        query: &MessageQuery,
    ) -> Result<MessageListResult, HistoryError> {
        let conn = self.conn.lock().unwrap();

        // Get total count
        let total: u32 = if let Some(role) = &query.role {
            conn.query_row(
                "SELECT COUNT(*) FROM messages WHERE session_id = ?1 AND role = ?2",
                rusqlite::params![session_id, role.as_str()],
                |row| row.get(0),
            )?
        } else {
            conn.query_row(
                "SELECT COUNT(*) FROM messages WHERE session_id = ?1",
                [session_id],
                |row| row.get(0),
            )?
        };

        // Build query
        let sql = if query.role.is_some() {
            "SELECT id, session_id, role, content, tool_name, tool_id, created_at, input_tokens, output_tokens, source
             FROM messages WHERE session_id = ?1 AND role = ?2
             ORDER BY created_at ASC LIMIT ?3 OFFSET ?4"
        } else {
            "SELECT id, session_id, role, content, tool_name, tool_id, created_at, input_tokens, output_tokens, source
             FROM messages WHERE session_id = ?1
             ORDER BY created_at ASC LIMIT ?2 OFFSET ?3"
        };

        let messages = if let Some(role) = &query.role {
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map(
                rusqlite::params![
                    session_id,
                    role.as_str(),
                    query.effective_limit(),
                    query.offset
                ],
                |row| self.row_to_message(row),
            )?;
            rows.collect::<Result<Vec<_>, _>>()?
        } else {
            let mut stmt = conn.prepare(sql)?;
            let rows = stmt.query_map(
                rusqlite::params![session_id, query.effective_limit(), query.offset],
                |row| self.row_to_message(row),
            )?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        Ok(MessageListResult { messages, total })
    }

    fn update_session_stats(
        &self,
        session_id: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<(), HistoryError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE sessions SET
                total_input_tokens = total_input_tokens + ?2,
                total_output_tokens = total_output_tokens + ?3,
                last_accessed_at = strftime('%s', 'now')
             WHERE id = ?1",
            rusqlite::params![session_id, input_tokens, output_tokens],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Task 7: Session CRUD tests
    #[test]
    fn test_session_save_and_get() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        let session = HistoricalSession::new("test-123".into(), Some("Test Session".into()));
        store.save_session(&session).unwrap();

        let loaded = store.get_session("test-123").unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.id, "test-123");
        assert_eq!(loaded.name, Some("Test Session".into()));
        assert!(matches!(loaded.state, SessionState::Idle));
    }

    #[test]
    fn test_session_not_found() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();
        let loaded = store.get_session("nonexistent").unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_session_update() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        let mut session = HistoricalSession::new("test-123".into(), Some("Original".into()));
        store.save_session(&session).unwrap();

        session.name = Some("Updated".into());
        session.state = SessionState::Finished;
        store.update_session(&session).unwrap();

        let loaded = store.get_session("test-123").unwrap().unwrap();
        assert_eq!(loaded.name, Some("Updated".into()));
        assert!(matches!(loaded.state, SessionState::Finished));
    }

    #[test]
    fn test_session_delete() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        let session = HistoricalSession::new("test-123".into(), None);
        store.save_session(&session).unwrap();

        store.delete_session("test-123").unwrap();

        let loaded = store.get_session("test-123").unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_update_session_stats() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        let session = HistoricalSession::new("test-123".into(), None);
        store.save_session(&session).unwrap();

        store.update_session_stats("test-123", 100, 200).unwrap();
        store.update_session_stats("test-123", 50, 100).unwrap();

        let loaded = store.get_session("test-123").unwrap().unwrap();
        assert_eq!(loaded.total_input_tokens, 150);
        assert_eq!(loaded.total_output_tokens, 300);
    }

    // Task 8: Message CRUD tests
    #[test]
    fn test_message_save_and_get() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        let session = HistoricalSession::new("sess-1".into(), None);
        store.save_session(&session).unwrap();

        let msg = HistoricalMessage::user("sess-1".into(), "Hello".into(), 1000);
        let id = store.save_message(&msg).unwrap();
        assert!(id > 0);

        let result = store.get_messages("sess-1", &MessageQuery::new()).unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.messages[0].content, "Hello");
    }

    #[test]
    fn test_message_updates_session_count() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        let session = HistoricalSession::new("sess-1".into(), None);
        store.save_session(&session).unwrap();

        store
            .save_message(&HistoricalMessage::user(
                "sess-1".into(),
                "One".into(),
                1000,
            ))
            .unwrap();
        store
            .save_message(&HistoricalMessage::assistant(
                "sess-1".into(),
                "Two".into(),
                1001,
            ))
            .unwrap();

        let loaded = store.get_session("sess-1").unwrap().unwrap();
        assert_eq!(loaded.message_count, 2);
    }

    #[test]
    fn test_message_filter_by_role() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        let session = HistoricalSession::new("sess-1".into(), None);
        store.save_session(&session).unwrap();

        store
            .save_message(&HistoricalMessage::user(
                "sess-1".into(),
                "User msg".into(),
                1000,
            ))
            .unwrap();
        store
            .save_message(&HistoricalMessage::assistant(
                "sess-1".into(),
                "Assistant msg".into(),
                1001,
            ))
            .unwrap();

        let mut query = MessageQuery::new();
        query.role = Some(MessageRole::User);
        let result = store.get_messages("sess-1", &query).unwrap();

        assert_eq!(result.total, 1);
        assert_eq!(result.messages[0].role, MessageRole::User);
    }

    #[test]
    fn test_message_cascade_delete() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        let session = HistoricalSession::new("sess-1".into(), None);
        store.save_session(&session).unwrap();
        store
            .save_message(&HistoricalMessage::user(
                "sess-1".into(),
                "Hello".into(),
                1000,
            ))
            .unwrap();

        store.delete_session("sess-1").unwrap();

        let result = store.get_messages("sess-1", &MessageQuery::new()).unwrap();
        assert_eq!(result.total, 0);
    }

    // Task 9: Search tests
    #[test]
    fn test_list_sessions_basic() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        store
            .save_session(&HistoricalSession::new(
                "sess-1".into(),
                Some("First".into()),
            ))
            .unwrap();
        store
            .save_session(&HistoricalSession::new(
                "sess-2".into(),
                Some("Second".into()),
            ))
            .unwrap();

        let result = store.list_sessions(&SessionQuery::new()).unwrap();
        assert_eq!(result.total, 2);
        assert_eq!(result.sessions.len(), 2);
    }

    #[test]
    fn test_list_sessions_filter_by_name() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        store
            .save_session(&HistoricalSession::new(
                "sess-1".into(),
                Some("Alpha Test".into()),
            ))
            .unwrap();
        store
            .save_session(&HistoricalSession::new(
                "sess-2".into(),
                Some("Beta Test".into()),
            ))
            .unwrap();
        store
            .save_session(&HistoricalSession::new(
                "sess-3".into(),
                Some("Gamma".into()),
            ))
            .unwrap();

        let mut query = SessionQuery::new();
        query.name = Some("Test".into());
        let result = store.list_sessions(&query).unwrap();

        assert_eq!(result.total, 2);
    }

    #[test]
    fn test_list_sessions_filter_by_state() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        let mut s1 = HistoricalSession::new("sess-1".into(), None);
        s1.state = SessionState::Finished;
        store.save_session(&s1).unwrap();

        let s2 = HistoricalSession::new("sess-2".into(), None);
        store.save_session(&s2).unwrap();

        let mut query = SessionQuery::new();
        query.state = Some(SessionState::Finished);
        let result = store.list_sessions(&query).unwrap();

        assert_eq!(result.total, 1);
        assert_eq!(result.sessions[0].id, "sess-1");
    }

    #[test]
    fn test_list_sessions_fts_search() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        store
            .save_session(&HistoricalSession::new("sess-1".into(), None))
            .unwrap();
        store
            .save_session(&HistoricalSession::new("sess-2".into(), None))
            .unwrap();

        store
            .save_message(&HistoricalMessage::user(
                "sess-1".into(),
                "How do I use Rust?".into(),
                1000,
            ))
            .unwrap();
        store
            .save_message(&HistoricalMessage::user(
                "sess-2".into(),
                "Hello world".into(),
                1000,
            ))
            .unwrap();

        let mut query = SessionQuery::new();
        query.search = Some("Rust".into());
        let result = store.list_sessions(&query).unwrap();

        assert_eq!(result.total, 1);
        assert_eq!(result.sessions[0].id, "sess-1");
    }

    #[test]
    fn test_list_sessions_pagination() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        for i in 0..5 {
            store
                .save_session(&HistoricalSession::new(format!("sess-{}", i), None))
                .unwrap();
        }

        let mut query = SessionQuery::new();
        query.limit = 2;
        query.offset = 0;
        let page1 = store.list_sessions(&query).unwrap();

        query.offset = 2;
        let page2 = store.list_sessions(&query).unwrap();

        assert_eq!(page1.total, 5);
        assert_eq!(page1.sessions.len(), 2);
        assert_eq!(page2.sessions.len(), 2);
        assert_ne!(page1.sessions[0].id, page2.sessions[0].id);
    }

    #[test]
    fn test_list_sessions_filter_by_tool() {
        let store = SqliteHistoryStore::open_in_memory().unwrap();

        store
            .save_session(&HistoricalSession::new("sess-1".into(), None))
            .unwrap();
        store
            .save_session(&HistoricalSession::new("sess-2".into(), None))
            .unwrap();

        store
            .save_message(&HistoricalMessage::tool_use(
                "sess-1".into(),
                "t1".into(),
                "Read".into(),
                "{}".into(),
                1000,
            ))
            .unwrap();
        store
            .save_message(&HistoricalMessage::tool_use(
                "sess-2".into(),
                "t2".into(),
                "Write".into(),
                "{}".into(),
                1000,
            ))
            .unwrap();

        let mut query = SessionQuery::new();
        query.tool = Some("Read".into());
        let result = store.list_sessions(&query).unwrap();

        assert_eq!(result.total, 1);
        assert_eq!(result.sessions[0].id, "sess-1");
    }
}
