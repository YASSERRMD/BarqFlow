use crate::auth::{require_authenticated_user, require_workspace_role, AuthenticatedUser};
use crate::contracts::{
    AuditLogResponse, PromotionRequestResponse, PromotionTargetResponse, SecretProviderResponse,
    WorkspacePolicyResponse,
};
use crate::governance::{
    default_workspace_policy, enforce_workflow_policy, record_governance_event,
    validate_secret_provider,
};
use crate::repositories::{
    api_key::ApiKeyRepository, governance::GovernanceRepository, workflow::WorkflowRepository,
    workspace::WorkspaceRepository,
};
use axum::{
    extract::{Json, Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json as JsonResponse, Router,
};
use barqflow_db::users::UserRepo;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub governance_repo: Arc<GovernanceRepository>,
    pub workflow_repo: Arc<WorkflowRepository>,
    pub user_repo: Arc<UserRepo>,
    pub workspace_repo: Arc<WorkspaceRepository>,
    pub api_key_repo: Arc<ApiKeyRepository>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GovernanceListQuery {
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateSecretProviderRequest {
    name: String,
    provider_type: String,
    #[serde(default)]
    config: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateWorkspacePolicyRequest {
    #[serde(default)]
    blocked_node_types: Vec<String>,
    #[serde(default)]
    blocked_support_tiers: Vec<String>,
    #[serde(default)]
    approval_required_node_types: Vec<String>,
    #[serde(default)]
    max_workflow_nodes: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreatePromotionTargetRequest {
    name: String,
    environment: String,
    git_repo_url: Option<String>,
    git_branch: Option<String>,
    #[serde(default = "default_requires_approval")]
    requires_approval: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreatePromotionRequestRequest {
    workflow_id: Uuid,
    target_id: Uuid,
    source_control_ref: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApprovePromotionRequestRequest {
    notes: Option<String>,
}

pub fn governance_routes(state: AppState) -> Router {
    Router::new()
        .route("/governance/audit-logs", get(list_audit_logs))
        .route(
            "/governance/secret-providers",
            get(list_secret_providers).post(create_secret_provider),
        )
        .route(
            "/governance/secret-providers/{id}/validate",
            post(validate_secret_provider_route),
        )
        .route(
            "/governance/workspace-policy",
            get(get_workspace_policy).put(update_workspace_policy),
        )
        .route(
            "/governance/promotion-targets",
            get(list_promotion_targets).post(create_promotion_target),
        )
        .route(
            "/governance/promotion-requests",
            get(list_promotion_requests).post(create_promotion_request),
        )
        .route(
            "/governance/promotion-requests/{id}/approve",
            post(approve_promotion_request),
        )
        .with_state(state)
}

async fn list_audit_logs(
    headers: HeaderMap,
    Query(query): Query<GovernanceListQuery>,
    State(state): State<AppState>,
) -> Result<JsonResponse<Vec<AuditLogResponse>>, (StatusCode, String)> {
    let auth = require_governance_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;

    let logs = state
        .governance_repo
        .list_audit_logs(auth.workspace_id, query.limit.unwrap_or(50).clamp(1, 200))
        .await
        .map_err(internal_error)?;

    Ok(JsonResponse(
        logs.into_iter().map(AuditLogResponse::from).collect(),
    ))
}

async fn list_secret_providers(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<JsonResponse<Vec<SecretProviderResponse>>, (StatusCode, String)> {
    let auth = require_governance_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;

    let providers = state
        .governance_repo
        .list_secret_providers(auth.workspace_id)
        .await
        .map_err(internal_error)?;

    Ok(JsonResponse(
        providers
            .into_iter()
            .map(SecretProviderResponse::from)
            .collect(),
    ))
}

async fn create_secret_provider(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<CreateSecretProviderRequest>,
) -> Result<JsonResponse<SecretProviderResponse>, (StatusCode, String)> {
    let auth = require_governance_auth(&headers, &state).await?;
    require_workspace_role(&auth, "admin")?;

    let name = payload.name.trim();
    let provider_type = payload.provider_type.trim();
    if name.is_empty() || provider_type.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Provider name and type are required".to_string(),
        ));
    }

    let provider = state
        .governance_repo
        .create_secret_provider(
            auth.workspace_id,
            name,
            provider_type,
            payload.config,
            "draft",
            None,
        )
        .await
        .map_err(internal_error)?;

    let validation = validate_secret_provider(&provider.provider_type, &provider.config).await;
    let provider = state
        .governance_repo
        .update_secret_provider_validation(
            auth.workspace_id,
            provider.id,
            &validation.status,
            validation.message.as_deref(),
        )
        .await
        .map_err(internal_error)?
        .ok_or_else(|| not_found("Secret provider not found after creation"))?;

    record_governance_event(
        &state.governance_repo,
        &auth,
        "governance.secretProvider.created",
        "secretProvider",
        Some(provider.id),
        &format!(
            "Created {} secret provider '{}'.",
            provider.provider_type, provider.name
        ),
        json!({
            "providerType": provider.provider_type,
            "status": provider.status,
        }),
    )
    .await
    .map_err(internal_error)?;

    Ok(JsonResponse(SecretProviderResponse::from(provider)))
}

async fn validate_secret_provider_route(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<JsonResponse<SecretProviderResponse>, (StatusCode, String)> {
    let auth = require_governance_auth(&headers, &state).await?;
    require_workspace_role(&auth, "admin")?;

    let provider = state
        .governance_repo
        .find_secret_provider_in_workspace(auth.workspace_id, id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| not_found("Secret provider not found"))?;
    let validation = validate_secret_provider(&provider.provider_type, &provider.config).await;

    let provider = state
        .governance_repo
        .update_secret_provider_validation(
            auth.workspace_id,
            provider.id,
            &validation.status,
            validation.message.as_deref(),
        )
        .await
        .map_err(internal_error)?
        .ok_or_else(|| not_found("Secret provider not found"))?;

    record_governance_event(
        &state.governance_repo,
        &auth,
        "governance.secretProvider.validated",
        "secretProvider",
        Some(provider.id),
        &format!("Validated secret provider '{}'.", provider.name),
        json!({
            "status": provider.status,
            "validatedAt": validation.validated_at,
            "lastError": provider.last_error,
        }),
    )
    .await
    .map_err(internal_error)?;

    Ok(JsonResponse(SecretProviderResponse::from(provider)))
}

async fn get_workspace_policy(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<JsonResponse<WorkspacePolicyResponse>, (StatusCode, String)> {
    let auth = require_governance_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;

    let policy = state
        .governance_repo
        .get_workspace_policy(auth.workspace_id)
        .await
        .map_err(internal_error)?
        .unwrap_or_else(|| default_workspace_policy(auth.workspace_id));

    Ok(JsonResponse(WorkspacePolicyResponse::from(policy)))
}

async fn update_workspace_policy(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<UpdateWorkspacePolicyRequest>,
) -> Result<JsonResponse<WorkspacePolicyResponse>, (StatusCode, String)> {
    let auth = require_governance_auth(&headers, &state).await?;
    require_workspace_role(&auth, "admin")?;

    let policy = state
        .governance_repo
        .upsert_workspace_policy(
            auth.workspace_id,
            json!(payload.blocked_node_types),
            json!(payload.blocked_support_tiers),
            json!(payload.approval_required_node_types),
            payload.max_workflow_nodes,
        )
        .await
        .map_err(internal_error)?;

    record_governance_event(
        &state.governance_repo,
        &auth,
        "governance.policy.updated",
        "workspacePolicy",
        Some(auth.workspace_id),
        "Updated workspace governance policy.",
        json!({
            "blockedNodeTypes": policy.blocked_node_types,
            "blockedSupportTiers": policy.blocked_support_tiers,
            "approvalRequiredNodeTypes": policy.approval_required_node_types,
            "maxWorkflowNodes": policy.max_workflow_nodes,
        }),
    )
    .await
    .map_err(internal_error)?;

    Ok(JsonResponse(WorkspacePolicyResponse::from(policy)))
}

async fn list_promotion_targets(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<JsonResponse<Vec<PromotionTargetResponse>>, (StatusCode, String)> {
    let auth = require_governance_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;

    let targets = state
        .governance_repo
        .list_promotion_targets(auth.workspace_id)
        .await
        .map_err(internal_error)?;

    Ok(JsonResponse(
        targets
            .into_iter()
            .map(PromotionTargetResponse::from)
            .collect(),
    ))
}

async fn create_promotion_target(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<CreatePromotionTargetRequest>,
) -> Result<JsonResponse<PromotionTargetResponse>, (StatusCode, String)> {
    let auth = require_governance_auth(&headers, &state).await?;
    require_workspace_role(&auth, "admin")?;

    let name = payload.name.trim();
    let environment = payload.environment.trim();
    if name.is_empty() || environment.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Promotion target name and environment are required".to_string(),
        ));
    }

    let target = state
        .governance_repo
        .create_promotion_target(
            auth.workspace_id,
            name,
            environment,
            payload.git_repo_url.as_deref(),
            payload.git_branch.as_deref(),
            payload.requires_approval,
        )
        .await
        .map_err(internal_error)?;

    record_governance_event(
        &state.governance_repo,
        &auth,
        "governance.promotionTarget.created",
        "promotionTarget",
        Some(target.id),
        &format!("Created promotion target '{}'.", target.name),
        json!({
            "environment": target.environment,
            "gitRepoUrl": target.git_repo_url,
            "gitBranch": target.git_branch,
            "requiresApproval": target.requires_approval,
        }),
    )
    .await
    .map_err(internal_error)?;

    Ok(JsonResponse(PromotionTargetResponse::from(target)))
}

async fn list_promotion_requests(
    headers: HeaderMap,
    Query(query): Query<GovernanceListQuery>,
    State(state): State<AppState>,
) -> Result<JsonResponse<Vec<PromotionRequestResponse>>, (StatusCode, String)> {
    let auth = require_governance_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;

    let requests = state
        .governance_repo
        .list_promotion_requests(auth.workspace_id, query.limit.unwrap_or(50).clamp(1, 200))
        .await
        .map_err(internal_error)?;

    Ok(JsonResponse(
        requests
            .into_iter()
            .map(PromotionRequestResponse::from)
            .collect(),
    ))
}

async fn create_promotion_request(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<CreatePromotionRequestRequest>,
) -> Result<JsonResponse<PromotionRequestResponse>, (StatusCode, String)> {
    let auth = require_governance_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;

    let workflow = state
        .workflow_repo
        .find_document_by_id_in_workspace(auth.workspace_id, payload.workflow_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| not_found("Workflow not found"))?;
    let target = state
        .governance_repo
        .find_promotion_target_in_workspace(auth.workspace_id, payload.target_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| not_found("Promotion target not found"))?;
    let policy_evaluation = enforce_workflow_policy(
        &state.governance_repo,
        auth.workspace_id,
        &workflow.workflow.nodes,
    )
    .await
    .map_err(|message| (StatusCode::FORBIDDEN, message))?;

    let auto_approved = !target.requires_approval && !policy_evaluation.requires_approval;
    let status = if auto_approved {
        "approved"
    } else {
        "pendingApproval"
    };
    let workflow_snapshot = json!({
        "workflowId": workflow.workflow.id,
        "name": workflow.workflow.name.clone(),
        "active": workflow.workflow.active,
        "tags": workflow.tags.iter().map(|tag| tag.name.clone()).collect::<Vec<_>>(),
        "nodes": workflow.workflow.nodes.clone(),
        "connections": workflow.workflow.connections.clone(),
        "settings": workflow.workflow.settings.clone(),
        "policy": {
            "nodeCount": policy_evaluation.node_count,
            "approvalReasons": policy_evaluation.approval_reasons,
        }
    });

    let request = state
        .governance_repo
        .create_promotion_request(
            auth.workspace_id,
            workflow.workflow.id,
            target.id,
            Some(auth.id),
            status,
            payload.source_control_ref.as_deref(),
            workflow_snapshot,
            payload.notes.as_deref(),
            if auto_approved { Some(auth.id) } else { None },
            if auto_approved {
                Some(chrono::Utc::now())
            } else {
                None
            },
        )
        .await
        .map_err(internal_error)?;

    record_governance_event(
        &state.governance_repo,
        &auth,
        "governance.promotionRequest.created",
        "promotionRequest",
        Some(request.id),
        &format!(
            "Created promotion request for workflow '{}' to target '{}'.",
            workflow.workflow.name, target.name
        ),
        json!({
            "targetId": target.id,
            "targetName": target.name,
            "status": request.status,
            "sourceControlRef": request.source_control_ref,
            "requiresApproval": !auto_approved,
        }),
    )
    .await
    .map_err(internal_error)?;

    Ok(JsonResponse(PromotionRequestResponse::from(request)))
}

async fn approve_promotion_request(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ApprovePromotionRequestRequest>,
) -> Result<JsonResponse<PromotionRequestResponse>, (StatusCode, String)> {
    let auth = require_governance_auth(&headers, &state).await?;
    require_workspace_role(&auth, "admin")?;

    let existing = state
        .governance_repo
        .find_promotion_request_in_workspace(auth.workspace_id, id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| not_found("Promotion request not found"))?;

    if existing.status.eq_ignore_ascii_case("approved") {
        return Err((
            StatusCode::CONFLICT,
            "Promotion request is already approved".to_string(),
        ));
    }

    let request = state
        .governance_repo
        .approve_promotion_request(auth.workspace_id, id, auth.id, payload.notes.as_deref())
        .await
        .map_err(internal_error)?
        .ok_or_else(|| not_found("Promotion request not found"))?;

    record_governance_event(
        &state.governance_repo,
        &auth,
        "governance.promotionRequest.approved",
        "promotionRequest",
        Some(request.id),
        "Approved promotion request.",
        json!({
            "workflowId": request.workflow_id,
            "targetId": request.target_id,
            "notes": request.notes,
        }),
    )
    .await
    .map_err(internal_error)?;

    Ok(JsonResponse(PromotionRequestResponse::from(request)))
}

async fn require_governance_auth(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<AuthenticatedUser, (StatusCode, String)> {
    require_authenticated_user(
        headers,
        Arc::clone(&state.user_repo),
        Arc::clone(&state.workspace_repo),
        Arc::clone(&state.api_key_repo),
    )
    .await
}

fn default_requires_approval() -> bool {
    true
}

fn internal_error(error: impl ToString) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn not_found(message: &str) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, message.to_string())
}
