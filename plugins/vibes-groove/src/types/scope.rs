//! Scope type for hierarchical learning isolation

use serde::{Deserialize, Serialize};

use crate::GrooveError;

/// Hierarchical scope for learning isolation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Scope {
    /// Shared across all users
    Global,
    /// Specific to a user
    User(String),
    /// Specific to a project path
    Project(String),
}

impl Scope {
    /// Convert to database string format (type:value)
    pub fn to_db_string(&self) -> String {
        match self {
            Scope::Global => "global".to_string(),
            Scope::User(id) => format!("user:{id}"),
            Scope::Project(path) => format!("project:{path}"),
        }
    }

    /// Parse from database string format
    pub fn from_db_string(s: &str) -> Result<Self, GrooveError> {
        if s == "global" {
            return Ok(Scope::Global);
        }
        if let Some(id) = s.strip_prefix("user:") {
            return Ok(Scope::User(id.to_string()));
        }
        if let Some(path) = s.strip_prefix("project:") {
            return Ok(Scope::Project(path.to_string()));
        }
        Err(GrooveError::InvalidScope(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_scope_roundtrip() {
        let scope = Scope::Global;
        let db_str = scope.to_db_string();
        assert_eq!(db_str, "global");
        let parsed = Scope::from_db_string(&db_str).unwrap();
        assert_eq!(parsed, Scope::Global);
    }

    #[test]
    fn test_user_scope_roundtrip() {
        let scope = Scope::User("alex".into());
        let db_str = scope.to_db_string();
        assert_eq!(db_str, "user:alex");
        let parsed = Scope::from_db_string(&db_str).unwrap();
        assert_eq!(parsed, scope);
    }

    #[test]
    fn test_project_scope_roundtrip() {
        let scope = Scope::Project("/home/alex/myrepo".into());
        let db_str = scope.to_db_string();
        assert_eq!(db_str, "project:/home/alex/myrepo");
        let parsed = Scope::from_db_string(&db_str).unwrap();
        assert_eq!(parsed, scope);
    }

    #[test]
    fn test_invalid_scope_parse() {
        let result = Scope::from_db_string("unknown:value");
        assert!(result.is_err());
    }

    #[test]
    fn test_scope_serialization() {
        let scope = Scope::User("test".into());
        let json = serde_json::to_string(&scope).unwrap();
        let parsed: Scope = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, scope);
    }
}
