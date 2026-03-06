use crate::auth::{generate_jwt, hash_password, verify_password, Claims};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use barqflow_db::users::UserRepo;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppState {
    pub user_repo: std::sync::Arc<UserRepo>,
}

pub fn user_routes(state: AppState) -> Router {
    Router::new()
        .route("/users", post(register_user))
        .route("/login", post(login_user))
        .route("/users/me", get(get_profile))
        .with_state(state)
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
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
    let hashed_pw = hash_password(&payload.password)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Hashing failed".into()))?;

    let new_user = state
        .user_repo
        .create(
            &payload.email,
            &hashed_pw,
            payload.first_name,
            payload.last_name,
            "user", // default role
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let token = generate_jwt(&new_user.id.to_string(), &new_user.global_role)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "JWT logic failed".into()))?;

    Ok(Json(AuthResponse {
        token,
        user_id: new_user.id.to_string(),
    }))
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

    Ok(Json(AuthResponse {
        token,
        user_id: user.id.to_string(),
    }))
}

#[derive(Serialize)]
pub struct ProfileResponse {
    pub id: String,
    pub email: String,
    pub role: String,
}

async fn get_profile(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<ProfileResponse>, (StatusCode, String)> {
    let user_uuid = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid UUID in token".into()))?;

    let user = state
        .user_repo
        .get_by_id(user_uuid)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".into()))?;

    Ok(Json(ProfileResponse {
        id: user.id.to_string(),
        email: user.email,
        role: user.global_role,
    }))
}
