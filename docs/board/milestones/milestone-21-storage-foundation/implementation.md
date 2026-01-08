# Milestone 4.2 Storage Foundation - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the storage layer for groove's continual learning system using CozoDB with RocksDB backend.

**Architecture:** Three-tier storage (user/project/enterprise) with separate RocksDB databases per tier. CozoDB provides unified relational + graph + vector queries via Datalog. Eager user DB, lazy project/enterprise DBs.

**Tech Stack:** CozoDB (cozo-core), RocksDB backend, uuid v7, chrono, serde, thiserror, async-trait, rand_distr (Beta distribution)

**Related Docs:**
- Design decisions: `docs/plans/14-continual-learning/milestone-4.2-decisions.md`
- Phase 4 design: `docs/plans/14-continual-learning/design.md`

---

## Task Overview

| Task | Component | Dependencies |
|------|-----------|--------------|
| 1 | Create vibes-groove crate | None |
| 2 | Error types | Task 1 |
| 3 | Scope types | Task 2 |
| 4 | Learning types | Tasks 2-3 |
| 5 | UsageStats and Outcome | Task 2 |
| 6 | Adaptive parameters | Task 2 |
| 7 | Relations types | Task 3 |
| 8 | Storage traits | Tasks 3-7 |
| 9 | Config types | Task 2 |
| 10 | Export/Import types | Tasks 4-7 |
| 11 | CozoStore: Schema | Task 1 |
| 12 | CozoStore: Schema migrations | Task 11 |
| 13 | CozoStore: Learning CRUD | Tasks 8, 11 |
| 14 | CozoStore: Usage stats | Task 13 |
| 15 | CozoStore: Relations | Tasks 7, 13 |
| 16 | CozoStore: Parameters | Tasks 6, 11 |
| 17 | CozoStore: Semantic search | Task 13 |
| 18 | GrooveStorage: Multi-tier | Tasks 9, 13 |
| 19 | Export/Import | Tasks 10, 18 |
| 20 | PluginContext extensions | vibes-introspection |

---

## Task 1: Create vibes-groove Crate

**Files:**
- Create: `vibes-groove/Cargo.toml`
- Create: `vibes-groove/src/lib.rs`
- Modify: `Cargo.toml` (workspace)

**Step 1: Add to workspace**

```toml
# Cargo.toml (workspace root) - add to members
members = [
    # ... existing members ...
    "vibes-groove",
]
```

**Step 2: Create Cargo.toml**

```toml
# vibes-groove/Cargo.toml
[package]
name = "vibes-groove"
version = "0.1.0"
edition = "2021"
description = "Continual learning storage for vibes"
license = "MIT"

[dependencies]
# Storage
cozo = { version = "0.7", default-features = false, features = ["storage-rocksdb"] }

# Async
async-trait = "0.1"
tokio = { version = "1", features = ["sync"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Types
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v7", "serde"] }
thiserror = "1"

# Bayesian sampling
rand = "0.8"
rand_distr = "0.4"

# Config paths
dirs = "5"

[dev-dependencies]
tempfile = "3"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

**Step 3: Create lib.rs stub**

```rust
// vibes-groove/src/lib.rs
//! vibes-groove - Continual learning storage

pub mod error;
pub mod types;
pub mod store;
pub mod storage;
pub mod config;
pub mod export;

pub use error::GrooveError;
pub use types::*;
pub use store::{LearningStore, ParamStore};
pub use storage::GrooveStorage;
pub use config::GrooveConfig;
pub use export::{GrooveExport, ImportStats};
```

**Step 4: Verify workspace builds**

Run: `just build`
Expected: Compilation succeeds (with module not found warnings - we'll create them)

**Step 5: Commit**

```bash
git add Cargo.toml vibes-groove/
git commit -m "feat(groove): create vibes-groove crate structure"
```

---

## Task 2: Error Types

**Files:**
- Create: `vibes-groove/src/error.rs`
- Modify: `vibes-groove/src/lib.rs`

**Step 1: Write the test**

```rust
// vibes-groove/src/error.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = GrooveError::Database("connection failed".into());
        assert!(err.to_string().contains("connection failed"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let groove_err: GrooveError = io_err.into();
        assert!(matches!(groove_err, GrooveError::Io(_)));
    }

    #[test]
    fn test_no_project_context() {
        let err = GrooveError::NoProjectContext;
        assert_eq!(err.to_string(), "No project context available");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-groove error::tests`
Expected: FAIL - module doesn't exist

**Step 3: Write implementation**

```rust
// vibes-groove/src/error.rs
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum GrooveError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("No project context available")]
    NoProjectContext,

    #[error("No enterprise context for org: {0}")]
    NoEnterpriseContext(String),

    #[error("Invalid scope: {0}")]
    InvalidScope(String),

    #[error("Learning not found: {0}")]
    NotFound(Uuid),

    #[error("Schema migration failed: {0}")]
    Migration(String),

    #[error("Export/import error: {0}")]
    Export(String),

    #[error("Embedding error: {0}")]
    Embedding(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type Result<T> = std::result::Result<T, GrooveError>;

#[cfg(test)]
mod tests {
    // ... tests from Step 1 ...
}
```

**Step 4: Update lib.rs**

```rust
// vibes-groove/src/lib.rs
pub mod error;

pub use error::{GrooveError, Result};
```

**Step 5: Run tests**

Run: `cargo test -p vibes-groove error::tests`
Expected: PASS

**Step 6: Commit**

```bash
git add vibes-groove/src/error.rs vibes-groove/src/lib.rs
git commit -m "feat(groove): add error types"
```

---

## Task 3: Scope Types

**Files:**
- Create: `vibes-groove/src/types/scope.rs`
- Create: `vibes-groove/src/types/mod.rs`
- Modify: `vibes-groove/src/lib.rs`

**Step 1: Write the test**

```rust
// vibes-groove/src/types/scope.rs
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-groove types::scope::tests`
Expected: FAIL - module doesn't exist

**Step 3: Write implementation**

```rust
// vibes-groove/src/types/scope.rs
use serde::{Deserialize, Serialize};
use crate::GrooveError;

/// Hierarchical scope for learning isolation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Scope {
    Global,
    User(String),
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
    // ... tests from Step 1 ...
}
```

**Step 4: Create types/mod.rs**

```rust
// vibes-groove/src/types/mod.rs
mod scope;

pub use scope::Scope;
```

**Step 5: Update lib.rs**

```rust
// vibes-groove/src/lib.rs
pub mod error;
pub mod types;

pub use error::{GrooveError, Result};
pub use types::*;
```

**Step 6: Run tests**

Run: `cargo test -p vibes-groove types::scope::tests`
Expected: PASS

**Step 7: Commit**

```bash
git add vibes-groove/src/types/
git commit -m "feat(groove): add Scope type with db string conversion"
```

---

## Task 4: Learning Types

**Files:**
- Create: `vibes-groove/src/types/learning.rs`
- Modify: `vibes-groove/src/types/mod.rs`

**Step 1: Write the test**

```rust
// vibes-groove/src/types/learning.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learning_category_roundtrip() {
        for category in [
            LearningCategory::CodePattern,
            LearningCategory::Preference,
            LearningCategory::Solution,
            LearningCategory::ErrorRecovery,
            LearningCategory::ToolUsage,
            LearningCategory::HarnessKnowledge,
        ] {
            let s = category.as_str();
            let parsed = LearningCategory::from_str(s).unwrap();
            assert_eq!(parsed, category);
        }
    }

    #[test]
    fn test_learning_source_type() {
        let source = LearningSource::UserCreated;
        assert_eq!(source.source_type(), "user_created");

        let source = LearningSource::Transcript {
            session_id: "sess-1".into(),
            message_index: 5,
        };
        assert_eq!(source.source_type(), "transcript");
    }

    #[test]
    fn test_learning_content_serialization() {
        let content = LearningContent {
            description: "Use Result for errors".into(),
            pattern: Some(serde_json::json!({"language": "rust"})),
            insight: "Prefer Result over panic".into(),
        };
        let json = serde_json::to_string(&content).unwrap();
        let parsed: LearningContent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.description, content.description);
    }

    #[test]
    fn test_learning_id_is_uuid_v7() {
        let id = LearningId::now_v7();
        // UUIDv7 starts with version nibble 7
        assert_eq!(id.get_version_num(), 7);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-groove types::learning::tests`
Expected: FAIL - module doesn't exist

**Step 3: Write implementation**

```rust
// vibes-groove/src/types/learning.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Scope;

/// UUIDv7 provides time-ordered unique identifiers
pub type LearningId = Uuid;

/// A captured piece of knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Learning {
    pub id: LearningId,
    pub scope: Scope,
    pub category: LearningCategory,
    pub content: LearningContent,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source: LearningSource,
}

