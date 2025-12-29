//! Groove security API handlers
//!
//! REST endpoints for quarantine management and security operations.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use vibes_groove::security::{OrgRole, Policy, ReviewOutcome, TrustLevel, load_policy_or_default};

use crate::AppState;

// ============================================================================
// Policy Types
// ============================================================================

/// Security policy response
#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyResponse {
    /// Injection policy settings
    pub injection: InjectionPolicyResponse,
    /// Quarantine policy settings
    pub quarantine: QuarantinePolicyResponse,
    /// Import/export policy settings
    pub import_export: ImportExportPolicyResponse,
    /// Audit policy settings
    pub audit: AuditPolicyResponse,
}

/// Injection policy in response
#[derive(Debug, Serialize, Deserialize)]
pub struct InjectionPolicyResponse {
    pub block_quarantined: bool,
    pub allow_personal_injection: bool,
    pub allow_unverified_injection: bool,
}

/// Quarantine policy in response
#[derive(Debug, Serialize, Deserialize)]
pub struct QuarantinePolicyResponse {
    pub reviewers: Vec<String>,
    pub visible_to: Vec<String>,
    pub auto_delete_after_days: Option<u32>,
}

/// Import/export policy in response
#[derive(Debug, Serialize, Deserialize)]
pub struct ImportExportPolicyResponse {
    pub allow_import_from_file: bool,
    pub allow_import_from_url: bool,
    pub allowed_import_sources: Vec<String>,
    pub allow_export_personal: bool,
    pub allow_export_enterprise: bool,
}

/// Audit policy in response
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

// ============================================================================
// Trust Types
// ============================================================================

/// Trust level information
#[derive(Debug, Serialize, Deserialize)]
pub struct TrustLevelInfo {
    /// Level name
    pub name: String,
    /// Trust score (0-100)
    pub score: u8,
    /// Description
    pub description: String,
}

/// Trust hierarchy response
#[derive(Debug, Serialize, Deserialize)]
pub struct TrustHierarchyResponse {
    /// Ordered list of trust levels (highest to lowest)
    pub levels: Vec<TrustLevelInfo>,
}

/// Role permissions response
#[derive(Debug, Serialize, Deserialize)]
pub struct RolePermissionsResponse {
    /// Role name
    pub role: String,
    /// Permission flags
    pub permissions: PermissionFlags,
}

/// Permission flags
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

// ============================================================================
// Quarantine Types
// ============================================================================

/// Summary of a quarantined learning
#[derive(Debug, Serialize, Deserialize)]
pub struct QuarantinedLearningSummary {
    /// Learning ID
    pub id: String,
    /// Description of the learning
    pub description: String,
    /// Trust level
    pub trust_level: String,
    /// Quarantine reason
    pub reason: String,
    /// When it was quarantined (ISO 8601)
    pub quarantined_at: String,
    /// Whether review is pending
    pub pending_review: bool,
}

/// Quarantine list response
#[derive(Debug, Serialize, Deserialize)]
pub struct QuarantineListResponse {
    /// List of quarantined learnings
    pub items: Vec<QuarantinedLearningSummary>,
    /// Total count
    pub total: usize,
}

/// Quarantine statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct QuarantineStatsResponse {
    /// Total quarantined items
    pub total: usize,
    /// Pending review
    pub pending_review: usize,
    /// Approved (restored)
    pub approved: usize,
    /// Rejected (deleted)
    pub rejected: usize,
    /// Escalated
    pub escalated: usize,
}

/// Review request body
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewRequest {
    /// Review outcome: "approve", "reject", or "escalate"
    pub outcome: String,
    /// Optional reviewer notes
    pub notes: Option<String>,
}

/// Review response
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewResponse {
    /// The action taken
    pub outcome: String,
    /// Whether the learning was restored
    pub restored: bool,
    /// Whether the learning was deleted
    pub deleted: bool,
}

/// Error response
#[derive(Debug, Serialize, Deserialize)]
pub struct GrooveErrorResponse {
    /// Error message
    pub error: String,
    /// Error code
    pub code: String,
}

// ============================================================================
// Policy Endpoints
// ============================================================================

/// GET /api/groove/policy - Get current security policy
pub async fn get_policy(State(_state): State<Arc<AppState>>) -> Json<PolicyResponse> {
    let policy = load_policy_or_default("groove-policy.toml");
    Json(PolicyResponse::from(policy))
}

// ============================================================================
// Trust Endpoints
// ============================================================================

/// GET /api/groove/trust/levels - Get trust level hierarchy
pub async fn get_trust_levels() -> Json<TrustHierarchyResponse> {
    // TrustLevel uses integer discriminants for scores
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

    Json(TrustHierarchyResponse { levels })
}

/// GET /api/groove/trust/role/:role - Get permissions for a role
pub async fn get_role_permissions(
    Path(role): Path<String>,
) -> Result<Json<RolePermissionsResponse>, (StatusCode, Json<GrooveErrorResponse>)> {
    let parsed_role: OrgRole = role.parse().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(GrooveErrorResponse {
                error: format!(
                    "Invalid role: {}. Use: admin, curator, member, viewer",
                    role
                ),
                code: "INVALID_ROLE".to_string(),
            }),
        )
    })?;

    let perms = parsed_role.permissions();

    Ok(Json(RolePermissionsResponse {
        role: parsed_role.as_str().to_string(),
        permissions: PermissionFlags {
            can_create: perms.can_create,
            can_read: perms.can_read,
            can_modify: perms.can_modify,
            can_delete: perms.can_delete,
            can_publish: perms.can_publish,
            can_review: perms.can_review,
            can_admin: perms.can_admin,
        },
    }))
}

