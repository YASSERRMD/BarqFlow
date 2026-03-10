use std::sync::Arc;

use axum::{
    extract::{Json, Path, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, post},
    Router,
};
use barqflow_db::users::UserRepo;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::{
        hash_password, require_authenticated_user, require_workspace_role, verify_password,
        workspace_role_rank, AuthMethod,
    },
    contracts::{
        ApiKeyCreateResponse, ApiKeyResponse, UserProfileResponse, WorkspaceMemberResponse,
        WorkspaceSummaryResponse,
    },
    repositories::{
        api_key::{
            build_api_key_secret, build_api_key_token, ApiKeyRepository, API_KEY_TOKEN_PREFIX,
        },
        workspace::{WorkspaceMemberRecord, WorkspaceMembershipDocument, WorkspaceRepository},
    },
};

#[derive(Clone)]
pub struct AppState {
    pub user_repo: Arc<UserRepo>,
    pub workspace_repo: Arc<WorkspaceRepository>,
    pub api_key_repo: Arc<ApiKeyRepository>,
}

pub fn identity_routes(state: AppState) -> Router {
    Router::new()
        .route("/users/change-password", post(change_password))
        .route("/workspaces", get(list_workspaces).post(create_workspace))
        .route("/workspaces/current", get(get_current_workspace))
        .route("/workspaces/{id}/select", post(select_workspace))
        .route(
            "/workspaces/current/members",
            get(list_workspace_members).post(add_workspace_member),
        )
        .route("/api-keys", get(list_api_keys).post(create_api_key))
        .route("/api-keys/{id}", delete(delete_api_key))
        .with_state(state)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkspaceRequest {
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddWorkspaceMemberRequest {
    pub email: String,
    pub role: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub expires_at: Option<DateTime<Utc>>,
}

async fn change_password(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<UserProfileResponse>, (StatusCode, String)> {
    let auth = require_authenticated_user(
        &headers,
        Arc::clone(&state.user_repo),
        Arc::clone(&state.workspace_repo),
        Arc::clone(&state.api_key_repo),
    )
    .await?;

    if auth.auth_method != AuthMethod::Jwt {
        return Err((
            StatusCode::FORBIDDEN,
            "Password changes require a signed-in user session".to_string(),
        ));
    }

    if payload.new_password.trim().len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            "New password must be at least 8 characters".to_string(),
        ));
    }

    let user = state
        .user_repo
        .get_by_id(auth.id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".to_string()))?;

    let valid = verify_password(&user.password_hash, &payload.current_password).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Password verification failed".to_string(),
        )
    })?;
    if !valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Current password is incorrect".to_string(),
        ));
    }

    let password_hash = hash_password(&payload.new_password).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Hashing failed".to_string(),
        )
    })?;
    state
        .user_repo
        .update_password(auth.id, &password_hash)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".to_string()))?;

    Ok(Json(
        build_user_profile_response(
            state.user_repo.as_ref(),
            state.workspace_repo.as_ref(),
            auth.id,
        )
        .await?,
    ))
}

async fn list_workspaces(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Vec<WorkspaceSummaryResponse>>, (StatusCode, String)> {
    let auth = require_authenticated_user(
        &headers,
        Arc::clone(&state.user_repo),
        Arc::clone(&state.workspace_repo),
        Arc::clone(&state.api_key_repo),
    )
    .await?;

    let workspaces = state
        .workspace_repo
        .list_for_user(auth.id)
        .await
        .map_err(internal_error)?;

    Ok(Json(
        workspaces
            .into_iter()
            .map(workspace_summary_response)
            .collect(),
    ))
}

async fn create_workspace(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<CreateWorkspaceRequest>,
) -> Result<Json<WorkspaceSummaryResponse>, (StatusCode, String)> {
    let auth = require_authenticated_user(
        &headers,
        Arc::clone(&state.user_repo),
        Arc::clone(&state.workspace_repo),
        Arc::clone(&state.api_key_repo),
    )
    .await?;

    let name = payload.name.trim();
    if name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Workspace name is required".to_string(),
        ));
    }

    let workspace = state
        .workspace_repo
        .create_workspace(name, auth.id)
        .await
        .map_err(internal_error)?;

    state
        .user_repo
        .set_active_workspace(auth.id, workspace.workspace.id)
        .await
        .map_err(internal_error)?;

    Ok(Json(workspace_summary_response(workspace)))
}

