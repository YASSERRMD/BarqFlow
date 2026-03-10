use crate::auth::Claims;
use crate::contracts::{
    CredentialOAuthConnectResponse, CredentialResponse, CredentialValidationResponse,
};
use crate::repositories::credential::CredentialRepository;
use axum::http::{header, HeaderMap, StatusCode};
use axum::{
    extract::{Json, Path, Query, State},
    routing::{get, post},
    Router,
};
use barqflow_core::types::GenericValue;
use barqflow_db::models::CredentialEntity;
use barqflow_registry::registry::CredentialRegistry;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone)]
pub struct AppState {
    pub credential_repo: std::sync::Arc<CredentialRepository>,
    pub credential_registry: std::sync::Arc<CredentialRegistry>,
}

impl From<CredentialEntity> for CredentialResponse {
    fn from(value: CredentialEntity) -> Self {
        CredentialResponse::from_masked_entity(value.clone(), mask_credential_data(&value.data))
    }
}

fn mask_credential_data(data: &serde_json::Value) -> serde_json::Value {
    let Some(object) = data.as_object() else {
        return serde_json::json!({});
    };

    let masked: serde_json::Map<String, serde_json::Value> = object
        .keys()
        .map(|k| (k.clone(), serde_json::json!("******")))
        .collect();
    serde_json::Value::Object(masked)
}

pub fn credential_routes(state: AppState) -> Router {
    Router::new()
        .route("/credentials", get(get_credentials).post(create_credential))
        .route(
            "/credentials/{id}",
            get(get_credential)
                .put(update_credential)
                .delete(delete_credential),
        )
        .route("/credentials/{id}/test", post(test_saved_credential))
        .route("/credentials/{id}/rotate", post(rotate_credential))
        .route("/credentials/{id}/oauth2/connect", post(start_oauth_connect))
        .route("/credentials/types", get(get_credential_types))
        .route("/credentials/test", post(test_credential))
        .with_state(state)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCredentialRequest {
    pub name: String,
    #[serde(alias = "cred_type")]
    pub credential_type: String,
    pub data: serde_json::Value,
}

#[derive(Deserialize)]
pub struct UpdateCredentialRequest {
    pub name: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestCredentialRequest {
    #[serde(alias = "cred_type")]
    pub credential_type: String,
    pub data: std::collections::HashMap<String, barqflow_core::types::GenericValue>,
}

#[derive(Deserialize)]
pub struct CredentialListQuery {
    pub r#type: Option<String>,
}

fn build_validation_response(
    valid: bool,
    status: impl Into<String>,
    message: impl Into<String>,
    credential_id: Option<uuid::Uuid>,
    credential_type: Option<String>,
) -> CredentialValidationResponse {
    CredentialValidationResponse {
        valid,
        status: status.into(),
        message: message.into(),
        credential_id,
        credential_type,
    }
}

fn validation_status_from_error(status_code: StatusCode) -> &'static str {
    if status_code.is_server_error() || status_code == StatusCode::BAD_GATEWAY {
        "error"
    } else {
        "invalid"
    }
}

fn read_credential_string(data: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| data.get(*key).and_then(|value| value.as_str()))
        .map(|value| value.to_string())
}

fn request_origin(headers: &HeaderMap) -> Option<String> {
    if let Some(origin) = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Some(origin.trim_end_matches('/').to_string());
    }

    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("http");

    let host = headers
        .get("x-forwarded-host")
        .or_else(|| headers.get(header::HOST))
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())?;

    Some(format!("{}://{}", proto, host))
}

fn default_oauth_redirect_uri(headers: &HeaderMap) -> Result<String, (StatusCode, String)> {
    let origin = request_origin(headers).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "Unable to determine request origin for OAuth callback".to_string(),
        )
    })?;

    Ok(format!("{}/rest/oauth2-credential/callback", origin))
}

