//! Database migrations for chat history

use crate::history::HistoryError;
use rusqlite::Connection;

/// SQL for each migration version
const MIGRATIONS: &[(&str, &str)] = &[
    ("v001_initial", include_str!("v001_initial.sql")),
    ("v002_fts", include_str!("v002_fts.sql")),
];

/// Runs database migrations
pub struct Migrator<'a> {
    conn: &'a Connection,
}

impl<'a> Migrator<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Get current schema version
    pub fn current_version(&self) -> Result<i32, HistoryError> {
        let version: i32 = self
            .conn
            .pragma_query_value(None, "user_version", |row| row.get(0))?;
        Ok(version)
    }

    /// Set schema version
    fn set_version(&self, version: i32) -> Result<(), HistoryError> {
        self.conn.pragma_update(None, "user_version", version)?;
        Ok(())
    }

    /// Run all pending migrations
    pub fn migrate(&self) -> Result<(), HistoryError> {
        let current = self.current_version()?;
        let target = MIGRATIONS.len() as i32;

        if current >= target {
            return Ok(());
        }

        for (idx, (name, sql)) in MIGRATIONS.iter().enumerate() {
            let version = (idx + 1) as i32;
            if version > current {
                tracing::info!("Running migration {}: {}", version, name);
                self.conn
                    .execute_batch(sql)
                    .map_err(|e| HistoryError::Migration(format!("{}: {}", name, e)))?;
                self.set_version(version)?;
            }
        }

        Ok(())
    }

    /// Get target version (latest migration)
    #[allow(dead_code)]
    pub fn target_version(&self) -> i32 {
        MIGRATIONS.len() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_fresh_database() {
        let conn = Connection::open_in_memory().unwrap();
        let migrator = Migrator::new(&conn);

        assert_eq!(migrator.current_version().unwrap(), 0);
        migrator.migrate().unwrap();
        assert_eq!(
            migrator.current_version().unwrap(),
            migrator.target_version()
        );
    }

    #[test]
    fn test_migrate_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        let migrator = Migrator::new(&conn);

        migrator.migrate().unwrap();
        let v1 = migrator.current_version().unwrap();

        migrator.migrate().unwrap();
        let v2 = migrator.current_version().unwrap();

        assert_eq!(v1, v2);
    }

    #[test]
    fn test_tables_created() {
        let conn = Connection::open_in_memory().unwrap();
        let migrator = Migrator::new(&conn);
        migrator.migrate().unwrap();

        // Verify sessions table exists
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='sessions'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // Verify messages table exists
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='messages'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_fts_table_created() {
        let conn = Connection::open_in_memory().unwrap();
        let migrator = Migrator::new(&conn);
        migrator.migrate().unwrap();

        assert_eq!(migrator.current_version().unwrap(), 2);

        // Verify FTS table exists
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='messages_fts'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_fts_triggers_created() {
        let conn = Connection::open_in_memory().unwrap();
        let migrator = Migrator::new(&conn);
        migrator.migrate().unwrap();

        // Count triggers
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='trigger' AND name LIKE 'messages_%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 3); // ai, ad, au
    }
}