impl Learning {
    /// Create a new learning with generated ID and timestamps
    pub fn new(
        scope: Scope,
        category: LearningCategory,
        content: LearningContent,
        source: LearningSource,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            scope,
            category,
            content,
            confidence: 0.5, // Neutral starting confidence
            created_at: now,
            updated_at: now,
            source,
        }
    }
}

/// Category of learning for filtering and organization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LearningCategory {
    CodePattern,
    Preference,
    Solution,
    ErrorRecovery,
    ToolUsage,
    HarnessKnowledge,
}

impl LearningCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CodePattern => "code_pattern",
            Self::Preference => "preference",
            Self::Solution => "solution",
            Self::ErrorRecovery => "error_recovery",
            Self::ToolUsage => "tool_usage",
            Self::HarnessKnowledge => "harness_knowledge",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "code_pattern" => Some(Self::CodePattern),
            "preference" => Some(Self::Preference),
            "solution" => Some(Self::Solution),
            "error_recovery" => Some(Self::ErrorRecovery),
            "tool_usage" => Some(Self::ToolUsage),
            "harness_knowledge" => Some(Self::HarnessKnowledge),
            _ => None,
        }
    }
}

/// The actual content of a learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningContent {
    /// Human-readable description of what was learned
    pub description: String,

    /// Structured pattern data (flexible JSON for different pattern types)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<serde_json::Value>,

    /// Actionable insight for injection into sessions
    pub insight: String,
}

/// Where this learning originated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningSource {
    Transcript {
        session_id: String,
        message_index: usize,
    },
    UserCreated,
    Promoted {
        from_scope: Scope,
        original_id: LearningId,
    },
    Imported {
        source_file: String,
        imported_at: DateTime<Utc>,
    },
    EnterpriseCurated {
        curator: String,
    },
}

impl LearningSource {
    pub fn source_type(&self) -> &'static str {
        match self {
            Self::Transcript { .. } => "transcript",
            Self::UserCreated => "user_created",
            Self::Promoted { .. } => "promoted",
            Self::Imported { .. } => "imported",
            Self::EnterpriseCurated { .. } => "enterprise_curated",
        }
    }
}

#[cfg(test)]
mod tests {
    // ... tests from Step 1 ...
}
```

**Step 4: Update types/mod.rs**

```rust
// vibes-groove/src/types/mod.rs
mod scope;
mod learning;

