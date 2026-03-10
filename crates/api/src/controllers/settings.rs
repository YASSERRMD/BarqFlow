use crate::auth::Claims;
use crate::contracts::RuntimeSettingsResponse;
use axum::{extract::State, routing::get, Json, Router};
use chrono::Utc;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub node_registry: Arc<barqflow_registry::registry::NodeRegistry>,
    pub credential_registry: Arc<barqflow_registry::registry::CredentialRegistry>,
}

pub fn settings_routes(state: AppState) -> Router {
    Router::new()
        .route("/settings/runtime", get(get_runtime_settings))
        .with_state(state)
}

async fn get_runtime_settings(
    _claims: Claims,
    State(state): State<AppState>,
) -> Json<RuntimeSettingsResponse> {
    let encryption_key_configured = std::env::var("BARQFLOW_ENCRYPTION_KEY")
        .map(|key| key.len() >= 32)
        .unwrap_or(false);

    Json(RuntimeSettingsResponse {
        server_time: Utc::now(),
        environment: std::env::var("BARQFLOW_ENV").unwrap_or_else(|_| "development".to_string()),
        node_types_count: state.node_registry.get_all_node_names().len(),
        credential_types_count: state.credential_registry.get_all_credentials().len(),
        encryption_key_configured,
    })
}
