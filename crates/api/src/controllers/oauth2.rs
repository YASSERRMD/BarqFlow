use crate::repositories::credential::CredentialRepository;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// State for OAuth2 controller
#[derive(Clone)]
pub struct OAuth2State {
    pub credential_repo: Arc<CredentialRepository>,
}

/// Query parameters received at the OAuth2 callback URL
#[derive(Debug, Deserialize)]
pub struct OAuth2CallbackQuery {
    /// The authorization code returned by the OAuth2 provider
    pub code: Option<String>,
    /// The credential ID to update (state used to track which credential to update)
    pub state: String,
    /// Optional provider error code
    pub error: Option<String>,
    /// Optional provider error details
    pub error_description: Option<String>,
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

fn parse_callback_state(state: &str) -> Result<(Uuid, Option<String>), (StatusCode, String)> {
    let mut parts = state.splitn(2, ':');
    let credential_part = parts.next().unwrap_or_default();
    let csrf_part = parts
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned);

    let credential_id = Uuid::parse_str(credential_part).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Invalid credential ID in state".into(),
        )
    })?;
    Ok((credential_id, csrf_part))
}

fn validate_state_nonce(
    credential_data: &serde_json::Value,
    provided_nonce: Option<&str>,
) -> Result<(), (StatusCode, String)> {
    let expected_nonce = credential_data
        .get("oauthState")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .or_else(|| {
            credential_data
                .get("oauthCsrfState")
                .and_then(|v| v.as_str())
                .filter(|v| !v.trim().is_empty())
        });

    let Some(expected) = expected_nonce else {
        return Ok(()); // Backward compatibility for legacy credentials.
    };

    let Some(provided) = provided_nonce else {
        return Err((
            StatusCode::BAD_REQUEST,
            "Missing OAuth2 state token".to_string(),
        ));
    };

    if provided != expected {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid OAuth2 state token".to_string(),
        ));
    }

    Ok(())
}

fn read_credential_string(data: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|k| data.get(*k).and_then(|v| v.as_str()))
        .map(|v| v.to_string())
}

fn oauth_validation_status(status_code: StatusCode) -> &'static str {
    if status_code.is_server_error() || status_code == StatusCode::BAD_GATEWAY {
        "error"
    } else {
        "invalid"
    }
}

async fn persist_oauth_status(
    repo: &CredentialRepository,
    credential_id: Uuid,
    status: &str,
    message: &str,
) {
    let _ = repo
        .record_test_result(credential_id, status, Some(message))
        .await;
}

fn render_callback_page(success: bool, credential_id: Option<&str>, message: &str) -> Html<String> {
    let payload = serde_json::json!({
        "source": "barqflow-oauth2",
        "success": success,
        "credentialId": credential_id,
        "message": message,
    });
    let payload_json = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
    let title = if success {
        "Credential Connected"
    } else {
        "Credential Connection Failed"
    };
    let badge_class = if success {
        "background:#dcfce7;color:#166534;"
    } else {
        "background:#fee2e2;color:#991b1b;"
    };

    Html(format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>{title}</title>
    <style>
      body {{
        margin: 0;
        min-height: 100vh;
        display: grid;
        place-items: center;
        background: linear-gradient(160deg, #f8fafc 0%, #e2e8f0 100%);
        color: #0f172a;
        font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      }}
      main {{
        width: min(420px, calc(100vw - 32px));
        border-radius: 20px;
        background: rgba(255, 255, 255, 0.96);
        border: 1px solid rgba(148, 163, 184, 0.24);
        box-shadow: 0 24px 80px rgba(15, 23, 42, 0.14);
        padding: 28px;
      }}
      .badge {{
        display: inline-flex;
        align-items: center;
        border-radius: 999px;
        padding: 6px 12px;
        font-size: 12px;
        font-weight: 700;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        {badge_class}
      }}
      h1 {{
        margin: 16px 0 10px;
        font-size: 28px;
        line-height: 1.1;
      }}
      p {{
        margin: 0;
        color: #475569;
        line-height: 1.6;
      }}
      code {{
        display: inline-block;
        margin-top: 14px;
        padding: 8px 10px;
        border-radius: 10px;
        background: #f8fafc;
        color: #0f172a;
      }}
    </style>
  </head>
  <body>
    <main>
      <span class="badge">{title}</span>
      <h1>{title}</h1>
      <p>{message}</p>
      <code>You can close this window.</code>
    </main>
    <script>
      const payload = {payload_json};
      if (window.opener && !window.opener.closed) {{
        window.opener.postMessage(payload, window.location.origin);
        window.close();
      }}
    </script>
  </body>
</html>"#
    ))
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
) -> impl IntoResponse {
    match oauth2_callback_inner(params, state).await {
        Ok(response) => (
            StatusCode::OK,
            render_callback_page(
                true,
                Some(response.credential_id.as_str()),
                &response.message,
            ),
        )
            .into_response(),
        Err((status_code, credential_id, message)) => (
            status_code,
            render_callback_page(false, credential_id.as_deref(), &message),
        )
            .into_response(),
    }
}

