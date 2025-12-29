//! Groove continual learning commands
//!
//! Security and trust management commands for vibes groove.

use anyhow::Result;
use clap::{Args, Subcommand};

use vibes_groove::security::{
    OrgRole,
    load_policy_or_default,
};

#[derive(Args)]
pub struct GrooveArgs {
    #[command(subcommand)]
    pub command: GrooveCommands,
}

#[derive(Subcommand)]
pub enum GrooveCommands {
    /// View trust hierarchy information
    Trust(TrustArgs),
    /// View and manage security policy
    Policy(PolicyArgs),
    /// View quarantine queue (placeholder)
    Quarantine(QuarantineArgs),
}

#[derive(Args)]
pub struct TrustArgs {
    #[command(subcommand)]
    pub command: TrustCommands,
}

#[derive(Subcommand)]
pub enum TrustCommands {
    /// Show trust level hierarchy
    Levels,
    /// Show trust level for a specific role
    Role {
        /// Role name (admin, curator, member, viewer)
        role: String,
    },
}

#[derive(Args)]
pub struct PolicyArgs {
    #[command(subcommand)]
    pub command: PolicyCommands,
}

#[derive(Subcommand)]
pub enum PolicyCommands {
    /// Show current policy
    Show,
    /// Show policy file path
    Path,
}

#[derive(Args)]
pub struct QuarantineArgs {
    #[command(subcommand)]
    pub command: QuarantineCommands,
}

#[derive(Subcommand)]
pub enum QuarantineCommands {
    /// List quarantined learnings (placeholder)
    List,
    /// Show quarantine statistics (placeholder)
    Stats,
}

pub fn run(args: GrooveArgs) -> Result<()> {
    match args.command {
        GrooveCommands::Trust(trust_args) => run_trust(trust_args),
        GrooveCommands::Policy(policy_args) => run_policy(policy_args),
        GrooveCommands::Quarantine(quarantine_args) => run_quarantine(quarantine_args),
    }
}

fn run_trust(args: TrustArgs) -> Result<()> {
    match args.command {
        TrustCommands::Levels => {
            println!("Trust Level Hierarchy (highest to lowest):");
            println!();
            println!("  {:20} {:>6}  {}", "Level", "Score", "Description");
            println!("  {:20} {:>6}  {}", "─".repeat(20), "─".repeat(6), "─".repeat(40));
            println!("  {:20} {:>6}  {}", "Local", "100", "Locally created content (full trust)");
            println!("  {:20} {:>6}  {}", "OrganizationVerified", "80", "Verified by organization admin");
            println!("  {:20} {:>6}  {}", "OrganizationMember", "60", "From organization member");
            println!("  {:20} {:>6}  {}", "CommunityVerified", "40", "Community-verified public content");
            println!("  {:20} {:>6}  {}", "PublicUnverified", "10", "Unverified public content");
            println!("  {:20} {:>6}  {}", "Quarantined", "0", "Quarantined (blocked)");
            println!();
            println!("Injection Policy:");
            println!("  - Local & OrganizationVerified: Allowed without scanning");
            println!("  - OrganizationMember & CommunityVerified: Requires scanning");
            println!("  - PublicUnverified: Requires scanning, may show warnings");
            println!("  - Quarantined: Blocked from injection");
            Ok(())
        }
        TrustCommands::Role { role } => {
            let parsed_role: OrgRole = role
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid role: {}. Use: admin, curator, member, viewer", role))?;

            let perms = parsed_role.permissions();
            println!("Role: {}", parsed_role.as_str());
            println!();
            println!("Permissions:");
            println!("  Create:   {}", if perms.can_create { "✓" } else { "✗" });
            println!("  Read:     {}", if perms.can_read { "✓" } else { "✗" });
            println!("  Modify:   {}", if perms.can_modify { "✓" } else { "✗" });
            println!("  Delete:   {}", if perms.can_delete { "✓" } else { "✗" });
            println!("  Publish:  {}", if perms.can_publish { "✓" } else { "✗" });
            println!("  Review:   {}", if perms.can_review { "✓" } else { "✗" });
            println!("  Admin:    {}", if perms.can_admin { "✓" } else { "✗" });
            Ok(())
        }
    }
}

fn run_policy(args: PolicyArgs) -> Result<()> {
    match args.command {
        PolicyCommands::Show => {
            // Look for policy in standard locations
            let policy = load_policy_or_default("groove-policy.toml");

            println!("Current Security Policy:");
            println!();

            // Injection policy
            println!("Injection Policy:");
            println!("  Block quarantined:       {}", policy.injection.block_quarantined);
            println!("  Allow personal:          {}", policy.injection.allow_personal_injection);
            println!("  Allow unverified:        {}", policy.injection.allow_unverified_injection);
            println!();

            // Quarantine policy
            println!("Quarantine Policy:");
            println!("  Reviewers:               {:?}", policy.quarantine.reviewers);
            println!("  Visible to:              {:?}", policy.quarantine.visible_to);
            println!("  Auto-delete after days:  {:?}", policy.quarantine.auto_delete_after_days);
            println!();

            // Import/Export policy
            println!("Import/Export Policy:");
            println!("  Allow import from file:  {}", policy.import_export.allow_import_from_file);
            println!("  Allow import from URL:   {}", policy.import_export.allow_import_from_url);
            println!("  Allowed import sources:  {:?}", policy.import_export.allowed_import_sources);
            println!("  Allow export personal:   {}", policy.import_export.allow_export_personal);
            println!("  Allow export enterprise: {}", policy.import_export.allow_export_enterprise);
            println!();

            // Audit policy
            println!("Audit Policy:");
            println!("  Enabled:                 {}", policy.audit.enabled);
            println!("  Retention days:          {:?}", policy.audit.retention_days);

            Ok(())
        }
        PolicyCommands::Path => {
            println!("Policy search paths:");
            println!("  1. ./groove-policy.toml");
            println!("  2. ~/.config/vibes/groove-policy.toml");
            println!("  3. /etc/vibes/groove-policy.toml");
            println!();
            println!("If no policy file is found, defaults are used.");
            Ok(())
        }
    }
}

fn run_quarantine(args: QuarantineArgs) -> Result<()> {
    match args.command {
        QuarantineCommands::List => {
            println!("Quarantine queue listing not yet implemented.");
            println!("This will show learnings pending review.");
            Ok(())
        }
        QuarantineCommands::Stats => {
            println!("Quarantine statistics not yet implemented.");
            println!("This will show quarantine queue metrics.");
            Ok(())
        }
    }
}