fn build_oauth_connect_payload(
    credential_id: uuid::Uuid,
    credential_type: &str,
    current_data: &serde_json::Value,
    headers: &HeaderMap,
) -> Result<(serde_json::Value, CredentialOAuthConnectResponse), (StatusCode, String)> {
    let grant_type = read_credential_string(current_data, &["grantType"])
        .unwrap_or_else(|| "authorizationCode".to_string());
    if grant_type != "authorizationCode" {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "OAuth connect only supports authorizationCode grant credentials, received '{}'",
                grant_type
            ),
        ));
    }

    let auth_url = read_credential_string(current_data, &["authUrl", "authorizationUrl"])
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                "Credential is missing authUrl required for OAuth connect".to_string(),
            )
        })?;
    let client_id = read_credential_string(current_data, &["clientId"]).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "Credential is missing clientId required for OAuth connect".to_string(),
        )
    })?;
    let redirect_uri = read_credential_string(current_data, &["redirectUri"])
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(default_oauth_redirect_uri(headers)?);

    let nonce = uuid::Uuid::new_v4().to_string();
    let state_token = format!("{}:{}", credential_id, nonce);

    let mut updated_data = current_data.clone();
    let object = updated_data.as_object_mut().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "Credential payload must be a JSON object".to_string(),
        )
    })?;
    object.insert("oauthState".to_string(), serde_json::json!(nonce));
    object.insert("oauthCsrfState".to_string(), serde_json::json!(nonce));
    object.insert("redirectUri".to_string(), serde_json::json!(redirect_uri.clone()));

    let mut connect_url = reqwest::Url::parse(&auth_url).map_err(|error| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid OAuth authorization URL: {}", error),
        )
    })?;

    {
        let mut query_pairs = connect_url.query_pairs_mut();
        query_pairs.append_pair("response_type", "code");
        query_pairs.append_pair("client_id", &client_id);
        query_pairs.append_pair("redirect_uri", &redirect_uri);
        query_pairs.append_pair("state", &state_token);

        if let Some(scope) = read_credential_string(current_data, &["scope", "scopes"])
            .filter(|value| !value.trim().is_empty())
        {
            query_pairs.append_pair("scope", &scope);
        }

        for (source_key, query_key) in [
            ("accessType", "access_type"),
            ("prompt", "prompt"),
            ("audience", "audience"),
            ("resource", "resource"),
            ("includeGrantedScopes", "include_granted_scopes"),
        ] {
            if let Some(value) = read_credential_string(current_data, &[source_key])
                .filter(|value| !value.trim().is_empty())
            {
                query_pairs.append_pair(query_key, &value);
            }
        }
    }

    Ok((
        updated_data,
        CredentialOAuthConnectResponse {
            credential_id,
            credential_type: credential_type.to_string(),
            connect_url: connect_url.to_string(),
            redirect_uri,
            state: state_token,
        },
    ))
}

async fn get_credentials(
    _claims: Claims,
    State(state): State<AppState>,
    Query(query): Query<CredentialListQuery>,
) -> Result<Json<Vec<CredentialResponse>>, (StatusCode, String)> {
    let creds = if let Some(cred_type) = query.r#type {
        state
            .credential_repo
            .find_by_type(&cred_type)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        state
            .credential_repo
            .find_all()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    Ok(Json(
        creds.into_iter().map(CredentialResponse::from).collect(),
    ))
}

async fn get_credential(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<CredentialResponse>, (StatusCode, String)> {
    let credential = state
        .credential_repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Credential not found".to_string()))?;

    Ok(Json(CredentialResponse::from(credential)))
}

async fn get_credential_types(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Vec<barqflow_core::properties::ICredentialProperties>>, (StatusCode, String)> {
    let creds = state.credential_registry.get_all_credentials();
    let schema_list: Vec<_> = creds
        .into_iter()
        .map(|info| info.cred_impl.get_description())
        .collect();

    Ok(Json(schema_list))
}

async fn create_credential(
    _claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateCredentialRequest>,
) -> Result<Json<CredentialResponse>, (StatusCode, String)> {
    let new_cred = state
        .credential_repo
        .create(&payload.name, &payload.credential_type, payload.data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(CredentialResponse::from(new_cred)))
}

fn merge_credential_data(
    existing: &serde_json::Value,
    patch: Option<&serde_json::Value>,
) -> Result<serde_json::Value, (StatusCode, String)> {
    let mut merged = existing.as_object().cloned().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Stored credential payload is not an object".to_string(),
        )
    })?;

    let Some(patch_value) = patch else {
        return Ok(serde_json::Value::Object(merged));
    };

    let patch_obj = patch_value.as_object().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "Credential update payload `data` must be a JSON object".to_string(),
        )
    })?;

    for (key, value) in patch_obj {
        merged.insert(key.clone(), value.clone());
    }

    Ok(serde_json::Value::Object(merged))
}