async fn get_current_workspace(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<WorkspaceSummaryResponse>, (StatusCode, String)> {
    let auth = require_authenticated_user(
        &headers,
        Arc::clone(&state.user_repo),
        Arc::clone(&state.workspace_repo),
        Arc::clone(&state.api_key_repo),
    )
    .await?;

    let current = state
        .workspace_repo
        .find_membership(auth.id, auth.workspace_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Workspace not found".to_string()))?;

    Ok(Json(workspace_summary_response(current)))
}

async fn select_workspace(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkspaceSummaryResponse>, (StatusCode, String)> {
    let auth = require_authenticated_user(
        &headers,
        Arc::clone(&state.user_repo),
        Arc::clone(&state.workspace_repo),
        Arc::clone(&state.api_key_repo),
    )
    .await?;

    let membership = state
        .workspace_repo
        .find_membership(auth.id, id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::FORBIDDEN, "Workspace access denied".to_string()))?;

    state
        .user_repo
        .set_active_workspace(auth.id, id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".to_string()))?;

    Ok(Json(workspace_summary_response(membership)))
}

async fn list_workspace_members(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Vec<WorkspaceMemberResponse>>, (StatusCode, String)> {
    let auth = require_authenticated_user(
        &headers,
        Arc::clone(&state.user_repo),
        Arc::clone(&state.workspace_repo),
        Arc::clone(&state.api_key_repo),
    )
    .await?;

    let members = state
        .workspace_repo
        .list_members(auth.workspace_id)
        .await
        .map_err(internal_error)?;

    Ok(Json(
        members.into_iter().map(workspace_member_response).collect(),
    ))
}

async fn add_workspace_member(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<AddWorkspaceMemberRequest>,
) -> Result<Json<WorkspaceMemberResponse>, (StatusCode, String)> {
    let auth = require_authenticated_user(
        &headers,
        Arc::clone(&state.user_repo),
        Arc::clone(&state.workspace_repo),
        Arc::clone(&state.api_key_repo),
    )
    .await?;
    require_workspace_role(&auth, "admin")?;

    let role = normalize_workspace_role(&payload.role)?;
    if role == "owner" && workspace_role_rank(&auth.workspace_role) < workspace_role_rank("owner") {
        return Err((
            StatusCode::FORBIDDEN,
            "Only workspace owners can assign the owner role".to_string(),
        ));
    }

    let email = payload.email.trim();
    if email.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Member email is required".to_string(),
        ));
    }

    let user = state
        .user_repo
        .get_by_email(email)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".to_string()))?;

    let membership = state
        .workspace_repo
        .add_or_update_member(auth.workspace_id, user.id, role)
        .await
        .map_err(internal_error)?;

    if user.active_workspace_id.is_none() {
        let _ = state
            .user_repo
            .set_active_workspace(user.id, auth.workspace_id)
            .await;
    }

    Ok(Json(WorkspaceMemberResponse {
        membership_id: membership.id,
        user_id: user.id,
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
        role: membership.role,
        created_at: membership.created_at,
        updated_at: membership.updated_at,
    }))
}

async fn list_api_keys(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Vec<ApiKeyResponse>>, (StatusCode, String)> {
    let auth = require_authenticated_user(
        &headers,
        Arc::clone(&state.user_repo),
        Arc::clone(&state.workspace_repo),
        Arc::clone(&state.api_key_repo),
    )
    .await?;
    require_workspace_role(&auth, "member")?;

    let keys = state
        .api_key_repo
        .list_for_workspace(auth.workspace_id)
        .await
        .map_err(internal_error)?;

    Ok(Json(keys.into_iter().map(ApiKeyResponse::from).collect()))
}

async fn create_api_key(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<Json<ApiKeyCreateResponse>, (StatusCode, String)> {
    let auth = require_authenticated_user(
        &headers,
        Arc::clone(&state.user_repo),
        Arc::clone(&state.workspace_repo),
        Arc::clone(&state.api_key_repo),
    )
    .await?;
    require_workspace_role(&auth, "member")?;

    let name = payload.name.trim();
    if name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "API key name is required".to_string(),
        ));
    }

    let secret = build_api_key_secret();
    let hash = hash_password(&secret).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Hashing failed".to_string(),
        )
    })?;
    let prefix = format!("{}_{}", API_KEY_TOKEN_PREFIX, &secret[..8]);

    let key = state
        .api_key_repo
        .create(
            auth.workspace_id,
            auth.id,
            name,
            &prefix,
            &hash,
            payload.expires_at,
        )
        .await
        .map_err(internal_error)?;

    Ok(Json(ApiKeyCreateResponse {
        api_key: build_api_key_token(key.id, &secret),
        key: ApiKeyResponse::from(key),
    }))
}

