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

use crate::assessment::{AssessmentConfig, SyncAssessmentProcessor};

use crate::CozoStore;
use crate::paths::GroovePaths;
use crate::security::load_policy_or_default;
use crate::security::{OrgRole, Policy, ReviewOutcome, TrustLevel};

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

/// Assessment status response for HTTP API
#[derive(Debug, Serialize, Deserialize)]
pub struct AssessmentStatusResponse {
    pub circuit_breaker: CircuitBreakerStatus,
    pub sampling: SamplingStatus,
    pub activity: ActivityStatus,
}

/// Circuit breaker status for API
#[derive(Debug, Serialize, Deserialize)]
pub struct CircuitBreakerStatus {
    pub enabled: bool,
    pub cooldown_seconds: u32,
    pub max_interventions_per_session: u32,
}

/// Sampling status for API
#[derive(Debug, Serialize, Deserialize)]
pub struct SamplingStatus {
    pub base_rate: f64,
    pub burnin_sessions: u32,
}

/// Activity status for API
#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityStatus {
    pub active_sessions: usize,
    pub events_stored: usize,
    pub sessions: Vec<String>,
}

/// Assessment history response for HTTP API
#[derive(Debug, Serialize, Deserialize)]
pub struct AssessmentHistoryResponse {
    pub sessions: Vec<SessionHistoryItem>,
    pub has_more: bool,
}

/// Single session's history
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionHistoryItem {
    pub session_id: String,
    pub event_count: usize,
    pub result_types: Vec<String>,
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
        let base = format!("{}{}", self.base_url(), API_ASSESS_HISTORY_PATH);
        match session_id {
            Some(id) => format!("{}?session={}", base, id),
            None => base,
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
        let url = config.history_url(session_id);

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
    ) -> Result<AssessmentHistoryResponse, String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;
        let config = config.clone();
        rt.block_on(Self::fetch_history_with_config(session_id, &config))
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
            output.push_str("Recent Activity:\n");
            output.push_str(&format!("  Active sessions: {}\n", sessions.len()));
            output.push_str(&format!("  Events stored:   {}\n\n", event_count));
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
                        "  Events stored:   {}\n\n",
                        status.activity.events_stored
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
                   --url <URL>  Server URL (default: from .vibes/config.toml or http://127.0.0.1:7432)\n\
                   --help, -h   Show this help message\n"
                    .to_string(),
            ));
        }

        // Parse --url flag
        let (url_config, remaining) = Self::parse_url_flag(&args.args);

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

            match Self::fetch_history_blocking(session_id, &config) {
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

        if let Some(processor) = &self.processor {
            let sessions = processor.active_sessions();

            // Filter sessions if a filter is provided
            let filtered_sessions: Vec<_> = if let Some(ref filter) = session_filter {
                sessions.into_iter().filter(|s| s == filter).collect()
            } else {
                sessions
            };

            // Get event counts and types for each session
            let items: Vec<SessionHistoryItem> = filtered_sessions
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
                    has_more: false,
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
}
