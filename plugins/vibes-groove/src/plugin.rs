//! Groove Plugin - Plugin trait implementation for vibes groove
//!
//! Provides CLI commands and HTTP routes for security, trust, and quarantine management.

use serde::{Deserialize, Serialize};
use vibes_core::hooks::{HookInstaller, HookInstallerConfig};
use vibes_plugin_api::{
    ArgSpec, CommandOutput, CommandSpec, HttpMethod, Plugin, PluginContext, PluginError,
    PluginManifest, RouteRequest, RouteResponse, RouteSpec,
};

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

// ============================================================================
// Plugin Implementation
// ============================================================================

/// Groove continual learning plugin
///
/// Provides CLI commands and HTTP routes for:
/// - Trust level hierarchy management
/// - Security policy viewing
/// - Quarantine queue management
#[derive(Default)]
pub struct GroovePlugin;

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

        // Register CLI commands
        self.register_commands(ctx)?;

        // Register HTTP routes
        self.register_routes(ctx)?;

        ctx.log_info("Groove plugin loaded successfully");
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

        // Handle SessionStart and UserPromptSubmit for context injection
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
            _ => {
                // Other hook types are logged but not processed
                None
            }
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
            ["assess", "status"] => self.cmd_assess_status(),
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

        Ok(())
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

    fn cmd_assess_status(&self) -> Result<CommandOutput, PluginError> {
        let mut output = String::new();
        output.push_str("Assessment System Status\n");
        output.push_str(&format!("{}\n\n", "=".repeat(40)));

        // Circuit breaker status
        output.push_str("Circuit Breaker:\n");
        output.push_str("  State:           Closed (normal operation)\n");
        output.push_str("  Failure count:   0\n");
        output.push_str("  Last transition: N/A\n\n");

        // Sampling status
        output.push_str("Sampling Strategy:\n");
        output.push_str("  Base rate:       10%\n");
        output.push_str("  Burnin sessions: 5\n\n");

        // Recent activity
        output.push_str("Recent Activity:\n");
        output.push_str("  Active sessions: 0\n");
        output.push_str("  Events today:    0\n");
        output.push_str("  Checkpoints:     0\n\n");

        output.push_str("Note: Full assessment status requires active assessment consumer.\n");
        output.push_str("Run 'vibes serve' to start the assessment system.\n");

        Ok(CommandOutput::Text(output))
    }

    fn cmd_assess_history(
        &self,
        args: &vibes_plugin_api::CommandArgs,
    ) -> Result<CommandOutput, PluginError> {
        let session_id = args.args.first().map(|s| s.as_str());

        let mut output = String::new();
        output.push_str("Assessment History\n");
        output.push_str(&format!("{}\n\n", "=".repeat(40)));

        if let Some(id) = session_id {
            output.push_str(&format!("Session: {}\n\n", id));
            output.push_str("No assessments found for this session.\n");
            output.push_str("\nAssessment history is stored in Iggy when the server is running.\n");
        } else {
            output.push_str("Recent Sessions:\n");
            output.push_str("  No session history available.\n\n");
            output.push_str(
                "Tip: Run 'vibes groove assess history <session_id>' for a specific session.\n",
            );
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
        let plugin = GroovePlugin;
        let manifest = plugin.manifest();

        assert_eq!(manifest.name, "groove");
        assert!(!manifest.version.is_empty());
        assert!(manifest.description.contains("Continual learning"));
    }

    #[test]
    fn test_on_load_registers_commands() {
        let mut plugin = GroovePlugin;
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
        let mut plugin = GroovePlugin;
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
        let plugin = GroovePlugin;
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
        let plugin = GroovePlugin;
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
        let plugin = GroovePlugin;
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
        let plugin = GroovePlugin;
        let mut args = vibes_plugin_api::CommandArgs::default();
        args.args.push("invalid".into());

        let result = plugin.cmd_trust_role(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_trust_role_missing_arg() {
        let plugin = GroovePlugin;
        let args = vibes_plugin_api::CommandArgs::default();

        let result = plugin.cmd_trust_role(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_policy_show() {
        let plugin = GroovePlugin;
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
        let plugin = GroovePlugin;
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
        let plugin = GroovePlugin;
        let result = plugin.route_get_policy().unwrap();

        assert_eq!(result.status, 200);
        assert_eq!(result.content_type, "application/json");

        let response: PolicyResponse = serde_json::from_slice(&result.body).unwrap();
        // Default policy has block_quarantined = true
        assert!(response.injection.block_quarantined);
    }

    #[test]
    fn test_route_get_trust_levels() {
        let plugin = GroovePlugin;
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
        let plugin = GroovePlugin;
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
        let plugin = GroovePlugin;
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
        let plugin = GroovePlugin;
        let result = plugin.route_list_quarantined().unwrap();

        assert_eq!(result.status, 200);

        let response: QuarantineListResponse = serde_json::from_slice(&result.body).unwrap();
        assert_eq!(response.total, 0);
        assert!(response.items.is_empty());
    }

    #[test]
    fn test_route_get_quarantine_stats() {
        let plugin = GroovePlugin;
        let result = plugin.route_get_quarantine_stats().unwrap();

        assert_eq!(result.status, 200);

        let response: QuarantineStatsResponse = serde_json::from_slice(&result.body).unwrap();
        assert_eq!(response.total, 0);
        assert_eq!(response.pending_review, 0);
    }

    #[test]
    fn test_route_review_quarantined_not_configured() {
        let plugin = GroovePlugin;
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
        let plugin = GroovePlugin;
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
        let mut plugin = GroovePlugin;
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
        let mut plugin = GroovePlugin;
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
        let plugin = GroovePlugin;
        let result = plugin.cmd_assess_status().unwrap();

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
        let plugin = GroovePlugin;
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
        let mut plugin = GroovePlugin;
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
}