pub use scope::Scope;
pub use learning::*;
```

**Step 5: Run tests**

Run: `cargo test -p vibes-groove types::learning::tests`
Expected: PASS

**Step 6: Commit**

```bash
git add vibes-groove/src/types/learning.rs vibes-groove/src/types/mod.rs
git commit -m "feat(groove): add Learning, LearningCategory, LearningContent, LearningSource types"
```

---

## Task 5: UsageStats and Outcome

**Files:**
- Create: `vibes-groove/src/types/usage.rs`
- Modify: `vibes-groove/src/types/mod.rs`

**Step 1: Write the test**

```rust
// vibes-groove/src/types/usage.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_usage_stats() {
        let stats = UsageStats::default();
        assert_eq!(stats.times_injected, 0);
        assert_eq!(stats.confidence_alpha, 1.0);
        assert_eq!(stats.confidence_beta, 1.0);
        // Uniform prior = 0.5 confidence
        assert!((stats.confidence() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_helpful_outcome_increases_confidence() {
        let mut stats = UsageStats::default();
        let initial = stats.confidence();
        stats.record_outcome(Outcome::Helpful);
        assert!(stats.confidence() > initial);
        assert_eq!(stats.times_helpful, 1);
        assert_eq!(stats.times_injected, 1);
    }

    #[test]
    fn test_contradicted_outcome_decreases_confidence() {
        let mut stats = UsageStats::default();
        let initial = stats.confidence();
        stats.record_outcome(Outcome::Contradicted);
        assert!(stats.confidence() < initial);
        assert_eq!(stats.times_contradicted, 1);
    }

    #[test]
    fn test_ignored_outcome_slight_decay() {
        let mut stats = UsageStats::default();
        let initial = stats.confidence();
        stats.record_outcome(Outcome::Ignored);
        // Ignored causes slight decay
        assert!(stats.confidence() < initial);
        assert_eq!(stats.times_ignored, 1);
    }

    #[test]
    fn test_last_used_updated_on_outcome() {
        let mut stats = UsageStats::default();
        assert!(stats.last_used.is_none());
        stats.record_outcome(Outcome::Helpful);
        assert!(stats.last_used.is_some());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-groove types::usage::tests`
Expected: FAIL - module doesn't exist

**Step 3: Write implementation**

```rust
// vibes-groove/src/types/usage.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Per-learning usage statistics (updated frequently)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub times_injected: u32,
    pub times_helpful: u32,
    pub times_ignored: u32,
    pub times_contradicted: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<DateTime<Utc>>,
    pub confidence_alpha: f64,
    pub confidence_beta: f64,
}

impl Default for UsageStats {
    fn default() -> Self {
        Self {
            times_injected: 0,
            times_helpful: 0,
            times_ignored: 0,
            times_contradicted: 0,
            last_used: None,
            confidence_alpha: 1.0, // Uniform prior
            confidence_beta: 1.0,
        }
    }
}

impl UsageStats {
    /// Calculate current confidence from Bayesian priors
    pub fn confidence(&self) -> f64 {
        self.confidence_alpha / (self.confidence_alpha + self.confidence_beta)
    }

    /// Update confidence based on outcome
    pub fn record_outcome(&mut self, outcome: Outcome) {
        self.times_injected += 1;
        self.last_used = Some(Utc::now());

        match outcome {
            Outcome::Helpful => {
                self.times_helpful += 1;
                self.confidence_alpha += 1.0;
            }
            Outcome::Ignored => {
                self.times_ignored += 1;
                // Neutral - slight decay
                self.confidence_beta += 0.1;
            }
            Outcome::Contradicted => {
                self.times_contradicted += 1;
                self.confidence_beta += 1.5; // Strong negative signal
            }
        }
    }
}

/// Outcome of a learning injection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Helpful,
    Ignored,
    Contradicted,
}

#[cfg(test)]
mod tests {
    // ... tests from Step 1 ...
}
```

**Step 4: Update types/mod.rs**

```rust
// vibes-groove/src/types/mod.rs
mod scope;
mod learning;
mod usage;

pub use scope::Scope;
pub use learning::*;
pub use usage::*;
```

**Step 5: Run tests**

Run: `cargo test -p vibes-groove types::usage::tests`
Expected: PASS

**Step 6: Commit**

```bash
git add vibes-groove/src/types/usage.rs vibes-groove/src/types/mod.rs
git commit -m "feat(groove): add UsageStats with Bayesian confidence updates"
```

---

## Task 6: Adaptive Parameters

**Files:**
- Create: `vibes-groove/src/types/params.rs`
- Modify: `vibes-groove/src/types/mod.rs`

**Step 1: Write the test**

```rust
// vibes-groove/src/types/params.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uninformed_prior() {
        let param = AdaptiveParam::new_uninformed();
        assert!((param.value - 0.5).abs() < 0.001);
        assert_eq!(param.observations, 0);
    }

    #[test]
    fn test_informed_prior() {
        // Prior of alpha=8, beta=2 should give ~0.8 value
        let param = AdaptiveParam::new_with_prior(8.0, 2.0);
        assert!((param.value - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_update_moves_toward_outcome() {
        let mut param = AdaptiveParam::new_uninformed();
        // Positive outcome (1.0) should increase value
        param.update(1.0, 1.0);
        assert!(param.value > 0.5);
        assert_eq!(param.observations, 1);
    }

    #[test]
    fn test_uncertainty_decreases_with_observations() {
        let mut param = AdaptiveParam::new_uninformed();
        let initial_uncertainty = param.uncertainty;
        param.update(0.5, 1.0);
        assert!(param.uncertainty < initial_uncertainty);
    }

    #[test]
    fn test_sample_returns_valid_probability() {
        let param = AdaptiveParam::new_uninformed();
        for _ in 0..100 {
            let sample = param.sample();
            assert!(sample >= 0.0 && sample <= 1.0);
        }
    }

    #[test]
    fn test_system_param_creation() {
        let param = SystemParam::new("injection_budget");
        assert_eq!(param.name, "injection_budget");
        assert!((param.param.value - 0.5).abs() < 0.001);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-groove types::params::tests`
Expected: FAIL - module doesn't exist

**Step 3: Write implementation**

```rust
// vibes-groove/src/types/params.rs
use chrono::{DateTime, Utc};
use rand::Rng;
use rand_distr::{Beta, Distribution};
use serde::{Deserialize, Serialize};

/// A parameter that learns via Bayesian updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveParam {
    pub value: f64,
    pub uncertainty: f64,
    pub observations: u64,
    pub prior_alpha: f64,
    pub prior_beta: f64,
}

impl Default for AdaptiveParam {
    fn default() -> Self {
        Self::new_uninformed()
    }
}

impl AdaptiveParam {
    /// Create with uninformed (uniform) prior
    pub fn new_uninformed() -> Self {
        Self {
            value: 0.5,
            uncertainty: 1.0,
            observations: 0,
            prior_alpha: 1.0,
            prior_beta: 1.0,
        }
    }

    /// Create with informed prior
    pub fn new_with_prior(alpha: f64, beta: f64) -> Self {
        let value = alpha / (alpha + beta);
        Self {
            value,
            uncertainty: 1.0,
            observations: 0,
            prior_alpha: alpha,
            prior_beta: beta,
        }
    }

    /// Bayesian update based on outcome
    pub fn update(&mut self, outcome: f64, weight: f64) {
        self.observations += 1;
        let effective_weight = weight / (1.0 + self.uncertainty);
        self.prior_alpha += outcome * effective_weight;
        self.prior_beta += (1.0 - outcome) * effective_weight;
        self.value = self.prior_alpha / (self.prior_alpha + self.prior_beta);
        self.uncertainty = 1.0 / (1.0 + (self.observations as f64).sqrt());
    }

    /// Thompson sampling for exploration
    pub fn sample(&self) -> f64 {
        let beta = Beta::new(self.prior_alpha, self.prior_beta)
            .unwrap_or_else(|_| Beta::new(1.0, 1.0).unwrap());
        beta.sample(&mut rand::thread_rng())
    }
}

/// Named system-wide parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemParam {
    pub name: String,
    pub param: AdaptiveParam,
    pub updated_at: DateTime<Utc>,
}

impl SystemParam {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            param: AdaptiveParam::new_uninformed(),
            updated_at: Utc::now(),
        }
    }

    pub fn with_prior(name: impl Into<String>, alpha: f64, beta: f64) -> Self {
        Self {
            name: name.into(),
            param: AdaptiveParam::new_with_prior(alpha, beta),
            updated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    // ... tests from Step 1 ...
}
```

**Step 4: Update types/mod.rs**

```rust
// vibes-groove/src/types/mod.rs
mod scope;
mod learning;
mod usage;
mod params;

pub use scope::Scope;
pub use learning::*;
pub use usage::*;
pub use params::*;
```

**Step 5: Run tests**

Run: `cargo test -p vibes-groove types::params::tests`
Expected: PASS

**Step 6: Commit**

```bash
git add vibes-groove/src/types/params.rs vibes-groove/src/types/mod.rs
git commit -m "feat(groove): add AdaptiveParam with Thompson sampling"
```

---

## Task 7: Relations Types

**Files:**
- Create: `vibes-groove/src/types/relations.rs`
- Modify: `vibes-groove/src/types/mod.rs`

**Step 1: Write the test**

```rust
// vibes-groove/src/types/relations.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relation_type_as_str() {
        assert_eq!(RelationType::Supersedes.as_str(), "supersedes");
        assert_eq!(RelationType::Contradicts.as_str(), "contradicts");
        assert_eq!(RelationType::DerivedFrom.as_str(), "derived_from");
        assert_eq!(RelationType::RelatedTo.as_str(), "related_to");
        assert_eq!(RelationType::Specializes.as_str(), "specializes");
    }

    #[test]
    fn test_relation_type_from_str() {
        assert_eq!(RelationType::from_str("supersedes"), Some(RelationType::Supersedes));
        assert_eq!(RelationType::from_str("unknown"), None);
    }

    #[test]
    fn test_learning_relation_serialization() {
        let relation = LearningRelation {
            from_id: uuid::Uuid::now_v7(),
            relation_type: RelationType::Supersedes,
            to_id: uuid::Uuid::now_v7(),
            weight: 1.0,
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&relation).unwrap();
        let parsed: LearningRelation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.from_id, relation.from_id);
        assert_eq!(parsed.relation_type, relation.relation_type);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-groove types::relations::tests`
Expected: FAIL - module doesn't exist

**Step 3: Write implementation**

```rust
// vibes-groove/src/types/relations.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::LearningId;

/// Relationship between two learnings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningRelation {
    pub from_id: LearningId,
    pub relation_type: RelationType,
    pub to_id: LearningId,
    pub weight: f64,
    pub created_at: DateTime<Utc>,
}