async fn update_credential(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    Json(payload): Json<UpdateCredentialRequest>,
) -> Result<Json<CredentialResponse>, (StatusCode, String)> {
    let existing = state
        .credential_repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Credential not found".to_string()))?;

    let next_name = payload
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or(existing.name.as_str())
        .to_string();

    let merged_data = merge_credential_data(&existing.data, payload.data.as_ref())?;
    let credential_changed = merged_data != existing.data;

    let mut updated = state
        .credential_repo
        .update(id, &next_name, merged_data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Credential not found".to_string()))?;

    if credential_changed {
        updated = state
            .credential_repo
            .clear_test_result(id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .ok_or_else(|| (StatusCode::NOT_FOUND, "Credential not found".to_string()))?;
    }

    Ok(Json(CredentialResponse::from(updated)))
}

async fn rotate_credential(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    Json(payload): Json<UpdateCredentialRequest>,
) -> Result<Json<CredentialResponse>, (StatusCode, String)> {
    let existing = state
        .credential_repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Credential not found".to_string()))?;

    let next_name = payload
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or(existing.name.as_str())
        .to_string();
    let merged_data = merge_credential_data(&existing.data, payload.data.as_ref())?;

    let rotated = state
        .credential_repo
        .rotate(id, &next_name, merged_data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Credential not found".to_string()))?;

    Ok(Json(CredentialResponse::from(rotated)))
}

async fn start_oauth_connect(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    headers: HeaderMap,
) -> Result<Json<CredentialOAuthConnectResponse>, (StatusCode, String)> {
    let existing = state
        .credential_repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Credential not found".to_string()))?;

    let (updated_data, response) =
        build_oauth_connect_payload(existing.id, &existing.cred_type, &existing.data, &headers)?;

    state
        .credential_repo
        .update(existing.id, &existing.name, updated_data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Credential not found".to_string()))?;

    Ok(Json(response))
}

fn credential_data_to_map(
    data: &serde_json::Value,
) -> Result<HashMap<String, GenericValue>, (StatusCode, String)> {
    let object = data.as_object().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "Credential payload must be a JSON object".to_string(),
        )
    })?;

    Ok(object
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect())
}

async fn validate_credential_data(
    state: &AppState,
    cred_type: &str,
    data: &HashMap<String, GenericValue>,
) -> Result<bool, (StatusCode, String)> {
    let cred_info = state
        .credential_registry
        .get_credential(cred_type)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Credential type '{}' not found in registry", cred_type),
            )
        })?;

    if let Some(rules) = cred_info.cred_impl.test_request() {
        let client = reqwest::Client::new();
        let method = match rules.method.to_uppercase().as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            "PATCH" => reqwest::Method::PATCH,
            _ => reqwest::Method::GET,
        };

        let response = client
            .request(method, &rules.url)
            .send()
            .await
            .map_err(|e| {
                (
                    StatusCode::BAD_GATEWAY,
                    format!("Failed to reach test URL: {}", e),
                )
            })?;

        let status = response.status().as_u16();
        if !rules.expected_status.contains(&status) {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("API replied with unexpected status: {}", status),
            ));
        }
    }

    cred_info
        .cred_impl
        .test_credential(data)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn test_credential(
    _claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<TestCredentialRequest>,
) -> Result<Json<CredentialValidationResponse>, (StatusCode, String)> {
    match validate_credential_data(&state, &payload.credential_type, &payload.data).await {
        Ok(true) => Ok(Json(build_validation_response(
            true,
            "valid",
            "Credential validated successfully.",
            None,
            Some(payload.credential_type),
        ))),
        Ok(false) => Ok(Json(build_validation_response(
            false,
            "invalid",
            "Credential validation returned false.",
            None,
            Some(payload.credential_type),
        ))),
        Err((status_code, message)) => Ok(Json(build_validation_response(
            false,
            validation_status_from_error(status_code),
            message,
            None,
            Some(payload.credential_type),
        ))),
    }
}

