use crate::auth::Claims;
use axum::http::StatusCode;
use axum::{
    extract::{Json, State},
    routing::get,
    Router,
};
use barqflow_db::credentials::CredentialRepo;
use barqflow_db::models::CredentialEntity;
use serde::Deserialize;

#[derive(Clone)]
pub struct AppState {
    pub credential_repo: std::sync::Arc<CredentialRepo>,
}

pub fn credential_routes(state: AppState) -> Router {
    Router::new()
        .route("/credentials", get(get_credentials).post(create_credential))
        .with_state(state)
}

#[derive(Deserialize)]
pub struct CreateCredentialRequest {
    pub name: String,
    pub cred_type: String,
    pub data: serde_json::Value,
}

async fn get_credentials(
    _claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Vec<CredentialEntity>>, (StatusCode, String)> {
    let creds = state
        .credential_repo
        .get_all()
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