impl LearningRelation {
    pub fn new(from_id: LearningId, relation_type: RelationType, to_id: LearningId) -> Self {
        Self {
            from_id,
            relation_type,
            to_id,
            weight: 1.0,
            created_at: Utc::now(),
        }
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }
}

/// Types of relationships between learnings
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationType {
    /// from_id replaces/deprecates to_id
    Supersedes,
    /// from_id conflicts with to_id
    Contradicts,
    /// from_id was derived/generalized from to_id
    DerivedFrom,
    /// from_id is related to to_id (same topic)
    RelatedTo,
    /// from_id is a specific case of general to_id
    Specializes,
}

impl RelationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Supersedes => "supersedes",
            Self::Contradicts => "contradicts",
            Self::DerivedFrom => "derived_from",
            Self::RelatedTo => "related_to",
            Self::Specializes => "specializes",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "supersedes" => Some(Self::Supersedes),
            "contradicts" => Some(Self::Contradicts),
            "derived_from" => Some(Self::DerivedFrom),
            "related_to" => Some(Self::RelatedTo),
            "specializes" => Some(Self::Specializes),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    // ... tests from Step 1 ...
}
```

**Step 4: Update types/mod.rs**

```rust
// vibes-groove/src/types/mod.rs
mod scope;
mod learning;
mod usage;
mod params;
mod relations;

pub use scope::Scope;
pub use learning::*;
pub use usage::*;
pub use params::*;
pub use relations::*;
```

**Step 5: Run tests**

Run: `cargo test -p vibes-groove types::relations::tests`
Expected: PASS

**Step 6: Commit**

```bash
git add vibes-groove/src/types/relations.rs vibes-groove/src/types/mod.rs
git commit -m "feat(groove): add LearningRelation and RelationType"
```

---

## Task 8: Storage Traits

**Files:**
- Create: `vibes-groove/src/store/traits.rs`
- Create: `vibes-groove/src/store/mod.rs`
- Modify: `vibes-groove/src/lib.rs`

**Step 1: Write the test**

```rust
// vibes-groove/src/store/traits.rs
#[cfg(test)]
mod tests {
    use super::*;

    // Verify traits are object-safe
    #[test]
    fn test_learning_store_is_object_safe() {
        fn _takes_boxed(_: Box<dyn LearningStore>) {}
    }

    #[test]
    fn test_param_store_is_object_safe() {
        fn _takes_boxed(_: Box<dyn ParamStore>) {}
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-groove store::traits::tests`
Expected: FAIL - module doesn't exist

**Step 3: Write implementation**

```rust
// vibes-groove/src/store/traits.rs
use async_trait::async_trait;

use crate::{
    GrooveError, Learning, LearningCategory, LearningId, LearningRelation,
    RelationType, Scope, SystemParam, UsageStats,
};

/// Storage operations for a single learning database
#[async_trait]
pub trait LearningStore: Send + Sync {
    /// Store a new learning, returns its ID
    async fn store(&self, learning: &Learning) -> Result<LearningId, GrooveError>;

    /// Retrieve a learning by ID
    async fn get(&self, id: LearningId) -> Result<Option<Learning>, GrooveError>;

    /// Find all learnings in a scope
    async fn find_by_scope(&self, scope: &Scope) -> Result<Vec<Learning>, GrooveError>;

    /// Find learnings by category
    async fn find_by_category(&self, category: &LearningCategory) -> Result<Vec<Learning>, GrooveError>;

    /// Semantic search using embedding vector
    async fn semantic_search(
        &self,
        embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<(Learning, f64)>, GrooveError>;

    /// Update usage statistics for a learning
    async fn update_usage(&self, id: LearningId, stats: &UsageStats) -> Result<(), GrooveError>;

    /// Find related learnings by relation type
    async fn find_related(
        &self,
        id: LearningId,
        relation_type: Option<&RelationType>,
    ) -> Result<Vec<Learning>, GrooveError>;

    /// Store a relation between learnings
    async fn store_relation(&self, relation: &LearningRelation) -> Result<(), GrooveError>;

    /// Delete a learning
    async fn delete(&self, id: LearningId) -> Result<bool, GrooveError>;

    /// Count learnings (for stats)
    async fn count(&self) -> Result<u64, GrooveError>;
}

/// System parameter storage
#[async_trait]
pub trait ParamStore: Send + Sync {
    /// Get a system parameter by name
    async fn get_param(&self, name: &str) -> Result<Option<SystemParam>, GrooveError>;

    /// Store/update a system parameter
    async fn store_param(&self, param: &SystemParam) -> Result<(), GrooveError>;

    /// Get all system parameters
    async fn all_params(&self) -> Result<Vec<SystemParam>, GrooveError>;
}

#[cfg(test)]
mod tests {
    // ... tests from Step 1 ...
}
```

**Step 4: Create store/mod.rs**

```rust
// vibes-groove/src/store/mod.rs
mod traits;

pub use traits::{LearningStore, ParamStore};
```

**Step 5: Update lib.rs**

```rust
// vibes-groove/src/lib.rs
pub mod error;
pub mod types;
pub mod store;

pub use error::{GrooveError, Result};
pub use types::*;
pub use store::{LearningStore, ParamStore};
```

**Step 6: Run tests**

Run: `cargo test -p vibes-groove store::traits::tests`
Expected: PASS

**Step 7: Commit**

```bash
git add vibes-groove/src/store/
git commit -m "feat(groove): add LearningStore and ParamStore traits"
```

---

## Task 9: Config Types

**Files:**
- Create: `vibes-groove/src/config.rs`
- Modify: `vibes-groove/src/lib.rs`

**Step 1: Write the test**

```rust
// vibes-groove/src/config.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GrooveConfig::default();
        assert!(config.user_db_path.to_string_lossy().contains("vibes"));
        assert!(config.enterprises.is_empty());
    }

    #[test]
    fn test_project_context_personal() {
        let ctx = ProjectContext::Personal;
        assert!(matches!(ctx, ProjectContext::Personal));
    }

    #[test]
    fn test_project_context_enterprise() {
        let ctx = ProjectContext::Enterprise {
            org_id: "acme".into(),
        };
        if let ProjectContext::Enterprise { org_id } = ctx {
            assert_eq!(org_id, "acme");
        } else {
            panic!("Expected Enterprise variant");
        }
    }

    #[test]
    fn test_config_serialization() {
        let config = GrooveConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let parsed: GrooveConfig = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.user_db_path, config.user_db_path);
    }
}
```

**Step 2: Add toml to dependencies**

```toml
# vibes-groove/Cargo.toml - add to [dependencies]
toml = "0.8"

