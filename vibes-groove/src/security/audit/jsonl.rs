//! JSONL audit log implementation
//!
//! Provides append-only JSONL file storage for audit entries.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use super::{AuditFilter, AuditLog, AuditLogEntry};
use crate::security::{SecurityError, SecurityResult};

/// JSONL file-based audit log
pub struct JsonlAuditLog {
    path: PathBuf,
}

impl JsonlAuditLog {
    /// Create a new JSONL audit log at the given path
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Get the path to the audit log file
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Ensure the parent directory exists
    async fn ensure_parent_dir(&self) -> SecurityResult<()> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                SecurityError::AuditLog(format!("failed to create audit dir: {}", e))
            })?;
        }
        Ok(())
    }

    /// Parse a single line into an audit entry
    fn parse_line(line: &str) -> Option<AuditLogEntry> {
        if line.trim().is_empty() {
            return None;
        }
        serde_json::from_str(line).ok()
    }

    /// Check if an entry matches the filter
    fn matches_filter(entry: &AuditLogEntry, filter: &AuditFilter) -> bool {
        if filter.actor.as_ref().is_some_and(|a| &entry.actor != a) {
            return false;
        }

        if filter.action.as_ref().is_some_and(|a| &entry.action != a) {
            return false;
        }

        if filter
            .resource
            .as_ref()
            .is_some_and(|r| &entry.resource != r)
        {
            return false;
        }

        if filter.from.is_some_and(|from| entry.timestamp < from) {
            return false;
        }

        if filter.to.is_some_and(|to| entry.timestamp > to) {
            return false;
        }

        true
    }
}

#[async_trait]
impl AuditLog for JsonlAuditLog {
    async fn log(&self, entry: AuditLogEntry) -> SecurityResult<()> {
        self.ensure_parent_dir().await?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await
            .map_err(|e| SecurityError::AuditLog(format!("failed to open audit log: {}", e)))?;

        let json = serde_json::to_string(&entry)
            .map_err(|e| SecurityError::AuditLog(format!("failed to serialize entry: {}", e)))?;

        file.write_all(json.as_bytes())
            .await
            .map_err(|e| SecurityError::AuditLog(format!("failed to write entry: {}", e)))?;
        file.write_all(b"\n")
            .await
            .map_err(|e| SecurityError::AuditLog(format!("failed to write newline: {}", e)))?;
        file.flush()
            .await
            .map_err(|e| SecurityError::AuditLog(format!("failed to flush: {}", e)))?;

        Ok(())
    }

    async fn query(&self, filter: AuditFilter) -> SecurityResult<Vec<AuditLogEntry>> {
        // If file doesn't exist, return empty results
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.path)
            .await
            .map_err(|e| SecurityError::AuditLog(format!("failed to open audit log: {}", e)))?;

        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut results = Vec::new();

