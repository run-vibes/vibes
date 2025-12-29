//! Role-based access control types
//!
//! Provides organization roles and permissions (enforcement deferred).

use serde::{Deserialize, Serialize};

/// Organization role for RBAC
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OrgRole {
    /// Full control over everything
    Admin,
    /// Can create, review, and publish learnings
    Curator,
    /// Can create and share learnings
    Member,
    /// Read-only access
    Viewer,
}

impl OrgRole {
    /// Get permissions for this role
    pub fn permissions(&self) -> Permissions {
        match self {
            Self::Admin => Permissions {
                can_create: true,
                can_read: true,
                can_modify: true,
                can_delete: true,
                can_publish: true,
                can_review: true,
                can_admin: true,
            },
            Self::Curator => Permissions {
                can_create: true,
                can_read: true,
                can_modify: true,
                can_delete: false,
                can_publish: true,
                can_review: true,
                can_admin: false,
            },
            Self::Member => Permissions {
                can_create: true,
                can_read: true,
                can_modify: false,
                can_delete: false,
                can_publish: false,
                can_review: false,
                can_admin: false,
            },
            Self::Viewer => Permissions {
                can_create: false,
                can_read: true,
                can_modify: false,
                can_delete: false,
                can_publish: false,
                can_review: false,
                can_admin: false,
            },
        }
    }

    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Curator => "curator",
            Self::Member => "member",
            Self::Viewer => "viewer",
        }
    }
}

impl std::str::FromStr for OrgRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Self::Admin),
            "curator" => Ok(Self::Curator),
            "member" => Ok(Self::Member),
            "viewer" => Ok(Self::Viewer),
            _ => Err(format!("unknown role: {}", s)),
        }
    }
}

/// Permission flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permissions {
    pub can_create: bool,
    pub can_read: bool,
    pub can_modify: bool,
    pub can_delete: bool,
    pub can_publish: bool,
    pub can_review: bool,
    pub can_admin: bool,
}

impl Default for Permissions {
    fn default() -> Self {
        // Default to viewer permissions
        OrgRole::Viewer.permissions()
    }
}

impl Permissions {
    /// Check if this permission set allows an operation
    pub fn allows(&self, operation: Operation) -> bool {
        match operation {
            Operation::Create => self.can_create,
            Operation::Read => self.can_read,
            Operation::Modify => self.can_modify,
            Operation::Delete => self.can_delete,
            Operation::Publish => self.can_publish,
            Operation::Review => self.can_review,
            Operation::Admin => self.can_admin,
        }
    }
}

/// Operations that can be performed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Create,
    Read,
    Modify,
    Delete,
    Publish,
    Review,
    Admin,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_has_all_permissions() {
        let perms = OrgRole::Admin.permissions();
        assert!(perms.can_create);
        assert!(perms.can_read);
        assert!(perms.can_modify);
        assert!(perms.can_delete);
        assert!(perms.can_publish);
        assert!(perms.can_review);
        assert!(perms.can_admin);
    }

    #[test]
    fn test_curator_permissions() {
        let perms = OrgRole::Curator.permissions();
        assert!(perms.can_create);
        assert!(perms.can_read);
        assert!(perms.can_modify);
        assert!(!perms.can_delete);
        assert!(perms.can_publish);
        assert!(perms.can_review);
        assert!(!perms.can_admin);
    }

    #[test]
    fn test_member_permissions() {
        let perms = OrgRole::Member.permissions();
        assert!(perms.can_create);
        assert!(perms.can_read);
        assert!(!perms.can_modify);
        assert!(!perms.can_delete);
        assert!(!perms.can_publish);
        assert!(!perms.can_review);
    }

    #[test]
    fn test_viewer_read_only() {
        let perms = OrgRole::Viewer.permissions();
        assert!(!perms.can_create);
        assert!(perms.can_read);
        assert!(!perms.can_modify);
        assert!(!perms.can_delete);
    }

    #[test]
    fn test_role_str_roundtrip() {
        for role in [OrgRole::Admin, OrgRole::Curator, OrgRole::Member, OrgRole::Viewer] {
            let s = role.as_str();
            let parsed: OrgRole = s.parse().unwrap();
            assert_eq!(parsed, role);
        }
    }

    #[test]
    fn test_permissions_allows() {
        let admin = OrgRole::Admin.permissions();
        assert!(admin.allows(Operation::Delete));
        assert!(admin.allows(Operation::Admin));

        let viewer = OrgRole::Viewer.permissions();
        assert!(!viewer.allows(Operation::Create));
        assert!(viewer.allows(Operation::Read));
    }

    #[test]
    fn test_default_permissions() {
        let perms = Permissions::default();
        assert_eq!(perms, OrgRole::Viewer.permissions());
    }
}