# Add to [dev-dependencies]
toml = "0.8"
```

**Step 3: Run test to verify it fails**

Run: `cargo test -p vibes-groove config::tests`
Expected: FAIL - module doesn't exist

**Step 4: Write implementation**

```rust
// vibes-groove/src/config.rs
use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// groove plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrooveConfig {
    /// Path to user tier database
    pub user_db_path: PathBuf,

    /// Configured enterprises (org_id -> config)
    #[serde(default)]
    pub enterprises: HashMap<String, EnterpriseConfig>,
}

impl Default for GrooveConfig {
    fn default() -> Self {
        Self {
            user_db_path: default_user_db_path(),
            enterprises: HashMap::new(),
        }
    }
}

fn default_user_db_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vibes")
        .join("groove")
        .join("user")
}

/// Enterprise configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseConfig {
    /// Organization identifier
    pub org_id: String,

    /// Path to enterprise database
    pub db_path: PathBuf,

    /// Optional: remote sync endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_url: Option<String>,
}

/// Current project context
#[derive(Debug, Clone)]
pub enum ProjectContext {
    Personal,
    Enterprise { org_id: String },
}

#[cfg(test)]
mod tests {
    // ... tests from Step 1 ...
}
```

**Step 5: Update lib.rs**

```rust
// vibes-groove/src/lib.rs
pub mod error;
pub mod types;
pub mod store;
pub mod config;

pub use error::{GrooveError, Result};
pub use types::*;
pub use store::{LearningStore, ParamStore};
pub use config::{GrooveConfig, EnterpriseConfig, ProjectContext};
```

**Step 6: Run tests**

Run: `cargo test -p vibes-groove config::tests`
Expected: PASS

**Step 7: Commit**

```bash
git add vibes-groove/src/config.rs vibes-groove/Cargo.toml vibes-groove/src/lib.rs
git commit -m "feat(groove): add GrooveConfig, EnterpriseConfig, ProjectContext"
```

---

## Task 10: Export/Import Types

**Files:**
- Create: `vibes-groove/src/export.rs`
- Modify: `vibes-groove/src/lib.rs`

**Step 1: Write the test**

```rust
// vibes-groove/src/export.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_version() {
        assert_eq!(EXPORT_VERSION, 1);
    }

    #[test]
    fn test_import_stats_default() {
        let stats = ImportStats::default();
        assert_eq!(stats.learnings_imported, 0);
        assert_eq!(stats.embeddings_queued, 0);
    }

    #[test]
    fn test_groove_export_serialization() {
        let export = GrooveExport {
            version: EXPORT_VERSION,
            exported_at: chrono::Utc::now(),
            learnings: vec![],
            params: vec![],
            relations: vec![],
        };
        let json = serde_json::to_string(&export).unwrap();
        let parsed: GrooveExport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, EXPORT_VERSION);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-groove export::tests`
Expected: FAIL - module doesn't exist

**Step 3: Write implementation**

```rust
// vibes-groove/src/export.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    Learning, LearningCategory, LearningContent, LearningId, LearningRelation,
    LearningSource, Scope, SystemParam, UsageStats,
};

/// Current export format version
pub const EXPORT_VERSION: u32 = 1;

/// Export format for backup/portability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrooveExport {
    /// Export format version
    pub version: u32,

    /// When export was created
    pub exported_at: DateTime<Utc>,

    /// Learnings (without embeddings)
    pub learnings: Vec<LearningExport>,

    /// System parameters
    pub params: Vec<SystemParam>,

    /// Relations
    pub relations: Vec<LearningRelation>,
}

impl GrooveExport {
    pub fn new() -> Self {
        Self {
            version: EXPORT_VERSION,
            exported_at: Utc::now(),
            learnings: vec![],
            params: vec![],
            relations: vec![],
        }
    }
}

impl Default for GrooveExport {
    fn default() -> Self {
        Self::new()
    }
}

/// Learning without embedding (for export)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningExport {
    pub id: LearningId,
    pub scope: Scope,
    pub category: LearningCategory,
    pub content: LearningContent,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source: LearningSource,
    pub usage_stats: UsageStats,
}

impl From<Learning> for LearningExport {
    fn from(learning: Learning) -> Self {
        Self {
            id: learning.id,
            scope: learning.scope,
            category: learning.category,
            content: learning.content,
            confidence: learning.confidence,
            created_at: learning.created_at,
            updated_at: learning.updated_at,
            source: learning.source,
            usage_stats: UsageStats::default(),
        }
    }
}

/// Import statistics
#[derive(Debug, Clone, Default)]
pub struct ImportStats {
    pub learnings_imported: u32,
    pub learnings_skipped: u32,
    pub params_imported: u32,
    pub relations_imported: u32,
    pub embeddings_queued: u32,
}

#[cfg(test)]
mod tests {
    // ... tests from Step 1 ...
}
```

**Step 4: Update lib.rs**

```rust
// vibes-groove/src/lib.rs
pub mod error;
pub mod types;
pub mod store;
pub mod config;
pub mod export;