        while let Some(line) = lines
            .next_line()
            .await
            .map_err(|e| SecurityError::AuditLog(format!("failed to read line: {}", e)))?
        {
            let Some(entry) = Self::parse_line(&line) else {
                continue;
            };
            if !Self::matches_filter(&entry, &filter) {
                continue;
            }
            results.push(entry);

            // Check limit
            if filter.limit.is_some_and(|limit| results.len() >= limit) {
                break;
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::{ActionOutcome, ActorId, AuditAction, ResourceRef};
    use chrono::{Duration, Utc};
    use tempfile::TempDir;

    async fn create_test_log() -> (TempDir, JsonlAuditLog) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("audit.jsonl");
        let log = JsonlAuditLog::new(path);
        (dir, log)
    }

    #[tokio::test]
    async fn test_jsonl_log_and_query() {
        let (_dir, log) = create_test_log().await;

        let entry = AuditLogEntry::new(
            ActorId::User("alice".into()),
            AuditAction::LearningCreated,
            ResourceRef::Learning(uuid::Uuid::new_v4()),
            ActionOutcome::Success,
        );

        log.log(entry.clone()).await.unwrap();

        let results = log.query(AuditFilter::default()).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].actor, ActorId::User("alice".into()));
    }

    #[tokio::test]
    async fn test_jsonl_multiple_entries() {
        let (_dir, log) = create_test_log().await;

        for i in 0..5 {
            let entry = AuditLogEntry::new(
                ActorId::User(format!("user{}", i)),
                AuditAction::LearningCreated,
                ResourceRef::Learning(uuid::Uuid::new_v4()),
                ActionOutcome::Success,
            );
            log.log(entry).await.unwrap();
        }

        let results = log.query(AuditFilter::default()).await.unwrap();
        assert_eq!(results.len(), 5);
    }

    #[tokio::test]
    async fn test_jsonl_filter_by_actor() {
        let (_dir, log) = create_test_log().await;

        log.log(AuditLogEntry::new(
            ActorId::User("alice".into()),
            AuditAction::LearningCreated,
            ResourceRef::Learning(uuid::Uuid::new_v4()),
            ActionOutcome::Success,
        ))
        .await
        .unwrap();

        log.log(AuditLogEntry::new(
            ActorId::System,
            AuditAction::PolicyLoaded,
            ResourceRef::Policy("default".into()),
            ActionOutcome::Success,
        ))
        .await
        .unwrap();

        let filter = AuditFilter {
            actor: Some(ActorId::System),
            ..Default::default()
        };
        let results = log.query(filter).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].action, AuditAction::PolicyLoaded);
    }

    #[tokio::test]
    async fn test_jsonl_filter_by_action() {
        let (_dir, log) = create_test_log().await;

        log.log(AuditLogEntry::new(
            ActorId::User("alice".into()),
            AuditAction::LearningCreated,
            ResourceRef::Learning(uuid::Uuid::new_v4()),
            ActionOutcome::Success,
        ))
        .await
        .unwrap();

        log.log(AuditLogEntry::new(
            ActorId::User("bob".into()),
            AuditAction::ImportAttempted,
            ResourceRef::Import("file.json".into()),
            ActionOutcome::Success,
        ))
        .await
        .unwrap();

        let filter = AuditFilter {
            action: Some(AuditAction::ImportAttempted),
            ..Default::default()
        };
        let results = log.query(filter).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].actor, ActorId::User("bob".into()));
    }

    #[tokio::test]
    async fn test_jsonl_filter_by_time_range() {
        let (_dir, log) = create_test_log().await;

        // Create entries
        for _ in 0..3 {
            log.log(AuditLogEntry::new(
                ActorId::System,
                AuditAction::LearningCreated,
                ResourceRef::Learning(uuid::Uuid::new_v4()),
                ActionOutcome::Success,
            ))
            .await
            .unwrap();
        }

        // Query with time filter for future (should be empty)
        let filter = AuditFilter {
            from: Some(Utc::now() + Duration::hours(1)),
            ..Default::default()
        };
        let results = log.query(filter).await.unwrap();
        assert!(results.is_empty());

        // Query with time filter for past (should get all)
        let filter = AuditFilter {
            from: Some(Utc::now() - Duration::hours(1)),
            ..Default::default()
        };
        let results = log.query(filter).await.unwrap();
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_jsonl_filter_with_limit() {
        let (_dir, log) = create_test_log().await;

        for _ in 0..10 {
            log.log(AuditLogEntry::new(
                ActorId::System,
                AuditAction::LearningCreated,
                ResourceRef::Learning(uuid::Uuid::new_v4()),
                ActionOutcome::Success,
            ))
            .await
            .unwrap();
        }

        let filter = AuditFilter {
            limit: Some(3),
            ..Default::default()
        };
        let results = log.query(filter).await.unwrap();
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_jsonl_query_empty_file() {
        let (_dir, log) = create_test_log().await;

        // Query without writing anything
        let results = log.query(AuditFilter::default()).await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_jsonl_creates_parent_dir() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested").join("deep").join("audit.jsonl");
        let log = JsonlAuditLog::new(path);

        log.log(AuditLogEntry::new(
            ActorId::System,
            AuditAction::PolicyLoaded,
            ResourceRef::Policy("test".into()),
            ActionOutcome::Success,
        ))
        .await
        .unwrap();

        let results = log.query(AuditFilter::default()).await.unwrap();
        assert_eq!(results.len(), 1);
    }
}
