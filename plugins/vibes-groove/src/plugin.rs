//! Groove Plugin - Plugin trait implementation for vibes groove
//!
//! Provides CLI commands and HTTP routes for security, trust, and quarantine management.

use serde::{Deserialize, Serialize};
use vibes_core::hooks::{HookInstaller, HookInstallerConfig};
use vibes_plugin_api::{
    ArgSpec, AssessmentQuery, AssessmentQueryResponse, CommandOutput, CommandSpec, HttpMethod,
    Plugin, PluginAssessmentResult, PluginContext, PluginError, PluginManifest, RawEvent,
    RouteRequest, RouteResponse, RouteSpec,
};

use crate::assessment::{
    ActivityStatus, AssessmentConfig, AssessmentHistoryResponse, AssessmentStatsResponse,
    AssessmentStatusResponse, CircuitBreakerStatus, SamplingStatus, SessionHistoryItem,
    SessionStats, SyncAssessmentProcessor, TierDistribution,
};

use crate::CozoStore;
use crate::paths::GroovePaths;
use crate::security::load_policy_or_default;
use crate::security::{OrgRole, Policy, ReviewOutcome, TrustLevel};
use crate::strategy::{CozoStrategyStore, StrategyStore};
use crate::types::{Learning, LearningCategory, Scope};

/// Initialize the groove database at the configured path
///
/// Creates and initializes the CozoDB database with the groove schema.
/// This is called during `groove init` to ensure the database is ready.
///
/// If already in an async context, reuses the existing runtime handle for efficiency.
pub fn init_database(paths: &GroovePaths) -> Result<(), crate::GrooveError> {
    // Try to reuse existing runtime if we're already in an async context
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.block_on(async {
            // Open (and create if needed) the database - this also runs schema migrations
            let _store = CozoStore::open(&paths.db_path).await?;
            Ok::<(), crate::GrooveError>(())
        })
    } else {
        // Create new runtime if not in async context
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            crate::GrooveError::Database(format!("Failed to create runtime: {}", e))
        })?;

        rt.block_on(async {
            // Open (and create if needed) the database - this also runs schema migrations
            let _store = CozoStore::open(&paths.db_path).await?;
            Ok(())
        })
    }
}

// ============================================================================
// Response Types (mirrored from vibes-server for independence)
// ============================================================================

/// Security policy response
#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyResponse {
    pub injection: InjectionPolicyResponse,
    pub quarantine: QuarantinePolicyResponse,
    pub import_export: ImportExportPolicyResponse,
    pub audit: AuditPolicyResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InjectionPolicyResponse {
    pub block_quarantined: bool,
    pub allow_personal_injection: bool,
    pub allow_unverified_injection: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuarantinePolicyResponse {
    pub reviewers: Vec<String>,
    pub visible_to: Vec<String>,
    pub auto_delete_after_days: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportExportPolicyResponse {
    pub allow_import_from_file: bool,
    pub allow_import_from_url: bool,
    pub allowed_import_sources: Vec<String>,
    pub allow_export_personal: bool,
    pub allow_export_enterprise: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditPolicyResponse {
    pub enabled: bool,
    pub retention_days: u32,
}

impl From<Policy> for PolicyResponse {
    fn from(policy: Policy) -> Self {
        Self {
            injection: InjectionPolicyResponse {
                block_quarantined: policy.injection.block_quarantined,
                allow_personal_injection: policy.injection.allow_personal_injection,
                allow_unverified_injection: policy.injection.allow_unverified_injection,
            },
            quarantine: QuarantinePolicyResponse {
                reviewers: policy.quarantine.reviewers.clone(),
                visible_to: policy.quarantine.visible_to.clone(),
                auto_delete_after_days: policy.quarantine.auto_delete_after_days,
            },
            import_export: ImportExportPolicyResponse {
                allow_import_from_file: policy.import_export.allow_import_from_file,
                allow_import_from_url: policy.import_export.allow_import_from_url,
                allowed_import_sources: policy.import_export.allowed_import_sources.clone(),
                allow_export_personal: policy.import_export.allow_export_personal,
                allow_export_enterprise: policy.import_export.allow_export_enterprise,
            },
            audit: AuditPolicyResponse {
                enabled: policy.audit.enabled,
                retention_days: policy.audit.retention_days,
            },
        }
    }
}

/// Trust level information
#[derive(Debug, Serialize, Deserialize)]
pub struct TrustLevelInfo {
    pub name: String,
    pub score: u8,
    pub description: String,
}

/// Trust hierarchy response
#[derive(Debug, Serialize, Deserialize)]
pub struct TrustHierarchyResponse {
    pub levels: Vec<TrustLevelInfo>,
}

/// Role permissions response
#[derive(Debug, Serialize, Deserialize)]
pub struct RolePermissionsResponse {
    pub role: String,
    pub permissions: PermissionFlags,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionFlags {
    pub can_create: bool,
    pub can_read: bool,
    pub can_modify: bool,
    pub can_delete: bool,
    pub can_publish: bool,
    pub can_review: bool,
    pub can_admin: bool,
}

/// Quarantine list response
#[derive(Debug, Serialize, Deserialize)]
pub struct QuarantineListResponse {
    pub items: Vec<QuarantinedLearningSummary>,
    pub total: usize,
}

/// Summary of a quarantined learning
#[derive(Debug, Serialize, Deserialize)]
pub struct QuarantinedLearningSummary {
    pub id: String,
    pub description: String,
    pub trust_level: String,
    pub reason: String,
    pub quarantined_at: String,
    pub pending_review: bool,
}

/// Quarantine statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct QuarantineStatsResponse {
    pub total: usize,
    pub pending_review: usize,
    pub approved: usize,
    pub rejected: usize,
    pub escalated: usize,
}

/// Review request body
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewRequest {
    pub outcome: String,
    pub notes: Option<String>,
}

/// Review response
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewResponse {
    pub outcome: String,
    pub restored: bool,
    pub deleted: bool,
}

/// Error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

// ─── Learning API Response Types ──────────────────────────────────────────────

/// Learning extraction status response
#[derive(Debug, Serialize, Deserialize)]
pub struct LearningStatusResponse {
    pub counts_by_scope: ScopeCounts,
    pub counts_by_category: CategoryCounts,
    pub embedder: EmbedderStatus,
    pub last_extraction: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScopeCounts {
    pub project: u64,
    pub user: u64,
    pub global: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryCounts {
    pub correction: u64,
    pub error_recovery: u64,
    pub pattern: u64,
    pub preference: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbedderStatus {
    pub model: String,
    pub dimensions: usize,
    pub healthy: bool,
}

/// Learning list response
#[derive(Debug, Serialize, Deserialize)]
pub struct LearningListResponse {
    pub learnings: Vec<LearningSummary>,
    pub total: u64,
    pub page: usize,
    pub per_page: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LearningSummary {
    pub id: String,
    pub category: String,
    pub confidence: f64,
    pub description: String,
    pub scope: String,
    pub created_at: String,
}

/// Learning detail response
#[derive(Debug, Serialize, Deserialize)]
pub struct LearningDetailResponse {
    pub id: String,
    pub scope: String,
    pub category: String,
    pub description: String,
    pub insight: String,
    pub pattern: Option<serde_json::Value>,
    pub confidence: f64,
    pub created_at: String,
    pub updated_at: String,
    pub source: LearningSourceResponse,
    pub embedding_dimensions: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LearningSourceResponse {
    pub source_type: String,
    pub session_id: Option<String>,
    pub message_index: Option<u32>,
}

/// Learning export response
#[derive(Debug, Serialize, Deserialize)]
pub struct LearningExportResponse {
    pub learnings: Vec<LearningDetailResponse>,
    pub exported_at: String,
}

/// Delete response
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteResponse {
    pub deleted: bool,
    pub id: String,
}

// Assessment API types are now in crate::assessment::api_types
// (AssessmentStatusResponse, CircuitBreakerStatus, SamplingStatus, ActivityStatus,
//  AssessmentHistoryResponse, SessionHistoryItem, AssessmentStatsResponse,
//  TierDistribution, SessionStats)

// ─── Attribution API Response Types ───────────────────────────────────────────

/// Attribution engine status response
#[derive(Debug, Serialize, Deserialize)]
pub struct AttributionStatusResponse {
    pub total_learnings: u64,
    pub active_learnings: u64,
    pub deprecated_learnings: u64,
    pub experimental_learnings: u64,
    pub attributions_24h: u64,
    pub average_value: f64,
    pub consumer_running: bool,
}

/// Attribution values list response
#[derive(Debug, Serialize, Deserialize)]
pub struct AttributionValuesResponse {
    pub values: Vec<LearningValueSummary>,
    pub total: u64,
}

/// Summary of a learning's attribution value
#[derive(Debug, Serialize, Deserialize)]
pub struct LearningValueSummary {
    pub learning_id: String,
    pub estimated_value: f64,
    pub confidence: f64,
    pub session_count: u32,
    pub status: String,
}

/// Detailed attribution response for a specific learning
#[derive(Debug, Serialize, Deserialize)]
pub struct AttributionDetailResponse {
    pub learning_id: String,
    pub estimated_value: f64,
    pub confidence: f64,
    pub session_count: u32,
    pub activation_rate: f64,
    pub temporal_value: f64,
    pub temporal_confidence: f64,
    pub ablation_value: Option<f64>,
    pub ablation_confidence: Option<f64>,
    pub status: String,
    pub status_reason: Option<String>,
    pub recent_sessions: Vec<AttributionSessionSummary>,
}

/// Summary of attribution for a specific session
#[derive(Debug, Serialize, Deserialize)]
pub struct AttributionSessionSummary {
    pub session_id: String,
    pub timestamp: String,
    pub was_activated: bool,
    pub attributed_value: f64,
}

/// Attribution explanation response
#[derive(Debug, Serialize, Deserialize)]
pub struct AttributionExplainResponse {
    pub learning_id: String,
    pub session_id: String,
    pub timestamp: String,
    pub was_activated: bool,
    pub activation_confidence: f64,
    pub activation_signals: Vec<String>,
    pub temporal_positive: f64,
    pub temporal_negative: f64,
    pub net_temporal: f64,
    pub was_withheld: bool,
    pub session_outcome: f64,
    pub attributed_value: f64,
}

// ─── Strategy API Response Types ─────────────────────────────────────────────

/// Strategy learner status response
#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyStatusResponse {
    pub total_distributions: u64,
    pub total_overrides: u64,
    pub events_24h: u64,
    pub consumer_running: bool,
    pub top_strategies: Vec<TopStrategyInfo>,
}

/// Information about a top-performing strategy
#[derive(Debug, Serialize, Deserialize)]
pub struct TopStrategyInfo {
    pub variant: String,
    pub category: String,
    pub weight: f64,
    pub session_count: u32,
}

/// List of strategy distributions response
#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyDistributionsResponse {
    pub distributions: Vec<DistributionSummary>,
}

/// Summary of a single distribution
#[derive(Debug, Serialize, Deserialize)]
pub struct DistributionSummary {
    pub category: String,
    pub context_type: String,
    pub session_count: u32,
    pub leading_strategy: String,
    pub leading_weight: f64,
}

/// Detailed distribution response
#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyDistributionDetail {
    pub category: String,
    pub context_type: String,
    pub session_count: u32,
    pub weights: Vec<StrategyWeightInfo>,
    pub specialized_learnings: Vec<String>,
    pub updated_at: String,
}

/// Weight information for a strategy variant
#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyWeightInfo {
    pub variant: String,
    pub weight: f64,
    pub alpha: f64,
    pub beta: f64,
}

/// Learning strategy override response
#[derive(Debug, Serialize, Deserialize)]
pub struct LearningStrategyResponse {
    pub learning_id: String,
    pub base_category: String,
    pub is_specialized: bool,
    pub session_count: u32,
    pub specialization_threshold: u32,
    pub effective_weights: Vec<StrategyWeightInfo>,
    pub category_weights: Vec<StrategyWeightInfo>,
}

/// Strategy selection history response
#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyHistoryResponse {
    pub events: Vec<StrategyEventSummary>,
}

/// Summary of a strategy selection event
#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyEventSummary {
    pub event_id: String,
    pub session_id: String,
    pub strategy_variant: String,
    pub outcome_value: f64,
    pub outcome_confidence: f64,
    pub outcome_source: String,
    pub timestamp: String,
}

// ============================================================================
// Server URL Configuration (for CLI → Server HTTP)
// ============================================================================

/// Default vibes server port (matches vibes-cli DEFAULT_PORT)
pub const DEFAULT_SERVER_PORT: u16 = 7432;

/// Default vibes server host
pub const DEFAULT_SERVER_HOST: &str = "127.0.0.1";

/// API endpoint path for assessment status
pub const API_ASSESS_STATUS_PATH: &str = "/api/groove/assess/status";

/// API endpoint path for assessment history
pub const API_ASSESS_HISTORY_PATH: &str = "/api/groove/assess/history";

/// API endpoint path for assessment stats
pub const API_ASSESS_STATS_PATH: &str = "/api/groove/assess/stats";

/// API endpoint path for learning status
pub const API_LEARN_STATUS_PATH: &str = "/api/groove/learnings/status";

/// API endpoint path for learning list
pub const API_LEARN_LIST_PATH: &str = "/api/groove/learnings";

/// Server configuration for CLI HTTP calls
#[derive(Debug, Clone)]
pub struct ServerUrlConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerUrlConfig {
    fn default() -> Self {
        Self {
            host: DEFAULT_SERVER_HOST.to_string(),
            port: DEFAULT_SERVER_PORT,
        }
    }
}

impl ServerUrlConfig {
    /// Build base URL from config
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }

    /// Build full URL for assessment status endpoint
    pub fn status_url(&self) -> String {
        format!("{}{}", self.base_url(), API_ASSESS_STATUS_PATH)
    }

    /// Build full URL for assessment history endpoint
    pub fn history_url(&self, session_id: Option<&str>) -> String {
        self.history_url_with_pagination(session_id, None, None)
    }

    /// Build URL for assessment history with pagination parameters
    pub fn history_url_with_pagination(
        &self,
        session_id: Option<&str>,
        page: Option<usize>,
        per_page: Option<usize>,
    ) -> String {
        let base = format!("{}{}", self.base_url(), API_ASSESS_HISTORY_PATH);
        let mut params: Vec<String> = Vec::new();

        if let Some(id) = session_id {
            params.push(format!("session={}", id));
        }
        if let Some(p) = page {
            params.push(format!("page={}", p));
        }
        if let Some(pp) = per_page {
            params.push(format!("per_page={}", pp));
        }

        if params.is_empty() {
            base
        } else {
            format!("{}?{}", base, params.join("&"))
        }
    }

    /// Parse from --url flag value (e.g., "http://localhost:8080")
    pub fn from_url(url_str: &str) -> Result<Self, String> {
        // Strip trailing slash
        let url_str = url_str.trim_end_matches('/');

        // Parse URL
        let url = url::Url::parse(url_str).map_err(|e| format!("Invalid URL: {}", e))?;

        let host = url.host_str().ok_or("URL must have a host")?.to_string();
        let port = url.port().unwrap_or(DEFAULT_SERVER_PORT);

        Ok(Self { host, port })
    }

    /// Build full URL for learnings status endpoint
    pub fn learnings_status_url(&self) -> String {
        format!("{}{}", self.base_url(), API_LEARN_STATUS_PATH)
    }

    /// Build URL for learnings list with pagination and filters
    pub fn learnings_list_url(
        &self,
        scope: Option<&str>,
        category: Option<&str>,
        page: Option<usize>,
        per_page: Option<usize>,
    ) -> String {
        let base = format!("{}{}", self.base_url(), API_LEARN_LIST_PATH);
        let mut params: Vec<String> = Vec::new();

        if let Some(s) = scope {
            params.push(format!("scope={}", s));
        }
        if let Some(c) = category {
            params.push(format!("category={}", c));
        }
        if let Some(p) = page {
            params.push(format!("page={}", p));
        }
        if let Some(pp) = per_page {
            params.push(format!("per_page={}", pp));
        }

        if params.is_empty() {
            base
        } else {
            format!("{}?{}", base, params.join("&"))
        }
    }

    /// Build URL for a specific learning by ID
    pub fn learning_url(&self, id: &str) -> String {
        format!("{}{}/{}", self.base_url(), API_LEARN_LIST_PATH, id)
    }
}

// ============================================================================
// Assessment Consumer
// ============================================================================
//
// The assessment consumer is spawned in on_ready() using the host's runtime
// handle. Since the plugin runs in-process (not as a separate .so), we can
// safely use async code by spawning tasks through the PluginContext's
// runtime_handle.
//
// The consumer replays from the beginning of the event log to process full
// session history for pattern detection.

// ============================================================================
// Plugin Implementation
// ============================================================================

/// Groove continual learning plugin
///
/// Provides CLI commands and HTTP routes for:
/// - Trust level hierarchy management
/// - Security policy viewing
/// - Quarantine queue management
///
/// Implements `on_event()` for synchronous event processing, enabling the
/// host to call plugin assessment logic without async complications.
#[derive(Default)]
pub struct GroovePlugin {
    /// Synchronous assessment processor for event callbacks.
    /// Initialized during `on_load()` with default config.
    processor: Option<SyncAssessmentProcessor>,
}

impl Plugin for GroovePlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            name: "groove".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Continual learning system for vibes".to_string(),
            author: "vibes".to_string(),
            ..Default::default()
        }
    }

    fn on_load(&mut self, ctx: &mut PluginContext) -> Result<(), PluginError> {
        ctx.log_info("Loading groove plugin");

        // NOTE: Don't initialize processor here - it's created lazily in on_event()
        // This ensures CLI mode (no events) queries the server instead of empty local state

        // Register CLI commands
        self.register_commands(ctx)?;

        // Register HTTP routes
        self.register_routes(ctx)?;

        ctx.log_info("Groove plugin loaded successfully");
        Ok(())
    }

    fn on_ready(&mut self, ctx: &mut PluginContext) -> Result<(), PluginError> {
        ctx.log_info("Groove plugin ready");
        // Assessment processing is handled via on_event() callback which is called
        // synchronously by the host for each event. The host owns the AssessmentLog
        // due to TypeId mismatch issues with dynamic libraries.
        Ok(())
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        Ok(())
    }

    fn on_hook(
        &mut self,
        session_id: Option<&str>,
        hook_type: &str,
        project_path: Option<&str>,
        ctx: &mut PluginContext,
    ) -> Option<String> {
        ctx.log_debug(&format!(
            "Received hook: type={}, session={:?}, project={:?}",
            hook_type, session_id, project_path
        ));

        // Handle hook types - some can inject context, others are fire-and-forget
        match hook_type {
            "SessionStart" => {
                // In the future, this will query learned context and return it
                // For now, just log that we received the event
                ctx.log_info(&format!("Session started for project: {:?}", project_path));
                // Return None for now - context injection will be implemented in Milestone 4.5
                None
            }
            "UserPromptSubmit" => {
                // In the future, this will capture the prompt and potentially inject context
                ctx.log_debug("User prompt submitted");
                None
            }
            "PermissionRequest" => {
                // PermissionRequest can block or modify dangerous operations
                // In the future, this will check against learned policies
                ctx.log_debug("Permission request received");
                // Return None to allow the operation (no blocking for now)
                None
            }
            "SessionEnd" => {
                // Session ended - good time to finalize any pending assessments
                ctx.log_info(&format!("Session ended for session: {:?}", session_id));
                None
            }
            "Notification" | "SubagentStop" | "PreCompact" => {
                // Fire-and-forget hooks - just log for now
                ctx.log_debug(&format!("Received {} hook", hook_type));
                None
            }
            _ => {
                // Unknown hook types are logged but not processed
                ctx.log_debug(&format!("Unknown hook type: {}", hook_type));
                None
            }
        }
    }

    fn on_event(
        &mut self,
        event: RawEvent,
        _ctx: &mut PluginContext,
    ) -> Vec<PluginAssessmentResult> {
        // Lazily initialize the processor on first event (server mode only)
        if self.processor.is_none() {
            let config = AssessmentConfig::default();
            self.processor = Some(SyncAssessmentProcessor::new(config));
        }

        // Delegate to the processor
        self.processor
            .as_ref()
            .map(|p| p.process(&event))
            .unwrap_or_default()
    }

    fn query_assessment_results(
        &self,
        query: AssessmentQuery,
        _ctx: &PluginContext,
    ) -> AssessmentQueryResponse {
        // Delegate to the synchronous processor if initialized
        match &self.processor {
            Some(processor) => processor.query(query),
            None => AssessmentQueryResponse::default(),
        }
    }

    fn handle_command(
        &mut self,
        path: &[&str],
        args: &vibes_plugin_api::CommandArgs,
        _ctx: &mut PluginContext,
    ) -> Result<CommandOutput, PluginError> {
        match path {
            ["init"] => self.cmd_init(args),
            ["list"] => self.cmd_list(args),
            ["status"] => self.cmd_status(),
            ["trust", "levels"] => self.cmd_trust_levels(),
            ["trust", "role"] => self.cmd_trust_role(args),
            ["policy", "show"] => self.cmd_policy_show(),
            ["policy", "path"] => self.cmd_policy_path(),
            ["quarantine", "list"] => self.cmd_quarantine_list(),
            ["quarantine", "stats"] => self.cmd_quarantine_stats(),
            ["assess", "status"] => self.cmd_assess_status(args),
            ["assess", "history"] => self.cmd_assess_history(args),
            ["learn", "status"] => self.cmd_learn_status(args),
            ["learn", "list"] => self.cmd_learn_list(args),
            ["learn", "show"] => self.cmd_learn_show(args),
            ["learn", "delete"] => self.cmd_learn_delete(args),
            ["learn", "export"] => self.cmd_learn_export(args),
            ["learn", "enable"] => self.cmd_learn_enable(args),
            ["learn", "disable"] => self.cmd_learn_disable(args),
            ["attr", "status"] => self.cmd_attr_status(args),
            ["attr", "values"] => self.cmd_attr_values(args),
            ["attr", "show"] => self.cmd_attr_show(args),
            ["attr", "explain"] => self.cmd_attr_explain(args),
            _ => Err(PluginError::UnknownCommand(path.join(" "))),
        }
    }

    fn handle_route(
        &mut self,
        method: HttpMethod,
        path: &str,
        request: RouteRequest,
        _ctx: &mut PluginContext,
    ) -> Result<RouteResponse, PluginError> {
        match (method, path) {
            (HttpMethod::Get, "/policy") => self.route_get_policy(),
            (HttpMethod::Get, "/trust/levels") => self.route_get_trust_levels(),
            (HttpMethod::Get, "/trust/role/:role") => self.route_get_role_permissions(&request),
            (HttpMethod::Get, "/quarantine") => self.route_list_quarantined(),
            (HttpMethod::Get, "/quarantine/stats") => self.route_get_quarantine_stats(),
            (HttpMethod::Post, "/quarantine/:id/review") => self.route_review_quarantined(&request),
            (HttpMethod::Get, "/assess/status") => self.route_assess_status(),
            (HttpMethod::Get, "/assess/history") => self.route_assess_history(&request),
            (HttpMethod::Get, "/assess/stats") => self.route_assess_stats(),
            (HttpMethod::Get, "/learnings/status") => self.route_learnings_status(),
            (HttpMethod::Get, "/learnings") => self.route_learnings_list(&request),
            (HttpMethod::Get, "/learnings/:id") => self.route_learnings_get(&request),
            (HttpMethod::Delete, "/learnings/:id") => self.route_learnings_delete(&request),
            (HttpMethod::Post, "/learnings/:id/enable") => self.route_learnings_enable(&request),
            (HttpMethod::Post, "/learnings/:id/disable") => self.route_learnings_disable(&request),
            (HttpMethod::Get, "/attr/status") => self.route_attr_status(),
            (HttpMethod::Get, "/attr/values") => self.route_attr_values(&request),
            (HttpMethod::Get, "/attr/show/:id") => self.route_attr_show(&request),
            (HttpMethod::Get, "/attr/explain/:learning_id/:session_id") => {
                self.route_attr_explain(&request)
            }
            // Strategy routes
            (HttpMethod::Get, "/strategy/status") => self.route_strategy_status(),
            (HttpMethod::Get, "/strategy/distributions") => self.route_strategy_distributions(),
            (HttpMethod::Get, "/strategy/show/:category/:context_type") => {
                self.route_strategy_show(&request)
            }
            (HttpMethod::Get, "/strategy/learning/:id") => self.route_strategy_learning(&request),
            (HttpMethod::Get, "/strategy/history/:learning_id") => {
                self.route_strategy_history(&request)
            }
            (HttpMethod::Post, "/strategy/reset/:category/:context_type") => {
                self.route_strategy_reset(&request)
            }
            (HttpMethod::Post, "/strategy/reset-learning/:id") => {
                self.route_strategy_reset_learning(&request)
            }
            _ => Err(PluginError::UnknownRoute(format!("{:?} {}", method, path))),
        }
    }
}

