use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, HeaderMap, StatusCode},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc, sync::OnceLock};
use uuid::Uuid;

use crate::repositories::{
    api_key::{parse_api_key_token, ApiKeyRepository},
    workspace::WorkspaceRepository,
};
use barqflow_db::users::UserRepo;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID (UUID string)
    pub role: String,
    pub exp: usize,
}

#[derive(Debug)]
pub struct AuthError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AuthMethod {
    Jwt,
    ApiKey,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub global_role: String,
    pub workspace_id: Uuid,
    pub workspace_name: String,
    pub workspace_slug: String,
    pub workspace_role: String,
    pub auth_method: AuthMethod,
}

static JWT_SECRET_CACHE: OnceLock<String> = OnceLock::new();

pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AuthError)?
        .to_string();
    Ok(password_hash)
}

pub fn verify_password(hash: &str, password: &str) -> Result<bool, AuthError> {
    let parsed_hash = PasswordHash::new(hash).map_err(|_| AuthError)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

fn resolve_jwt_secret(
    jwt_secret_env: Option<String>,
    barqflow_env: Option<String>,
) -> Result<String, &'static str> {
    if let Some(secret) = jwt_secret_env {
        let trimmed = secret.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    let runtime_env = barqflow_env
        .unwrap_or_else(|| "development".to_string())
        .to_lowercase();
    if runtime_env == "production" {
        return Err("JWT_SECRET must be set when BARQFLOW_ENV=production");
    }

    let generated = format!(
        "dev-jwt-{}{}",
        Uuid::new_v4().simple(),
        Uuid::new_v4().simple()
    );
    eprintln!(
        "WARNING: JWT_SECRET is not set. Using an ephemeral development secret for this process."
    );
    Ok(generated)
}

fn jwt_secret() -> &'static str {
    JWT_SECRET_CACHE
        .get_or_init(|| {
            resolve_jwt_secret(env::var("JWT_SECRET").ok(), env::var("BARQFLOW_ENV").ok())
                .unwrap_or_else(|message| panic!("{message}"))
        })
        .as_str()
}

pub fn decode_jwt_token(token: &str) -> Result<Claims, &'static str> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )
    .map(|token_data| token_data.claims)
    .map_err(|_| "Invalid token")
}

pub fn decode_claims_from_auth_header(
    auth_header: Option<&str>,
) -> Result<Option<Claims>, &'static str> {
    let Some(auth_header) = auth_header else {
        return Ok(None);
    };

    let Some(token) = auth_header.strip_prefix("Bearer ") else {
        return Ok(None);
    };

    decode_jwt_token(token).map(Some)
}

fn bearer_token_from_headers(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
}

pub async fn require_authenticated_user(
    headers: &HeaderMap,
    user_repo: Arc<UserRepo>,
    workspace_repo: Arc<WorkspaceRepository>,
    api_key_repo: Arc<ApiKeyRepository>,
) -> Result<AuthenticatedUser, (StatusCode, String)> {
    let token = bearer_token_from_headers(headers).ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            "Missing or invalid authorization header".to_string(),
        )
    })?;

    if let Some((api_key_id, secret)) = parse_api_key_token(token) {
        return authenticate_with_api_key(
            api_key_id,
            secret,
            user_repo,
            workspace_repo,
            api_key_repo,
        )
        .await;
    }

    let claims = decode_jwt_token(token)
        .map_err(|message| (StatusCode::UNAUTHORIZED, message.to_string()))?;
    authenticate_with_claims(claims, user_repo, workspace_repo).await
}

pub fn workspace_role_rank(role: &str) -> u8 {
    match role.trim().to_ascii_lowercase().as_str() {
        "owner" => 4,
        "admin" => 3,
        "member" => 2,
        "viewer" => 1,
        _ => 0,
    }
}

pub fn require_workspace_role(
    auth: &AuthenticatedUser,
    minimum_role: &str,
) -> Result<(), (StatusCode, String)> {
    let minimum_rank = workspace_role_rank(minimum_role);
    let current_rank = workspace_role_rank(&auth.workspace_role);
    let is_global_admin = auth.global_role.eq_ignore_ascii_case("admin");

    if is_global_admin || current_rank >= minimum_rank {
        return Ok(());
    }

    Err((
        StatusCode::FORBIDDEN,
        format!("This action requires '{}' workspace access", minimum_role),
    ))
}