pub use error::{GrooveError, Result};
pub use types::*;
pub use store::{LearningStore, ParamStore};
pub use config::{GrooveConfig, EnterpriseConfig, ProjectContext};
pub use export::{GrooveExport, LearningExport, ImportStats, EXPORT_VERSION};
```

**Step 5: Run tests**

Run: `cargo test -p vibes-groove export::tests`
Expected: PASS

**Step 6: Commit**

```bash
git add vibes-groove/src/export.rs vibes-groove/src/lib.rs
git commit -m "feat(groove): add GrooveExport and ImportStats types"
```

---

## Task 11: CozoStore Schema

**Files:**
- Create: `vibes-groove/src/store/schema.rs`
- Modify: `vibes-groove/src/store/mod.rs`

**Step 1: Write the test**

```rust
// vibes-groove/src/store/schema.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version_constant() {
        assert_eq!(CURRENT_SCHEMA_VERSION, 1);
    }

    #[test]
    fn test_initial_schema_contains_learning_table() {
        assert!(INITIAL_SCHEMA.contains(":create learning {"));
    }

    #[test]
    fn test_initial_schema_contains_usage_stats() {
        assert!(INITIAL_SCHEMA.contains(":create usage_stats {"));
    }

    #[test]
    fn test_initial_schema_contains_embeddings() {
        assert!(INITIAL_SCHEMA.contains(":create learning_embeddings {"));
    }

    #[test]
    fn test_initial_schema_contains_relations() {
        assert!(INITIAL_SCHEMA.contains(":create learning_relations {"));
    }

    #[test]
    fn test_initial_schema_contains_params() {
        assert!(INITIAL_SCHEMA.contains(":create adaptive_params {"));
    }

    #[test]
    fn test_initial_schema_contains_version_table() {
        assert!(INITIAL_SCHEMA.contains(":create schema_version {"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-groove store::schema::tests`
Expected: FAIL - module doesn't exist

**Step 3: Write implementation**

```rust
// vibes-groove/src/store/schema.rs
/// Current schema version
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Initial schema for groove database
pub const INITIAL_SCHEMA: &str = r#"
# Schema version tracking
:create schema_version {
    version: Int =>
    applied_at: Int,
    description: String
}

# Core learning entity
:create learning {
    id: String =>
    scope: String,
    category: String,
    description: String,
    pattern_json: String?,
    insight: String,
    confidence: Float,
    created_at: Int,
    updated_at: Int,
    source_type: String,
    source_json: String
}

# Per-learning usage statistics (updated frequently)
:create usage_stats {
    learning_id: String =>
    times_injected: Int,
    times_helpful: Int,
    times_ignored: Int,
    times_contradicted: Int,
    last_used: Int?,
    confidence_alpha: Float,
    confidence_beta: Float
}

# Embeddings for semantic search (384-dim GteSmall)
:create learning_embeddings {
    learning_id: String =>
    embedding: <F32; 384>
}

# Learning relationships
:create learning_relations {
    from_id: String,
    relation_type: String,
    to_id: String =>
    weight: Float,
    created_at: Int
}

# System-wide adaptive parameters
:create adaptive_params {
    param_name: String =>
    value: Float,
    uncertainty: Float,
    observations: Int,
    prior_alpha: Float,
    prior_beta: Float,
    updated_at: Int
}

# Indexes
::index create learning:by_scope { scope }
::index create learning:by_category { category }
::index create learning_relations:by_from { from_id }
::index create learning_relations:by_to { to_id }

# HNSW index for semantic search
::hnsw create learning_embeddings:semantic_idx {
    dim: 384,
    m: 16,
    ef_construction: 200,
    fields: [embedding]
}
"#;

/// Schema migration definition
pub struct Migration {
    pub version: u32,
    pub description: &'static str,
    pub script: &'static str,
}

/// All migrations in order
pub static MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "Initial schema",
        script: INITIAL_SCHEMA,
    },
];

#[cfg(test)]
mod tests {
    // ... tests from Step 1 ...
}
```

**Step 4: Update store/mod.rs**

```rust
// vibes-groove/src/store/mod.rs
mod traits;
mod schema;

pub use traits::{LearningStore, ParamStore};
pub use schema::{CURRENT_SCHEMA_VERSION, INITIAL_SCHEMA, MIGRATIONS, Migration};
```

**Step 5: Run tests**

Run: `cargo test -p vibes-groove store::schema::tests`
Expected: PASS

**Step 6: Commit**

```bash
git add vibes-groove/src/store/schema.rs vibes-groove/src/store/mod.rs
git commit -m "feat(groove): add CozoDB schema with HNSW index"
```

---

## Task 12: CozoStore Schema Migrations

**Files:**
- Create: `vibes-groove/src/store/cozo.rs`
- Modify: `vibes-groove/src/store/mod.rs`

**Step 1: Write the test**

```rust
// vibes-groove/src/store/cozo.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_cozo_store() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();
        assert!(store.is_initialized());
    }

    #[tokio::test]
    async fn test_schema_version_recorded() {
        let tmp = TempDir::new().unwrap();
        let store = CozoStore::open(tmp.path()).await.unwrap();
        let version = store.get_schema_version().await.unwrap();
        assert_eq!(version, CURRENT_SCHEMA_VERSION);
    }

    #[tokio::test]
    async fn test_reopen_existing_db() {
        let tmp = TempDir::new().unwrap();

        // Create and close
        {
            let store = CozoStore::open(tmp.path()).await.unwrap();
            assert_eq!(store.get_schema_version().await.unwrap(), CURRENT_SCHEMA_VERSION);
        }

        // Reopen
        {
            let store = CozoStore::open(tmp.path()).await.unwrap();
            assert_eq!(store.get_schema_version().await.unwrap(), CURRENT_SCHEMA_VERSION);
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-groove store::cozo::tests`
Expected: FAIL - module doesn't exist

**Step 3: Write implementation**

```rust
// vibes-groove/src/store/cozo.rs
use std::path::Path;
use std::sync::Arc;

use cozo::{DataValue, DbInstance, NamedRows};
use chrono::Utc;

use crate::{GrooveError, Result};
use super::schema::{CURRENT_SCHEMA_VERSION, MIGRATIONS};

/// CozoDB-backed learning store
pub struct CozoStore {
    db: Arc<DbInstance>,
    initialized: bool,
}

impl CozoStore {
    /// Open or create a groove database at the given path
    pub async fn open(path: &Path) -> Result<Self> {
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| GrooveError::Database(format!("Failed to create directory: {e}")))?;
        }

        let path_str = path.to_string_lossy();
        let db = DbInstance::new("rocksdb", &path_str, "")
            .map_err(|e| GrooveError::Database(format!("Failed to open database: {e}")))?;

        let db = Arc::new(db);
        let mut store = Self {
            db,
            initialized: false,
        };

        store.ensure_schema().await?;
        store.initialized = true;

        Ok(store)
    }

    /// Check if store is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get current schema version from database
    pub async fn get_schema_version(&self) -> Result<u32> {
        let query = "?[version] := *schema_version{version}, version = max(version)";

        match self.run_query(query, Default::default()).await {
            Ok(rows) if !rows.rows.is_empty() => {
                let version = rows.rows[0][0].get_int()
                    .ok_or_else(|| GrooveError::Database("Invalid version type".into()))?;
                Ok(version as u32)
            }
            Ok(_) => Ok(0), // No version recorded
            Err(e) => {
                // Table might not exist yet
                if e.to_string().contains("not found") {
                    Ok(0)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Ensure schema is up to date
    async fn ensure_schema(&mut self) -> Result<()> {
        let current = self.get_schema_version().await?;

        for migration in MIGRATIONS.iter().filter(|m| m.version > current) {
            self.apply_migration(migration).await?;
        }

        Ok(())
    }

    /// Apply a single migration
    async fn apply_migration(&self, migration: &super::schema::Migration) -> Result<()> {
        // Run migration script
        self.db.run_script(migration.script, Default::default(), cozo::ScriptMutability::Mutable)
            .map_err(|e| GrooveError::Migration(format!(
                "Migration {} failed: {e}", migration.version
            )))?;

        // Record migration
        let now = Utc::now().timestamp();
        let record_query = format!(
            "?[version, applied_at, description] <- [[{}, {}, '{}']] :put schema_version {{version => applied_at, description}}",
            migration.version,
            now,
            migration.description.replace('\'', "''")
        );

        self.db.run_script(&record_query, Default::default(), cozo::ScriptMutability::Mutable)
            .map_err(|e| GrooveError::Migration(format!(
                "Failed to record migration {}: {e}", migration.version
            )))?;

        Ok(())
    }

    /// Run a query and return results
    async fn run_query(
        &self,
        query: &str,
        params: cozo::BTreeMap<String, DataValue>,
    ) -> Result<NamedRows> {
        self.db.run_script(query, params, cozo::ScriptMutability::Immutable)
            .map_err(|e| GrooveError::Database(format!("Query failed: {e}")))
    }

    /// Run a mutation query
    async fn run_mutation(
        &self,
        query: &str,
        params: cozo::BTreeMap<String, DataValue>,
    ) -> Result<NamedRows> {
        self.db.run_script(query, params, cozo::ScriptMutability::Mutable)
            .map_err(|e| GrooveError::Database(format!("Mutation failed: {e}")))
    }

    /// Get a reference to the underlying database
    pub fn db(&self) -> &DbInstance {
        &self.db
    }
}

#[cfg(test)]
mod tests {
    // ... tests from Step 1 ...
}
```

**Step 4: Update store/mod.rs**

```rust
// vibes-groove/src/store/mod.rs
mod traits;
mod schema;
mod cozo;

pub use traits::{LearningStore, ParamStore};
pub use schema::{CURRENT_SCHEMA_VERSION, INITIAL_SCHEMA, MIGRATIONS, Migration};
pub use cozo::CozoStore;
```

**Step 5: Run tests**

Run: `cargo test -p vibes-groove store::cozo::tests`
Expected: PASS

**Step 6: Commit**

```bash
git add vibes-groove/src/store/cozo.rs vibes-groove/src/store/mod.rs
git commit -m "feat(groove): add CozoStore with schema migrations"
```

---

## Task 13: CozoStore Learning CRUD

**Files:**
- Modify: `vibes-groove/src/store/cozo.rs`

**Step 1: Write the test**

```rust
// Add to vibes-groove/src/store/cozo.rs tests module

#[tokio::test]
async fn test_store_and_get_learning() {
    let tmp = TempDir::new().unwrap();
    let store = CozoStore::open(tmp.path()).await.unwrap();

    let learning = Learning::new(
        Scope::User("test".into()),
        LearningCategory::Preference,
        LearningContent {
            description: "Test learning".into(),
            pattern: None,
            insight: "Test insight".into(),
        },
        LearningSource::UserCreated,
    );

    let id = store.store(&learning).await.unwrap();
    assert_eq!(id, learning.id);

    let retrieved = store.get(id).await.unwrap().unwrap();
    assert_eq!(retrieved.id, learning.id);
    assert_eq!(retrieved.content.description, learning.content.description);
}

#[tokio::test]
async fn test_find_by_scope() {
    let tmp = TempDir::new().unwrap();
    let store = CozoStore::open(tmp.path()).await.unwrap();

    let user_scope = Scope::User("alice".into());
    let other_scope = Scope::User("bob".into());

    // Create learnings for Alice
    for i in 0..3 {
        let learning = Learning::new(
            user_scope.clone(),
            LearningCategory::Preference,
            LearningContent {
                description: format!("Alice learning {i}"),
                pattern: None,
                insight: "insight".into(),
            },
            LearningSource::UserCreated,
        );
        store.store(&learning).await.unwrap();
    }

    // Create one for Bob
    let bob_learning = Learning::new(
        other_scope.clone(),
        LearningCategory::Preference,
        LearningContent {
            description: "Bob learning".into(),
            pattern: None,
            insight: "insight".into(),
        },
        LearningSource::UserCreated,
    );
    store.store(&bob_learning).await.unwrap();

    let alice_learnings = store.find_by_scope(&user_scope).await.unwrap();
    assert_eq!(alice_learnings.len(), 3);

    let bob_learnings = store.find_by_scope(&other_scope).await.unwrap();
    assert_eq!(bob_learnings.len(), 1);
}

#[tokio::test]
async fn test_delete_learning() {
    let tmp = TempDir::new().unwrap();
    let store = CozoStore::open(tmp.path()).await.unwrap();

    let learning = Learning::new(
        Scope::Global,
        LearningCategory::Solution,
        LearningContent {
            description: "To be deleted".into(),
            pattern: None,
            insight: "insight".into(),
        },
        LearningSource::UserCreated,
    );

    let id = store.store(&learning).await.unwrap();
    assert!(store.get(id).await.unwrap().is_some());

    let deleted = store.delete(id).await.unwrap();
    assert!(deleted);

    assert!(store.get(id).await.unwrap().is_none());
}

#[tokio::test]
async fn test_count_learnings() {
    let tmp = TempDir::new().unwrap();
    let store = CozoStore::open(tmp.path()).await.unwrap();

    assert_eq!(store.count().await.unwrap(), 0);

    for i in 0..5 {
        let learning = Learning::new(
            Scope::Global,
            LearningCategory::CodePattern,
            LearningContent {
                description: format!("Learning {i}"),
                pattern: None,
                insight: "insight".into(),
            },
            LearningSource::UserCreated,
        );
        store.store(&learning).await.unwrap();
    }

    assert_eq!(store.count().await.unwrap(), 5);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p vibes-groove store::cozo::tests::test_store_and_get`
Expected: FAIL - methods don't exist

**Step 3: Write implementation**

```rust
// Add to vibes-groove/src/store/cozo.rs (impl CozoStore)

use crate::{
    Learning, LearningCategory, LearningContent, LearningId, LearningSource, Scope,
};

impl CozoStore {
    // ... existing methods ...

    /// Store a new learning
    pub async fn store(&self, learning: &Learning) -> Result<LearningId> {
        let id_str = learning.id.to_string();
        let scope_str = learning.scope.to_db_string();
        let category_str = learning.category.as_str();
        let pattern_json = learning.content.pattern
            .as_ref()
            .map(|p| serde_json::to_string(p).unwrap_or_default());
        let source_json = serde_json::to_string(&learning.source)
            .map_err(|e| GrooveError::Serialization(e.to_string()))?;

        let query = format!(
            r#"?[id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json] <- [[
                '{}', '{}', '{}', '{}', {}, '{}', {}, {}, {}, '{}', '{}'
            ]]
            :put learning {{
                id => scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json
            }}"#,
            id_str,
            scope_str,
            category_str,
            learning.content.description.replace('\'', "''"),
            pattern_json.as_ref().map(|p| format!("'{}'", p.replace('\'', "''"))).unwrap_or_else(|| "null".to_string()),
            learning.content.insight.replace('\'', "''"),
            learning.confidence,
            learning.created_at.timestamp(),
            learning.updated_at.timestamp(),
            learning.source.source_type(),
            source_json.replace('\'', "''"),
        );

        self.run_mutation(&query, Default::default()).await?;

        // Also create default usage stats
        let stats_query = format!(
            r#"?[learning_id, times_injected, times_helpful, times_ignored, times_contradicted, last_used, confidence_alpha, confidence_beta] <- [[
                '{}', 0, 0, 0, 0, null, 1.0, 1.0
            ]]
            :put usage_stats {{
                learning_id => times_injected, times_helpful, times_ignored, times_contradicted, last_used, confidence_alpha, confidence_beta
            }}"#,
            id_str,
        );
        self.run_mutation(&stats_query, Default::default()).await?;

        Ok(learning.id)
    }

    /// Get a learning by ID
    pub async fn get(&self, id: LearningId) -> Result<Option<Learning>> {
        let query = format!(
            r#"?[id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json] :=
                *learning{{id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json}},
                id = '{}'"#,
            id
        );

        let rows = self.run_query(&query, Default::default()).await?;

        if rows.rows.is_empty() {
            return Ok(None);
        }

        let row = &rows.rows[0];
        self.row_to_learning(row)
    }

    /// Find all learnings in a scope
    pub async fn find_by_scope(&self, scope: &Scope) -> Result<Vec<Learning>> {
        let scope_str = scope.to_db_string();
        let query = format!(
            r#"?[id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json] :=
                *learning{{id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json}},
                scope = '{}'"#,
            scope_str
        );

        let rows = self.run_query(&query, Default::default()).await?;

        rows.rows.iter()
            .map(|row| self.row_to_learning(row))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .collect::<Vec<_>>()
            .into_iter()
            .map(|opt| opt.ok_or_else(|| GrooveError::Database("Invalid row".into())))
            .collect()
    }

    /// Find learnings by category
    pub async fn find_by_category(&self, category: &LearningCategory) -> Result<Vec<Learning>> {
        let category_str = category.as_str();
        let query = format!(
            r#"?[id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json] :=
                *learning{{id, scope, category, description, pattern_json, insight, confidence, created_at, updated_at, source_type, source_json}},
                category = '{}'"#,
            category_str
        );

        let rows = self.run_query(&query, Default::default()).await?;

        rows.rows.iter()
            .map(|row| self.row_to_learning(row))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .filter_map(|opt| opt)
            .collect::<Vec<_>>()
            .into_iter()
            .map(Ok)
            .collect()
    }

    /// Delete a learning
    pub async fn delete(&self, id: LearningId) -> Result<bool> {
        let id_str = id.to_string();

        // Check if exists first
        let exists = self.get(id).await?.is_some();
        if !exists {
            return Ok(false);
        }

        // Delete from all related tables
        let queries = [
            format!("?[id] <- [['{}']]:rm learning {{id}}", id_str),
            format!("?[learning_id] <- [['{}']]:rm usage_stats {{learning_id}}", id_str),
            format!("?[learning_id] <- [['{}']]:rm learning_embeddings {{learning_id}}", id_str),
        ];

        for query in queries {
            self.run_mutation(&query, Default::default()).await?;
        }

        Ok(true)
    }

    /// Count total learnings
    pub async fn count(&self) -> Result<u64> {
        let query = "?[count(id)] := *learning{id}";
        let rows = self.run_query(query, Default::default()).await?;

        if rows.rows.is_empty() {
            return Ok(0);
        }

        rows.rows[0][0].get_int()
            .map(|n| n as u64)
            .ok_or_else(|| GrooveError::Database("Invalid count".into()))
    }

    /// Convert a database row to a Learning struct
    fn row_to_learning(&self, row: &[DataValue]) -> Result<Option<Learning>> {
        let id_str = row[0].get_str()
            .ok_or_else(|| GrooveError::Database("Invalid id".into()))?;
        let id = uuid::Uuid::parse_str(id_str)
            .map_err(|e| GrooveError::Database(format!("Invalid UUID: {e}")))?;

        let scope_str = row[1].get_str()
            .ok_or_else(|| GrooveError::Database("Invalid scope".into()))?;
        let scope = Scope::from_db_string(scope_str)?;

        let category_str = row[2].get_str()
            .ok_or_else(|| GrooveError::Database("Invalid category".into()))?;
        let category = LearningCategory::from_str(category_str)
            .ok_or_else(|| GrooveError::Database(format!("Unknown category: {category_str}")))?;

        let description = row[3].get_str()
            .ok_or_else(|| GrooveError::Database("Invalid description".into()))?
            .to_string();

        let pattern: Option<serde_json::Value> = match &row[4] {
            DataValue::Null => None,
            DataValue::Str(s) => serde_json::from_str(s).ok(),
            _ => None,
        };

        let insight = row[5].get_str()
            .ok_or_else(|| GrooveError::Database("Invalid insight".into()))?
            .to_string();

        let confidence = row[6].get_float()
            .ok_or_else(|| GrooveError::Database("Invalid confidence".into()))?;

        let created_at = chrono::DateTime::from_timestamp(
            row[7].get_int().ok_or_else(|| GrooveError::Database("Invalid created_at".into()))?,
            0
        ).ok_or_else(|| GrooveError::Database("Invalid timestamp".into()))?;

        let updated_at = chrono::DateTime::from_timestamp(
            row[8].get_int().ok_or_else(|| GrooveError::Database("Invalid updated_at".into()))?,
            0
        ).ok_or_else(|| GrooveError::Database("Invalid timestamp".into()))?;

        let source_json = row[10].get_str()
            .ok_or_else(|| GrooveError::Database("Invalid source_json".into()))?;
        let source: LearningSource = serde_json::from_str(source_json)
            .map_err(|e| GrooveError::Serialization(format!("Invalid source: {e}")))?;

        Ok(Some(Learning {
            id,
            scope,
            category,
            content: LearningContent {
                description,
                pattern,
                insight,
            },
            confidence,
            created_at,
            updated_at,
            source,
        }))
    }
}
```

**Step 4: Run tests**

Run: `cargo test -p vibes-groove store::cozo::tests`
Expected: PASS

**Step 5: Commit**

```bash
git add vibes-groove/src/store/cozo.rs
git commit -m "feat(groove): add CozoStore learning CRUD operations"
```

---

## Tasks 14-20: Remaining Implementation

The remaining tasks follow the same TDD pattern:

| Task | Focus | Key Methods |
|------|-------|-------------|
| 14 | Usage stats | `update_usage()`, `get_usage()` |
| 15 | Relations | `store_relation()`, `find_related()` |
| 16 | Parameters | `get_param()`, `store_param()`, `all_params()` |
| 17 | Semantic search | `store_embedding()`, `semantic_search()` |
| 18 | GrooveStorage multi-tier | `new()`, `with_project()`, `with_enterprise()`, `get_read_tiers()` |
| 19 | Export/Import | `export()`, `import()` |
| 20 | PluginContext extensions | Modify vibes-plugin-api to add `harness()`, `capabilities()` |

Each task follows the same structure:
1. Write failing test
2. Run to verify failure
3. Implement minimal code
4. Run to verify pass
5. Commit

---

## Execution Checklist

- [ ] Task 1: Create vibes-groove crate
- [ ] Task 2: Error types
- [ ] Task 3: Scope types
- [ ] Task 4: Learning types
- [ ] Task 5: UsageStats and Outcome
- [ ] Task 6: Adaptive parameters
- [ ] Task 7: Relations types
- [ ] Task 8: Storage traits
- [ ] Task 9: Config types
- [ ] Task 10: Export/Import types
- [ ] Task 11: CozoStore schema
- [ ] Task 12: CozoStore schema migrations
- [ ] Task 13: CozoStore learning CRUD
- [ ] Task 14: CozoStore usage stats
- [ ] Task 15: CozoStore relations
- [ ] Task 16: CozoStore parameters
- [ ] Task 17: CozoStore semantic search
- [ ] Task 18: GrooveStorage multi-tier
- [ ] Task 19: Export/Import functionality
- [ ] Task 20: PluginContext extensions

---

## Final Verification

After completing all tasks:

```bash
# Run all tests
just test

# Run pre-commit checks
just pre-commit

# Verify schema creation works
cargo test -p vibes-groove -- --nocapture
```
