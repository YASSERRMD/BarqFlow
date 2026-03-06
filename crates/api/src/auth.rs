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
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID (UUID string)
    pub role: String,
    pub exp: usize,
}

#[derive(Debug)]
pub struct AuthError;

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

pub fn generate_jwt(user_id: &str, role: &str) -> Result<String, AuthError> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "SUPER_SECRET_TEMP_KEY".to_string());

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

        if let Some(auth_header) = auth_header {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                let secret =
                    env::var("JWT_SECRET").unwrap_or_else(|_| "SUPER_SECRET_TEMP_KEY".to_string());

                let token_data = decode::<Claims>(
                    token,
                    &DecodingKey::from_secret(secret.as_bytes()),
                    &Validation::default(),
                )
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token"))?;

                return Ok(token_data.claims);
            }
        }

        Err((
            StatusCode::UNAUTHORIZED,
            "Missing or invalid authorization header",
        ))
    }
}
