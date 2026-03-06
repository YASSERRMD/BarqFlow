use crate::auth::Claims;
use axum::http::StatusCode;
use axum::{
    extract::{Json, State},
    routing::{get, post},
    Router,
};
use barqflow_registry::registry::CredentialRegistry;
use crate::repositories::credential::CredentialRepository;
use barqflow_db::models::CredentialEntity;
use serde::Deserialize;

#[derive(Clone)]
pub struct AppState {
    pub credential_repo: std::sync::Arc<CredentialRepository>,
    pub credential_registry: std::sync::Arc<CredentialRegistry>,
}

pub fn credential_routes(state: AppState) -> Router {
    Router::new()
        .route("/credentials", get(get_credentials).post(create_credential))
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
pub struct TestCredentialRequest {
    pub cred_type: String,
    pub data: std::collections::HashMap<String, barqflow_core::types::GenericValue>,
}

async fn get_credentials(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Vec<CredentialEntity>>, (StatusCode, String)> {
    let creds = state
        .credential_repo
        .find_all()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(creds))
}

async fn create_credential(
    _claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<CreateCredentialRequest>,
) -> Result<Json<CredentialEntity>, (StatusCode, String)> {
    let new_cred = state
        .credential_repo
        .create(&payload.name, &payload.cred_type, payload.data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(new_cred))
}

async fn test_credential(
    _claims: Claims,
    State(state): State<AppState>,
    Json(payload): Json<TestCredentialRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let cred_info = state
        .credential_registry
        .get_credential(&payload.cred_type)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Credential type '{}' not found in registry", payload.cred_type),
            )
        })?;

    // Check if the credential type provides standard API ping rules
    if let Some(rules) = cred_info.cred_impl.test_request() {
        let client = reqwest::Client::new();
        
        // Parse HTTP Method
        let method = match rules.method.to_uppercase().as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            "PATCH" => reqwest::Method::PATCH,
            _ => reqwest::Method::GET,
        };

        // Execute Ping (Authentication injection can be done inside test_credential override)
        // For generic ping tests, if the API requires the username/password in the URL or Body,
        // it must be handled by the specialized `test_credential` method on the trait implementation.
        // This is a basic generic ping.
        let response = client
            .request(method, &rules.url)
            .send()
            .await
            .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Failed to reach test URL: {}", e)))?;

        let status = response.status().as_u16();
        if !rules.expected_status.contains(&status) {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("API replied with unexpected status: {}", status),
            ));
        }
    }

    // Call the specific implementation's test_credential method for deep validation
    let is_valid = cred_info
        .cred_impl
        .test_credential(&payload.data)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "valid": is_valid
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use serde_json::json;
    use sqlx::PgPool;
    use std::sync::Arc;
    use tower::ServiceExt;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use barqflow_registry::registry::CredentialInfo;
    use barqflow_core::traits::{ICredentialType, ICredentialTestRequest};

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
        std::env::set_var("BARQFLOW_ENCRYPTION_KEY", "12345678901234567890123456789012");
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ping"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let registry = CredentialRegistry::new();
        registry.register_credential(CredentialInfo {
            name: "testAuth".to_string(),
            cred_impl: Arc::new(TestCredential {
                test_url: format!("{}/ping", mock_server.uri()),
            }),
        }).unwrap();

        let state = AppState {
            credential_repo: Arc::new(CredentialRepository::new(pool)),
            credential_registry: Arc::new(registry),
        };

        let app = credential_routes(state);

        let req_body = json!({
            "cred_type": "testAuth",
            "data": {}
        });

        let request = Request::builder()
            .uri("/credentials/test")
            .method("POST")
            .header("content-type", "application/json")
            // Pass authorization if needed? We need claims to bypass auth middleware!
            // Wait, does the router have auth layer? `credential_routes` currently does not wrap the routes in `from_extractor::<Claims>()` as middleware, it uses `_claims: Claims` as an extractor in the handler.
            // If the handler uses `Claims`, then we MUST provide a valid JWT!
            .body(Body::from(req_body.to_string()))
            .unwrap();
        
        // Wait, if we use `Claims`, we must send `Authorization: Bearer <valid_token>`.
        // Let's generate a valid token.
        let token = crate::auth::generate_jwt("00000000-0000-0000-0000-000000000000", "admin").unwrap();
        
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
}
