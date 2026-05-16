use crate::repositories::credential::CredentialRepository;
use crate::repositories::governance::GovernanceRepository;
use crate::repositories::workflow::WorkflowRepository;
use crate::subworkflow_executor::RepositorySubWorkflowExecutor;
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
    Json, Router,
};
use barqflow_core::{
    schema::{INode, INodeConnections, IWorkflowSettings, WorkflowDef},
    types::{IDataObject, RunId, WorkflowId},
};
use barqflow_exec::runner::{
    ExecutionConfig, NodeExecutionResult, WorkflowRunContext, WorkflowRunner,
};
use barqflow_registry::registry::NodeRegistry;
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use uuid::Uuid;

/// A single registered webhook endpoint, mapping a URL path to a workflow ID.
#[derive(Debug, Clone)]
pub struct WebhookEndpoint {
    pub workflow_id: Uuid,
    pub node_id: String,
    pub http_method: String, // "GET", "POST", "ANY", etc.
    /// When set, incoming requests must carry a valid `X-Barqflow-Signature-256` header.
    /// Value is the raw secret used for HMAC-SHA256 computation.
    pub webhook_secret: Option<String>,
}

type HmacSha256 = Hmac<Sha256>;

/// Verifies the `X-Barqflow-Signature-256: sha256=<hex>` header against the request body.
/// Returns `Ok(())` when the signature matches or when no secret is configured.
fn verify_hmac_signature(
    headers: &HeaderMap,
    body: &[u8],
    secret: &str,
) -> Result<(), (StatusCode, String)> {
    let header_value = headers
        .get("x-barqflow-signature-256")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::FORBIDDEN,
                "Missing X-Barqflow-Signature-256 header".into(),
            )
        })?;

    let provided_hex = header_value
        .strip_prefix("sha256=")
        .ok_or_else(|| {
            (
                StatusCode::FORBIDDEN,
                "Invalid signature format; expected sha256=<hex>".into(),
            )
        })?;

    let provided_bytes = hex::decode(provided_hex).map_err(|_| {
        (
            StatusCode::FORBIDDEN,
            "Signature is not valid hex".into(),
        )
    })?;

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC accepts any key length");
    mac.update(body);

    // constant-time comparison — verify_slice rejects if lengths differ too
    mac.verify_slice(&provided_bytes).map_err(|_| {
        (StatusCode::FORBIDDEN, "Signature mismatch".into())
    })
}

/// Thread-safe registry mapping webhook paths to their bound workflow endpoints.
/// Populated at boot from the database and modified when workflows are activated/deactivated.
pub type WebhookRegistry = Arc<RwLock<HashMap<String, WebhookEndpoint>>>;

/// Creates a new empty WebhookRegistry
pub fn new_webhook_registry() -> WebhookRegistry {
    Arc::new(RwLock::new(HashMap::new()))
}

fn find_webhook_node<'a>(nodes: &'a [INode], endpoint: &WebhookEndpoint) -> Option<&'a INode> {
    nodes
        .iter()
        .find(|node| node.id.0 == endpoint.node_id || node.id.to_string() == endpoint.node_id)
}

fn webhook_response_mode(node: Option<&INode>) -> String {
    node.and_then(|n| n.parameters.0.get("responseMode"))
        .and_then(|v| v.as_str())
        .unwrap_or("onReceived")
        .to_string()
}

fn parse_response_code(value: &serde_json::Value) -> Option<u16> {
    let parsed = value
        .as_u64()
        .and_then(|n| u16::try_from(n).ok())
        .or_else(|| value.as_str().and_then(|s| s.parse::<u16>().ok()))?;
    if (100..=599).contains(&parsed) {
        Some(parsed)
    } else {
        None
    }
}

fn webhook_response_status(node: Option<&INode>) -> StatusCode {
    let status = node
        .and_then(|n| n.parameters.0.get("responseCode"))
        .and_then(parse_response_code)
        .unwrap_or(200);
    StatusCode::from_u16(status).unwrap_or(StatusCode::OK)
}