impl GroovePlugin {
    // ─── Command Registration ─────────────────────────────────────────

    fn register_commands(&self, ctx: &mut PluginContext) -> Result<(), PluginError> {
        // init [project_path]
        ctx.register_command(CommandSpec {
            path: vec!["init".into()],
            description: "Initialize groove for a project".into(),
            args: vec![ArgSpec {
                name: "project_path".into(),
                description: "Project path (defaults to current directory)".into(),
                required: false,
            }],
        })?;

        // list [limit]
        ctx.register_command(CommandSpec {
            path: vec!["list".into()],
            description: "List captured learnings".into(),
            args: vec![ArgSpec {
                name: "limit".into(),
                description: "Maximum number of learnings to show (default: 10)".into(),
                required: false,
            }],
        })?;

        // status
        ctx.register_command(CommandSpec {
            path: vec!["status".into()],
            description: "Show groove system status".into(),
            args: vec![],
        })?;

        // trust levels
        ctx.register_command(CommandSpec {
            path: vec!["trust".into(), "levels".into()],
            description: "Show trust level hierarchy".into(),
            args: vec![],
        })?;

        // trust role <role>
        ctx.register_command(CommandSpec {
            path: vec!["trust".into(), "role".into()],
            description: "Show permissions for a role".into(),
            args: vec![ArgSpec {
                name: "role".into(),
                description: "Role name (admin, curator, member, viewer)".into(),
                required: true,
            }],
        })?;

        // policy show
        ctx.register_command(CommandSpec {
            path: vec!["policy".into(), "show".into()],
            description: "Show current security policy".into(),
            args: vec![],
        })?;

        // policy path
        ctx.register_command(CommandSpec {
            path: vec!["policy".into(), "path".into()],
            description: "Show policy file search paths".into(),
            args: vec![],
        })?;

        // quarantine list
        ctx.register_command(CommandSpec {
            path: vec!["quarantine".into(), "list".into()],
            description: "List quarantined learnings".into(),
            args: vec![],
        })?;

        // quarantine stats
        ctx.register_command(CommandSpec {
            path: vec!["quarantine".into(), "stats".into()],
            description: "Show quarantine statistics".into(),
            args: vec![],
        })?;

        // assess status
        ctx.register_command(CommandSpec {
            path: vec!["assess".into(), "status".into()],
            description: "Show assessment system status (circuit state, recent events)".into(),
            args: vec![],
        })?;

        // assess history [session_id]
        ctx.register_command(CommandSpec {
            path: vec!["assess".into(), "history".into()],
            description: "Show past assessments for a session".into(),
            args: vec![ArgSpec {
                name: "session_id".into(),
                description: "Session ID to show history for".into(),
                required: false,
            }],
        })?;

        // learn status
        ctx.register_command(CommandSpec {
            path: vec!["learn".into(), "status".into()],
            description: "Show extraction status and counts".into(),
            args: vec![],
        })?;

        // learn list [--scope] [--category]
        ctx.register_command(CommandSpec {
            path: vec!["learn".into(), "list".into()],
            description: "List learnings with optional filters".into(),
            args: vec![
                ArgSpec {
                    name: "scope".into(),
                    description: "Filter by scope (project, user, global)".into(),
                    required: false,
                },
                ArgSpec {
                    name: "category".into(),
                    description:
                        "Filter by category (correction, error_recovery, pattern, preference)"
                            .into(),
                    required: false,
                },
            ],
        })?;

        // learn show <id>
        ctx.register_command(CommandSpec {
            path: vec!["learn".into(), "show".into()],
            description: "Show full learning details".into(),
            args: vec![ArgSpec {
                name: "id".into(),
                description: "Learning ID to show".into(),
                required: true,
            }],
        })?;

        // learn delete <id>
        ctx.register_command(CommandSpec {
            path: vec!["learn".into(), "delete".into()],
            description: "Delete a learning".into(),
            args: vec![ArgSpec {
                name: "id".into(),
                description: "Learning ID to delete".into(),
                required: true,
            }],
        })?;

        // learn export [--scope]
        ctx.register_command(CommandSpec {
            path: vec!["learn".into(), "export".into()],
            description: "Export learnings as JSON".into(),
            args: vec![ArgSpec {
                name: "scope".into(),
                description: "Filter by scope (project, user, global)".into(),
                required: false,
            }],
        })?;

        // learn enable <id>
        ctx.register_command(CommandSpec {
            path: vec!["learn".into(), "enable".into()],
            description: "Re-enable a deprecated learning".into(),
            args: vec![ArgSpec {
                name: "id".into(),
                description: "Learning ID to enable".into(),
                required: true,
            }],
        })?;

        // learn disable <id> [reason]
        ctx.register_command(CommandSpec {
            path: vec!["learn".into(), "disable".into()],
            description: "Manually deprecate a learning".into(),
            args: vec![
                ArgSpec {
                    name: "id".into(),
                    description: "Learning ID to disable".into(),
                    required: true,
                },
                ArgSpec {
                    name: "reason".into(),
                    description: "Reason for deprecation".into(),
                    required: false,
                },
            ],
        })?;

        // attr status
        ctx.register_command(CommandSpec {
            path: vec!["attr".into(), "status".into()],
            description: "Show attribution engine status".into(),
            args: vec![],
        })?;

        // attr values [--sort] [--limit]
        ctx.register_command(CommandSpec {
            path: vec!["attr".into(), "values".into()],
            description: "List learning values".into(),
            args: vec![
                ArgSpec {
                    name: "sort".into(),
                    description: "Sort by: value, confidence, sessions (default: value)".into(),
                    required: false,
                },
                ArgSpec {
                    name: "limit".into(),
                    description: "Maximum results (default: 20)".into(),
                    required: false,
                },
            ],
        })?;

        // attr show <id>
        ctx.register_command(CommandSpec {
            path: vec!["attr".into(), "show".into()],
            description: "Show detailed attribution for a learning".into(),
            args: vec![ArgSpec {
                name: "id".into(),
                description: "Learning ID to show".into(),
                required: true,
            }],
        })?;

        // attr explain <learning_id> <session_id>
        ctx.register_command(CommandSpec {
            path: vec!["attr".into(), "explain".into()],
            description: "Explain attribution for a specific session".into(),
            args: vec![
                ArgSpec {
                    name: "learning_id".into(),
                    description: "Learning ID".into(),
                    required: true,
                },
                ArgSpec {
                    name: "session_id".into(),
                    description: "Session ID".into(),
                    required: true,
                },
            ],
        })?;

        // strategy status
        ctx.register_command(CommandSpec {
            path: vec!["strategy".into(), "status".into()],
            description: "Show strategy learner status".into(),
            args: vec![],
        })?;

        // strategy distributions
        ctx.register_command(CommandSpec {
            path: vec!["strategy".into(), "distributions".into()],
            description: "List category distributions".into(),
            args: vec![],
        })?;

        // strategy show <category> <context_type>
        ctx.register_command(CommandSpec {
            path: vec!["strategy".into(), "show".into()],
            description: "Detailed distribution breakdown".into(),
            args: vec![
                ArgSpec {
                    name: "category".into(),
                    description:
                        "Learning category (code_pattern, preference, solution, correction)".into(),
                    required: true,
                },
                ArgSpec {
                    name: "context_type".into(),
                    description: "Context type (interactive, code_review, batch)".into(),
                    required: true,
                },
            ],
        })?;

        // strategy learning <id>
        ctx.register_command(CommandSpec {
            path: vec!["strategy".into(), "learning".into()],
            description: "Show learning's strategy override".into(),
            args: vec![ArgSpec {
                name: "id".into(),
                description: "Learning ID".into(),
                required: true,
            }],
        })?;

        // strategy history <learning_id> [limit]
        ctx.register_command(CommandSpec {
            path: vec!["strategy".into(), "history".into()],
            description: "Strategy selection history for a learning".into(),
            args: vec![
                ArgSpec {
                    name: "learning_id".into(),
                    description: "Learning ID".into(),
                    required: true,
                },
                ArgSpec {
                    name: "limit".into(),
                    description: "Maximum events to show (default: 20)".into(),
                    required: false,
                },
            ],
        })?;

        // strategy reset <category> <context_type> --confirm
        ctx.register_command(CommandSpec {
            path: vec!["strategy".into(), "reset".into()],
            description: "Reset category to default priors".into(),
            args: vec![
                ArgSpec {
                    name: "category".into(),
                    description: "Learning category".into(),
                    required: true,
                },
                ArgSpec {
                    name: "context_type".into(),
                    description: "Context type".into(),
                    required: true,
                },
            ],
        })?;

        // strategy reset-learning <id> --confirm
        ctx.register_command(CommandSpec {
            path: vec!["strategy".into(), "reset-learning".into()],
            description: "Clear learning specialization".into(),
            args: vec![ArgSpec {
                name: "id".into(),
                description: "Learning ID".into(),
                required: true,
            }],
        })?;

        Ok(())
    }

    // ─── Route Registration ───────────────────────────────────────────

