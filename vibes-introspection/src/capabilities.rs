//! Harness capability types

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Hook types that can be observed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookType {
    PreToolUse,
    PostToolUse,
    Stop,
    Notification,
}

/// Config file format
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigFormat {
    Json,
    Toml,
    Markdown,
}

/// Scope for injection targets
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InjectionScope {
    System,
    User,
    Project,
}

/// An installed hook we detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledHook {
    pub hook_type: HookType,
    pub name: String,
    pub path: PathBuf,
}

/// Hook system capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HookCapabilities {
    pub supported_types: Vec<HookType>,
    pub hooks_dir: Option<PathBuf>,
    pub installed_hooks: Vec<InstalledHook>,
}

/// A config file we detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub path: PathBuf,
    pub format: ConfigFormat,
    pub writable: bool,
}

/// A file we can inject learnings into
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionTarget {
    pub path: PathBuf,
    pub format: ConfigFormat,
    pub writable: bool,
    pub scope: InjectionScope,
}

/// Capabilities at a single scope level
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScopedCapabilities {
    pub hooks: Option<HookCapabilities>,
    pub config_files: Vec<ConfigFile>,
    pub injection_targets: Vec<InjectionTarget>,
}

/// Full harness capabilities across all scopes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessCapabilities {
    pub harness_type: String,
    pub version: Option<String>,
    pub system: Option<ScopedCapabilities>,
    pub user: ScopedCapabilities,
    pub project: Option<ScopedCapabilities>,
}

impl HarnessCapabilities {
    /// Get effective hooks (project -> user -> system precedence)
    pub fn effective_hooks(&self) -> Option<&HookCapabilities> {
        self.project
            .as_ref()
            .and_then(|p| p.hooks.as_ref())
            .or(self.user.hooks.as_ref())
            .or(self.system.as_ref().and_then(|s| s.hooks.as_ref()))
    }

    /// Get all injection targets across scopes
    pub fn effective_injection_targets(&self) -> Vec<&InjectionTarget> {
        let mut targets = Vec::new();
        if let Some(sys) = &self.system {
            targets.extend(sys.injection_targets.iter());
        }
        targets.extend(self.user.injection_targets.iter());
        if let Some(proj) = &self.project {
            targets.extend(proj.injection_targets.iter());
        }
        targets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_hooks_prefers_project() {
        let project_hooks = HookCapabilities {
            supported_types: vec![HookType::Stop],
            hooks_dir: Some(PathBuf::from("/project/.claude/hooks")),
            installed_hooks: vec![],
        };

        let user_hooks = HookCapabilities {
            supported_types: vec![HookType::PreToolUse, HookType::PostToolUse],
            hooks_dir: Some(PathBuf::from("/home/user/.claude/hooks")),
            installed_hooks: vec![],
        };

        let caps = HarnessCapabilities {
            harness_type: "claude".to_string(),
            version: None,
            system: None,
            user: ScopedCapabilities {
                hooks: Some(user_hooks),
                ..Default::default()
            },
            project: Some(ScopedCapabilities {
                hooks: Some(project_hooks),
                ..Default::default()
            }),
        };

        let effective = caps.effective_hooks().unwrap();
        assert_eq!(effective.supported_types, vec![HookType::Stop]);
    }

    #[test]
    fn test_effective_hooks_falls_back_to_user() {
        let user_hooks = HookCapabilities {
            supported_types: vec![HookType::PreToolUse],
            hooks_dir: Some(PathBuf::from("/home/user/.claude/hooks")),
            installed_hooks: vec![],
        };

        let caps = HarnessCapabilities {
            harness_type: "claude".to_string(),
            version: None,
            system: None,
            user: ScopedCapabilities {
                hooks: Some(user_hooks),
                ..Default::default()
            },
            project: None,
        };

        let effective = caps.effective_hooks().unwrap();
        assert_eq!(effective.supported_types, vec![HookType::PreToolUse]);
    }

    #[test]
    fn test_effective_injection_targets_collects_all_scopes() {
        let caps = HarnessCapabilities {
            harness_type: "claude".to_string(),
            version: None,
            system: Some(ScopedCapabilities {
                injection_targets: vec![InjectionTarget {
                    path: PathBuf::from("/etc/claude/CLAUDE.md"),
                    format: ConfigFormat::Markdown,
                    writable: false,
                    scope: InjectionScope::System,
                }],
                ..Default::default()
            }),
            user: ScopedCapabilities {
                injection_targets: vec![InjectionTarget {
                    path: PathBuf::from("/home/user/.claude/CLAUDE.md"),
                    format: ConfigFormat::Markdown,
                    writable: true,
                    scope: InjectionScope::User,
                }],
                ..Default::default()
            },
            project: Some(ScopedCapabilities {
                injection_targets: vec![InjectionTarget {
                    path: PathBuf::from("/project/CLAUDE.md"),
                    format: ConfigFormat::Markdown,
                    writable: true,
                    scope: InjectionScope::Project,
                }],
                ..Default::default()
            }),
        };

        let targets = caps.effective_injection_targets();
        assert_eq!(targets.len(), 3);
    }

    #[test]
    fn test_effective_hooks_falls_back_to_system() {
        let system_hooks = HookCapabilities {
            supported_types: vec![HookType::Notification],
            hooks_dir: Some(PathBuf::from("/etc/claude/hooks")),
            installed_hooks: vec![],
        };

        let caps = HarnessCapabilities {
            harness_type: "claude".to_string(),
            version: None,
            system: Some(ScopedCapabilities {
                hooks: Some(system_hooks),
                ..Default::default()
            }),
            user: ScopedCapabilities::default(),
            project: None,
        };

        let effective = caps.effective_hooks().unwrap();
        assert_eq!(effective.supported_types, vec![HookType::Notification]);
    }

    #[test]
    fn test_effective_hooks_returns_none_when_no_hooks() {
        let caps = HarnessCapabilities {
            harness_type: "claude".to_string(),
            version: None,
            system: None,
            user: ScopedCapabilities::default(),
            project: None,
        };

        assert!(caps.effective_hooks().is_none());
    }
}