async fn oauth2_callback_inner(
    params: OAuth2CallbackQuery,
    state: OAuth2State,
) -> Result<OAuth2CallbackResponse, (StatusCode, Option<String>, String)> {
    // Parse the credential ID and optional CSRF token from OAuth2 `state`.
    let (credential_id, csrf_token) = parse_callback_state(&params.state)
        .map_err(|(status_code, message)| (status_code, None, message))?;

    // Retrieve the existing credential (needed for current data e.g. client_id/secret if not in query)
    let existing = state
        .credential_repo
        .find_by_id(credential_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(credential_id.to_string()),
                e.to_string(),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Some(credential_id.to_string()),
                "Credential not found".to_string(),
            )
        })?;

    if let Some(error) = params.error.as_deref() {
        let message = params
            .error_description
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(|description| format!("OAuth provider returned {}: {}", error, description))
            .unwrap_or_else(|| format!("OAuth provider returned {}", error));
        persist_oauth_status(&state.credential_repo, credential_id, "invalid", &message).await;
        return Err((
            StatusCode::BAD_REQUEST,
            Some(credential_id.to_string()),
            message,
        ));
    }

    if let Err((status_code, message)) = validate_state_nonce(&existing.data, csrf_token.as_deref())
    {
        persist_oauth_status(
            &state.credential_repo,
            credential_id,
            oauth_validation_status(status_code),
            &message,
        )
        .await;
        return Err((status_code, Some(credential_id.to_string()), message));
    }

    // Build the token exchange request body
    let token_url = params
        .token_url
        .clone()
        .or_else(|| read_credential_string(&existing.data, &["accessTokenUrl", "tokenUrl"]))
        .ok_or_else(|| {
            let message = "token_url is required".to_string();
            (
                StatusCode::BAD_REQUEST,
                Some(credential_id.to_string()),
                message,
            )
        })?;

    let redirect_uri = params
        .redirect_uri
        .clone()
        .or_else(|| read_credential_string(&existing.data, &["redirectUri"]))
        .unwrap_or_default();

    let client_id = read_credential_string(&existing.data, &["clientId"])
        .or_else(|| params.client_id.clone())
        .unwrap_or_default();

    let client_secret = read_credential_string(&existing.data, &["clientSecret"])
        .or_else(|| params.client_secret.clone())
        .unwrap_or_default();
    let code = params.code.as_deref().ok_or_else(|| {
        let message = "Missing OAuth2 authorization code".to_string();
        (
            StatusCode::BAD_REQUEST,
            Some(credential_id.to_string()),
            message,
        )
    })?;

    // Exchange code for tokens via POST to the provider token endpoint
    let client = reqwest::Client::new();
    let token_res = client
        .post(&token_url)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &redirect_uri),
            ("client_id", &client_id),
            ("client_secret", &client_secret),
        ])
        .send()
        .await
        .map_err(|e| {
            let message = format!("Token exchange failed: {}", e);
            (
                StatusCode::BAD_GATEWAY,
                Some(credential_id.to_string()),
                message,
            )
        })?;

    if !token_res.status().is_success() {
        let status = token_res.status().as_u16();
        let body = token_res.text().await.unwrap_or_default();
        let message = format!("Provider returned {}: {}", status, body);
        persist_oauth_status(&state.credential_repo, credential_id, "error", &message).await;
        return Err((
            StatusCode::BAD_GATEWAY,
            Some(credential_id.to_string()),
            message,
        ));
    }

    let token_body: OAuth2TokenResponse = token_res.json().await.map_err(|e| {
        let message = format!("Failed to parse token response: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Some(credential_id.to_string()),
            message,
        )
    })?;

    // Merge the new tokens into the existing credential data
    let mut updated_data = existing.data.clone();
    if let Some(obj) = updated_data.as_object_mut() {
        obj.insert(
            "oauthTokenData".to_string(),
            serde_json::json!({
                "accessToken": token_body.access_token,
                "tokenType": token_body.token_type,
                "refreshToken": token_body.refresh_token,
                "expiresIn": token_body.expires_in,
                "scope": token_body.scope,
            }),
        );
        // One-time nonce should not persist after successful callback.
        obj.remove("oauthState");
        obj.remove("oauthCsrfState");
    }

    // Persist the updated credential (CredentialRepository will automatically re-encrypt)
    state
        .credential_repo
        .update(credential_id, &existing.name, updated_data)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(credential_id.to_string()),
                e.to_string(),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Some(credential_id.to_string()),
                "Credential not found".to_string(),
            )
        })?;

    let message = "OAuth2 tokens exchanged and saved successfully".to_string();
    persist_oauth_status(&state.credential_repo, credential_id, "valid", &message).await;

    Ok(OAuth2CallbackResponse {
        success: true,
        credential_id: credential_id.to_string(),
        message,
    })
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

    #[test]
    fn parse_callback_state_supports_uuid_and_nonce() {
        let id = Uuid::new_v4();
        let state = format!("{}:csrf-token-123", id);
        let (parsed_id, nonce) = parse_callback_state(&state).unwrap();
        assert_eq!(parsed_id, id);
        assert_eq!(nonce.as_deref(), Some("csrf-token-123"));
    }

    #[test]
    fn validate_state_nonce_accepts_matching_nonce() {
        let data = json!({ "oauthState": "csrf-abc" });
        assert!(validate_state_nonce(&data, Some("csrf-abc")).is_ok());
    }

    #[test]
    fn validate_state_nonce_rejects_missing_or_mismatched_nonce() {
        let data = json!({ "oauthState": "csrf-abc" });
        assert!(validate_state_nonce(&data, None).is_err());
        assert!(validate_state_nonce(&data, Some("wrong")).is_err());
    }

    #[test]
    fn render_callback_page_embeds_post_message_payload() {
        let html = render_callback_page(true, Some("cred-1"), "Connected").0;
        assert!(html.contains("barqflow-oauth2"));
        assert!(html.contains("window.opener.postMessage"));
        assert!(html.contains("cred-1"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_oauth2_token_exchange(pool: PgPool) {
        std::env::set_var(
            "BARQFLOW_ENCRYPTION_KEY",
            "12345678901234567890123456789012",
        );

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
                json!({
                    "clientId": "test-client-id",
                    "clientSecret": "test-client-secret",
                    "oauthState": "csrf-123"
                }),
            )
            .await
            .unwrap();

        let state = OAuth2State {
            credential_repo: Arc::new(CredentialRepository::new(pool)),
        };
        let app = oauth2_routes(state);

        let token_url = format!("{}/oauth/token", mock_server.uri());
        let uri = format!(
            "/oauth2-credential/callback?code=auth_code_123&state={}%3Acsrf-123&token_url={}&client_id=test-client-id&client_secret=test-client-secret",
            cred.id,
            urlencoding::encode(&token_url)
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
