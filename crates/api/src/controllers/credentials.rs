use crate::auth::Claims;
use crate::repositories::credential::CredentialRepository;
use axum::http::StatusCode;
use axum::{
    extract::{Json, Path, Query, State},
    routing::{get, post, put},
    Router,
};
use barqflow_core::types::GenericValue;
use barqflow_db::models::CredentialEntity;
use barqflow_registry::registry::CredentialRegistry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone)]
pub struct AppState {
    pub credential_repo: std::sync::Arc<CredentialRepository>,
    pub credential_registry: std::sync::Arc<CredentialRegistry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CredentialResponse {
    pub id: uuid::Uuid,
    pub name: String,
    pub cred_type: String,
    pub credential_type: String,
    pub data: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<CredentialEntity> for CredentialResponse {
    fn from(value: CredentialEntity) -> Self {
        Self {
            id: value.id,
            name: value.name,
            cred_type: value.cred_type.clone(),
            credential_type: value.cred_type,
            data: mask_credential_data(&value.data),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
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
            put(update_credential).delete(delete_credential),
        )
        .route("/credentials/{id}/test", post(test_saved_credential))
        .route("/credentials/types", get(get_credential_types))
        .route("/credentials/test", post(test_credential))
        .with_state(state)
}

#[derive(Deserialize)]
pub struct CreateCredentialRequest {
    pub name: String,
    pub cred_type: String,
    pub data: serde_json::Value,
}

#[derive(Deserialize)]
pub struct UpdateCredentialRequest {
    pub name: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct TestCredentialRequest {
    pub cred_type: String,
    pub data: std::collections::HashMap<String, barqflow_core::types::GenericValue>,
}

#[derive(Deserialize)]
pub struct CredentialListQuery {
    pub r#type: Option<String>,
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
        .create(&payload.name, &payload.cred_type, payload.data)
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

    let updated = state
        .credential_repo
        .update(id, &next_name, merged_data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Credential not found".to_string()))?;

    Ok(Json(CredentialResponse::from(updated)))
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
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let is_valid = validate_credential_data(&state, &payload.cred_type, &payload.data).await?;

    Ok(Json(serde_json::json!({
        "valid": is_valid
    })))
}

async fn test_saved_credential(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let credential = state
        .credential_repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Credential not found".to_string()))?;

    let data_map = credential_data_to_map(&credential.data)?;
    let is_valid = validate_credential_data(&state, &credential.cred_type, &data_map).await?;

    Ok(Json(serde_json::json!({
        "valid": is_valid,
        "credentialId": credential.id,
        "credentialType": credential.cred_type,
    })))
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