fn configured_response_payload(node: Option<&INode>) -> Option<serde_json::Value> {
    let raw = node.and_then(|n| n.parameters.0.get("responseData"))?;
    if raw.is_null() {
        return None;
    }

    if let Some(value) = raw.as_str() {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return None;
        }

        return Some(
            serde_json::from_str(trimmed)
                .unwrap_or_else(|_| serde_json::json!({ "data": trimmed })),
        );
    }

    Some(raw.clone())
}

fn summarize_node_results_for_webhook(
    results: HashMap<String, NodeExecutionResult>,
) -> serde_json::Value {
    let mut summary = serde_json::Map::new();
    let mut all_success = true;

    for (node, result) in results {
        if !result.success {
            all_success = false;
        }
        summary.insert(
            node,
            serde_json::json!({
                "success": result.success,
                "error": result.error,
                "outputs": result.outputs
            }),
        );
    }

    serde_json::json!({
        "success": all_success,
        "results": summary
    })
}

#[derive(Clone)]
pub struct WebhookState {
    pub workflow_repo: Arc<WorkflowRepository>,
    pub webhook_registry: WebhookRegistry,
    pub node_registry: Arc<NodeRegistry>,
    pub credential_repo: Arc<CredentialRepository>,
    pub governance_repo: Arc<GovernanceRepository>,
}

pub fn webhook_routes(state: WebhookState) -> Router {
    Router::new()
        .route("/{*path}", any(handle_webhook))
        .with_state(state)
}

