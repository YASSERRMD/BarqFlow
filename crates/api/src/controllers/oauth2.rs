use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use crate::repositories::credential::CredentialRepository;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// State for OAuth2 controller
#[derive(Clone)]
pub struct OAuth2State {
    pub credential_repo: Arc<CredentialRepository>,
}

/// Query parameters received at the OAuth2 callback URL
#[derive(Debug, Deserialize)]
pub struct OAuth2CallbackQuery {
    /// The authorization code returned by the OAuth2 provider
    pub code: String,
    /// The credential ID to update (state used to track which credential to update)
    pub state: String,
    /// The token exchange URL — provided by the credential type
    pub token_url: Option<String>,
    /// Optional redirect URI to pass to the token exchange
    pub redirect_uri: Option<String>,
    /// Optional client_id for the token exchange
    pub client_id: Option<String>,
    /// Optional client_secret for the token exchange
    pub client_secret: Option<String>,
}

/// Response format returned after successful OAuth2 callback exchange
#[derive(Debug, Serialize)]
pub struct OAuth2CallbackResponse {
    pub success: bool,
    pub credential_id: String,
    pub message: String,
}

/// The OAuth2 token exchange response body received from the provider
#[derive(Debug, Deserialize)]
pub struct OAuth2TokenResponse {
    pub access_token: String,
    pub token_type: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub scope: Option<String>,
}

pub fn oauth2_routes(state: OAuth2State) -> Router {
    Router::new()
        .route("/oauth2-credential/callback", get(oauth2_callback))
        .with_state(state)
}

/// Handles the OAuth2 redirect callback from an external provider.
///
/// Flow:
/// 1. Receive `code` + `state` (credential ID) from the OAuth2 provider
/// 2. Exchange `code` for tokens via `reqwest` POST to `token_url`
/// 3. Securely update credential data in the database with the new tokens
async fn oauth2_callback(
    Query(params): Query<OAuth2CallbackQuery>,
    State(state): State<OAuth2State>,
) -> Result<Json<OAuth2CallbackResponse>, (StatusCode, String)> {
    // Parse the credential ID from the OAuth2 `state` parameter
    let credential_id = uuid::Uuid::parse_str(&params.state)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid credential ID in state".into()))?;

    // Retrieve the existing credential (needed for current data e.g. client_id/secret if not in query)
    let existing = state
        .credential_repo
        .find_by_id(credential_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Credential not found".into()))?;

    // Build the token exchange request body
    let token_url = params
        .token_url
        .clone()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "token_url is required".into()))?;

    let redirect_uri = params
        .redirect_uri
        .clone()
        .unwrap_or_else(|| "".to_string());

    let client_id = params
        .client_id
        .clone()
        .or_else(|| existing.data.get("clientId").and_then(|v| v.as_str()).map(String::from))
        .unwrap_or_default();

    let client_secret = params
        .client_secret
        .clone()
        .or_else(|| existing.data.get("clientSecret").and_then(|v| v.as_str()).map(String::from))
        .unwrap_or_default();

    // Exchange code for tokens via POST to the provider token endpoint
    let client = reqwest::Client::new();
    let token_res = client
        .post(&token_url)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &params.code),
            ("redirect_uri", &redirect_uri),
            ("client_id", &client_id),
            ("client_secret", &client_secret),
        ])
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Token exchange failed: {}", e)))?;

    if !token_res.status().is_success() {
        let status = token_res.status().as_u16();
        let body = token_res.text().await.unwrap_or_default();
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("Provider returned {}: {}", status, body),
        ));
    }

    let token_body: OAuth2TokenResponse = token_res
        .json()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse token response: {}", e)))?;

    // Merge the new tokens into the existing credential data
    let mut updated_data = existing.data.clone();
    if let Some(obj) = updated_data.as_object_mut() {
        obj.insert("oauthTokenData".to_string(), serde_json::json!({
            "accessToken": token_body.access_token,
            "tokenType": token_body.token_type,
            "refreshToken": token_body.refresh_token,
            "expiresIn": token_body.expires_in,
            "scope": token_body.scope,
        }));
    }

    // Persist the updated credential (CredentialRepository will automatically re-encrypt)
    state
        .credential_repo
        .update(credential_id, &existing.name, updated_data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(OAuth2CallbackResponse {
        success: true,
        credential_id: credential_id.to_string(),
        message: "OAuth2 tokens exchanged and saved successfully".to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use serde_json::json;
    use sqlx::PgPool;
    use tower::ServiceExt;
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[sqlx::test(migrations = "./migrations")]
    async fn test_oauth2_token_exchange(pool: PgPool) {
        std::env::set_var("BARQFLOW_ENCRYPTION_KEY", "12345678901234567890123456789012");

        // Setup a mock OAuth2 token endpoint
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .and(body_string_contains("grant_type=authorization_code"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "mocked_access_token_xyz",
                "token_type": "Bearer",
                "refresh_token": "mocked_refresh_token_abc",
                "expires_in": 3600,
            })))
            .mount(&mock_server)
            .await;

        // Create a pre-existing credential in the DB
        let repo = CredentialRepository::new(pool.clone());
        let cred = repo
            .create(
                "Test OAuth2 Credential",
                "myOAuth2Api",
                json!({ "clientId": "test-client-id", "clientSecret": "test-client-secret" }),
            )
            .await
            .unwrap();

        let state = OAuth2State {
            credential_repo: Arc::new(CredentialRepository::new(pool)),
        };
        let app = oauth2_routes(state);

        let token_url = format!("{}/oauth/token", mock_server.uri());
        let uri = format!(
            "/oauth2-credential/callback?code=auth_code_123&state={}&token_url={}&client_id=test-client-id&client_secret=test-client-secret",
            cred.id, urlencoding::encode(&token_url)
        );

        let request = Request::builder()
            .uri(&uri)
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