    fn register_routes(&self, ctx: &mut PluginContext) -> Result<(), PluginError> {
        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/policy".into(),
        })?;

        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/trust/levels".into(),
        })?;

        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/trust/role/:role".into(),
        })?;

        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/quarantine".into(),
        })?;

        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/quarantine/stats".into(),
        })?;

        ctx.register_route(RouteSpec {
            method: HttpMethod::Post,
            path: "/quarantine/:id/review".into(),
        })?;

        // Assessment routes
        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/assess/status".into(),
        })?;

        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/assess/history".into(),
        })?;

        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/assess/stats".into(),
        })?;

        // Learning routes
        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/learnings/status".into(),
        })?;

        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/learnings".into(),
        })?;

        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/learnings/:id".into(),
        })?;

        ctx.register_route(RouteSpec {
            method: HttpMethod::Delete,
            path: "/learnings/:id".into(),
        })?;

        // Attribution routes
        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/attr/status".into(),
        })?;

        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/attr/values".into(),
        })?;

        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/attr/show/:id".into(),
        })?;

        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/attr/explain/:learning_id/:session_id".into(),
        })?;

        // Learn enable/disable routes
        ctx.register_route(RouteSpec {
            method: HttpMethod::Post,
            path: "/learnings/:id/enable".into(),
        })?;

        ctx.register_route(RouteSpec {
            method: HttpMethod::Post,
            path: "/learnings/:id/disable".into(),
        })?;

        // Strategy routes
        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/strategy/status".into(),
        })?;

        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/strategy/distributions".into(),
        })?;

        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/strategy/show/:category/:context_type".into(),
        })?;

        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/strategy/learning/:id".into(),
        })?;

        ctx.register_route(RouteSpec {
            method: HttpMethod::Get,
            path: "/strategy/history/:learning_id".into(),
        })?;

        ctx.register_route(RouteSpec {
            method: HttpMethod::Post,
            path: "/strategy/reset/:category/:context_type".into(),
        })?;

        ctx.register_route(RouteSpec {
            method: HttpMethod::Post,
            path: "/strategy/reset-learning/:id".into(),
        })?;

        Ok(())
    }

    // ─── Server Client (CLI → Server HTTP) ────────────────────────────

    /// Find .vibes/config.toml by searching up from current directory
    pub fn find_config_file() -> Option<std::path::PathBuf> {
        let mut dir = std::env::current_dir().ok()?;
        loop {
            let config_path = dir.join(".vibes/config.toml");
            if config_path.exists() {
                return Some(config_path);
            }
            if !dir.pop() {
                return None;
            }
        }
    }

    /// Load server URL config from .vibes/config.toml (searching up directories)
    pub fn load_server_config() -> ServerUrlConfig {
        if let Some(config_path) = Self::find_config_file()
            && let Ok(contents) = std::fs::read_to_string(&config_path)
        {
            // Parse just the server section
            #[derive(serde::Deserialize)]
            struct ConfigFile {
                #[serde(default)]
                server: ServerSection,
            }
            #[derive(serde::Deserialize, Default)]
            struct ServerSection {
                host: Option<String>,
                port: Option<u16>,
            }

            if let Ok(config) = toml::from_str::<ConfigFile>(&contents) {
                return ServerUrlConfig {
                    host: config
                        .server
                        .host
                        .unwrap_or_else(|| DEFAULT_SERVER_HOST.to_string()),
                    port: config.server.port.unwrap_or(DEFAULT_SERVER_PORT),
                };
            }
        }
        ServerUrlConfig::default()
    }

    /// Parse --url flag from command args, returning (url_config, remaining_args)
    pub fn parse_url_flag(args: &[String]) -> (Option<ServerUrlConfig>, Vec<String>) {
        let mut url_config = None;
        let mut remaining = Vec::new();
        let mut skip_next = false;

        for (i, arg) in args.iter().enumerate() {
            if skip_next {
                skip_next = false;
                continue;
            }

            if arg == "--url"
                && let Some(url) = args.get(i + 1)
                && let Ok(config) = ServerUrlConfig::from_url(url)
            {
                url_config = Some(config);
                skip_next = true;
                continue;
            } else if arg.starts_with("--url=")
                && let Some(url) = arg.strip_prefix("--url=")
                && let Ok(config) = ServerUrlConfig::from_url(url)
            {
                url_config = Some(config);
                continue;
            }

            remaining.push(arg.clone());
        }

        (url_config, remaining)
    }

    /// Parse a single flag value from arguments (e.g., --scope project)
    pub fn parse_flag(args: &[String], flag: &str) -> Option<String> {
        for (i, arg) in args.iter().enumerate() {
            if arg == flag {
                return args.get(i + 1).cloned();
            }
            if let Some(value) = arg.strip_prefix(&format!("{}=", flag)) {
                return Some(value.to_string());
            }
        }
        None
    }

    /// Parse --page and --per-page flags from arguments
    pub fn parse_pagination_flags(args: &[String]) -> (Option<usize>, Option<usize>, Vec<String>) {
        let mut page = None;
        let mut per_page = None;
        let mut remaining = Vec::new();
        let mut skip_next = false;

        for (i, arg) in args.iter().enumerate() {
            if skip_next {
                skip_next = false;
                continue;
            }

            if arg == "--page" {
                if let Some(val) = args.get(i + 1) {
                    page = val.parse().ok();
                    skip_next = true;
                }
                continue;
            } else if arg.starts_with("--page=") {
                if let Some(val) = arg.strip_prefix("--page=") {
                    page = val.parse().ok();
                }
                continue;
            }

            if arg == "--per-page" {
                if let Some(val) = args.get(i + 1) {
                    per_page = val.parse().ok();
                    skip_next = true;
                }
                continue;
            } else if arg.starts_with("--per-page=") {
                if let Some(val) = arg.strip_prefix("--per-page=") {
                    per_page = val.parse().ok();
                }
                continue;
            }

            remaining.push(arg.clone());
        }

        (page, per_page, remaining)
    }

    /// Check if args contain --help or -h
    pub fn wants_help(args: &[String]) -> bool {
        args.iter().any(|a| a == "--help" || a == "-h")
    }

    /// Build the base URL for the vibes server (legacy, for tests)
    pub fn server_base_url(port: Option<u16>) -> String {
        let port = port.unwrap_or(DEFAULT_SERVER_PORT);
        format!("http://{}:{}", DEFAULT_SERVER_HOST, port)
    }

    /// Fetch assessment status from the running vibes server
    pub async fn fetch_status_from_server(
        port: Option<u16>,
    ) -> Result<AssessmentStatusResponse, String> {
        let config = port.map_or_else(Self::load_server_config, |p| ServerUrlConfig {
            host: DEFAULT_SERVER_HOST.to_string(),
            port: p,
        });
        Self::fetch_status_with_config(&config).await
    }

    /// Fetch assessment status using explicit config
    pub async fn fetch_status_with_config(
        config: &ServerUrlConfig,
    ) -> Result<AssessmentStatusResponse, String> {
        let url = config.status_url();

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client.get(&url).send().await.map_err(|e| {
            format!(
                "Failed to connect to server at {}: {}",
                config.base_url(),
                e
            )
        })?;

        if !response.status().is_success() {
            return Err(format!("Server returned error: {}", response.status()));
        }

        response
            .json::<AssessmentStatusResponse>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Fetch assessment history from the running vibes server
    pub async fn fetch_history_from_server(
        session_id: Option<&str>,
        port: Option<u16>,
    ) -> Result<AssessmentHistoryResponse, String> {
        let config = port.map_or_else(Self::load_server_config, |p| ServerUrlConfig {
            host: DEFAULT_SERVER_HOST.to_string(),
            port: p,
        });
        Self::fetch_history_with_config(session_id, &config).await
    }

    /// Fetch assessment history using explicit config
    pub async fn fetch_history_with_config(
        session_id: Option<&str>,
        config: &ServerUrlConfig,
    ) -> Result<AssessmentHistoryResponse, String> {
        Self::fetch_history_with_pagination(session_id, config, None, None).await
    }

    /// Fetch assessment history with pagination using explicit config
    pub async fn fetch_history_with_pagination(
        session_id: Option<&str>,
        config: &ServerUrlConfig,
        page: Option<usize>,
        per_page: Option<usize>,
    ) -> Result<AssessmentHistoryResponse, String> {
        let url = config.history_url_with_pagination(session_id, page, per_page);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client.get(&url).send().await.map_err(|e| {
            format!(
                "Failed to connect to server at {}: {}",
                config.base_url(),
                e
            )
        })?;

        if !response.status().is_success() {
            return Err(format!("Server returned error: {}", response.status()));
        }

        response
            .json::<AssessmentHistoryResponse>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Blocking version of fetch_status using config
    fn fetch_status_blocking(config: &ServerUrlConfig) -> Result<AssessmentStatusResponse, String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;
        let config = config.clone();
        rt.block_on(Self::fetch_status_with_config(&config))
    }

    /// Blocking version of fetch_history using config
    fn fetch_history_blocking(
        session_id: Option<&str>,
        config: &ServerUrlConfig,
        page: Option<usize>,
        per_page: Option<usize>,
    ) -> Result<AssessmentHistoryResponse, String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;
        let config = config.clone();
        rt.block_on(Self::fetch_history_with_pagination(
            session_id, &config, page, per_page,
        ))
    }

    // ─── Learnings HTTP Client Methods ────────────────────────────────

    /// Fetch learnings status from server
    pub async fn fetch_learnings_status_with_config(
        config: &ServerUrlConfig,
    ) -> Result<LearningStatusResponse, String> {
        let url = config.learnings_status_url();

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client.get(&url).send().await.map_err(|e| {
            format!(
                "Failed to connect to server at {}: {}",
                config.base_url(),
                e
            )
        })?;

        if !response.status().is_success() {
            return Err(format!("Server returned error: {}", response.status()));
        }

        response
            .json::<LearningStatusResponse>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Blocking version of fetch_learnings_status
    fn fetch_learnings_status_blocking(
        config: &ServerUrlConfig,
    ) -> Result<LearningStatusResponse, String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;
        let config = config.clone();
        rt.block_on(Self::fetch_learnings_status_with_config(&config))
    }

    /// Fetch learnings list from server with optional filters
    pub async fn fetch_learnings_list_with_config(
        config: &ServerUrlConfig,
        scope: Option<&str>,
        category: Option<&str>,
        page: Option<usize>,
        per_page: Option<usize>,
    ) -> Result<LearningListResponse, String> {
        let url = config.learnings_list_url(scope, category, page, per_page);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client.get(&url).send().await.map_err(|e| {
            format!(
                "Failed to connect to server at {}: {}",
                config.base_url(),
                e
            )
        })?;

        if !response.status().is_success() {
            return Err(format!("Server returned error: {}", response.status()));
        }

        response
            .json::<LearningListResponse>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Blocking version of fetch_learnings_list
    fn fetch_learnings_list_blocking(
        config: &ServerUrlConfig,
        scope: Option<&str>,
        category: Option<&str>,
        page: Option<usize>,
        per_page: Option<usize>,
    ) -> Result<LearningListResponse, String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;
        let config = config.clone();
        let scope = scope.map(|s| s.to_string());
        let category = category.map(|c| c.to_string());
        rt.block_on(Self::fetch_learnings_list_with_config(
            &config,
            scope.as_deref(),
            category.as_deref(),
            page,
            per_page,
        ))
    }

    /// Fetch a specific learning by ID
    pub async fn fetch_learning_with_config(
        config: &ServerUrlConfig,
        id: &str,
    ) -> Result<LearningDetailResponse, String> {
        let url = config.learning_url(id);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client.get(&url).send().await.map_err(|e| {
            format!(
                "Failed to connect to server at {}: {}",
                config.base_url(),
                e
            )
        })?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(format!("Learning not found: {}", id));
        }

        if !response.status().is_success() {
            return Err(format!("Server returned error: {}", response.status()));
        }

        response
            .json::<LearningDetailResponse>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Blocking version of fetch_learning
    fn fetch_learning_blocking(
        config: &ServerUrlConfig,
        id: &str,
    ) -> Result<LearningDetailResponse, String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;
        let config = config.clone();
        let id = id.to_string();
        rt.block_on(Self::fetch_learning_with_config(&config, &id))
    }

    /// Delete a learning by ID
    pub async fn delete_learning_with_config(
        config: &ServerUrlConfig,
        id: &str,
    ) -> Result<DeleteResponse, String> {
        let url = config.learning_url(id);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client.delete(&url).send().await.map_err(|e| {
            format!(
                "Failed to connect to server at {}: {}",
                config.base_url(),
                e
            )
        })?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(format!("Learning not found: {}", id));
        }

        if !response.status().is_success() {
            return Err(format!("Server returned error: {}", response.status()));
        }

        response
            .json::<DeleteResponse>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    /// Blocking version of delete_learning
    fn delete_learning_blocking(
        config: &ServerUrlConfig,
        id: &str,
    ) -> Result<DeleteResponse, String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;
        let config = config.clone();
        let id = id.to_string();
        rt.block_on(Self::delete_learning_with_config(&config, &id))
    }

    // ─── Command Handlers ─────────────────────────────────────────────

    fn cmd_init(&self, args: &vibes_plugin_api::CommandArgs) -> Result<CommandOutput, PluginError> {
        // Get project path from args or use current directory
        let project_path = args
            .args
            .first()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        let mut output = String::new();
        output.push_str("Initializing groove for continual learning...\n\n");

        // Create groove directories
        let paths = GroovePaths::default();
        if let Err(e) = paths.ensure_dirs() {
            return Err(PluginError::custom(format!(
                "Failed to create directories: {}",
                e
            )));
        }
        output.push_str(&format!(
            "✓ Created data directory: {}\n",
            paths.data_dir.display()
        ));
        output.push_str(&format!(
            "✓ Created transcripts directory: {}\n",
            paths.transcripts_dir.display()
        ));
        output.push_str(&format!(
            "✓ Created learnings directory: {}\n",
            paths.learnings_dir.display()
        ));

        // Initialize database
        match init_database(&paths) {
            Ok(()) => {
                output.push_str(&format!(
                    "✓ Initialized database: {}\n",
                    paths.db_path.display()
                ));
            }
            Err(e) => {
                output.push_str(&format!("⚠ Could not initialize database: {}\n", e));
                output.push_str("  Database will be created on first use.\n");
            }
        }

        // Install hooks
        let hook_config = HookInstallerConfig::default();
        let installer = HookInstaller::new(hook_config);

        match installer.install() {
            Ok(()) => {
                output.push_str("✓ Installed Claude Code hooks\n");
            }
            Err(e) => {
                output.push_str(&format!("⚠ Could not install hooks: {}\n", e));
                output
                    .push_str("  You can manually install hooks by running 'vibes claude' once.\n");
            }
        }

        output.push('\n');
        output.push_str(&format!(
            "Groove initialized for project: {}\n",
            project_path.display()
        ));
        output.push_str("\nNext steps:\n");
        output.push_str("  1. Run 'vibes claude' to start a session with learning capture\n");
        output.push_str("  2. Run 'vibes groove status' to check system status\n");
        output.push_str("  3. Run 'vibes groove list' to view captured learnings\n");

        Ok(CommandOutput::Text(output))
    }

    fn cmd_list(&self, args: &vibes_plugin_api::CommandArgs) -> Result<CommandOutput, PluginError> {
        let limit = args
            .args
            .first()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(10);

        let mut output = String::new();
        output.push_str(&format!("Learnings (limit: {}):\n\n", limit));

        // Stub implementation - full implementation requires storage integration
        output.push_str("No learnings captured yet.\n");
        output.push_str("\nStart a session with 'vibes claude' to begin capturing learnings.\n");

        Ok(CommandOutput::Text(output))
    }

    fn cmd_status(&self) -> Result<CommandOutput, PluginError> {
        let mut output = String::new();
        output.push_str("Groove System Status\n");
        output.push_str(&format!("{}\n\n", "=".repeat(40)));

        // Check paths
        let paths = GroovePaths::default();
        output.push_str("Directories:\n");
        let check = |exists: bool| if exists { "✓" } else { "✗" };
        output.push_str(&format!(
            "  {} Data:        {}\n",
            check(paths.data_dir.exists()),
            paths.data_dir.display()
        ));
        output.push_str(&format!(
            "  {} Transcripts: {}\n",
            check(paths.transcripts_dir.exists()),
            paths.transcripts_dir.display()
        ));
        output.push_str(&format!(
            "  {} Learnings:   {}\n",
            check(paths.learnings_dir.exists()),
            paths.learnings_dir.display()
        ));
        output.push_str(&format!(
            "  {} Database:    {}\n",
            check(paths.db_path.exists()),
            paths.db_path.display()
        ));

        // Check hooks
        output.push_str("\nHooks:\n");
        if let Some(hooks_dir) = GroovePaths::claude_projects_dir() {
            let hooks_path = hooks_dir.parent().unwrap_or(&hooks_dir).join("hooks");
            if hooks_path.exists() {
                output.push_str(&format!("  ✓ Hooks directory: {}\n", hooks_path.display()));
            } else {
                output.push_str("  ✗ Hooks not installed\n");
                output.push_str("    Run 'vibes groove init' to install hooks\n");
            }
        } else {
            output.push_str("  ? Could not determine hooks directory\n");
        }

        // Summary
        output.push_str("\nStatus: ");
        if paths.data_dir.exists() && paths.transcripts_dir.exists() {
            output.push_str("Ready\n");
        } else {
            output.push_str("Not initialized\n");
            output.push_str("  Run 'vibes groove init' to set up groove\n");
        }

        Ok(CommandOutput::Text(output))
    }

    fn cmd_trust_levels(&self) -> Result<CommandOutput, PluginError> {
        let mut output = String::new();
        output.push_str("Trust Level Hierarchy (highest to lowest):\n\n");
        output.push_str(&format!("  {:<24} {:>6}  Description\n", "Level", "Score"));
        output.push_str(&format!(
            "  {} {} {}\n",
            "-".repeat(24),
            "-".repeat(6),
            "-".repeat(36)
        ));
        output.push_str(&format!(
            "  {:<24} {:>6}  Locally created content (full trust)\n",
            "Local", "100"
        ));
        output.push_str(&format!(
            "  {:<24} {:>6}  Synced from user's own cloud\n",
            "PrivateCloud", "90"
        ));
        output.push_str(&format!(
            "  {:<24} {:>6}  Enterprise content, curator approved\n",
            "OrganizationVerified", "70"
        ));
        output.push_str(&format!(
            "  {:<24} {:>6}  Enterprise content, not yet approved\n",
            "OrganizationUnverified", "50"
        ));
        output.push_str(&format!(
            "  {:<24} {:>6}  Community content, verified\n",
            "PublicVerified", "30"
        ));
        output.push_str(&format!(
            "  {:<24} {:>6}  Community content, unverified\n",
            "PublicUnverified", "10"
        ));
        output.push_str(&format!(
            "  {:<24} {:>6}  Quarantined (blocked)\n",
            "Quarantined", "0"
        ));
        output.push_str("\nInjection Policy:\n");
        output
            .push_str("  - Local, PrivateCloud, OrganizationVerified: Allowed without scanning\n");
        output.push_str("  - OrganizationUnverified, PublicVerified: Requires scanning\n");
        output.push_str("  - PublicUnverified: Requires scanning, may show warnings\n");
        output.push_str("  - Quarantined: Blocked from injection\n");

        Ok(CommandOutput::Text(output))
    }

    fn cmd_trust_role(
        &self,
        args: &vibes_plugin_api::CommandArgs,
    ) -> Result<CommandOutput, PluginError> {
        let role_str = args
            .args
            .first()
            .ok_or_else(|| PluginError::InvalidInput("Missing role argument".into()))?;

        let role: OrgRole = role_str.parse().map_err(|_| {
            PluginError::InvalidInput(format!(
                "Invalid role: {}. Use: admin, curator, member, viewer",
                role_str
            ))
        })?;

        let perms = role.permissions();

        let check = |b: bool| if b { "Y" } else { "N" };

        let mut output = String::new();
        output.push_str(&format!("Role: {}\n\n", role.as_str()));
        output.push_str("Permissions:\n");
        output.push_str(&format!("  Create:   {}\n", check(perms.can_create)));
        output.push_str(&format!("  Read:     {}\n", check(perms.can_read)));
        output.push_str(&format!("  Modify:   {}\n", check(perms.can_modify)));
        output.push_str(&format!("  Delete:   {}\n", check(perms.can_delete)));
        output.push_str(&format!("  Publish:  {}\n", check(perms.can_publish)));
        output.push_str(&format!("  Review:   {}\n", check(perms.can_review)));
        output.push_str(&format!("  Admin:    {}\n", check(perms.can_admin)));

        Ok(CommandOutput::Text(output))
    }

    fn cmd_policy_show(&self) -> Result<CommandOutput, PluginError> {
        let policy = load_policy_or_default("groove-policy.toml");

        let mut output = String::new();
        output.push_str("Current Security Policy:\n\n");

        output.push_str("Injection Policy:\n");
        output.push_str(&format!(
            "  Block quarantined:       {}\n",
            policy.injection.block_quarantined
        ));
        output.push_str(&format!(
            "  Allow personal:          {}\n",
            policy.injection.allow_personal_injection
        ));
        output.push_str(&format!(
            "  Allow unverified:        {}\n",
            policy.injection.allow_unverified_injection
        ));
        output.push('\n');

        output.push_str("Quarantine Policy:\n");
        output.push_str(&format!(
            "  Reviewers:               {:?}\n",
            policy.quarantine.reviewers
        ));
        output.push_str(&format!(
            "  Visible to:              {:?}\n",
            policy.quarantine.visible_to
        ));
        output.push_str(&format!(
            "  Auto-delete after days:  {:?}\n",
            policy.quarantine.auto_delete_after_days
        ));
        output.push('\n');

        output.push_str("Import/Export Policy:\n");
        output.push_str(&format!(
            "  Allow import from file:  {}\n",
            policy.import_export.allow_import_from_file
        ));
        output.push_str(&format!(
            "  Allow import from URL:   {}\n",
            policy.import_export.allow_import_from_url
        ));
        output.push_str(&format!(
            "  Allowed import sources:  {:?}\n",
            policy.import_export.allowed_import_sources
        ));
        output.push_str(&format!(
            "  Allow export personal:   {}\n",
            policy.import_export.allow_export_personal
        ));
        output.push_str(&format!(
            "  Allow export enterprise: {}\n",
            policy.import_export.allow_export_enterprise
        ));
        output.push('\n');

        output.push_str("Audit Policy:\n");
        output.push_str(&format!(
            "  Enabled:                 {}\n",
            policy.audit.enabled
        ));
        output.push_str(&format!(
            "  Retention days:          {:?}\n",
            policy.audit.retention_days
        ));

        Ok(CommandOutput::Text(output))
    }

    fn cmd_policy_path(&self) -> Result<CommandOutput, PluginError> {
        let mut output = String::new();
        output.push_str("Policy search paths:\n");
        output.push_str("  1. ./groove-policy.toml\n");
        output.push_str("  2. ~/.config/vibes/groove-policy.toml\n");
        output.push_str("  3. /etc/vibes/groove-policy.toml\n\n");
        output.push_str("If no policy file is found, defaults are used.\n");

        Ok(CommandOutput::Text(output))
    }

    fn cmd_quarantine_list(&self) -> Result<CommandOutput, PluginError> {
        // Placeholder - full implementation requires storage integration
        let mut output = String::new();
        output.push_str("Quarantine queue listing not yet implemented.\n");
        output.push_str("This will show learnings pending review.\n");

        Ok(CommandOutput::Text(output))
    }

    fn cmd_quarantine_stats(&self) -> Result<CommandOutput, PluginError> {
        // Placeholder - full implementation requires storage integration
        let mut output = String::new();
        output.push_str("Quarantine statistics not yet implemented.\n");
        output.push_str("This will show quarantine queue metrics.\n");

        Ok(CommandOutput::Text(output))
    }

    fn cmd_assess_status(
        &self,
        args: &vibes_plugin_api::CommandArgs,
    ) -> Result<CommandOutput, PluginError> {
        // Handle --help
        if Self::wants_help(&args.args) {
            return Ok(CommandOutput::Text(
                "Usage: vibes groove assess status [OPTIONS]\n\n\
                 Show assessment system status including circuit breaker state,\n\
                 sampling configuration, and recent activity.\n\n\
                 Options:\n\
                   --url <URL>  Server URL (default: from .vibes/config.toml or http://127.0.0.1:7432)\n\
                   --help, -h   Show this help message\n"
                    .to_string(),
            ));
        }

        // Parse --url flag
        let (url_config, _remaining) = Self::parse_url_flag(&args.args);

        let mut output = String::new();
        output.push_str("Assessment System Status\n");
        output.push_str(&format!("{}\n\n", "=".repeat(40)));

        // Get real data from processor if available (server context)
        if let Some(processor) = &self.processor {
            // Circuit breaker status
            let cb_summary = processor.circuit_breaker_summary();
            output.push_str("Circuit Breaker:\n");
            let cb_state = if cb_summary.enabled {
                "Closed (normal operation)"
            } else {
                "Disabled"
            };
            output.push_str(&format!("  State:           {}\n", cb_state));
            output.push_str(&format!(
                "  Cooldown:        {} seconds\n",
                cb_summary.cooldown_seconds
            ));
            output.push_str(&format!(
                "  Max per session: {}\n\n",
                cb_summary.max_interventions_per_session
            ));

            // Sampling status
            let sampling = processor.sampling_summary();
            output.push_str("Sampling Strategy:\n");
            output.push_str(&format!(
                "  Base rate:       {:.0}%\n",
                sampling.base_rate * 100.0
            ));
            output.push_str(&format!(
                "  Burnin sessions: {}\n\n",
                sampling.burnin_sessions
            ));

            // Recent activity from actual data
            let sessions = processor.active_sessions();
            let event_count = processor.stored_results_count();
            let intervention_count = processor.total_intervention_count();
            output.push_str("Recent Activity:\n");
            output.push_str(&format!("  Active sessions: {}\n", sessions.len()));
            output.push_str(&format!("  Events stored:   {}\n", event_count));
            output.push_str(&format!("  Interventions:   {}\n\n", intervention_count));
        } else {
            // No processor (CLI mode) - try to fetch from running server
            let config = url_config.unwrap_or_else(Self::load_server_config);
            output.push_str(&format!("(Querying server at {})\n\n", config.base_url()));

            match Self::fetch_status_blocking(&config) {
                Ok(status) => {
                    // Circuit breaker status
                    output.push_str("Circuit Breaker:\n");
                    let cb_state = if status.circuit_breaker.enabled {
                        "Closed (normal operation)"
                    } else {
                        "Disabled"
                    };
                    output.push_str(&format!("  State:           {}\n", cb_state));
                    output.push_str(&format!(
                        "  Cooldown:        {} seconds\n",
                        status.circuit_breaker.cooldown_seconds
                    ));
                    output.push_str(&format!(
                        "  Max per session: {}\n\n",
                        status.circuit_breaker.max_interventions_per_session
                    ));

                    // Sampling status
                    output.push_str("Sampling Strategy:\n");
                    output.push_str(&format!(
                        "  Base rate:       {:.0}%\n",
                        status.sampling.base_rate * 100.0
                    ));
                    output.push_str(&format!(
                        "  Burnin sessions: {}\n\n",
                        status.sampling.burnin_sessions
                    ));

                    // Recent activity
                    output.push_str("Recent Activity:\n");
                    output.push_str(&format!(
                        "  Active sessions: {}\n",
                        status.activity.active_sessions
                    ));
                    output.push_str(&format!(
                        "  Events stored:   {}\n",
                        status.activity.events_stored
                    ));
                    output.push_str(&format!(
                        "  Interventions:   {}\n\n",
                        status.activity.intervention_count
                    ));
                }
                Err(e) => {
                    // Server not running or not responding
                    output.push_str(&format!("Error: {}\n\n", e));
                    output.push_str("Circuit Breaker:\n");
                    output.push_str("  Status: Not initialized\n\n");
                    output.push_str("Sampling Strategy:\n");
                    output.push_str("  Status: Not initialized\n\n");
                    output.push_str("Recent Activity:\n");
                    output.push_str(
                        "  No data available. Start server with 'vibes serve' first.\n\n",
                    );
                }
            }
        }

        output.push_str("Note: Assessment consumer starts automatically with 'vibes claude'.\n");

        Ok(CommandOutput::Text(output))
    }

    fn cmd_assess_history(
        &self,
        args: &vibes_plugin_api::CommandArgs,
    ) -> Result<CommandOutput, PluginError> {
        // Handle --help
        if Self::wants_help(&args.args) {
            return Ok(CommandOutput::Text(
                "Usage: vibes groove assess history [OPTIONS] [SESSION_ID]\n\n\
                 Show assessment history. Without SESSION_ID, lists all sessions.\n\
                 With SESSION_ID, shows events for that specific session.\n\n\
                 Arguments:\n\
                   [SESSION_ID]  Session ID to show details for (optional)\n\n\
                 Options:\n\
                   --url <URL>       Server URL (default: from .vibes/config.toml or http://127.0.0.1:7432)\n\
                   --page <N>        Page number (default: 1)\n\
                   --per-page <N>    Items per page (default: 20, max: 100)\n\
                   --help, -h        Show this help message\n"
                    .to_string(),
            ));
        }

        // Parse --url flag
        let (url_config, remaining) = Self::parse_url_flag(&args.args);

        // Parse --page and --per-page flags
        let (page, per_page, remaining) = Self::parse_pagination_flags(&remaining);

        // Session ID is the first non-flag argument
        let session_id = remaining.first().map(|s| s.as_str());

        let mut output = String::new();
        output.push_str("Assessment History\n");
        output.push_str(&format!("{}\n\n", "=".repeat(40)));

        if let Some(processor) = &self.processor {
            // Server context - query local processor
            if let Some(id) = session_id {
                // Query events for specific session
                output.push_str(&format!("Session: {}\n\n", id));

                let query = AssessmentQuery::new().with_session(id).with_limit(20);
                let response = processor.query(query);

                if response.results.is_empty() {
                    output.push_str("No assessments found for this session.\n");
                } else {
                    output.push_str(&format!(
                        "Showing {} most recent events:\n\n",
                        response.results.len()
                    ));
                    for result in &response.results {
                        output.push_str(&format!(
                            "  [{:12}] {}\n",
                            result.result_type, result.event_id
                        ));
                    }
                    if response.has_more {
                        output.push_str("\n  ... and more events available.\n");
                    }
                }
            } else {
                // List all sessions
                let sessions = processor.active_sessions();
                output.push_str("Recent Sessions:\n");

                if sessions.is_empty() {
                    output.push_str("  No session history available.\n");
                } else {
                    for session in &sessions {
                        // Get event count for each session
                        let query = AssessmentQuery::new().with_session(session);
                        let response = processor.query(query);
                        output.push_str(&format!(
                            "  {} ({} events)\n",
                            session,
                            response.results.len()
                        ));
                    }
                }
                output.push_str(
                    "\nTip: Run 'vibes groove assess history <session_id>' for details.\n",
                );
            }
        } else {
            // No processor (CLI mode) - try to fetch from running server
            let config = url_config.unwrap_or_else(Self::load_server_config);
            output.push_str(&format!("(Querying server at {})\n\n", config.base_url()));

            match Self::fetch_history_blocking(session_id, &config, page, per_page) {
                Ok(history) => {
                    if history.sessions.is_empty() {
                        output.push_str("Recent Sessions:\n");
                        output.push_str("  No session history available.\n");
                    } else {
                        output.push_str("Recent Sessions:\n");
                        for session in &history.sessions {
                            let types_str = if session.result_types.is_empty() {
                                String::new()
                            } else {
                                format!(" [{}]", session.result_types.join(", "))
                            };
                            output.push_str(&format!(
                                "  {} ({} events){}\n",
                                session.session_id, session.event_count, types_str
                            ));
                        }
                        if history.has_more {
                            output.push_str("\n  ... and more sessions available.\n");
                        }
                    }
                    output.push_str(
                        "\nTip: Run 'vibes groove assess history <session_id>' for details.\n",
                    );
                }
                Err(e) => {
                    // Server not running or not responding
                    output.push_str(&format!("Error: {}\n\n", e));
                    output.push_str("Assessment processor not initialized.\n");
                    output.push_str("Start server with 'vibes serve' first.\n");
                }
            }
        }

        Ok(CommandOutput::Text(output))
    }

    // ─── Learn Commands ───────────────────────────────────────────────

    fn cmd_learn_status(
        &self,
        args: &vibes_plugin_api::CommandArgs,
    ) -> Result<CommandOutput, PluginError> {
        if Self::wants_help(&args.args) {
            return Ok(CommandOutput::Text(
                "Usage: vibes groove learn status\n\n\
                 Show learning extraction status and counts.\n"
                    .to_string(),
            ));
        }

        // Load server config and fetch status
        let config = Self::load_server_config();
        let status = Self::fetch_learnings_status_blocking(&config)
            .map_err(|e| PluginError::custom(format!("Failed to fetch status: {}", e)))?;

        let mut output = String::new();
        output.push_str("Learning Extraction Status\n");
        output.push_str("========================================\n\n");

        output.push_str("Learnings by scope:\n");
        output.push_str(&format!("  Project: {}\n", status.counts_by_scope.project));
        output.push_str(&format!("  User: {}\n", status.counts_by_scope.user));
        output.push_str(&format!("  Global: {}\n\n", status.counts_by_scope.global));

        output.push_str("Learnings by category:\n");
        output.push_str(&format!(
            "  Correction: {}\n",
            status.counts_by_category.correction
        ));
        output.push_str(&format!(
            "  ErrorRecovery: {}\n",
            status.counts_by_category.error_recovery
        ));
        output.push_str(&format!(
            "  Pattern: {}\n",
            status.counts_by_category.pattern
        ));
        output.push_str(&format!(
            "  Preference: {}\n\n",
            status.counts_by_category.preference
        ));

        let embedder_status = if status.embedder.healthy {
            "loaded"
        } else {
            "not loaded"
        };
        output.push_str(&format!(
            "Embedder: {} ({} dims) - {}\n",
            status.embedder.model, status.embedder.dimensions, embedder_status
        ));

        match status.last_extraction {
            Some(ts) => output.push_str(&format!("Last extraction: {}\n", ts)),
            None => output.push_str("Last extraction: never\n"),
        }

        Ok(CommandOutput::Text(output))
    }

    fn cmd_learn_list(
        &self,
        args: &vibes_plugin_api::CommandArgs,
    ) -> Result<CommandOutput, PluginError> {
        if Self::wants_help(&args.args) {
            return Ok(CommandOutput::Text(
                "Usage: vibes groove learn list [OPTIONS]\n\n\
                 List learnings with optional filters.\n\n\
                 Options:\n\
                   --scope <SCOPE>      Filter by scope (project, user, global)\n\
                   --category <CAT>     Filter by category (correction, error_recovery, pattern, preference)\n\
                   --help, -h           Show this help message\n"
                    .to_string(),
            ));
        }

        // Parse optional filters
        let scope = Self::parse_flag(&args.args, "--scope");
        let category = Self::parse_flag(&args.args, "--category");

        // Load server config and fetch list
        let config = Self::load_server_config();
        let list = Self::fetch_learnings_list_blocking(
            &config,
            scope.as_deref(),
            category.as_deref(),
            None,
            None,
        )
        .map_err(|e| PluginError::custom(format!("Failed to fetch learnings: {}", e)))?;

        if list.learnings.is_empty() {
            return Ok(CommandOutput::Text("No learnings found.\n".to_string()));
        }

        let mut output = String::new();
        output.push_str("ID       Category        Confidence  Scope    Description\n");
        output.push_str(
            "──────── ─────────────── ─────────── ──────── ─────────────────────────────\n",
        );

        for learning in &list.learnings {
            let id_short = if learning.id.len() > 8 {
                &learning.id[..8]
            } else {
                &learning.id
            };
            let desc_truncated = if learning.description.len() > 29 {
                format!("{}...", &learning.description[..26])
            } else {
                learning.description.clone()
            };
            output.push_str(&format!(
                "{:<8} {:<15} {:>10.2}  {:<8} {}\n",
                id_short, learning.category, learning.confidence, learning.scope, desc_truncated
            ));
        }

        output.push_str(&format!(
            "\nShowing {} of {} learnings\n",
            list.learnings.len(),
            list.total
        ));

        Ok(CommandOutput::Text(output))
    }

    fn cmd_learn_show(
        &self,
        args: &vibes_plugin_api::CommandArgs,
    ) -> Result<CommandOutput, PluginError> {
        if Self::wants_help(&args.args) || args.args.is_empty() {
            return Ok(CommandOutput::Text(
                "Usage: vibes groove learn show <ID>\n\n\
                 Show full details for a learning.\n\n\
                 Arguments:\n\
                   <ID>  Learning ID to show\n"
                    .to_string(),
            ));
        }

        let id = &args.args[0];

        // Load server config and fetch learning
        let config = Self::load_server_config();
        let learning = Self::fetch_learning_blocking(&config, id).map_err(PluginError::custom)?;

        let mut output = String::new();
        output.push_str(&format!("Learning: {}\n", learning.id));
        output.push_str("========================================\n\n");
        output.push_str(&format!("Category:    {}\n", learning.category));
        output.push_str(&format!("Confidence:  {:.2}\n", learning.confidence));
        output.push_str(&format!("Scope:       {}\n", learning.scope));
        output.push_str(&format!("Created:     {}\n\n", learning.created_at));

        output.push_str("Description:\n");
        output.push_str(&format!("  {}\n\n", learning.description));

        output.push_str("Insight:\n");
        output.push_str(&format!("  {}\n\n", learning.insight));

        output.push_str("Source:\n");
        output.push_str(&format!("  Type: {}\n", learning.source.source_type));
        if let Some(session) = &learning.source.session_id {
            output.push_str(&format!("  Session: {}\n", session));
        }
        if let Some(idx) = &learning.source.message_index {
            output.push_str(&format!("  Message: {}\n", idx));
        }

        if let Some(pattern) = &learning.pattern {
            output.push_str(&format!("\nPattern: {}\n", pattern));
        }

        Ok(CommandOutput::Text(output))
    }

    fn cmd_learn_delete(
        &self,
        args: &vibes_plugin_api::CommandArgs,
    ) -> Result<CommandOutput, PluginError> {
        if Self::wants_help(&args.args) || args.args.is_empty() {
            return Ok(CommandOutput::Text(
                "Usage: vibes groove learn delete <ID>\n\n\
                 Delete a learning.\n\n\
                 Arguments:\n\
                   <ID>  Learning ID to delete\n"
                    .to_string(),
            ));
        }

        let id = &args.args[0];

        // Load server config and delete learning
        let config = Self::load_server_config();
        let result = Self::delete_learning_blocking(&config, id).map_err(PluginError::custom)?;

        if result.deleted {
            Ok(CommandOutput::Text(format!(
                "Deleted learning: {}\n",
                result.id
            )))
        } else {
            Ok(CommandOutput::Text(format!("Learning not found: {}\n", id)))
        }
    }

    fn cmd_learn_export(
        &self,
        args: &vibes_plugin_api::CommandArgs,
    ) -> Result<CommandOutput, PluginError> {
        if Self::wants_help(&args.args) {
            return Ok(CommandOutput::Text(
                "Usage: vibes groove learn export [OPTIONS]\n\n\
                 Export learnings as JSON.\n\n\
                 Options:\n\
                   --scope <SCOPE>  Filter by scope (project, user, global)\n\
                   --help, -h       Show this help message\n"
                    .to_string(),
            ));
        }

        // Parse optional scope filter
        let scope = Self::parse_flag(&args.args, "--scope");

        // Load server config and fetch learnings
        let config = Self::load_server_config();
        let list = Self::fetch_learnings_list_blocking(
            &config,
            scope.as_deref(),
            None,       // No category filter for export
            None,       // No pagination for export
            Some(1000), // Large page size to get all learnings
        )
        .map_err(|e| PluginError::custom(format!("Failed to fetch learnings: {}", e)))?;

        // Build export response
        let export = LearningExportResponse {
            learnings: list
                .learnings
                .into_iter()
                .map(|l| LearningDetailResponse {
                    id: l.id,
                    scope: l.scope,
                    category: l.category,
                    description: l.description,
                    insight: String::new(), // Not available in summary
                    pattern: None,
                    confidence: l.confidence,
                    created_at: l.created_at,
                    updated_at: String::new(), // Not available in summary
                    source: LearningSourceResponse {
                        source_type: "unknown".to_string(),
                        session_id: None,
                        message_index: None,
                    },
                    embedding_dimensions: None,
                })
                .collect(),
            exported_at: chrono::Utc::now().to_rfc3339(),
        };

        let json = serde_json::to_string_pretty(&export)
            .map_err(|e| PluginError::custom(format!("Failed to serialize export: {}", e)))?;

        Ok(CommandOutput::Text(json))
    }

    // ─── Learn Enable/Disable Commands ────────────────────────────────

    fn cmd_learn_enable(
        &self,
        args: &vibes_plugin_api::CommandArgs,
    ) -> Result<CommandOutput, PluginError> {
        if Self::wants_help(&args.args) {
            return Ok(CommandOutput::Text(
                "Usage: vibes groove learn enable <id>\n\n\
                 Re-enable a deprecated learning.\n\n\
                 Arguments:\n\
                   <id>         Learning ID to enable\n"
                    .to_string(),
            ));
        }

        let id = args
            .args
            .first()
            .ok_or_else(|| PluginError::custom("Missing learning ID"))?;

        let mut output = String::new();
        output.push_str(&format!("Enabling learning: {}\n", id));
        output.push_str("(Not yet implemented - requires server connection)\n");

        Ok(CommandOutput::Text(output))
    }

    fn cmd_learn_disable(
        &self,
        args: &vibes_plugin_api::CommandArgs,
    ) -> Result<CommandOutput, PluginError> {
        if Self::wants_help(&args.args) {
            return Ok(CommandOutput::Text(
                "Usage: vibes groove learn disable <id> [reason]\n\n\
                 Manually deprecate a learning.\n\n\
                 Arguments:\n\
                   <id>         Learning ID to disable\n\
                   [reason]     Optional reason for deprecation\n"
                    .to_string(),
            ));
        }

        let id = args
            .args
            .first()
            .ok_or_else(|| PluginError::custom("Missing learning ID"))?;
        let reason = args.args.get(1).cloned();

        let mut output = String::new();
        output.push_str(&format!("Disabling learning: {}\n", id));
        if let Some(r) = reason {
            output.push_str(&format!("Reason: {}\n", r));
        }
        output.push_str("(Not yet implemented - requires server connection)\n");

        Ok(CommandOutput::Text(output))
    }

    // ─── Attribution Commands ─────────────────────────────────────────

    fn cmd_attr_status(
        &self,
        args: &vibes_plugin_api::CommandArgs,
    ) -> Result<CommandOutput, PluginError> {
        if Self::wants_help(&args.args) {
            return Ok(CommandOutput::Text(
                "Usage: vibes groove attr status\n\n\
                 Show attribution engine status.\n"
                    .to_string(),
            ));
        }

        let mut output = String::new();
        output.push_str("Attribution Engine Status\n");
        output.push_str("========================================\n\n");

        // In CLI mode, we'd fetch from server
        // For now, return placeholder data
        output.push_str("Learnings:\n");
        output.push_str("  Total:        0\n");
        output.push_str("  Active:       0\n");
        output.push_str("  Deprecated:   0\n");
        output.push_str("  Experimental: 0\n\n");

        output.push_str("Activity (24h):\n");
        output.push_str("  Attributions: 0\n");
        output.push_str("  Avg value:    N/A\n\n");

        output.push_str("Consumer: not running\n");

        Ok(CommandOutput::Text(output))
    }

    fn cmd_attr_values(
        &self,
        args: &vibes_plugin_api::CommandArgs,
    ) -> Result<CommandOutput, PluginError> {
        if Self::wants_help(&args.args) {
            return Ok(CommandOutput::Text(
                "Usage: vibes groove attr values [OPTIONS]\n\n\
                 List learning values.\n\n\
                 Options:\n\
                   --sort <SORT>    Sort by: value, confidence, sessions (default: value)\n\
                   --limit <N>      Maximum results (default: 20)\n\
                   --help, -h       Show this help message\n"
                    .to_string(),
            ));
        }

        let _sort = Self::parse_flag(&args.args, "--sort").unwrap_or_else(|| "value".to_string());
        let _limit: usize = Self::parse_flag(&args.args, "--limit")
            .and_then(|s| s.parse().ok())
            .unwrap_or(20);

        let mut output = String::new();
        output.push_str("Learning Values\n");
        output.push_str("========================================\n\n");
        output.push_str("ID                                    Value   Conf    Sessions  Status\n");
        output.push_str("------------------------------------  ------  ------  --------  ------\n");
        output.push_str("\nNo learnings with attribution data.\n");

        Ok(CommandOutput::Text(output))
    }

    fn cmd_attr_show(
        &self,
        args: &vibes_plugin_api::CommandArgs,
    ) -> Result<CommandOutput, PluginError> {
        if Self::wants_help(&args.args) {
            return Ok(CommandOutput::Text(
                "Usage: vibes groove attr show <id>\n\n\
                 Show detailed attribution for a learning.\n\n\
                 Arguments:\n\
                   <id>         Learning ID to show\n"
                    .to_string(),
            ));
        }

        let id = args
            .args
            .first()
            .ok_or_else(|| PluginError::custom("Missing learning ID"))?;

        let mut output = String::new();
        output.push_str(&format!("Attribution Details: {}\n", id));
        output.push_str("========================================\n\n");
        output.push_str("Learning not found or no attribution data available.\n");

        Ok(CommandOutput::Text(output))
    }

    fn cmd_attr_explain(
        &self,
        args: &vibes_plugin_api::CommandArgs,
    ) -> Result<CommandOutput, PluginError> {
        if Self::wants_help(&args.args) {
            return Ok(CommandOutput::Text(
                "Usage: vibes groove attr explain <learning_id> <session_id>\n\n\
                 Explain attribution for a specific session.\n\n\
                 Arguments:\n\
                   <learning_id>  Learning ID\n\
                   <session_id>   Session ID\n"
                    .to_string(),
            ));
        }

        let learning_id = args
            .args
            .first()
            .ok_or_else(|| PluginError::custom("Missing learning ID"))?;
        let session_id = args
            .args
            .get(1)
            .ok_or_else(|| PluginError::custom("Missing session ID"))?;

        let mut output = String::new();
        output.push_str(&format!(
            "Attribution Explanation\n\
             ========================================\n\n\
             Learning: {}\n\
             Session:  {}\n\n\
             No attribution record found for this combination.\n",
            learning_id, session_id
        ));

        Ok(CommandOutput::Text(output))
    }

    // ─── Route Handlers ───────────────────────────────────────────────

    fn route_get_policy(&self) -> Result<RouteResponse, PluginError> {
        let policy = load_policy_or_default("groove-policy.toml");
        RouteResponse::json(200, &PolicyResponse::from(policy))
    }

    fn route_get_trust_levels(&self) -> Result<RouteResponse, PluginError> {
        let levels = vec![
            TrustLevelInfo {
                name: "Local".to_string(),
                score: TrustLevel::Local as u8,
                description: "Locally created content (full trust)".to_string(),
            },
            TrustLevelInfo {
                name: "PrivateCloud".to_string(),
                score: TrustLevel::PrivateCloud as u8,
                description: "Synced from user's own cloud".to_string(),
            },
            TrustLevelInfo {
                name: "OrganizationVerified".to_string(),
                score: TrustLevel::OrganizationVerified as u8,
                description: "Enterprise content, curator approved".to_string(),
            },
            TrustLevelInfo {
                name: "OrganizationUnverified".to_string(),
                score: TrustLevel::OrganizationUnverified as u8,
                description: "Enterprise content, not yet approved".to_string(),
            },
            TrustLevelInfo {
                name: "PublicVerified".to_string(),
                score: TrustLevel::PublicVerified as u8,
                description: "Community content, verified by community".to_string(),
            },
            TrustLevelInfo {
                name: "PublicUnverified".to_string(),
                score: TrustLevel::PublicUnverified as u8,
                description: "Community content, no verification".to_string(),
            },
            TrustLevelInfo {
                name: "Quarantined".to_string(),
                score: TrustLevel::Quarantined as u8,
                description: "Quarantined (blocked from injection)".to_string(),
            },
        ];

        RouteResponse::json(200, &TrustHierarchyResponse { levels })
    }

    fn route_get_role_permissions(
        &self,
        request: &RouteRequest,
    ) -> Result<RouteResponse, PluginError> {
        let role_str = request
            .params
            .get("role")
            .ok_or_else(|| PluginError::InvalidInput("Missing role parameter".into()))?;

        let role: OrgRole = role_str.parse().map_err(|_| {
            // Return as JSON error response
            PluginError::InvalidInput(format!(
                "Invalid role: {}. Use: admin, curator, member, viewer",
                role_str
            ))
        })?;

        let perms = role.permissions();

        RouteResponse::json(
            200,
            &RolePermissionsResponse {
                role: role.as_str().to_string(),
                permissions: PermissionFlags {
                    can_create: perms.can_create,
                    can_read: perms.can_read,
                    can_modify: perms.can_modify,
                    can_delete: perms.can_delete,
                    can_publish: perms.can_publish,
                    can_review: perms.can_review,
                    can_admin: perms.can_admin,
                },
            },
        )
    }

    fn route_list_quarantined(&self) -> Result<RouteResponse, PluginError> {
        // Placeholder - full implementation requires storage integration
        RouteResponse::json(
            200,
            &QuarantineListResponse {
                items: vec![],
                total: 0,
            },
        )
    }

    fn route_get_quarantine_stats(&self) -> Result<RouteResponse, PluginError> {
        // Placeholder - full implementation requires storage integration
        RouteResponse::json(
            200,
            &QuarantineStatsResponse {
                total: 0,
                pending_review: 0,
                approved: 0,
                rejected: 0,
                escalated: 0,
            },
        )
    }

    fn route_review_quarantined(
        &self,
        request: &RouteRequest,
    ) -> Result<RouteResponse, PluginError> {
        let id = request
            .params
            .get("id")
            .ok_or_else(|| PluginError::InvalidInput("Missing id parameter".into()))?;

        // Parse request body
        let review_request: ReviewRequest =
            serde_json::from_slice(&request.body).map_err(|e| PluginError::Json(e.to_string()))?;

        // Parse the outcome
        let outcome = match review_request.outcome.to_lowercase().as_str() {
            "approve" | "approved" => ReviewOutcome::Approved,
            "reject" | "rejected" => ReviewOutcome::Rejected,
            "escalate" | "escalated" => ReviewOutcome::Escalated,
            _ => {
                return RouteResponse::json(
                    400,
                    &ErrorResponse {
                        error: format!(
                            "Invalid outcome: {}. Use: approve, reject, or escalate",
                            review_request.outcome
                        ),
                        code: "INVALID_OUTCOME".to_string(),
                    },
                );
            }
        };

        // Placeholder - full implementation requires storage integration
        let _ = (id, outcome);
        RouteResponse::json(
            404,
            &ErrorResponse {
                error: "Quarantine storage not configured".to_string(),
                code: "NOT_CONFIGURED".to_string(),
            },
        )
    }

    // ─── Assessment Routes ────────────────────────────────────────────────

    fn route_assess_status(&self) -> Result<RouteResponse, PluginError> {
        if let Some(processor) = &self.processor {
            let cb = processor.circuit_breaker_summary();
            let sampling = processor.sampling_summary();
            let sessions = processor.active_sessions();
            let event_count = processor.stored_results_count();

            RouteResponse::json(
                200,
                &AssessmentStatusResponse {
                    circuit_breaker: CircuitBreakerStatus {
                        enabled: cb.enabled,
                        cooldown_seconds: cb.cooldown_seconds,
                        max_interventions_per_session: cb.max_interventions_per_session,
                    },
                    sampling: SamplingStatus {
                        base_rate: sampling.base_rate,
                        burnin_sessions: sampling.burnin_sessions,
                    },
                    activity: ActivityStatus {
                        active_sessions: sessions.len(),
                        events_stored: event_count,
                        sessions,
                        intervention_count: processor.total_intervention_count(),
                    },
                },
            )
        } else {
            RouteResponse::json(
                503,
                &ErrorResponse {
                    error: "Assessment processor not initialized".to_string(),
                    code: "NOT_INITIALIZED".to_string(),
                },
            )
        }
    }

    fn route_assess_history(&self, request: &RouteRequest) -> Result<RouteResponse, PluginError> {
        let session_filter = request.query.get("session").cloned();

        // Parse pagination parameters with defaults
        let page: usize = request
            .query
            .get("page")
            .and_then(|p| p.parse().ok())
            .unwrap_or(1)
            .max(1); // Ensure minimum of 1
        let per_page: usize = request
            .query
            .get("per_page")
            .and_then(|p| p.parse().ok())
            .unwrap_or(20)
            .clamp(1, 100); // Clamp between 1 and 100

        if let Some(processor) = &self.processor {
            let sessions = processor.active_sessions();

            // Filter sessions if a filter is provided
            let filtered_sessions: Vec<_> = if let Some(ref filter) = session_filter {
                sessions.into_iter().filter(|s| s == filter).collect()
            } else {
                sessions
            };

            let total = filtered_sessions.len();
            let total_pages = total.div_ceil(per_page);
            let start = (page - 1) * per_page;
            let end = (start + per_page).min(total);

            // Paginate the sessions
            let paginated_sessions: Vec<_> = if start < total {
                filtered_sessions[start..end].to_vec()
            } else {
                vec![]
            };

            // Get event counts and types for each session in this page
            let items: Vec<SessionHistoryItem> = paginated_sessions
                .into_iter()
                .map(|session_id| {
                    let query = AssessmentQuery::new().with_session(&session_id);
                    let response = processor.query(query);

                    // Collect unique result types
                    let result_types: Vec<_> = response
                        .results
                        .iter()
                        .map(|r| r.result_type.clone())
                        .collect::<std::collections::HashSet<_>>()
                        .into_iter()
                        .collect();

                    SessionHistoryItem {
                        session_id,
                        event_count: response.results.len(),
                        result_types,
                    }
                })
                .collect();

            RouteResponse::json(
                200,
                &AssessmentHistoryResponse {
                    sessions: items,
                    has_more: page < total_pages,
                    page,
                    per_page,
                    total,
                    total_pages,
                },
            )
        } else {
            RouteResponse::json(
                503,
                &ErrorResponse {
                    error: "Assessment processor not initialized".to_string(),
                    code: "NOT_INITIALIZED".to_string(),
                },
            )
        }
    }

    fn route_assess_stats(&self) -> Result<RouteResponse, PluginError> {
        if let Some(processor) = &self.processor {
            // Use pre-computed stats from accumulator (O(1) instead of O(n))
            let tier_counts = processor.global_tier_counts();
            let total = processor.total_assessment_count();
            let top = processor.top_sessions(10);

            // Convert to API response format
            let top_sessions: Vec<SessionStats> = top
                .into_iter()
                .map(|(session_id, count)| SessionStats {
                    session_id,
                    assessment_count: count,
                })
                .collect();

            RouteResponse::json(
                200,
                &AssessmentStatsResponse {
                    tier_distribution: TierDistribution {
                        lightweight: tier_counts.lightweight,
                        medium: tier_counts.medium,
                        heavy: tier_counts.heavy,
                        checkpoint: tier_counts.checkpoint,
                    },
                    total_assessments: total,
                    top_sessions,
                },
            )
        } else {
            RouteResponse::json(
                503,
                &ErrorResponse {
                    error: "Assessment processor not initialized".to_string(),
                    code: "NOT_INITIALIZED".to_string(),
                },
            )
        }
    }

    // ─── Learning Route Handlers ──────────────────────────────────────────────

    fn route_learnings_status(&self) -> Result<RouteResponse, PluginError> {
        // Open store on-demand
        let paths = match GroovePaths::new() {
            Some(p) => p,
            None => {
                return RouteResponse::json(
                    503,
                    &ErrorResponse {
                        error: "Groove not initialized. Run 'vibes groove init' first.".to_string(),
                        code: "NOT_INITIALIZED".to_string(),
                    },
                );
            }
        };

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PluginError::custom(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let store = CozoStore::open(&paths.db_path)
                .await
                .map_err(|e| PluginError::custom(format!("Failed to open store: {}", e)))?;

            // Query counts by scope
            let project_count = store
                .count_by_scope(&Scope::Project("*".to_string()))
                .await
                .unwrap_or(0);
            let user_count = store
                .count_by_scope(&Scope::User("*".to_string()))
                .await
                .unwrap_or(0);
            let global_count = store.count_by_scope(&Scope::Global).await.unwrap_or(0);

            // Query counts by category
            let correction_count = store
                .count_by_category(&LearningCategory::Correction)
                .await
                .unwrap_or(0);
            let error_recovery_count = store
                .count_by_category(&LearningCategory::ErrorRecovery)
                .await
                .unwrap_or(0);
            let pattern_count = store
                .count_by_category(&LearningCategory::CodePattern)
                .await
                .unwrap_or(0);
            let preference_count = store
                .count_by_category(&LearningCategory::Preference)
                .await
                .unwrap_or(0);

            RouteResponse::json(
                200,
                &LearningStatusResponse {
                    counts_by_scope: ScopeCounts {
                        project: project_count,
                        user: user_count,
                        global: global_count,
                    },
                    counts_by_category: CategoryCounts {
                        correction: correction_count,
                        error_recovery: error_recovery_count,
                        pattern: pattern_count,
                        preference: preference_count,
                    },
                    embedder: EmbedderStatus {
                        model: "gte-small".to_string(),
                        dimensions: 384,
                        healthy: true, // TODO: Check actual embedder status
                    },
                    last_extraction: None, // TODO: Track last extraction time
                },
            )
        })
    }

    fn route_learnings_list(&self, request: &RouteRequest) -> Result<RouteResponse, PluginError> {
        let scope_filter = request.query.get("scope").cloned();
        let category_filter = request.query.get("category").cloned();
        let page: usize = request
            .query
            .get("page")
            .and_then(|p| p.parse().ok())
            .unwrap_or(1)
            .max(1);
        let per_page: usize = request
            .query
            .get("per_page")
            .and_then(|p| p.parse().ok())
            .unwrap_or(20)
            .clamp(1, 100);

        let paths = match GroovePaths::new() {
            Some(p) => p,
            None => {
                return RouteResponse::json(
                    503,
                    &ErrorResponse {
                        error: "Groove not initialized".to_string(),
                        code: "NOT_INITIALIZED".to_string(),
                    },
                );
            }
        };

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PluginError::custom(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let store = CozoStore::open(&paths.db_path)
                .await
                .map_err(|e| PluginError::custom(format!("Failed to open store: {}", e)))?;

            // Get all learnings (with optional filters)
            let all_learnings: Vec<Learning> = if let Some(ref scope_str) = scope_filter {
                let scope = match scope_str.as_str() {
                    "project" => Scope::Project("*".to_string()),
                    "user" => Scope::User("*".to_string()),
                    "global" => Scope::Global,
                    _ => Scope::Project("*".to_string()),
                };
                store.find_by_scope(&scope).await.unwrap_or_default()
            } else if let Some(ref cat_str) = category_filter {
                let category = match cat_str.as_str() {
                    "correction" => LearningCategory::Correction,
                    "error_recovery" => LearningCategory::ErrorRecovery,
                    "pattern" => LearningCategory::CodePattern,
                    "preference" => LearningCategory::Preference,
                    _ => LearningCategory::CodePattern,
                };
                store.find_by_category(&category).await.unwrap_or_default()
            } else {
                // Get all learnings by querying each scope
                let mut all = Vec::new();
                if let Ok(learnings) = store.find_by_scope(&Scope::Global).await {
                    all.extend(learnings);
                }
                all
            };

            let total = all_learnings.len() as u64;
            let start = (page - 1) * per_page;
            let end = (start + per_page).min(all_learnings.len());

            let paginated: Vec<LearningSummary> = if start < all_learnings.len() {
                all_learnings[start..end]
                    .iter()
                    .map(|l| LearningSummary {
                        id: l.id.to_string(),
                        category: l.category.as_str().to_string(),
                        confidence: l.confidence,
                        description: l.content.description.clone(),
                        scope: l.scope.to_db_string(),
                        created_at: l.created_at.to_rfc3339(),
                    })
                    .collect()
            } else {
                vec![]
            };

            RouteResponse::json(
                200,
                &LearningListResponse {
                    learnings: paginated,
                    total,
                    page,
                    per_page,
                },
            )
        })
    }

    fn route_learnings_get(&self, request: &RouteRequest) -> Result<RouteResponse, PluginError> {
        let id_str = request
            .params
            .get("id")
            .ok_or_else(|| PluginError::InvalidInput("Missing id parameter".into()))?;

        let id: uuid::Uuid = id_str
            .parse()
            .map_err(|_| PluginError::InvalidInput(format!("Invalid UUID: {}", id_str)))?;

        let paths = match GroovePaths::new() {
            Some(p) => p,
            None => {
                return RouteResponse::json(
                    503,
                    &ErrorResponse {
                        error: "Groove not initialized".to_string(),
                        code: "NOT_INITIALIZED".to_string(),
                    },
                );
            }
        };

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PluginError::custom(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let store = CozoStore::open(&paths.db_path)
                .await
                .map_err(|e| PluginError::custom(format!("Failed to open store: {}", e)))?;

            match store.get(id).await {
                Ok(Some(learning)) => {
                    let (source_type, session_id, message_index): (
                        String,
                        Option<String>,
                        Option<u32>,
                    ) = match &learning.source {
                        crate::LearningSource::Transcript {
                            session_id,
                            message_index,
                        } => (
                            "transcript".to_string(),
                            Some(session_id.clone()),
                            Some(*message_index as u32),
                        ),
                        crate::LearningSource::UserCreated => ("manual".to_string(), None, None),
                        crate::LearningSource::Imported { .. } => {
                            ("import".to_string(), None, None)
                        }
                        crate::LearningSource::Promoted { .. } => {
                            ("promoted".to_string(), None, None)
                        }
                        crate::LearningSource::EnterpriseCurated { .. } => {
                            ("enterprise".to_string(), None, None)
                        }
                    };

                    RouteResponse::json(
                        200,
                        &LearningDetailResponse {
                            id: learning.id.to_string(),
                            scope: learning.scope.to_db_string(),
                            category: learning.category.as_str().to_string(),
                            description: learning.content.description.clone(),
                            insight: learning.content.insight.clone(),
                            pattern: learning.content.pattern.clone(),
                            confidence: learning.confidence,
                            created_at: learning.created_at.to_rfc3339(),
                            updated_at: learning.updated_at.to_rfc3339(),
                            source: LearningSourceResponse {
                                source_type,
                                session_id,
                                message_index,
                            },
                            embedding_dimensions: None, // Embeddings stored separately
                        },
                    )
                }
                Ok(None) => RouteResponse::json(
                    404,
                    &ErrorResponse {
                        error: format!("Learning not found: {}", id),
                        code: "NOT_FOUND".to_string(),
                    },
                ),
                Err(e) => RouteResponse::json(
                    500,
                    &ErrorResponse {
                        error: format!("Database error: {}", e),
                        code: "DB_ERROR".to_string(),
                    },
                ),
            }
        })
    }

    fn route_learnings_delete(&self, request: &RouteRequest) -> Result<RouteResponse, PluginError> {
        let id_str = request
            .params
            .get("id")
            .ok_or_else(|| PluginError::InvalidInput("Missing id parameter".into()))?;

        let id: uuid::Uuid = id_str
            .parse()
            .map_err(|_| PluginError::InvalidInput(format!("Invalid UUID: {}", id_str)))?;

        let paths = match GroovePaths::new() {
            Some(p) => p,
            None => {
                return RouteResponse::json(
                    503,
                    &ErrorResponse {
                        error: "Groove not initialized".to_string(),
                        code: "NOT_INITIALIZED".to_string(),
                    },
                );
            }
        };

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PluginError::custom(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let store = CozoStore::open(&paths.db_path)
                .await
                .map_err(|e| PluginError::custom(format!("Failed to open store: {}", e)))?;

            match store.delete(id).await {
                Ok(true) => RouteResponse::json(
                    200,
                    &DeleteResponse {
                        deleted: true,
                        id: id.to_string(),
                    },
                ),
                Ok(false) => RouteResponse::json(
                    404,
                    &ErrorResponse {
                        error: format!("Learning not found: {}", id),
                        code: "NOT_FOUND".to_string(),
                    },
                ),
                Err(e) => RouteResponse::json(
                    500,
                    &ErrorResponse {
                        error: format!("Database error: {}", e),
                        code: "DB_ERROR".to_string(),
                    },
                ),
            }
        })
    }

    // ─── Learn Enable/Disable Routes ──────────────────────────────────

    fn route_learnings_enable(&self, request: &RouteRequest) -> Result<RouteResponse, PluginError> {
        let id_str = request
            .params
            .get("id")
            .ok_or_else(|| PluginError::InvalidInput("Missing id parameter".into()))?;

        // Validate UUID format
        let _id: uuid::Uuid = id_str
            .parse()
            .map_err(|_| PluginError::InvalidInput(format!("Invalid UUID: {}", id_str)))?;

        // For now, return a placeholder response
        RouteResponse::json(
            200,
            &serde_json::json!({
                "success": true,
                "id": id_str,
                "status": "active",
                "message": "Learning re-enabled (not yet implemented)"
            }),
        )
    }

    fn route_learnings_disable(
        &self,
        request: &RouteRequest,
    ) -> Result<RouteResponse, PluginError> {
        let id_str = request
            .params
            .get("id")
            .ok_or_else(|| PluginError::InvalidInput("Missing id parameter".into()))?;

        // Validate UUID format
        let _id: uuid::Uuid = id_str
            .parse()
            .map_err(|_| PluginError::InvalidInput(format!("Invalid UUID: {}", id_str)))?;

        // For now, return a placeholder response
        RouteResponse::json(
            200,
            &serde_json::json!({
                "success": true,
                "id": id_str,
                "status": "deprecated",
                "message": "Learning disabled (not yet implemented)"
            }),
        )
    }

    // ─── Attribution Routes ───────────────────────────────────────────

    fn route_attr_status(&self) -> Result<RouteResponse, PluginError> {
        let response = AttributionStatusResponse {
            total_learnings: 0,
            active_learnings: 0,
            deprecated_learnings: 0,
            experimental_learnings: 0,
            attributions_24h: 0,
            average_value: 0.0,
            consumer_running: false,
        };

        RouteResponse::json(200, &response)
    }

    fn route_attr_values(&self, request: &RouteRequest) -> Result<RouteResponse, PluginError> {
        let _limit: usize = request
            .query
            .get("limit")
            .and_then(|s| s.parse().ok())
            .unwrap_or(20);

        let _sort = request
            .query
            .get("sort")
            .cloned()
            .unwrap_or_else(|| "value".to_string());

        let response = AttributionValuesResponse {
            values: vec![],
            total: 0,
        };

        RouteResponse::json(200, &response)
    }

    fn route_attr_show(&self, request: &RouteRequest) -> Result<RouteResponse, PluginError> {
        let id_str = request
            .params
            .get("id")
            .ok_or_else(|| PluginError::InvalidInput("Missing id parameter".into()))?;

        // Validate UUID format
        let _id: uuid::Uuid = id_str
            .parse()
            .map_err(|_| PluginError::InvalidInput(format!("Invalid UUID: {}", id_str)))?;

        RouteResponse::json(
            404,
            &ErrorResponse {
                error: format!("Attribution data not found for learning: {}", id_str),
                code: "NOT_FOUND".to_string(),
            },
        )
    }

    fn route_attr_explain(&self, request: &RouteRequest) -> Result<RouteResponse, PluginError> {
        let learning_id = request
            .params
            .get("learning_id")
            .ok_or_else(|| PluginError::InvalidInput("Missing learning_id parameter".into()))?;

        let session_id = request
            .params
            .get("session_id")
            .ok_or_else(|| PluginError::InvalidInput("Missing session_id parameter".into()))?;

        // Validate learning_id UUID format
        let _id: uuid::Uuid = learning_id
            .parse()
            .map_err(|_| PluginError::InvalidInput(format!("Invalid UUID: {}", learning_id)))?;

        RouteResponse::json(
            404,
            &ErrorResponse {
                error: format!(
                    "Attribution record not found for learning {} session {}",
                    learning_id, session_id
                ),
                code: "NOT_FOUND".to_string(),
            },
        )
    }

    // ─── Strategy Routes ─────────────────────────────────────────────────

    fn route_strategy_status(&self) -> Result<RouteResponse, PluginError> {
        let paths = match GroovePaths::new() {
            Some(p) => p,
            None => {
                return RouteResponse::json(
                    503,
                    &ErrorResponse {
                        error: "Groove not initialized. Run 'vibes groove init' first.".to_string(),
                        code: "NOT_INITIALIZED".to_string(),
                    },
                );
            }
        };

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PluginError::custom(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let store = CozoStore::open(&paths.db_path)
                .await
                .map_err(|e| PluginError::custom(format!("Failed to open store: {}", e)))?;

            // Initialize strategy schema if needed (idempotent)
            let _ = CozoStrategyStore::init_schema(&store.db());
            let strategy_store = CozoStrategyStore::new(store.db());

            // Load distributions and overrides
            let distributions = strategy_store
                .load_distributions()
                .await
                .unwrap_or_default();
            let overrides = strategy_store.load_overrides().await.unwrap_or_default();

            // Calculate top strategies across all distributions
            let mut top_strategies: Vec<TopStrategyInfo> = vec![];
            for ((category, _context_type), dist) in &distributions {
                if let Some((variant, weight)) = dist
                    .strategy_weights
                    .iter()
                    .max_by(|a, b| a.1.value.partial_cmp(&b.1.value).unwrap())
                {
                    top_strategies.push(TopStrategyInfo {
                        variant: variant.as_str().to_string(),
                        category: category.as_str().to_string(),
                        weight: weight.value,
                        session_count: dist.session_count,
                    });
                }
            }

            // Sort by weight descending and take top 5
            top_strategies.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
            top_strategies.truncate(5);

            RouteResponse::json(
                200,
                &StrategyStatusResponse {
                    total_distributions: distributions.len() as u64,
                    total_overrides: overrides.len() as u64,
                    events_24h: 0,           // TODO: query events with timestamp filter
                    consumer_running: false, // TODO: check consumer status
                    top_strategies,
                },
            )
        })
    }

    fn route_strategy_distributions(&self) -> Result<RouteResponse, PluginError> {
        let paths = match GroovePaths::new() {
            Some(p) => p,
            None => {
                return RouteResponse::json(
                    503,
                    &ErrorResponse {
                        error: "Groove not initialized. Run 'vibes groove init' first.".to_string(),
                        code: "NOT_INITIALIZED".to_string(),
                    },
                );
            }
        };

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PluginError::custom(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let store = CozoStore::open(&paths.db_path)
                .await
                .map_err(|e| PluginError::custom(format!("Failed to open store: {}", e)))?;

            let _ = CozoStrategyStore::init_schema(&store.db());
            let strategy_store = CozoStrategyStore::new(store.db());

            let distributions = strategy_store
                .load_distributions()
                .await
                .unwrap_or_default();

            let summaries: Vec<DistributionSummary> = distributions
                .iter()
                .map(|((category, context_type), dist)| {
                    let (leading_strategy, leading_weight) = dist
                        .strategy_weights
                        .iter()
                        .max_by(|a, b| a.1.value.partial_cmp(&b.1.value).unwrap())
                        .map(|(v, w)| (v.as_str().to_string(), w.value))
                        .unwrap_or(("unknown".to_string(), 0.0));

                    DistributionSummary {
                        category: category.as_str().to_string(),
                        context_type: context_type.as_str().to_string(),
                        session_count: dist.session_count,
                        leading_strategy,
                        leading_weight,
                    }
                })
                .collect();

            RouteResponse::json(
                200,
                &StrategyDistributionsResponse {
                    distributions: summaries,
                },
            )
        })
    }

    fn route_strategy_show(&self, request: &RouteRequest) -> Result<RouteResponse, PluginError> {
        let category_str = request
            .params
            .get("category")
            .ok_or_else(|| PluginError::InvalidInput("Missing category parameter".into()))?;

        let context_type_str = request
            .params
            .get("context_type")
            .ok_or_else(|| PluginError::InvalidInput("Missing context_type parameter".into()))?;

        let paths = match GroovePaths::new() {
            Some(p) => p,
            None => {
                return RouteResponse::json(
                    503,
                    &ErrorResponse {
                        error: "Groove not initialized.".to_string(),
                        code: "NOT_INITIALIZED".to_string(),
                    },
                );
            }
        };

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PluginError::custom(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let store = CozoStore::open(&paths.db_path)
                .await
                .map_err(|e| PluginError::custom(format!("Failed to open store: {}", e)))?;

            let _ = CozoStrategyStore::init_schema(&store.db());
            let strategy_store = CozoStrategyStore::new(store.db());

            let distributions = strategy_store
                .load_distributions()
                .await
                .unwrap_or_default();
            let overrides = strategy_store.load_overrides().await.unwrap_or_default();

            // Find matching distribution
            let matching = distributions.iter().find(|((cat, ctx), _)| {
                cat.as_str() == category_str && ctx.as_str() == context_type_str
            });

            match matching {
                Some(((_cat, _ctx), dist)) => {
                    let weights: Vec<StrategyWeightInfo> = dist
                        .strategy_weights
                        .iter()
                        .map(|(variant, param)| StrategyWeightInfo {
                            variant: variant.as_str().to_string(),
                            weight: param.value,
                            alpha: param.prior_alpha,
                            beta: param.prior_beta,
                        })
                        .collect();

                    // Find learnings specialized in this category
                    let specialized_learnings: Vec<String> = overrides
                        .iter()
                        .filter(|(_, o)| {
                            o.base_category.as_str() == category_str
                                && o.specialized_weights.is_some()
                        })
                        .map(|(id, _)| id.to_string())
                        .collect();

                    RouteResponse::json(
                        200,
                        &StrategyDistributionDetail {
                            category: category_str.clone(),
                            context_type: context_type_str.clone(),
                            session_count: dist.session_count,
                            weights,
                            specialized_learnings,
                            updated_at: dist.updated_at.to_rfc3339(),
                        },
                    )
                }
                None => RouteResponse::json(
                    404,
                    &ErrorResponse {
                        error: format!(
                            "Distribution not found for category '{}' context '{}'",
                            category_str, context_type_str
                        ),
                        code: "NOT_FOUND".to_string(),
                    },
                ),
            }
        })
    }

    fn route_strategy_learning(
        &self,
        request: &RouteRequest,
    ) -> Result<RouteResponse, PluginError> {
        let id_str = request
            .params
            .get("id")
            .ok_or_else(|| PluginError::InvalidInput("Missing id parameter".into()))?;

        let learning_id: uuid::Uuid = id_str
            .parse()
            .map_err(|_| PluginError::InvalidInput(format!("Invalid UUID: {}", id_str)))?;

        let paths = match GroovePaths::new() {
            Some(p) => p,
            None => {
                return RouteResponse::json(
                    503,
                    &ErrorResponse {
                        error: "Groove not initialized.".to_string(),
                        code: "NOT_INITIALIZED".to_string(),
                    },
                );
            }
        };

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PluginError::custom(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let store = CozoStore::open(&paths.db_path)
                .await
                .map_err(|e| PluginError::custom(format!("Failed to open store: {}", e)))?;

            let _ = CozoStrategyStore::init_schema(&store.db());
            let strategy_store = CozoStrategyStore::new(store.db());

            let overrides = strategy_store.load_overrides().await.unwrap_or_default();
            let distributions = strategy_store
                .load_distributions()
                .await
                .unwrap_or_default();

            match overrides.get(&learning_id) {
                Some(override_) => {
                    let is_specialized = override_.specialized_weights.is_some();

                    // Get effective weights (specialized or from category)
                    let effective_weights: Vec<StrategyWeightInfo> =
                        if let Some(ref specialized) = override_.specialized_weights {
                            specialized
                                .iter()
                                .map(|(v, p)| StrategyWeightInfo {
                                    variant: v.as_str().to_string(),
                                    weight: p.value,
                                    alpha: p.prior_alpha,
                                    beta: p.prior_beta,
                                })
                                .collect()
                        } else {
                            // Fall back to category distribution
                            distributions
                                .iter()
                                .find(|((cat, _), _)| cat == &override_.base_category)
                                .map(|(_, dist)| {
                                    dist.strategy_weights
                                        .iter()
                                        .map(|(v, p)| StrategyWeightInfo {
                                            variant: v.as_str().to_string(),
                                            weight: p.value,
                                            alpha: p.prior_alpha,
                                            beta: p.prior_beta,
                                        })
                                        .collect()
                                })
                                .unwrap_or_default()
                        };

                    // Get category weights for comparison
                    let category_weights: Vec<StrategyWeightInfo> = distributions
                        .iter()
                        .find(|((cat, _), _)| cat == &override_.base_category)
                        .map(|(_, dist)| {
                            dist.strategy_weights
                                .iter()
                                .map(|(v, p)| StrategyWeightInfo {
                                    variant: v.as_str().to_string(),
                                    weight: p.value,
                                    alpha: p.prior_alpha,
                                    beta: p.prior_beta,
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    RouteResponse::json(
                        200,
                        &LearningStrategyResponse {
                            learning_id: learning_id.to_string(),
                            base_category: override_.base_category.as_str().to_string(),
                            is_specialized,
                            session_count: override_.session_count,
                            specialization_threshold: override_.specialization_threshold,
                            effective_weights,
                            category_weights,
                        },
                    )
                }
                None => RouteResponse::json(
                    404,
                    &ErrorResponse {
                        error: format!("Strategy override not found for learning: {}", id_str),
                        code: "NOT_FOUND".to_string(),
                    },
                ),
            }
        })
    }

    fn route_strategy_history(&self, request: &RouteRequest) -> Result<RouteResponse, PluginError> {
        let learning_id_str = request
            .params
            .get("learning_id")
            .ok_or_else(|| PluginError::InvalidInput("Missing learning_id parameter".into()))?;

        let learning_id: uuid::Uuid = learning_id_str
            .parse()
            .map_err(|_| PluginError::InvalidInput(format!("Invalid UUID: {}", learning_id_str)))?;

        let limit: usize = request
            .query
            .get("limit")
            .and_then(|s| s.parse().ok())
            .unwrap_or(20);

        let paths = match GroovePaths::new() {
            Some(p) => p,
            None => {
                return RouteResponse::json(
                    503,
                    &ErrorResponse {
                        error: "Groove not initialized.".to_string(),
                        code: "NOT_INITIALIZED".to_string(),
                    },
                );
            }
        };

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PluginError::custom(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let store = CozoStore::open(&paths.db_path)
                .await
                .map_err(|e| PluginError::custom(format!("Failed to open store: {}", e)))?;

            let _ = CozoStrategyStore::init_schema(&store.db());
            let strategy_store = CozoStrategyStore::new(store.db());

            let events = strategy_store
                .get_strategy_history(learning_id, limit)
                .await
                .unwrap_or_default();

            let event_summaries: Vec<StrategyEventSummary> = events
                .iter()
                .map(|e| StrategyEventSummary {
                    event_id: e.event_id.to_string(),
                    session_id: e.session_id.to_string(),
                    strategy_variant: e.strategy.variant().as_str().to_string(),
                    outcome_value: e.outcome.value,
                    outcome_confidence: e.outcome.confidence,
                    outcome_source: e.outcome.source.as_str().to_string(),
                    timestamp: e.timestamp.to_rfc3339(),
                })
                .collect();

            RouteResponse::json(
                200,
                &StrategyHistoryResponse {
                    events: event_summaries,
                },
            )
        })
    }

    fn route_strategy_reset(&self, request: &RouteRequest) -> Result<RouteResponse, PluginError> {
        let category_str = request
            .params
            .get("category")
            .ok_or_else(|| PluginError::InvalidInput("Missing category parameter".into()))?;

        let context_type_str = request
            .params
            .get("context_type")
            .ok_or_else(|| PluginError::InvalidInput("Missing context_type parameter".into()))?;

        let confirm = request
            .query
            .get("confirm")
            .map(|s| s == "true")
            .unwrap_or(false);

        if !confirm {
            return RouteResponse::json(
                400,
                &ErrorResponse {
                    error: "Reset requires confirmation. Pass ?confirm=true".to_string(),
                    code: "CONFIRMATION_REQUIRED".to_string(),
                },
            );
        }

        let paths = match GroovePaths::new() {
            Some(p) => p,
            None => {
                return RouteResponse::json(
                    503,
                    &ErrorResponse {
                        error: "Groove not initialized.".to_string(),
                        code: "NOT_INITIALIZED".to_string(),
                    },
                );
            }
        };

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PluginError::custom(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let store = CozoStore::open(&paths.db_path)
                .await
                .map_err(|e| PluginError::custom(format!("Failed to open store: {}", e)))?;

            let _ = CozoStrategyStore::init_schema(&store.db());
            let strategy_store = CozoStrategyStore::new(store.db());

            let mut distributions = strategy_store.load_distributions().await.unwrap_or_default();

            // Find and reset the matching distribution
            let key_to_reset = distributions.keys().find(|(cat, ctx)| {
                cat.as_str() == category_str && ctx.as_str() == context_type_str
            }).cloned();

            match key_to_reset {
                Some(key) => {
                    // Create fresh distribution with default priors
                    use crate::strategy::StrategyDistribution;
                    distributions.insert(key.clone(), StrategyDistribution::new(key.0.clone(), key.1));

                    strategy_store.save_distributions(&distributions).await
                        .map_err(|e| PluginError::custom(format!("Failed to save: {}", e)))?;

                    RouteResponse::json(
                        200,
                        &serde_json::json!({
                            "message": format!("Distribution reset for {} / {}", category_str, context_type_str),
                            "category": category_str,
                            "context_type": context_type_str
                        }),
                    )
                }
                None => RouteResponse::json(
                    404,
                    &ErrorResponse {
                        error: format!("Distribution not found for {} / {}", category_str, context_type_str),
                        code: "NOT_FOUND".to_string(),
                    },
                ),
            }
        })
    }

    fn route_strategy_reset_learning(
        &self,
        request: &RouteRequest,
    ) -> Result<RouteResponse, PluginError> {
        let id_str = request
            .params
            .get("id")
            .ok_or_else(|| PluginError::InvalidInput("Missing id parameter".into()))?;

        let learning_id: uuid::Uuid = id_str
            .parse()
            .map_err(|_| PluginError::InvalidInput(format!("Invalid UUID: {}", id_str)))?;

        let confirm = request
            .query
            .get("confirm")
            .map(|s| s == "true")
            .unwrap_or(false);

        if !confirm {
            return RouteResponse::json(
                400,
                &ErrorResponse {
                    error: "Reset requires confirmation. Pass ?confirm=true".to_string(),
                    code: "CONFIRMATION_REQUIRED".to_string(),
                },
            );
        }

        let paths = match GroovePaths::new() {
            Some(p) => p,
            None => {
                return RouteResponse::json(
                    503,
                    &ErrorResponse {
                        error: "Groove not initialized.".to_string(),
                        code: "NOT_INITIALIZED".to_string(),
                    },
                );
            }
        };

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PluginError::custom(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let store = CozoStore::open(&paths.db_path)
                .await
                .map_err(|e| PluginError::custom(format!("Failed to open store: {}", e)))?;

            let _ = CozoStrategyStore::init_schema(&store.db());
            let strategy_store = CozoStrategyStore::new(store.db());

            let mut overrides = strategy_store.load_overrides().await.unwrap_or_default();

            match overrides.get_mut(&learning_id) {
                Some(override_) => {
                    // Clear specialization but keep the override record
                    override_.specialized_weights = None;
                    override_.session_count = 0;

                    strategy_store
                        .save_overrides(&overrides)
                        .await
                        .map_err(|e| PluginError::custom(format!("Failed to save: {}", e)))?;

                    RouteResponse::json(
                        200,
                        &serde_json::json!({
                            "message": format!("Learning specialization cleared for {}", id_str),
                            "learning_id": id_str
                        }),
                    )
                }
                None => RouteResponse::json(
                    404,
                    &ErrorResponse {
                        error: format!("Learning override not found: {}", id_str),
                        code: "NOT_FOUND".to_string(),
                    },
                ),
            }
        })
    }
}

