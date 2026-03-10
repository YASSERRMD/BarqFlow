use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::{env, sync::OnceLock};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID (UUID string)
    pub role: String,
    pub exp: usize,
}

#[derive(Debug)]
pub struct AuthError;

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

pub fn decode_claims_from_auth_header(auth_header: Option<&str>) -> Result<Option<Claims>, &'static str> {
    let Some(auth_header) = auth_header else {
        return Ok(None);
    };

    let Some(token) = auth_header.strip_prefix("Bearer ") else {
        return Ok(None);
    };

    decode_jwt_token(token).map(Some)
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