async fn delete_api_key(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let auth = require_authenticated_user(
        &headers,
        Arc::clone(&state.user_repo),
        Arc::clone(&state.workspace_repo),
        Arc::clone(&state.api_key_repo),
    )
    .await?;
    require_workspace_role(&auth, "member")?;

    let deleted = state
        .api_key_repo
        .revoke(auth.workspace_id, id)
        .await
        .map_err(internal_error)?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "API key not found".to_string()))
    }
}

pub async fn build_user_profile_response(
    user_repo: &UserRepo,
    workspace_repo: &WorkspaceRepository,
    user_id: Uuid,
) -> Result<UserProfileResponse, (StatusCode, String)> {
    let user = user_repo
        .get_by_id(user_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".to_string()))?;

    let workspaces = workspace_repo
        .list_for_user(user.id)
        .await
        .map_err(internal_error)?;
    let active_workspace = if let Some(current) = workspace_repo
        .get_current_for_user(user.id)
        .await
        .map_err(internal_error)?
    {
        current
    } else {
        workspaces.first().cloned().ok_or_else(|| {
            (
                StatusCode::FORBIDDEN,
                "User is not assigned to a workspace".to_string(),
            )
        })?
    };

    Ok(UserProfileResponse {
        id: user.id.to_string(),
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
        role: user.global_role,
        workspace_role: active_workspace.membership.role.clone(),
        active_workspace: workspace_summary_response(active_workspace.clone()),
        workspaces: workspaces
            .into_iter()
            .map(workspace_summary_response)
            .collect(),
    })
}

pub fn auth_user_response(profile: UserProfileResponse) -> crate::contracts::AuthUserResponse {
    crate::contracts::AuthUserResponse {
        id: profile.id,
        email: profile.email,
        first_name: profile.first_name,
        last_name: profile.last_name,
        role: profile.role,
        workspace_role: profile.workspace_role,
        active_workspace: profile.active_workspace,
        workspaces: profile.workspaces,
    }
}

fn workspace_summary_response(value: WorkspaceMembershipDocument) -> WorkspaceSummaryResponse {
    WorkspaceSummaryResponse {
        id: value.workspace.id,
        name: value.workspace.name,
        slug: value.workspace.slug,
        role: value.membership.role,
        created_at: value.workspace.created_at,
        updated_at: value.workspace.updated_at,
    }
}

fn workspace_member_response(value: WorkspaceMemberRecord) -> WorkspaceMemberResponse {
    WorkspaceMemberResponse {
        membership_id: value.membership_id,
        user_id: value.user_id,
        email: value.email,
        first_name: value.first_name,
        last_name: value.last_name,
        role: value.role,
        created_at: value.created_at,
        updated_at: value.updated_at,
    }
}

fn normalize_workspace_role(role: &str) -> Result<&str, (StatusCode, String)> {
    match role.trim().to_ascii_lowercase().as_str() {
        "owner" => Ok("owner"),
        "admin" => Ok("admin"),
        "member" => Ok("member"),
        "viewer" => Ok("viewer"),
        _ => Err((
            StatusCode::BAD_REQUEST,
            "Workspace role must be one of owner, admin, member, or viewer".to_string(),
        )),
    }
}

fn internal_error(error: impl ToString) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}