// Export the plugin for dynamic loading
vibes_plugin_api::export_plugin!(GroovePlugin);

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn create_test_context() -> PluginContext {
        PluginContext::new("groove".into(), PathBuf::from("/tmp/groove"))
    }

    #[test]
    fn test_manifest() {
        let plugin = GroovePlugin::default();
        let manifest = plugin.manifest();

        assert_eq!(manifest.name, "groove");
        assert!(!manifest.version.is_empty());
        assert!(manifest.description.contains("Continual learning"));
    }

    #[test]
    fn test_on_load_registers_commands() {
        let mut plugin = GroovePlugin::default();
        let mut ctx = create_test_context();

        plugin.on_load(&mut ctx).unwrap();

        let commands = ctx.pending_commands();
        let paths: Vec<_> = commands.iter().map(|c| c.path.join(" ")).collect();

        // Verify expected commands are registered (not checking count to avoid brittleness)
        let expected_commands = [
            // Groove commands
            "init",
            "list",
            "status",
            // Trust commands
            "trust levels",
            "trust role",
            // Policy commands
            "policy show",
            "policy path",
            // Quarantine commands
            "quarantine list",
            "quarantine stats",
        ];

        for cmd in expected_commands {
            assert!(
                paths.contains(&cmd.to_string()),
                "Expected command '{}' not found. Registered: {:?}",
                cmd,
                paths
            );
        }
    }

    #[test]
    fn test_on_load_registers_routes() {
        let mut plugin = GroovePlugin::default();
        let mut ctx = create_test_context();

        plugin.on_load(&mut ctx).unwrap();

        let routes = ctx.pending_routes();
        let paths: Vec<_> = routes.iter().map(|r| r.path.clone()).collect();

        // Verify expected routes are registered (not checking count to avoid brittleness)
        let expected_routes = [
            "/policy",
            "/trust/levels",
            "/trust/role/:role",
            "/quarantine",
            "/quarantine/stats",
            "/quarantine/:id/review",
            "/assess/status",
            "/assess/history",
            "/assess/stats",
        ];

        for route in expected_routes {
            assert!(
                paths.contains(&route.to_string()),
                "Expected route '{}' not found. Registered: {:?}",
                route,
                paths
            );
        }
    }

    #[test]
    fn test_cmd_trust_levels() {
        let plugin = GroovePlugin::default();
        let result = plugin.cmd_trust_levels().unwrap();

        match result {
            CommandOutput::Text(text) => {
                assert!(text.contains("Trust Level Hierarchy"));
                assert!(text.contains("Local"));
                assert!(text.contains("100"));
                assert!(text.contains("Quarantined"));
                assert!(text.contains("0"));
            }
            _ => panic!("Expected Text output"),
        }
    }

    #[test]
    fn test_cmd_trust_role_admin() {
        let plugin = GroovePlugin::default();
        let mut args = vibes_plugin_api::CommandArgs::default();
        args.args.push("admin".into());

        let result = plugin.cmd_trust_role(&args).unwrap();

        match result {
            CommandOutput::Text(text) => {
                assert!(text.contains("Role: admin"));
                assert!(text.contains("Admin:    Y"));
                assert!(text.contains("Review:   Y"));
            }
            _ => panic!("Expected Text output"),
        }
    }

    #[test]
    fn test_cmd_trust_role_viewer() {
        let plugin = GroovePlugin::default();
        let mut args = vibes_plugin_api::CommandArgs::default();
        args.args.push("viewer".into());

        let result = plugin.cmd_trust_role(&args).unwrap();

        match result {
            CommandOutput::Text(text) => {
                assert!(text.contains("Role: viewer"));
                assert!(text.contains("Read:     Y"));
                assert!(text.contains("Create:   N"));
                assert!(text.contains("Admin:    N"));
            }
            _ => panic!("Expected Text output"),
        }
    }

    #[test]
    fn test_cmd_trust_role_invalid() {
        let plugin = GroovePlugin::default();
        let mut args = vibes_plugin_api::CommandArgs::default();
        args.args.push("invalid".into());

        let result = plugin.cmd_trust_role(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_trust_role_missing_arg() {
        let plugin = GroovePlugin::default();
        let args = vibes_plugin_api::CommandArgs::default();

        let result = plugin.cmd_trust_role(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_policy_show() {
        let plugin = GroovePlugin::default();
        let result = plugin.cmd_policy_show().unwrap();

        match result {
            CommandOutput::Text(text) => {
                assert!(text.contains("Current Security Policy"));
                assert!(text.contains("Injection Policy"));
                assert!(text.contains("Quarantine Policy"));
                assert!(text.contains("Import/Export Policy"));
                assert!(text.contains("Audit Policy"));
            }
            _ => panic!("Expected Text output"),
        }
    }

    #[test]
    fn test_cmd_policy_path() {
        let plugin = GroovePlugin::default();
        let result = plugin.cmd_policy_path().unwrap();

        match result {
            CommandOutput::Text(text) => {
                assert!(text.contains("Policy search paths"));
                assert!(text.contains("groove-policy.toml"));
            }
            _ => panic!("Expected Text output"),
        }
    }

    #[test]
    fn test_route_get_policy() {
        let plugin = GroovePlugin::default();
        let result = plugin.route_get_policy().unwrap();

        assert_eq!(result.status, 200);
        assert_eq!(result.content_type, "application/json");

        let response: PolicyResponse = serde_json::from_slice(&result.body).unwrap();
        // Default policy has block_quarantined = true
        assert!(response.injection.block_quarantined);
    }

    #[test]
    fn test_route_get_trust_levels() {
        let plugin = GroovePlugin::default();
        let result = plugin.route_get_trust_levels().unwrap();

        assert_eq!(result.status, 200);

        let response: TrustHierarchyResponse = serde_json::from_slice(&result.body).unwrap();
        assert_eq!(response.levels.len(), 7);
        assert_eq!(response.levels[0].name, "Local");
        assert_eq!(response.levels[0].score, 100);
        assert_eq!(response.levels[6].name, "Quarantined");
        assert_eq!(response.levels[6].score, 0);
    }

    #[test]
    fn test_route_get_role_permissions_admin() {
        let plugin = GroovePlugin::default();
        let request = RouteRequest {
            params: [("role".into(), "admin".into())].into_iter().collect(),
            query: HashMap::new(),
            body: vec![],
            headers: HashMap::new(),
        };

        let result = plugin.route_get_role_permissions(&request).unwrap();

        assert_eq!(result.status, 200);

        let response: RolePermissionsResponse = serde_json::from_slice(&result.body).unwrap();
        assert_eq!(response.role, "admin");
        assert!(response.permissions.can_admin);
        assert!(response.permissions.can_review);
    }

    #[test]
    fn test_route_get_role_permissions_invalid() {
        let plugin = GroovePlugin::default();
        let request = RouteRequest {
            params: [("role".into(), "invalid".into())].into_iter().collect(),
            query: HashMap::new(),
            body: vec![],
            headers: HashMap::new(),
        };

        let result = plugin.route_get_role_permissions(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_route_list_quarantined() {
        let plugin = GroovePlugin::default();
        let result = plugin.route_list_quarantined().unwrap();

        assert_eq!(result.status, 200);

        let response: QuarantineListResponse = serde_json::from_slice(&result.body).unwrap();
        assert_eq!(response.total, 0);
        assert!(response.items.is_empty());
    }

    #[test]
    fn test_route_get_quarantine_stats() {
        let plugin = GroovePlugin::default();
        let result = plugin.route_get_quarantine_stats().unwrap();

        assert_eq!(result.status, 200);

        let response: QuarantineStatsResponse = serde_json::from_slice(&result.body).unwrap();
        assert_eq!(response.total, 0);
        assert_eq!(response.pending_review, 0);
    }

    #[test]
    fn test_route_review_quarantined_not_configured() {
        let plugin = GroovePlugin::default();
        let body = serde_json::to_vec(&ReviewRequest {
            outcome: "approve".into(),
            notes: None,
        })
        .unwrap();

        let request = RouteRequest {
            params: [("id".into(), "test-id".into())].into_iter().collect(),
            query: HashMap::new(),
            body,
            headers: HashMap::new(),
        };

        let result = plugin.route_review_quarantined(&request).unwrap();

        // Returns 404 because storage is not configured
        assert_eq!(result.status, 404);

        let response: ErrorResponse = serde_json::from_slice(&result.body).unwrap();
        assert_eq!(response.code, "NOT_CONFIGURED");
    }

    #[test]
    fn test_route_review_quarantined_invalid_outcome() {
        let plugin = GroovePlugin::default();
        let body = serde_json::to_vec(&ReviewRequest {
            outcome: "invalid".into(),
            notes: None,
        })
        .unwrap();

        let request = RouteRequest {
            params: [("id".into(), "test-id".into())].into_iter().collect(),
            query: HashMap::new(),
            body,
            headers: HashMap::new(),
        };

        let result = plugin.route_review_quarantined(&request).unwrap();

        assert_eq!(result.status, 400);

        let response: ErrorResponse = serde_json::from_slice(&result.body).unwrap();
        assert_eq!(response.code, "INVALID_OUTCOME");
    }

    #[test]
    fn test_handle_command_dispatch() {
        let mut plugin = GroovePlugin::default();
        let mut ctx = create_test_context();
        let args = vibes_plugin_api::CommandArgs::default();

        // Test valid command
        let result = plugin.handle_command(&["trust", "levels"], &args, &mut ctx);
        assert!(result.is_ok());

        // Test unknown command
        let result = plugin.handle_command(&["unknown"], &args, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_route_dispatch() {
        let mut plugin = GroovePlugin::default();
        let mut ctx = create_test_context();
        let request = RouteRequest {
            params: HashMap::new(),
            query: HashMap::new(),
            body: vec![],
            headers: HashMap::new(),
        };

        // Test valid route
        let result = plugin.handle_route(HttpMethod::Get, "/policy", request, &mut ctx);
        assert!(result.is_ok());

        // Test unknown route
        let request = RouteRequest {
            params: HashMap::new(),
            query: HashMap::new(),
            body: vec![],
            headers: HashMap::new(),
        };
        let result = plugin.handle_route(HttpMethod::Get, "/unknown", request, &mut ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_init_database_creates_db_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let paths = crate::paths::GroovePaths::from_base(temp_dir.path().to_path_buf());

        // Database should not exist before init
        assert!(
            !paths.db_path.exists(),
            "Database should not exist before init"
        );

        // Initialize database
        init_database(&paths).expect("Database initialization should succeed");

        // Database should exist after init
        assert!(paths.db_path.exists(), "Database should exist after init");
    }

    #[test]
    fn test_cli_assess_status() {
        let plugin = GroovePlugin::default();
        let args = vibes_plugin_api::CommandArgs::default();
        let result = plugin.cmd_assess_status(&args).unwrap();

        match result {
            CommandOutput::Text(text) => {
                // Should show assessment system status
                assert!(
                    text.contains("Assessment") || text.contains("Circuit"),
                    "Should contain assessment status info"
                );
            }
            _ => panic!("Expected Text output"),
        }
    }

    #[test]
    fn test_cli_assess_history() {
        let plugin = GroovePlugin::default();
        let mut args = vibes_plugin_api::CommandArgs::default();
        args.args.push("test-session".into());

        let result = plugin.cmd_assess_history(&args).unwrap();

        match result {
            CommandOutput::Text(text) => {
                // Should show assessment history for session
                assert!(
                    text.contains("Assessment")
                        || text.contains("History")
                        || text.contains("session"),
                    "Should contain history info"
                );
            }
            _ => panic!("Expected Text output"),
        }
    }

    #[test]
    fn test_on_load_registers_assess_commands() {
        let mut plugin = GroovePlugin::default();
        let mut ctx = create_test_context();

        plugin.on_load(&mut ctx).unwrap();

        let commands = ctx.pending_commands();
        let paths: Vec<_> = commands.iter().map(|c| c.path.join(" ")).collect();

        // Verify assess commands are registered
        assert!(
            paths.contains(&"assess status".to_string()),
            "Should register 'assess status'. Registered: {:?}",
            paths
        );
        assert!(
            paths.contains(&"assess history".to_string()),
            "Should register 'assess history'. Registered: {:?}",
            paths
        );
    }

    // ─── on_event Tests ──────────────────────────────────────────────

    fn make_raw_event(session_id: &str, text: &str) -> RawEvent {
        let event = vibes_core::VibesEvent::Claude {
            session_id: session_id.to_string(),
            event: vibes_core::ClaudeEvent::TextDelta {
                text: text.to_string(),
            },
        };
        let payload = serde_json::to_string(&event).unwrap();

        RawEvent::new(
            uuid::Uuid::now_v7().into_bytes(),
            chrono::Utc::now().timestamp_millis() as u64,
            Some(session_id.to_string()),
            "Claude".to_string(),
            payload,
        )
    }

    #[test]
    fn test_on_event_returns_results_when_enabled() {
        let mut plugin = GroovePlugin::default();
        let mut ctx = create_test_context();

        // Load plugin (should initialize processor)
        plugin.on_load(&mut ctx).unwrap();

        let raw = make_raw_event("test-session", "Hello, world!");
        let results = plugin.on_event(raw, &mut ctx);

        // Should emit lightweight event result
        assert!(!results.is_empty(), "Should return assessment results");
        assert_eq!(results[0].result_type, "lightweight");
        assert_eq!(results[0].session_id, "test-session");
    }

    #[test]
    fn test_on_event_skips_events_without_session() {
        let mut plugin = GroovePlugin::default();
        let mut ctx = create_test_context();

        plugin.on_load(&mut ctx).unwrap();

        let raw = RawEvent::new(
            [0u8; 16],
            0,
            None, // No session ID
            "Test".to_string(),
            "{}".to_string(),
        );
        let results = plugin.on_event(raw, &mut ctx);

        assert!(results.is_empty(), "Should skip events without session");
    }

    #[test]
    fn test_on_event_produces_valid_json_payloads() {
        let mut plugin = GroovePlugin::default();
        let mut ctx = create_test_context();

        plugin.on_load(&mut ctx).unwrap();

        let raw = make_raw_event("json-test", "Testing JSON serialization");
        let results = plugin.on_event(raw, &mut ctx);

        for result in &results {
            // All payloads should be valid JSON
            let value: serde_json::Value =
                serde_json::from_str(&result.payload).expect("Payload should be valid JSON");
            assert!(value.is_object(), "Payload should be a JSON object");
        }
    }

    #[test]
    fn test_on_event_maintains_session_state() {
        let mut plugin = GroovePlugin::default();
        let mut ctx = create_test_context();

        plugin.on_load(&mut ctx).unwrap();

        // Process multiple events for the same session
        for i in 0..3 {
            let raw = make_raw_event("stateful-session", &format!("Message {i}"));
            let results = plugin.on_event(raw, &mut ctx);
            assert!(!results.is_empty());

            // Parse the lightweight event to check message_idx increments
            let le: crate::assessment::LightweightEvent =
                serde_json::from_str(&results[0].payload).unwrap();
            assert_eq!(le.message_idx, i as u32);
        }
    }

    // ─── CLI Assess Commands with Real Data Tests ────────────────────────

    #[test]
    fn test_cmd_assess_status_shows_real_event_count() {
        let mut plugin = GroovePlugin::default();
        let mut ctx = create_test_context();

        // Initialize plugin (creates processor)
        plugin.on_load(&mut ctx).unwrap();

        // Process events to create some data
        for i in 0..5 {
            let raw = make_raw_event("test-session", &format!("Event {i}"));
            plugin.on_event(raw, &mut ctx);
        }

        // Now check assess status shows actual count (not hardcoded "0")
        let args = vibes_plugin_api::CommandArgs::default();
        let result = plugin.cmd_assess_status(&args).unwrap();
        match result {
            CommandOutput::Text(text) => {
                // Should contain at least "5" somewhere indicating event count
                // Current hardcoded output says "Events today: 0"
                // After fix, it should show actual counts
                assert!(
                    !text.contains("Events today:    0"),
                    "Should show actual event count, not hardcoded 0. Output:\n{}",
                    text
                );
            }
            _ => panic!("Expected Text output"),
        }
    }

    #[test]
    fn test_cmd_assess_status_shows_real_session_count() {
        let mut plugin = GroovePlugin::default();
        let mut ctx = create_test_context();

        plugin.on_load(&mut ctx).unwrap();

        // Process events for multiple sessions
        plugin.on_event(make_raw_event("session-a", "Hello"), &mut ctx);
        plugin.on_event(make_raw_event("session-b", "World"), &mut ctx);

        let args = vibes_plugin_api::CommandArgs::default();
        let result = plugin.cmd_assess_status(&args).unwrap();
        match result {
            CommandOutput::Text(text) => {
                // Should show actual session count (not hardcoded "0")
                assert!(
                    !text.contains("Active sessions: 0"),
                    "Should show actual session count, not hardcoded 0. Output:\n{}",
                    text
                );
            }
            _ => panic!("Expected Text output"),
        }
    }

    #[test]
    fn test_cmd_assess_history_shows_real_sessions() {
        let mut plugin = GroovePlugin::default();
        let mut ctx = create_test_context();

        plugin.on_load(&mut ctx).unwrap();

        // Process events for a session
        for i in 0..3 {
            let raw = make_raw_event("history-session", &format!("Message {i}"));
            plugin.on_event(raw, &mut ctx);
        }

        // Query history without session ID (list all)
        let args = vibes_plugin_api::CommandArgs::default();
        let result = plugin.cmd_assess_history(&args).unwrap();

        match result {
            CommandOutput::Text(text) => {
                // Should NOT say "No session history available"
                // Should list the session we created
                assert!(
                    !text.contains("No session history available"),
                    "Should show actual sessions, not hardcoded 'no history'. Output:\n{}",
                    text
                );
            }
            _ => panic!("Expected Text output"),
        }
    }

    #[test]
    fn test_cmd_assess_history_shows_session_events() {
        let mut plugin = GroovePlugin::default();
        let mut ctx = create_test_context();

        plugin.on_load(&mut ctx).unwrap();

        // Process events for specific session
        for i in 0..3 {
            let raw = make_raw_event("detail-session", &format!("Detail message {i}"));
            plugin.on_event(raw, &mut ctx);
        }

        // Query history for specific session
        let mut args = vibes_plugin_api::CommandArgs::default();
        args.args.push("detail-session".into());
        let result = plugin.cmd_assess_history(&args).unwrap();

        match result {
            CommandOutput::Text(text) => {
                // Should NOT say "No assessments found"
                assert!(
                    !text.contains("No assessments found"),
                    "Should show session events, not hardcoded 'no assessments'. Output:\n{}",
                    text
                );
            }
            _ => panic!("Expected Text output"),
        }
    }

    // ─── HTTP Route Tests ─────────────────────────────────────────────────

    #[test]
    fn test_route_assess_status_with_data() {
        let mut plugin = GroovePlugin::default();
        let mut ctx = create_test_context();

        plugin.on_load(&mut ctx).unwrap();

        // Process some events
        for i in 0..3 {
            let raw = make_raw_event("route-test-session", &format!("Message {i}"));
            plugin.on_event(raw, &mut ctx);
        }

        let result = plugin.route_assess_status().unwrap();
        assert_eq!(result.status, 200);

        let response: AssessmentStatusResponse = serde_json::from_slice(&result.body).unwrap();
        assert!(response.activity.events_stored > 0);
        assert!(response.activity.active_sessions > 0);
        assert!(
            response
                .activity
                .sessions
                .contains(&"route-test-session".to_string())
        );
    }

    #[test]
    fn test_route_assess_history_with_data() {
        let mut plugin = GroovePlugin::default();
        let mut ctx = create_test_context();

        plugin.on_load(&mut ctx).unwrap();

        // Process events for a session
        for i in 0..3 {
            let raw = make_raw_event("history-route-session", &format!("Message {i}"));
            plugin.on_event(raw, &mut ctx);
        }

        let request = RouteRequest {
            params: HashMap::new(),
            query: HashMap::new(),
            body: vec![],
            headers: HashMap::new(),
        };

        let result = plugin.route_assess_history(&request).unwrap();
        assert_eq!(result.status, 200);

        let response: AssessmentHistoryResponse = serde_json::from_slice(&result.body).unwrap();
        assert!(!response.sessions.is_empty());

        let session = response
            .sessions
            .iter()
            .find(|s| s.session_id == "history-route-session")
            .expect("Should find session");
        assert!(session.event_count > 0);
    }

    #[test]
    fn test_route_assess_status_not_initialized() {
        // Plugin without on_load called - processor is None
        let plugin = GroovePlugin::default();

        let result = plugin.route_assess_status().unwrap();
        assert_eq!(result.status, 503);

        let response: ErrorResponse = serde_json::from_slice(&result.body).unwrap();
        assert_eq!(response.code, "NOT_INITIALIZED");
    }

    #[test]
    fn test_route_assess_stats_with_data() {
        let mut plugin = GroovePlugin::default();
        let mut ctx = create_test_context();

        plugin.on_load(&mut ctx).unwrap();

        // Process events for multiple sessions
        for i in 0..5 {
            let raw = make_raw_event("stats-session-1", &format!("Message {i}"));
            plugin.on_event(raw, &mut ctx);
        }
        for i in 0..3 {
            let raw = make_raw_event("stats-session-2", &format!("Message {i}"));
            plugin.on_event(raw, &mut ctx);
        }

        let result = plugin.route_assess_stats().unwrap();
        assert_eq!(result.status, 200);

        let response: AssessmentStatsResponse = serde_json::from_slice(&result.body).unwrap();

        // Should have assessments
        assert!(response.total_assessments > 0);

        // Tier distribution should have values
        let tier_total = response.tier_distribution.lightweight
            + response.tier_distribution.medium
            + response.tier_distribution.heavy
            + response.tier_distribution.checkpoint;
        assert_eq!(tier_total, response.total_assessments);

        // Top sessions should be ordered by count
        assert!(!response.top_sessions.is_empty());
        if response.top_sessions.len() >= 2 {
            assert!(
                response.top_sessions[0].assessment_count
                    >= response.top_sessions[1].assessment_count
            );
        }
    }

    #[test]
    fn test_route_assess_stats_not_initialized() {
        let plugin = GroovePlugin::default();

        let result = plugin.route_assess_stats().unwrap();
        assert_eq!(result.status, 503);

        let response: ErrorResponse = serde_json::from_slice(&result.body).unwrap();
        assert_eq!(response.code, "NOT_INITIALIZED");
    }

    // ─── Server Client Tests (for CLI → Server HTTP calls) ───────────────

    #[test]
    fn test_server_base_url_uses_default_port() {
        let url = GroovePlugin::server_base_url(None);
        assert_eq!(url, "http://127.0.0.1:7432");
    }

    #[test]
    fn test_server_base_url_uses_custom_port() {
        let url = GroovePlugin::server_base_url(Some(8080));
        assert_eq!(url, "http://127.0.0.1:8080");
    }

    #[tokio::test]
    async fn test_fetch_status_from_server_returns_error_when_server_not_running() {
        // Try to connect to a port where no server is running
        let result = GroovePlugin::fetch_status_from_server(Some(19999)).await;
        assert!(result.is_err(), "Should fail when server is not running");
    }

    #[tokio::test]
    async fn test_fetch_history_from_server_returns_error_when_server_not_running() {
        // Try to connect to a port where no server is running
        let result = GroovePlugin::fetch_history_from_server(None, Some(19999)).await;
        assert!(result.is_err(), "Should fail when server is not running");
    }

    #[test]
    fn test_cmd_assess_status_cli_mode_shows_server_hint() {
        // Plugin without processor (CLI mode)
        let plugin = GroovePlugin::default();
        let args = vibes_plugin_api::CommandArgs::default();

        let result = plugin.cmd_assess_status(&args).unwrap();
        match result {
            CommandOutput::Text(text) => {
                // In CLI mode without processor, should indicate server is needed
                assert!(
                    text.contains("Not initialized")
                        || text.contains("server")
                        || text.contains("vibes serve"),
                    "CLI mode should hint about server. Output:\n{}",
                    text
                );
            }
            _ => panic!("Expected Text output"),
        }
    }

    #[test]
    fn test_cmd_assess_history_cli_mode_shows_server_hint() {
        // Plugin without processor (CLI mode)
        let plugin = GroovePlugin::default();
        let args = vibes_plugin_api::CommandArgs::default();

        let result = plugin.cmd_assess_history(&args).unwrap();
        match result {
            CommandOutput::Text(text) => {
                // In CLI mode without processor, should indicate how to get data
                assert!(
                    text.contains("not initialized")
                        || text.contains("server")
                        || text.contains("vibes serve")
                        || text.contains("vibes claude"),
                    "CLI mode should hint about server. Output:\n{}",
                    text
                );
            }
            _ => panic!("Expected Text output"),
        }
    }

    // ─── ServerUrlConfig Tests ──────────────────────────────────────────────

    #[test]
    fn test_server_url_config_default() {
        let config = ServerUrlConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 7432);
        assert_eq!(config.base_url(), "http://127.0.0.1:7432");
    }

    #[test]
    fn test_server_url_config_from_url_simple() {
        let config = ServerUrlConfig::from_url("http://localhost:8080").unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_server_url_config_from_url_with_trailing_slash() {
        let config = ServerUrlConfig::from_url("http://localhost:8080/").unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_server_url_config_from_url_without_port_uses_default() {
        let config = ServerUrlConfig::from_url("http://localhost").unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 7432); // Default port
    }

    #[test]
    fn test_server_url_config_from_url_with_path() {
        let config = ServerUrlConfig::from_url("http://localhost:9000/api").unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 9000);
    }

    #[test]
    fn test_server_url_config_from_url_invalid() {
        let result = ServerUrlConfig::from_url("not-a-valid-url");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid URL"));
    }

    #[test]
    fn test_server_url_config_from_url_ip_address() {
        let config = ServerUrlConfig::from_url("http://192.168.1.100:3000").unwrap();
        assert_eq!(config.host, "192.168.1.100");
        assert_eq!(config.port, 3000);
    }

    // ─── parse_url_flag Tests ───────────────────────────────────────────────

    #[test]
    fn test_parse_url_flag_with_url_space() {
        let args = vec![
            "--url".to_string(),
            "http://localhost:8080".to_string(),
            "other-arg".to_string(),
        ];
        let (config, remaining) = GroovePlugin::parse_url_flag(&args);

        assert!(config.is_some());
        let config = config.unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
        assert_eq!(remaining, vec!["other-arg".to_string()]);
    }

    #[test]
    fn test_parse_url_flag_with_url_equals() {
        let args = vec![
            "--url=http://localhost:9000".to_string(),
            "other-arg".to_string(),
        ];
        let (config, remaining) = GroovePlugin::parse_url_flag(&args);

        assert!(config.is_some());
        let config = config.unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 9000);
        assert_eq!(remaining, vec!["other-arg".to_string()]);
    }

    #[test]
    fn test_parse_url_flag_no_url() {
        let args = vec!["session-id".to_string(), "--other".to_string()];
        let (config, remaining) = GroovePlugin::parse_url_flag(&args);

        assert!(config.is_none());
        assert_eq!(remaining, args);
    }

    #[test]
    fn test_parse_url_flag_empty_args() {
        let args: Vec<String> = vec![];
        let (config, remaining) = GroovePlugin::parse_url_flag(&args);

        assert!(config.is_none());
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_parse_url_flag_url_only() {
        let args = vec!["--url".to_string(), "http://localhost:8080".to_string()];
        let (config, remaining) = GroovePlugin::parse_url_flag(&args);

        assert!(config.is_some());
        assert!(remaining.is_empty());
    }

    // ─── wants_help Tests ───────────────────────────────────────────────────

    #[test]
    fn test_wants_help_with_double_dash() {
        let args = vec!["--help".to_string()];
        assert!(GroovePlugin::wants_help(&args));
    }

    #[test]
    fn test_wants_help_with_short_flag() {
        let args = vec!["-h".to_string()];
        assert!(GroovePlugin::wants_help(&args));
    }

    #[test]
    fn test_wants_help_mixed_with_other_args() {
        let args = vec!["session-id".to_string(), "--help".to_string()];
        assert!(GroovePlugin::wants_help(&args));
    }

    #[test]
    fn test_wants_help_no_help_flag() {
        let args = vec!["session-id".to_string(), "--url".to_string()];
        assert!(!GroovePlugin::wants_help(&args));
    }

    #[test]
    fn test_wants_help_empty_args() {
        let args: Vec<String> = vec![];
        assert!(!GroovePlugin::wants_help(&args));
    }

    // ─── --help in assess commands Tests ────────────────────────────────────

    #[test]
    fn test_cmd_assess_status_help() {
        let plugin = GroovePlugin::default();
        let mut args = vibes_plugin_api::CommandArgs::default();
        args.args.push("--help".to_string());

        let result = plugin.cmd_assess_status(&args).unwrap();
        match result {
            CommandOutput::Text(text) => {
                assert!(text.contains("Usage:"), "Help should show usage");
                assert!(text.contains("--url"), "Help should mention --url option");
                assert!(text.contains("--help"), "Help should mention --help option");
            }
            _ => panic!("Expected Text output"),
        }
    }

    #[test]
    fn test_cmd_assess_status_help_short() {
        let plugin = GroovePlugin::default();
        let mut args = vibes_plugin_api::CommandArgs::default();
        args.args.push("-h".to_string());

        let result = plugin.cmd_assess_status(&args).unwrap();
        match result {
            CommandOutput::Text(text) => {
                assert!(text.contains("Usage:"), "Help should show usage");
            }
            _ => panic!("Expected Text output"),
        }
    }

    #[test]
    fn test_cmd_assess_history_help() {
        let plugin = GroovePlugin::default();
        let mut args = vibes_plugin_api::CommandArgs::default();
        args.args.push("--help".to_string());

        let result = plugin.cmd_assess_history(&args).unwrap();
        match result {
            CommandOutput::Text(text) => {
                assert!(text.contains("Usage:"), "Help should show usage");
                assert!(text.contains("--url"), "Help should mention --url option");
                assert!(
                    text.contains("SESSION_ID"),
                    "Help should mention session ID arg"
                );
            }
            _ => panic!("Expected Text output"),
        }
    }

    #[test]
    fn test_cmd_assess_history_help_short() {
        let plugin = GroovePlugin::default();
        let mut args = vibes_plugin_api::CommandArgs::default();
        args.args.push("-h".to_string());

        let result = plugin.cmd_assess_history(&args).unwrap();
        match result {
            CommandOutput::Text(text) => {
                assert!(text.contains("Usage:"), "Help should show usage");
            }
            _ => panic!("Expected Text output"),
        }
    }

    #[test]
    fn test_cmd_assess_history_help_shows_pagination_flags() {
        let plugin = GroovePlugin::default();
        let mut args = vibes_plugin_api::CommandArgs::default();
        args.args.push("--help".to_string());

        let result = plugin.cmd_assess_history(&args).unwrap();
        match result {
            CommandOutput::Text(text) => {
                assert!(text.contains("--page"), "Help should mention --page option");
                assert!(
                    text.contains("--per-page"),
                    "Help should mention --per-page option"
                );
            }
            _ => panic!("Expected Text output"),
        }
    }

    #[test]
    fn test_history_url_includes_pagination_params() {
        let config = ServerUrlConfig::from_url("http://localhost:7432").unwrap();

        // No pagination params by default
        let url = config.history_url_with_pagination(None, None, None);
        assert_eq!(url, "http://localhost:7432/api/groove/assess/history");

        // With page only
        let url = config.history_url_with_pagination(None, Some(2), None);
        assert!(url.contains("page=2"), "URL should contain page param");

        // With both pagination params
        let url = config.history_url_with_pagination(None, Some(3), Some(50));
        assert!(url.contains("page=3"), "URL should contain page param");
        assert!(
            url.contains("per_page=50"),
            "URL should contain per_page param"
        );

        // With session and pagination
        let url = config.history_url_with_pagination(Some("sess-123"), Some(1), Some(20));
        assert!(
            url.contains("session=sess-123"),
            "URL should contain session param"
        );
        assert!(url.contains("page=1"), "URL should contain page param");
        assert!(
            url.contains("per_page=20"),
            "URL should contain per_page param"
        );
    }

    // ─── API Endpoint URL Tests ────────────────────────────────────────────────

    #[test]
    fn test_api_assess_status_path_includes_groove_prefix() {
        // This test ensures the API path includes the /api/groove/ prefix
        // which is required by the server's route registration
        assert!(
            API_ASSESS_STATUS_PATH.starts_with("/api/groove/"),
            "Status path must include /api/groove/ prefix, got: {}",
            API_ASSESS_STATUS_PATH
        );
        assert_eq!(API_ASSESS_STATUS_PATH, "/api/groove/assess/status");
    }

    #[test]
    fn test_api_assess_history_path_includes_groove_prefix() {
        // This test ensures the API path includes the /api/groove/ prefix
        assert!(
            API_ASSESS_HISTORY_PATH.starts_with("/api/groove/"),
            "History path must include /api/groove/ prefix, got: {}",
            API_ASSESS_HISTORY_PATH
        );
        assert_eq!(API_ASSESS_HISTORY_PATH, "/api/groove/assess/history");
    }

    #[test]
    fn test_server_url_config_status_url() {
        let config = ServerUrlConfig {
            host: "localhost".to_string(),
            port: 8080,
        };
        assert_eq!(
            config.status_url(),
            "http://localhost:8080/api/groove/assess/status"
        );
    }

    #[test]
    fn test_server_url_config_history_url_without_session() {
        let config = ServerUrlConfig {
            host: "localhost".to_string(),
            port: 8080,
        };
        assert_eq!(
            config.history_url(None),
            "http://localhost:8080/api/groove/assess/history"
        );
    }

    #[test]
    fn test_server_url_config_history_url_with_session() {
        let config = ServerUrlConfig {
            host: "localhost".to_string(),
            port: 8080,
        };
        assert_eq!(
            config.history_url(Some("test-session-123")),
            "http://localhost:8080/api/groove/assess/history?session=test-session-123"
        );
    }

    // ─── CLI Mode Behavior Tests ───────────────────────────────────────────────

    #[test]
    fn test_plugin_default_has_no_processor() {
        // In CLI mode, the plugin should NOT have a processor initialized
        // This ensures CLI commands will query the server instead of local state
        let plugin = GroovePlugin::default();
        assert!(
            plugin.processor.is_none(),
            "Default plugin should have no processor (CLI mode)"
        );
    }

    #[test]
    fn test_cli_mode_status_shows_querying_server_message() {
        // When processor is None (CLI mode), the status command should
        // indicate it's querying the server
        let plugin = GroovePlugin::default();
        let args = vibes_plugin_api::CommandArgs::default();

        let result = plugin.cmd_assess_status(&args).unwrap();
        match result {
            CommandOutput::Text(text) => {
                assert!(
                    text.contains("Querying server at"),
                    "CLI mode should show 'Querying server at' message. Got:\n{}",
                    text
                );
            }
            _ => panic!("Expected Text output"),
        }
    }

    #[test]
    fn test_cli_mode_history_shows_querying_server_message() {
        // When processor is None (CLI mode), the history command should
        // indicate it's querying the server
        let plugin = GroovePlugin::default();
        let args = vibes_plugin_api::CommandArgs::default();

        let result = plugin.cmd_assess_history(&args).unwrap();
        match result {
            CommandOutput::Text(text) => {
                assert!(
                    text.contains("Querying server at"),
                    "CLI mode should show 'Querying server at' message. Got:\n{}",
                    text
                );
            }
            _ => panic!("Expected Text output"),
        }
    }

    // ─── Attribution CLI Tests ─────────────────────────────────────────────

    #[test]
    fn test_attribution_status_response_serialization() {
        // Test that AttributionStatusResponse can be serialized/deserialized
        let response = AttributionStatusResponse {
            total_learnings: 10,
            active_learnings: 8,
            deprecated_learnings: 2,
            experimental_learnings: 0,
            attributions_24h: 150,
            average_value: 0.65,
            consumer_running: true,
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: AttributionStatusResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.total_learnings, 10);
        assert_eq!(parsed.active_learnings, 8);
        assert_eq!(parsed.attributions_24h, 150);
    }

    #[test]
    fn test_attribution_values_response_serialization() {
        // Test that AttributionValuesResponse can be serialized/deserialized
        let response = AttributionValuesResponse {
            values: vec![LearningValueSummary {
                learning_id: "test-id".into(),
                estimated_value: 0.75,
                confidence: 0.9,
                session_count: 15,
                status: "active".into(),
            }],
            total: 1,
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: AttributionValuesResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.values.len(), 1);
        assert_eq!(parsed.values[0].estimated_value, 0.75);
    }

    #[test]
    fn test_on_load_registers_attr_commands() {
        let mut plugin = GroovePlugin::default();
        let mut ctx = create_test_context();

        plugin.on_load(&mut ctx).unwrap();

        let commands = ctx.pending_commands();
        let paths: Vec<_> = commands.iter().map(|c| c.path.join(" ")).collect();

        // Verify attr commands are registered
        assert!(
            paths.contains(&"attr status".to_string()),
            "Should register 'attr status'. Registered: {:?}",
            paths
        );
        assert!(
            paths.contains(&"attr values".to_string()),
            "Should register 'attr values'. Registered: {:?}",
            paths
        );
        assert!(
            paths.contains(&"attr show".to_string()),
            "Should register 'attr show'. Registered: {:?}",
            paths
        );
        assert!(
            paths.contains(&"attr explain".to_string()),
            "Should register 'attr explain'. Registered: {:?}",
            paths
        );
    }

    #[test]
    fn test_on_load_registers_attr_routes() {
        let mut plugin = GroovePlugin::default();
        let mut ctx = create_test_context();

        plugin.on_load(&mut ctx).unwrap();

        let routes = ctx.pending_routes();
        let paths: Vec<_> = routes.iter().map(|r| r.path.clone()).collect();

        // Verify attr routes are registered
        assert!(
            paths.contains(&"/attr/status".to_string()),
            "Should register '/attr/status'. Registered: {:?}",
            paths
        );
        assert!(
            paths.contains(&"/attr/values".to_string()),
            "Should register '/attr/values'. Registered: {:?}",
            paths
        );
        assert!(
            paths.contains(&"/attr/show/:id".to_string()),
            "Should register '/attr/show/:id'. Registered: {:?}",
            paths
        );
    }

    #[test]
    fn test_on_load_registers_learn_enable_disable_commands() {
        let mut plugin = GroovePlugin::default();
        let mut ctx = create_test_context();

        plugin.on_load(&mut ctx).unwrap();

        let commands = ctx.pending_commands();
        let paths: Vec<_> = commands.iter().map(|c| c.path.join(" ")).collect();

        // Verify learn enable/disable commands are registered
        assert!(
            paths.contains(&"learn enable".to_string()),
            "Should register 'learn enable'. Registered: {:?}",
            paths
        );
        assert!(
            paths.contains(&"learn disable".to_string()),
            "Should register 'learn disable'. Registered: {:?}",
            paths
        );
    }

    #[test]
    fn test_handle_command_attr_status() {
        let mut plugin = GroovePlugin::default();
        let mut ctx = create_test_context();
        let args = vibes_plugin_api::CommandArgs::default();

        // Test attr status command dispatch
        let result = plugin.handle_command(&["attr", "status"], &args, &mut ctx);
        assert!(result.is_ok(), "attr status command should succeed");
    }

    #[test]
    fn test_route_attr_status() {
        let plugin = GroovePlugin::default();
        let result = plugin.route_attr_status().unwrap();

        assert_eq!(result.status, 200);
        assert_eq!(result.content_type, "application/json");

        let response: AttributionStatusResponse = serde_json::from_slice(&result.body).unwrap();
        // Default plugin has no attribution data
        assert_eq!(response.total_learnings, 0);
    }

    #[test]
    fn test_route_attr_values() {
        let plugin = GroovePlugin::default();
        let request = RouteRequest {
            params: HashMap::new(),
            query: [("limit".into(), "10".into())].into_iter().collect(),
            body: vec![],
            headers: HashMap::new(),
        };

        let result = plugin.route_attr_values(&request).unwrap();

        assert_eq!(result.status, 200);

        let response: AttributionValuesResponse = serde_json::from_slice(&result.body).unwrap();
        assert_eq!(response.total, 0);
        assert!(response.values.is_empty());
    }
}
