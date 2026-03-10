use crate::auth::{generate_jwt, hash_password, verify_password, Claims};
use crate::contracts::{AuthResponse, UserProfileResponse};
use crate::controllers::identity::{auth_user_response, build_user_profile_response};
use crate::repositories::{api_key::ApiKeyRepository, workspace::WorkspaceRepository};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use barqflow_db::users::UserRepo;
use serde::Deserialize;
use sqlx::Error as SqlxError;

#[derive(Clone)]
pub struct AppState {
    pub user_repo: std::sync::Arc<UserRepo>,
    pub workspace_repo: std::sync::Arc<WorkspaceRepository>,
    pub api_key_repo: std::sync::Arc<ApiKeyRepository>,
}

pub fn user_routes(state: AppState) -> Router {
    Router::new()
        .route("/users", post(register_user))
        .route("/login", post(login_user))
        .route("/users/me", get(get_profile))
        .with_state(state)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    #[serde(alias = "first_name")]
    pub first_name: Option<String>,
    #[serde(alias = "last_name")]
    pub last_name: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

async fn register_user(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let first_name = payload.first_name.clone();
    let last_name = payload.last_name.clone();
    let hashed_pw = hash_password(&payload.password)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Hashing failed".into()))?;

    let new_user = state
        .user_repo
        .create(
            &payload.email,
            &hashed_pw,
            first_name.clone(),
            last_name,
            "user", // default role
        )
        .await
        .map_err(map_register_error)?;

    let default_workspace_name = payload
        .first_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("{} Workspace", value))
        .unwrap_or_else(|| {
            let seed = payload
                .email
                .split('@')
                .next()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("Workspace");
            format!("{} Workspace", seed)
        });

    state
        .workspace_repo
        .create_workspace(&default_workspace_name, new_user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let token = generate_jwt(&new_user.id.to_string(), &new_user.global_role)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "JWT logic failed".into()))?;

    let profile = build_user_profile_response(
        state.user_repo.as_ref(),
        state.workspace_repo.as_ref(),
        new_user.id,
    )
    .await?;

    Ok(Json(AuthResponse {
        token,
        user_id: new_user.id.to_string(),
        user: auth_user_response(profile),
    }))
}

fn map_register_error(err: SqlxError) -> (StatusCode, String) {
    if let Some(db_err) = err.as_database_error() {
        if db_err.code().as_deref() == Some("23505") {
            return (
                StatusCode::CONFLICT,
                "Email is already registered. Please log in or use another email.".to_string(),
            );
        }
    }

    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

async fn login_user(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let user = state
        .user_repo
        .get_by_email(&payload.email)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid credentials".into()))?;

    let is_valid = verify_password(&user.password_hash, &payload.password).unwrap_or(false);

    if !is_valid {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into()));
    }

    let token = generate_jwt(&user.id.to_string(), &user.global_role).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "JWT generation failed".into(),
        )
    })?;

    let profile = build_user_profile_response(
        state.user_repo.as_ref(),
        state.workspace_repo.as_ref(),
        user.id,
    )
    .await?;

    Ok(Json(AuthResponse {
        token,
        user_id: user.id.to_string(),
        user: auth_user_response(profile),
    }))
}

async fn get_profile(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<UserProfileResponse>, (StatusCode, String)> {
    let user_uuid = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid UUID in token".into()))?;
    let profile = build_user_profile_response(
        state.user_repo.as_ref(),
        state.workspace_repo.as_ref(),
        user_uuid,
    )
    .await?;
    Ok(Json(profile))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use serde_json::json;
    use sqlx::PgPool;
    use tower::ServiceExt;

    #[sqlx::test(migrations = "./migrations")]
    async fn register_user_returns_conflict_for_duplicate_email(pool: PgPool) {
        let app = user_routes(AppState {
            user_repo: std::sync::Arc::new(UserRepo::new(pool.clone())),
            workspace_repo: std::sync::Arc::new(WorkspaceRepository::new(pool.clone())),
            api_key_repo: std::sync::Arc::new(ApiKeyRepository::new(pool)),
        });

        let payload = json!({
            "email": "duplicate@example.com",
            "password": "StrongPass123!"
        })
        .to_string();

        let first = Request::builder()
            .uri("/users")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(payload.clone()))
            .unwrap();
        let first_response = app.clone().oneshot(first).await.unwrap();
        assert_eq!(first_response.status(), StatusCode::OK);

        let second = Request::builder()
            .uri("/users")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(payload))
            .unwrap();
        let second_response = app.oneshot(second).await.unwrap();
        assert_eq!(second_response.status(), StatusCode::CONFLICT);
    }
}