async fn test_saved_credential(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<CredentialValidationResponse>, (StatusCode, String)> {
    let credential = state
        .credential_repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Credential not found".to_string()))?;

    let data_map = match credential_data_to_map(&credential.data) {
        Ok(data_map) => data_map,
        Err((status_code, message)) => {
            state
                .credential_repo
                .record_test_result(id, validation_status_from_error(status_code), Some(&message))
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            return Ok(Json(build_validation_response(
                false,
                validation_status_from_error(status_code),
                message,
                Some(credential.id),
                Some(credential.cred_type),
            )));
        }
    };

    match validate_credential_data(&state, &credential.cred_type, &data_map).await {
        Ok(true) => {
            let message = "Credential validated successfully.";
            state
                .credential_repo
                .record_test_result(id, "valid", Some(message))
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            Ok(Json(build_validation_response(
                true,
                "valid",
                message,
                Some(credential.id),
                Some(credential.cred_type),
            )))
        }
        Ok(false) => {
            let message = "Credential validation returned false.";
            state
                .credential_repo
                .record_test_result(id, "invalid", Some(message))
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            Ok(Json(build_validation_response(
                false,
                "invalid",
                message,
                Some(credential.id),
                Some(credential.cred_type),
            )))
        }
        Err((status_code, message)) => {
            let validation_status = validation_status_from_error(status_code);
            state
                .credential_repo
                .record_test_result(id, validation_status, Some(&message))
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            Ok(Json(build_validation_response(
                false,
                validation_status,
                message,
                Some(credential.id),
                Some(credential.cred_type),
            )))
        }
    }
}

async fn delete_credential(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let deleted = state
        .credential_repo
        .delete(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, "Credential not found".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use barqflow_core::traits::{ICredentialTestRequest, ICredentialType};
    use barqflow_registry::registry::CredentialInfo;
    use serde_json::json;
    use sqlx::PgPool;
    use std::sync::Arc;
    use tower::ServiceExt;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_mask_credential_data_hides_values() {
        let masked = mask_credential_data(&serde_json::json!({
            "apiKey": "secret",
            "password": "123456"
        }));

        assert_eq!(masked["apiKey"], "******");
        assert_eq!(masked["password"], "******");
    }

    struct TestCredential {
        test_url: String,
    }

    #[async_trait::async_trait]
    impl ICredentialType for TestCredential {
        fn get_description(&self) -> barqflow_core::properties::ICredentialProperties {
            barqflow_core::properties::ICredentialProperties {
                name: "testAuth".to_string(),
                display_name: "Test Auth".to_string(),
                properties: vec![],
                authenticate: None,
                documentation_url: None,
                notice: None,
            }
        }

        fn test_request(&self) -> Option<ICredentialTestRequest> {
            Some(ICredentialTestRequest {
                method: "GET".to_string(),
                url: self.test_url.clone(),
                expected_status: vec![200],
            })
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_credential_ping_success(pool: PgPool) {
        std::env::set_var(
            "BARQFLOW_ENCRYPTION_KEY",
            "12345678901234567890123456789012",
        );
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ping"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let registry = CredentialRegistry::new();
        registry
            .register_credential(CredentialInfo {
                name: "testAuth".to_string(),
                cred_impl: Arc::new(TestCredential {
                    test_url: format!("{}/ping", mock_server.uri()),
                }),
            })
            .unwrap();

        let state = AppState {
            credential_repo: Arc::new(CredentialRepository::new(pool)),
            credential_registry: Arc::new(registry),
        };

        let app = credential_routes(state);

        let req_body = json!({
            "cred_type": "testAuth",
            "data": {}
        });

        let token =
            crate::auth::generate_jwt("00000000-0000-0000-0000-000000000000", "admin").unwrap();

        let request = Request::builder()
            .uri("/credentials/test")
            .method("POST")
            .header("content-type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::from(req_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_update_credential_merges_existing_payload(pool: PgPool) {
        std::env::set_var(
            "BARQFLOW_ENCRYPTION_KEY",
            "12345678901234567890123456789012",
        );

        let registry = CredentialRegistry::new();
        let repo = Arc::new(CredentialRepository::new(pool.clone()));
        let state = AppState {
            credential_repo: Arc::clone(&repo),
            credential_registry: Arc::new(registry),
        };
        let app = credential_routes(state);

        let created = repo
            .create(
                "OpenAI Prod",
                "openAiApi",
                json!({
                    "apiKey": "sk-old",
                    "baseUrl": "https://api.openai.com/v1"
                }),
            )
            .await
            .unwrap();

        let token =
            crate::auth::generate_jwt("00000000-0000-0000-0000-000000000000", "admin").unwrap();
        let update_body = json!({
            "name": "OpenAI Prod Renamed",
            "data": {
                "apiKey": "sk-new"
            }
        });

        let request = Request::builder()
            .uri(format!("/credentials/{}", created.id))
            .method("PUT")
            .header("content-type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::from(update_body.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let reloaded = repo.find_by_id(created.id).await.unwrap().unwrap();
        assert_eq!(reloaded.name, "OpenAI Prod Renamed");
        assert_eq!(reloaded.data["apiKey"], "sk-new");
        assert_eq!(reloaded.data["baseUrl"], "https://api.openai.com/v1");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_saved_credential_test_endpoint_uses_stored_data(pool: PgPool) {
        std::env::set_var(
            "BARQFLOW_ENCRYPTION_KEY",
            "12345678901234567890123456789012",
        );

        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ping"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let registry = CredentialRegistry::new();
        registry
            .register_credential(CredentialInfo {
                name: "testAuth".to_string(),
                cred_impl: Arc::new(TestCredential {
                    test_url: format!("{}/ping", mock_server.uri()),
                }),
            })
            .unwrap();

        let repo = Arc::new(CredentialRepository::new(pool.clone()));
        let state = AppState {
            credential_repo: Arc::clone(&repo),
            credential_registry: Arc::new(registry),
        };
        let app = credential_routes(state);

        let created = repo
            .create("Saved Credential", "testAuth", json!({"token":"abc"}))
            .await
            .unwrap();

        let token =
            crate::auth::generate_jwt("00000000-0000-0000-0000-000000000000", "admin").unwrap();

        let request = Request::builder()
            .uri(format!("/credentials/{}/test", created.id))
            .method("POST")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