async fn authenticate_with_claims(
    claims: Claims,
    user_repo: Arc<UserRepo>,
    workspace_repo: Arc<WorkspaceRepository>,
) -> Result<AuthenticatedUser, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid UUID in token".to_string()))?;

    let user = user_repo
        .get_by_id(user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".to_string()))?;

    let active_workspace = resolve_active_workspace(&user_repo, &workspace_repo, &user)
        .await?
        .ok_or_else(|| {
            (
                StatusCode::FORBIDDEN,
                "User is not assigned to a workspace".to_string(),
            )
        })?;

    Ok(AuthenticatedUser {
        id: user.id,
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
        global_role: user.global_role,
        workspace_id: active_workspace.workspace.id,
        workspace_name: active_workspace.workspace.name,
        workspace_slug: active_workspace.workspace.slug,
        workspace_role: active_workspace.membership.role,
        auth_method: AuthMethod::Jwt,
    })
}

async fn authenticate_with_api_key(
    api_key_id: Uuid,
    secret: &str,
    user_repo: Arc<UserRepo>,
    workspace_repo: Arc<WorkspaceRepository>,
    api_key_repo: Arc<ApiKeyRepository>,
) -> Result<AuthenticatedUser, (StatusCode, String)> {
    let api_key = api_key_repo
        .find_by_id(api_key_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid API key".to_string()))?;

    if api_key.revoked_at.is_some() {
        return Err((
            StatusCode::UNAUTHORIZED,
            "API key has been revoked".to_string(),
        ));
    }

    if api_key
        .expires_at
        .map(|expires_at| expires_at <= chrono::Utc::now())
        .unwrap_or(false)
    {
        return Err((StatusCode::UNAUTHORIZED, "API key has expired".to_string()));
    }

    let valid = verify_password(&api_key.key_hash, secret).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "API key verification failed".to_string(),
        )
    })?;
    if !valid {
        return Err((StatusCode::UNAUTHORIZED, "Invalid API key".to_string()));
    }

    let _ = api_key_repo.record_usage(api_key.id).await;

    let user = user_repo
        .get_by_id(api_key.user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".to_string()))?;

    let membership = workspace_repo
        .find_membership(user.id, api_key.workspace_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            (
                StatusCode::FORBIDDEN,
                "User no longer belongs to the API key workspace".to_string(),
            )
        })?;

    Ok(AuthenticatedUser {
        id: user.id,
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
        global_role: user.global_role,
        workspace_id: membership.workspace.id,
        workspace_name: membership.workspace.name,
        workspace_slug: membership.workspace.slug,
        workspace_role: membership.membership.role,
        auth_method: AuthMethod::ApiKey,
    })
}

async fn resolve_active_workspace(
    user_repo: &UserRepo,
    workspace_repo: &WorkspaceRepository,
    user: &barqflow_db::models::UserEntity,
) -> Result<Option<crate::repositories::workspace::WorkspaceMembershipDocument>, (StatusCode, String)>
{
    if let Some(current) = workspace_repo
        .get_current_for_user(user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        return Ok(Some(current));
    }

    let memberships = workspace_repo
        .list_for_user(user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let Some(first_membership) = memberships.into_iter().next() else {
        return Ok(None);
    };

    user_repo
        .set_active_workspace(user.id, first_membership.workspace.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Some(first_membership))
}

pub fn generate_jwt(user_id: &str, role: &str) -> Result<String, AuthError> {
    let secret = jwt_secret();

    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        exp: expiration as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| AuthError)
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok());

        match decode_claims_from_auth_header(auth_header) {
            Ok(Some(claims)) => Ok(claims),
            Ok(None) => Err((
                StatusCode::UNAUTHORIZED,
                "Missing or invalid authorization header",
            )),
            Err(message) => Err((StatusCode::UNAUTHORIZED, message)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_jwt_secret;

    #[test]
    fn resolve_jwt_secret_prefers_env_value() {
        let secret = resolve_jwt_secret(
            Some("my-explicit-secret".to_string()),
            Some("production".to_string()),
        )
        .unwrap();
        assert_eq!(secret, "my-explicit-secret");
    }

    #[test]
    fn resolve_jwt_secret_generates_ephemeral_secret_in_dev() {
        let secret = resolve_jwt_secret(None, Some("development".to_string())).unwrap();
        assert!(secret.starts_with("dev-jwt-"));
        assert!(secret.len() > "dev-jwt-".len() + 10);
    }

    #[test]
    fn resolve_jwt_secret_rejects_missing_secret_in_production() {
        let error = resolve_jwt_secret(None, Some("production".to_string())).unwrap_err();
        assert_eq!(error, "JWT_SECRET must be set when BARQFLOW_ENV=production");
    }
}
