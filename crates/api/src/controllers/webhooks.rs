use axum::{
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::IntoResponse,
    routing::any,
    Router,
    body::Bytes,
};
use barqflow_db::workflows::WorkflowRepo;
use serde_json::json;

#[derive(Clone)]
pub struct WebhookState {
    pub workflow_repo: std::sync::Arc<WorkflowRepo>,
}

pub fn webhook_routes(state: WebhookState) -> Router {
    Router::new()
        .route("/{*path}", any(handle_webhook))
        .with_state(state)
}

async fn handle_webhook(
    State(_state): State<WebhookState>,
    Path(path): Path<String>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 1. Convert everything to JSON
    let body_str = String::from_utf8(body.to_vec()).unwrap_or_default();
    
    // Attempt parsing as JSON for convenience
    let parsed_body: serde_json::Value = serde_json::from_str(&body_str)
        .unwrap_or_else(|_| json!({ "raw": body_str }));

    let mut headers_map = serde_json::Map::new();
    for (key, value) in headers.iter() {
        if let Ok(val_str) = value.to_str() {
            headers_map.insert(key.to_string(), serde_json::Value::String(val_str.to_string()));
        }
    }

    let payload = json!({
        "headers": headers_map,
        "method": method.as_str(),
        "path": path,
        "body": parsed_body
    });

    // TODO: In Phase 48/49 Global State, we will take `payload` and push it into 
    // the ExecutionEngine via an active workflow mapping. For now, just print!
    
    println!("Webhook Triggered! {:?}", payload);

    Ok((StatusCode::OK, "Webhook Received"))
}