// ============================================================================
// Quarantine Endpoints
// ============================================================================

/// GET /api/groove/quarantine - List quarantined learnings
pub async fn list_quarantined(State(_state): State<Arc<AppState>>) -> Json<QuarantineListResponse> {
    // Placeholder - full implementation requires storage integration
    Json(QuarantineListResponse {
        items: vec![],
        total: 0,
    })
}

/// GET /api/groove/quarantine/stats - Get quarantine statistics
pub async fn get_quarantine_stats(
    State(_state): State<Arc<AppState>>,
) -> Json<QuarantineStatsResponse> {
    // Placeholder - full implementation requires storage integration
    Json(QuarantineStatsResponse {
        total: 0,
        pending_review: 0,
        approved: 0,
        rejected: 0,
        escalated: 0,
    })
}

/// POST /api/groove/quarantine/:id/review - Review a quarantined learning
pub async fn review_quarantined(
    Path(id): Path<String>,
    State(_state): State<Arc<AppState>>,
    Json(request): Json<ReviewRequest>,
) -> Result<Json<ReviewResponse>, (StatusCode, Json<GrooveErrorResponse>)> {
    // Parse the outcome
    let outcome = match request.outcome.to_lowercase().as_str() {
        "approve" | "approved" => ReviewOutcome::Approved,
        "reject" | "rejected" => ReviewOutcome::Rejected,
        "escalate" | "escalated" => ReviewOutcome::Escalated,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(GrooveErrorResponse {
                    error: format!(
                        "Invalid outcome: {}. Use: approve, reject, or escalate",
                        request.outcome
                    ),
                    code: "INVALID_OUTCOME".to_string(),
                }),
            ));
        }
    };

    // Placeholder - full implementation requires storage integration
    // For now, return not found since we have no storage
    let _ = (id, outcome);
    Err((
        StatusCode::NOT_FOUND,
        Json(GrooveErrorResponse {
            error: "Quarantine storage not configured".to_string(),
            code: "NOT_CONFIGURED".to_string(),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, routing::get};
    use axum_test::TestServer;

    fn create_test_app() -> Router {
        let state = Arc::new(AppState::new());
        Router::new()
            .route("/api/groove/policy", get(get_policy))
            .route("/api/groove/trust/levels", get(get_trust_levels))
            .route("/api/groove/trust/role/:role", get(get_role_permissions))
            .route("/api/groove/quarantine", get(list_quarantined))
            .route("/api/groove/quarantine/stats", get(get_quarantine_stats))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_get_policy() {
        let server = TestServer::new(create_test_app()).unwrap();
        let response = server.get("/api/groove/policy").await;
        response.assert_status_ok();

        let body: PolicyResponse = response.json();
        assert!(body.injection.block_quarantined);
    }

    #[tokio::test]
    async fn test_get_trust_levels() {
        let server = TestServer::new(create_test_app()).unwrap();
        let response = server.get("/api/groove/trust/levels").await;
        response.assert_status_ok();

        let body: TrustHierarchyResponse = response.json();
        assert_eq!(body.levels.len(), 7);
        assert_eq!(body.levels[0].name, "Local");
        assert_eq!(body.levels[0].score, 100);
        // Last one should be Quarantined with score 0
        assert_eq!(body.levels[6].name, "Quarantined");
        assert_eq!(body.levels[6].score, 0);
    }

    #[tokio::test]
    async fn test_get_role_permissions_admin() {
        let server = TestServer::new(create_test_app()).unwrap();
        let response = server.get("/api/groove/trust/role/admin").await;
        response.assert_status_ok();

        let body: RolePermissionsResponse = response.json();
        assert_eq!(body.role, "admin");
        assert!(body.permissions.can_admin);
        assert!(body.permissions.can_review);
    }

    #[tokio::test]
    async fn test_get_role_permissions_viewer() {
        let server = TestServer::new(create_test_app()).unwrap();
        let response = server.get("/api/groove/trust/role/viewer").await;
        response.assert_status_ok();

        let body: RolePermissionsResponse = response.json();
        assert_eq!(body.role, "viewer");
        assert!(body.permissions.can_read);
        assert!(!body.permissions.can_create);
        assert!(!body.permissions.can_admin);
    }

    #[tokio::test]
    async fn test_get_role_permissions_invalid() {
        let server = TestServer::new(create_test_app()).unwrap();
        let response = server.get("/api/groove/trust/role/invalid").await;
        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_list_quarantined_empty() {
        let server = TestServer::new(create_test_app()).unwrap();
        let response = server.get("/api/groove/quarantine").await;
        response.assert_status_ok();

        let body: QuarantineListResponse = response.json();
        assert_eq!(body.total, 0);
        assert!(body.items.is_empty());
    }

    #[tokio::test]
    async fn test_get_quarantine_stats() {
        let server = TestServer::new(create_test_app()).unwrap();
        let response = server.get("/api/groove/quarantine/stats").await;
        response.assert_status_ok();

        let body: QuarantineStatsResponse = response.json();
        assert_eq!(body.total, 0);
        assert_eq!(body.pending_review, 0);
    }
}