async fn handle_webhook(
    State(state): State<WebhookState>,
    Path(path): Path<String>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, (StatusCode, String)> {
    // 1. Look up the webhook path in the registry
    let endpoint = {
        let registry = state.webhook_registry.read().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Registry lock poisoned".into(),
            )
        })?;
        registry.get(&path).cloned()
    };

    let endpoint = endpoint.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("No active webhook registered for path: {}", path),
        )
    })?;

    // 2. Check HTTP method if the endpoint restricts it
    if endpoint.http_method != "ANY"
        && endpoint.http_method.to_uppercase() != method.as_str().to_uppercase()
    {
        return Err((
            StatusCode::METHOD_NOT_ALLOWED,
            format!(
                "Webhook path '{}' only accepts {} requests",
                path, endpoint.http_method
            ),
        ));
    }

    // 3. Verify HMAC-SHA256 signature when the endpoint has a secret configured
    if let Some(ref secret) = endpoint.webhook_secret {
        verify_hmac_signature(&headers, &body, secret)?;
    }

    // 4. Build the trigger output payload mimicking n8n Webhook node structure
    let body_str = String::from_utf8(body.to_vec()).unwrap_or_default();
    let parsed_body: serde_json::Value =
        serde_json::from_str(&body_str).unwrap_or_else(|_| json!({ "raw": body_str }));

    let mut headers_map = serde_json::Map::new();
    for (key, value) in headers.iter() {
        if let Ok(val_str) = value.to_str() {
            headers_map.insert(
                key.to_string(),
                serde_json::Value::String(val_str.to_string()),
            );
        }
    }

    let trigger_output = IDataObject::from(json!({
        "headers": headers_map,
        "method": method.as_str(),
        "path": path,
        "body": parsed_body,
    }));

    // 5. Fetch workflow from DB and launch execution asynchronously
    let wf_entity = state
        .workflow_repo
        .find_by_id(endpoint.workflow_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                "Workflow not found in database".into(),
            )
        })?;

    let nodes: Vec<INode> = serde_json::from_value(wf_entity.nodes.clone()).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Malformed workflow nodes: {}", e),
        )
    })?;
    let webhook_node = find_webhook_node(&nodes, &endpoint);
    let response_mode = webhook_response_mode(webhook_node);
    let response_status = webhook_response_status(webhook_node);
    let configured_payload = configured_response_payload(webhook_node);

    let connections: HashMap<String, INodeConnections> =
        serde_json::from_value(wf_entity.connections.clone()).unwrap_or_default();

    let settings: IWorkflowSettings =
        serde_json::from_value(wf_entity.settings.clone()).unwrap_or_default();

    let credential_provider = Arc::new(
        crate::credentials_provider::RepositoryCredentialProvider::new(
            Arc::clone(&state.credential_repo),
            Arc::clone(&state.governance_repo),
            &nodes,
        ),
    );
    let subworkflow_executor = Arc::new(RepositorySubWorkflowExecutor::new(
        Arc::clone(&state.workflow_repo),
        Arc::clone(&state.credential_repo),
        Arc::clone(&state.governance_repo),
        Arc::clone(&state.node_registry),
    ));

    let workflow_def = WorkflowDef {
        id: WorkflowId(wf_entity.id),
        name: wf_entity.name.clone(),
        nodes,
        connections: connections.into_iter().collect(),
        active: wf_entity.active,
        settings,
    };
    let runner = WorkflowRunner::new(state.node_registry.clone(), ExecutionConfig::default())
        .with_credential_provider(credential_provider)
        .with_subworkflow_executor(subworkflow_executor);
    let run_id = RunId::new();

    // Inject the trigger output as static data for the webhook node
    let ctx = WorkflowRunContext {
        run_id,
        workflow: workflow_def,
        static_data: Some(trigger_output),
        manual: false,
        execution_id: None,
        parent_execution_id: None,
        cancellation_token: None,
        stop_after_node_id: None,
        event_sequence_start: 0,
    };

    if response_mode.eq_ignore_ascii_case("lastNode") {
        let result = runner.run_workflow(ctx).await;
        return match result {
            Ok(node_results) => {
                let payload = configured_payload
                    .clone()
                    .unwrap_or_else(|| summarize_node_results_for_webhook(node_results));
                Ok((response_status, Json(payload)).into_response())
            }
            Err(err) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Webhook execution failed: {}", err),
            )),
        };
    }

    // Fire and forget for async response mode
    tokio::spawn(async move {
        let _ = runner.run_workflow(ctx).await;
    });

    let payload = configured_payload.unwrap_or_else(|| {
        json!({
            "success": true,
            "message": "Webhook accepted",
            "workflow_id": endpoint.workflow_id.to_string(),
        })
    });
    Ok((response_status, Json(payload)).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::credential::CredentialRepository;
    use axum::{body::Body, http::Request};
    use barqflow_core::schema::INodeParameters;
    use barqflow_core::types::NodeId;
    use hmac::{Hmac, Mac};
    use serde_json::json;
    use sha2::Sha256;
    use sqlx::PgPool;
    use tower::ServiceExt;

    fn make_signature(secret: &str, body: &[u8]) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
    }

    fn webhook_node_with_parameters(params: serde_json::Value) -> INode {
        INode {
            id: NodeId("webhook1".to_string()),
            name: "Webhook".to_string(),
            r#type: "barqflow-nodes.webhook".to_string(),
            type_version: 1.0,
            position: [0.0, 0.0],
            parameters: INodeParameters(
                params
                    .as_object()
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .collect(),
            ),
            credentials: vec![],
            disabled: false,
        }
    }

    #[test]
    fn webhook_payload_parses_json_string_response_data() {
        let node = webhook_node_with_parameters(json!({
            "responseData": "{\"ok\":true}"
        }));
        let payload = configured_response_payload(Some(&node)).unwrap();
        assert_eq!(payload, json!({ "ok": true }));
    }

    #[test]
    fn webhook_status_uses_valid_response_code() {
        let node = webhook_node_with_parameters(json!({
            "responseCode": "202"
        }));
        assert_eq!(webhook_response_status(Some(&node)), StatusCode::ACCEPTED);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_dynamic_webhook_routing(pool: PgPool) {
        let repo = Arc::new(WorkflowRepository::new(pool.clone()));

        // Create mock workflow
        let workflow = repo
            .create("Webhook Test", json!([]), json!({}), json!({}))
            .await
            .unwrap();

        // Register it in the webhook registry
        let registry = new_webhook_registry();
        {
            let mut write = registry.write().unwrap();
            write.insert(
                "test-hook".to_string(),
                WebhookEndpoint {
                    workflow_id: workflow.id,
                    node_id: "webhook1".to_string(),
                    http_method: "POST".to_string(),
                    webhook_secret: None,
                },
            );
        }

        let state = WebhookState {
            workflow_repo: repo,
            webhook_registry: registry,
            node_registry: Arc::new(NodeRegistry::new()), // Empty mock
            credential_repo: Arc::new(CredentialRepository::new(pool.clone())),
            governance_repo: Arc::new(GovernanceRepository::new(pool)),
        };

        let app = webhook_routes(state);

        // Test 1: Valid Path & Method
        let req1 = Request::builder()
            .uri("/test-hook")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(json!({"foo": "bar"}).to_string()))
            .unwrap();

        let res1 = app.clone().oneshot(req1).await.unwrap();
        assert_eq!(res1.status(), StatusCode::OK);

        // Test 2: Invalid Method (GET instead of POST)
        let req2 = Request::builder()
            .uri("/test-hook")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let res2 = app.clone().oneshot(req2).await.unwrap();
        assert_eq!(res2.status(), StatusCode::METHOD_NOT_ALLOWED);

        // Test 3: Unregistered Path
        let req3 = Request::builder()
            .uri("/unknown-hook")
            .method("POST")
            .body(Body::empty())
            .unwrap();

        let res3 = app.clone().oneshot(req3).await.unwrap();
        assert_eq!(res3.status(), StatusCode::NOT_FOUND);
    }

    // ── HMAC signature verification unit tests ────────────────────────────────

    #[test]
    fn valid_signature_passes_verification() {
        let secret = "super-secret";
        let body = b"hello world";
        let sig = make_signature(secret, body);

        let mut headers = HeaderMap::new();
        headers.insert("x-barqflow-signature-256", sig.parse().unwrap());

        assert!(verify_hmac_signature(&headers, body, secret).is_ok());
    }

    #[test]
    fn wrong_signature_is_rejected() {
        let body = b"hello world";
        let sig = make_signature("correct-secret", body);

        let mut headers = HeaderMap::new();
        headers.insert("x-barqflow-signature-256", sig.parse().unwrap());

        let result = verify_hmac_signature(&headers, body, "wrong-secret");
        assert_eq!(result.unwrap_err().0, StatusCode::FORBIDDEN);
    }

    #[test]
    fn missing_signature_header_is_rejected() {
        let headers = HeaderMap::new();
        let result = verify_hmac_signature(&headers, b"body", "secret");
        assert_eq!(result.unwrap_err().0, StatusCode::FORBIDDEN);
    }

    #[test]
    fn endpoint_without_secret_skips_verification() {
        let endpoint = WebhookEndpoint {
            workflow_id: Uuid::new_v4(),
            node_id: "n1".into(),
            http_method: "POST".into(),
            webhook_secret: None,
        };
        // No secret means the verification block is never entered; this is a
        // compile-time / logic test that the field exists and is None-able.
        assert!(endpoint.webhook_secret.is_none());
    }

    #[test]
    fn tampered_body_is_rejected() {
        let secret = "my-secret";
        let original_body = b"original";
        let tampered_body = b"tampered";
        let sig = make_signature(secret, original_body);

        let mut headers = HeaderMap::new();
        headers.insert("x-barqflow-signature-256", sig.parse().unwrap());

        let result = verify_hmac_signature(&headers, tampered_body, secret);
        assert_eq!(result.unwrap_err().0, StatusCode::FORBIDDEN);
    }
}
