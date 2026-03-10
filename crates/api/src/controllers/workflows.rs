use crate::active_workflows::{ActiveCronJobs, ActiveWorkflowManager};
use crate::auth::{require_authenticated_user, require_workspace_role, AuthenticatedUser};
use crate::contracts::{
    TagResponse, WorkflowExportResponse, WorkflowHistoryDiffResponse, WorkflowHistoryEntryResponse,
    WorkflowNodeChangeResponse, WorkflowResponse, WorkflowTemplateResponse,
};
use crate::controllers::executions::AppState as ExecutionControllerState;
use crate::controllers::webhooks::WebhookRegistry;
use crate::governance::{enforce_workflow_policy, record_governance_event};
use crate::repositories::workflow::{
    SortDirection, WorkflowListFilters, WorkflowRepository, WorkflowSortBy, WorkflowUpsert,
};
use crate::repositories::{
    api_key::ApiKeyRepository, credential::CredentialRepository, governance::GovernanceRepository,
    workspace::WorkspaceRepository,
};
use crate::workflow_templates::{
    find_workflow_template, list_workflow_templates, WorkflowTemplateDefinition,
};
use axum::http::{HeaderMap, StatusCode};
use axum::{
    extract::{Json, Path, Query, State},
    routing::{delete, get, post, put},
    Router,
};
use barqflow_db::users::UserRepo;
use barqflow_registry::registry::NodeRegistry;
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use tokio_cron_scheduler::JobScheduler;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub workflow_repo: Arc<WorkflowRepository>,
    pub credential_repo: Arc<CredentialRepository>,
    pub governance_repo: Arc<GovernanceRepository>,
    pub user_repo: Arc<UserRepo>,
    pub workspace_repo: Arc<WorkspaceRepository>,
    pub api_key_repo: Arc<ApiKeyRepository>,
    pub node_registry: Arc<NodeRegistry>,
    pub execution_controller_state: ExecutionControllerState,
    pub webhook_registry: WebhookRegistry,
    pub job_scheduler: JobScheduler,
    pub active_cron_jobs: ActiveCronJobs,
}

pub fn workflow_routes(state: AppState) -> Router {
    Router::new()
        .route("/workflows", get(get_workflows).post(create_workflow))
        .route("/workflows/import", post(import_workflow))
        .route("/workflows/{id}/export", get(export_workflow))
        .route("/workflows/{id}/history", get(get_workflow_history))
        .route(
            "/workflows/{id}/history/diff",
            get(get_workflow_history_diff),
        )
        .route(
            "/workflows/{id}",
            get(get_workflow)
                .put(update_workflow)
                .delete(delete_workflow),
        )
        .route("/workflows/{id}/activate", put(toggle_workflow_active))
        .route("/workflows/{id}/duplicate", post(duplicate_workflow))
        .route("/workflow-templates", get(get_workflow_templates))
        .route(
            "/workflow-templates/{id}/instantiate",
            post(instantiate_workflow_template),
        )
        .route("/tags", get(get_tags).post(create_tag))
        .route("/tags/{id}", delete(delete_tag))
        .with_state(state)
}

#[derive(Deserialize)]
pub struct WorkflowUpsertRequest {
    pub name: String,
    pub nodes: Value,
    pub connections: Value,
    pub settings: Value,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
pub struct ToggleActiveRequest {
    pub active: bool,
}

#[derive(Deserialize)]
pub struct WorkflowListQuery {
    pub active: Option<bool>,
    pub search: Option<String>,
    pub tags: Option<String>,
    pub limit: Option<usize>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
}

#[derive(Deserialize)]
pub struct ImportWorkflowDocument {
    pub name: String,
    pub nodes: Value,
    pub connections: Value,
    pub settings: Value,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Deserialize)]
pub struct ImportWorkflowRequest {
    pub workflow: ImportWorkflowDocument,
    #[serde(default)]
    pub name_override: Option<String>,
}

#[derive(Deserialize)]
pub struct WorkflowHistoryDiffQuery {
    pub from_version: i32,
    pub to_version: i32,
}

#[derive(Deserialize)]
pub struct WorkflowTemplateInstantiateRequest {
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
}

async fn get_workflows(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(query): Query<WorkflowListQuery>,
) -> Result<Json<Vec<WorkflowResponse>>, (StatusCode, String)> {
    let auth = require_workflow_auth(&headers, &state).await?;
    let workflows = state
        .workflow_repo
        .find_documents_for_workspace(
            auth.workspace_id,
            WorkflowListFilters {
                active: query.active,
                search: query.search,
                tags: parse_tags_csv(query.tags.as_deref()),
                limit: query.limit,
                sort_by: parse_sort_by(query.sort_by.as_deref()),
                sort_direction: parse_sort_direction(query.sort_direction.as_deref()),
            },
        )
        .await
        .map_err(internal_error)?;

    Ok(Json(
        workflows.into_iter().map(WorkflowResponse::from).collect(),
    ))
}

async fn get_workflow(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkflowResponse>, (StatusCode, String)> {
    let auth = require_workflow_auth(&headers, &state).await?;
    let workflow = state
        .workflow_repo
        .find_document_by_id_in_workspace(auth.workspace_id, id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| not_found("Workflow not found"))?;

    Ok(Json(WorkflowResponse::from(workflow)))
}

async fn create_workflow(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<WorkflowUpsertRequest>,
) -> Result<Json<WorkflowResponse>, (StatusCode, String)> {
    let auth = require_workflow_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;
    let upsert = to_workflow_upsert(payload)?;
    enforce_policy_for_nodes(&state, &auth, &upsert.nodes).await?;
    let workflow = state
        .workflow_repo
        .create_document_in_workspace(auth.workspace_id, Some(auth.id), upsert, "create")
        .await
        .map_err(internal_error)?;

    Ok(Json(WorkflowResponse::from(workflow)))
}

async fn update_workflow(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<WorkflowUpsertRequest>,
) -> Result<Json<WorkflowResponse>, (StatusCode, String)> {
    let auth = require_workflow_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;
    let upsert = to_workflow_upsert(payload)?;
    enforce_policy_for_nodes(&state, &auth, &upsert.nodes).await?;
    let updated = state
        .workflow_repo
        .update_document_in_workspace(auth.workspace_id, id, upsert, "update")
        .await
        .map_err(internal_error)?
        .ok_or_else(|| not_found("Workflow not found"))?;

    Ok(Json(WorkflowResponse::from(updated)))
}

async fn toggle_workflow_active(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ToggleActiveRequest>,
) -> Result<Json<WorkflowResponse>, (StatusCode, String)> {
    let auth = require_workflow_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;
    let manager = ActiveWorkflowManager::new(
        Arc::clone(&state.workflow_repo),
        state.execution_controller_state.clone(),
        Arc::clone(&state.webhook_registry),
        state.job_scheduler.clone(),
        Arc::clone(&state.active_cron_jobs),
    );

    if payload.active {
        manager
            .activate(id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    } else {
        manager
            .deactivate(id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    state
        .workflow_repo
        .find_by_id_in_workspace(auth.workspace_id, id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| not_found("Workflow not found"))?;

    state
        .workflow_repo
        .record_snapshot_by_workflow_id(
            id,
            if payload.active {
                "activate"
            } else {
                "deactivate"
            },
        )
        .await
        .map_err(internal_error)?
        .ok_or_else(|| not_found("Workflow not found"))?;

    let workflow = state
        .workflow_repo
        .find_document_by_id_in_workspace(auth.workspace_id, id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| not_found("Workflow not found"))?;

    Ok(Json(WorkflowResponse::from(workflow)))
}

async fn delete_workflow(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let auth = require_workflow_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;
    let deleted = state
        .workflow_repo
        .delete_in_workspace(auth.workspace_id, id)
        .await
        .map_err(internal_error)?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(not_found("Workflow not found"))
    }
}

async fn duplicate_workflow(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkflowResponse>, (StatusCode, String)> {
    let auth = require_workflow_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;
    let workflow = state
        .workflow_repo
        .find_document_by_id_in_workspace(auth.workspace_id, id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| not_found("Workflow not found"))?;

    enforce_policy_for_nodes(&state, &auth, &workflow.workflow.nodes).await?;

    let duplicated = state
        .workflow_repo
        .create_document_in_workspace(
            auth.workspace_id,
            Some(auth.id),
            WorkflowUpsert {
                name: format!("{} (copy)", workflow.workflow.name),
                nodes: workflow.workflow.nodes.clone(),
                connections: workflow.workflow.connections.clone(),
                settings: workflow.workflow.settings.clone(),
                tags: workflow.tags.into_iter().map(|tag| tag.name).collect(),
            },
            "duplicate",
        )
        .await
        .map_err(internal_error)?;

    Ok(Json(WorkflowResponse::from(duplicated)))
}

async fn import_workflow(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<ImportWorkflowRequest>,
) -> Result<Json<WorkflowResponse>, (StatusCode, String)> {
    let auth = require_workflow_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;
    let name = payload
        .name_override
        .unwrap_or(payload.workflow.name)
        .trim()
        .to_string();

    if name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Workflow name is required".into()));
    }

    enforce_policy_for_nodes(&state, &auth, &payload.workflow.nodes).await?;

    let imported = state
        .workflow_repo
        .create_document_in_workspace(
            auth.workspace_id,
            Some(auth.id),
            WorkflowUpsert {
                name,
                nodes: payload.workflow.nodes,
                connections: payload.workflow.connections,
                settings: payload.workflow.settings,
                tags: payload.workflow.tags,
            },
            "import",
        )
        .await
        .map_err(internal_error)?;

    Ok(Json(WorkflowResponse::from(imported)))
}

async fn export_workflow(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkflowExportResponse>, (StatusCode, String)> {
    let auth = require_workflow_auth(&headers, &state).await?;
    let workflow = state
        .workflow_repo
        .find_document_by_id_in_workspace(auth.workspace_id, id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| not_found("Workflow not found"))?;

    Ok(Json(WorkflowExportResponse {
        format: "barqflow.workflow".to_string(),
        exported_at: Utc::now(),
        workflow: WorkflowResponse::from(workflow),
    }))
}

async fn get_workflow_history(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<WorkflowHistoryEntryResponse>>, (StatusCode, String)> {
    let auth = require_workflow_auth(&headers, &state).await?;
    state
        .workflow_repo
        .find_by_id_in_workspace(auth.workspace_id, id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| not_found("Workflow not found"))?;

    let history = state
        .workflow_repo
        .list_history(id)
        .await
        .map_err(internal_error)?;

    Ok(Json(
        history
            .into_iter()
            .map(WorkflowHistoryEntryResponse::from)
            .collect(),
    ))
}

async fn get_workflow_history_diff(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<WorkflowHistoryDiffQuery>,
) -> Result<Json<WorkflowHistoryDiffResponse>, (StatusCode, String)> {
    let auth = require_workflow_auth(&headers, &state).await?;
    if query.from_version == query.to_version {
        return Err((
            StatusCode::BAD_REQUEST,
            "Choose two different workflow versions to diff".into(),
        ));
    }

    state
        .workflow_repo
        .find_by_id_in_workspace(auth.workspace_id, id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| not_found("Workflow not found"))?;

    let from_entry = state
        .workflow_repo
        .find_history_entry(id, query.from_version)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| not_found("Source workflow history version not found"))?;

    let to_entry = state
        .workflow_repo
        .find_history_entry(id, query.to_version)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| not_found("Target workflow history version not found"))?;

    Ok(Json(build_history_diff(id, &from_entry, &to_entry)))
}

async fn get_workflow_templates(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Vec<WorkflowTemplateResponse>>, (StatusCode, String)> {
    let _auth = require_authenticated_user(
        &headers,
        Arc::clone(&state.user_repo),
        Arc::clone(&state.workspace_repo),
        Arc::clone(&state.api_key_repo),
    )
    .await?;
    Ok(Json(
        list_workflow_templates()
            .into_iter()
            .map(to_workflow_template_response)
            .collect(),
    ))
}

async fn instantiate_workflow_template(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<WorkflowTemplateInstantiateRequest>,
) -> Result<Json<WorkflowResponse>, (StatusCode, String)> {
    let auth = require_workflow_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;
    let template =
        find_workflow_template(&id).ok_or_else(|| not_found("Workflow template not found"))?;
    let tag_names = template.tag_names();
    let workflow_name = payload
        .name
        .unwrap_or_else(|| template.name.to_string())
        .trim()
        .to_string();
    if workflow_name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Workflow name is required".into()));
    }
    enforce_policy_for_nodes(&state, &auth, &template.nodes).await?;
    let workflow = state
        .workflow_repo
        .create_document_in_workspace(
            auth.workspace_id,
            Some(auth.id),
            WorkflowUpsert {
                name: workflow_name,
                nodes: template.nodes,
                connections: template.connections,
                settings: template.settings,
                tags: tag_names,
            },
            "template",
        )
        .await
        .map_err(internal_error)?;

    Ok(Json(WorkflowResponse::from(workflow)))
}

async fn get_tags(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Vec<TagResponse>>, (StatusCode, String)> {
    let auth = require_workflow_auth(&headers, &state).await?;
    let tags = state
        .workflow_repo
        .list_tags_in_workspace(auth.workspace_id)
        .await
        .map_err(internal_error)?;
    Ok(Json(tags.into_iter().map(TagResponse::from).collect()))
}

async fn create_tag(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<CreateTagRequest>,
) -> Result<Json<TagResponse>, (StatusCode, String)> {
    let auth = require_workflow_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;
    let name = payload.name.trim();
    if name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Tag name is required".into()));
    }

    let tag = state
        .workflow_repo
        .create_tag_in_workspace(auth.workspace_id, name)
        .await
        .map_err(internal_error)?;

    Ok(Json(TagResponse::from(tag)))
}

async fn delete_tag(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let auth = require_workflow_auth(&headers, &state).await?;
    require_workspace_role(&auth, "member")?;
    let deleted = state
        .workflow_repo
        .delete_tag_in_workspace(auth.workspace_id, id)
        .await
        .map_err(internal_error)?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(not_found("Tag not found"))
    }
}

fn to_workflow_upsert(
    payload: WorkflowUpsertRequest,
) -> Result<WorkflowUpsert, (StatusCode, String)> {
    let name = payload.name.trim().to_string();
    if name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Workflow name is required".into()));
    }

    if !payload.nodes.is_array() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Workflow nodes must be a JSON array".into(),
        ));
    }

    Ok(WorkflowUpsert {
        name,
        nodes: payload.nodes,
        connections: payload.connections,
        settings: payload.settings,
        tags: payload.tags,
    })
}

fn parse_tags_csv(raw: Option<&str>) -> Vec<String> {
    raw.unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(ToString::to_string)
        .collect()
}

async fn enforce_policy_for_nodes(
    state: &AppState,
    auth: &AuthenticatedUser,
    nodes: &Value,
) -> Result<(), (StatusCode, String)> {
    if let Err(message) =
        enforce_workflow_policy(&state.governance_repo, auth.workspace_id, nodes).await
    {
        let _ = record_governance_event(
            &state.governance_repo,
            auth,
            "governance.policy.workflowDenied",
            "workflow",
            None,
            "Blocked workflow write because it violates workspace policy.",
            serde_json::json!({ "message": message.clone() }),
        )
        .await;

        return Err((StatusCode::FORBIDDEN, message));
    }

    Ok(())
}

fn parse_sort_by(raw: Option<&str>) -> WorkflowSortBy {
    match raw.unwrap_or_default().trim().to_lowercase().as_str() {
        "name" => WorkflowSortBy::Name,
        "created" | "createdat" | "created_at" => WorkflowSortBy::CreatedAt,
        _ => WorkflowSortBy::UpdatedAt,
    }
}

fn parse_sort_direction(raw: Option<&str>) -> SortDirection {
    match raw.unwrap_or_default().trim().to_lowercase().as_str() {
        "asc" => SortDirection::Asc,
        _ => SortDirection::Desc,
    }
}

async fn require_workflow_auth(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<AuthenticatedUser, (StatusCode, String)> {
    require_authenticated_user(
        headers,
        Arc::clone(&state.user_repo),
        Arc::clone(&state.workspace_repo),
        Arc::clone(&state.api_key_repo),
    )
    .await
}

fn to_workflow_template_response(template: WorkflowTemplateDefinition) -> WorkflowTemplateResponse {
    WorkflowTemplateResponse {
        id: template.id.to_string(),
        name: template.name.to_string(),
        description: template.description.to_string(),
        category: template.category.to_string(),
        difficulty: template.difficulty.to_string(),
        tags: template.tag_names(),
        highlights: template.highlight_list(),
        summary: summarize_template(&template).into(),
        nodes: template.nodes,
        connections: template.connections,
        settings: template.settings,
    }
}

fn summarize_template(
    template: &WorkflowTemplateDefinition,
) -> crate::repositories::workflow::WorkflowSummaryEntity {
    let nodes = template.nodes.as_array().cloned().unwrap_or_default();
    let mut trigger_count = 0;
    let mut credential_binding_count = 0;

    for node in &nodes {
        let node_type = node.get("type").and_then(Value::as_str).unwrap_or_default();
        if is_trigger_node_type(node_type) {
            trigger_count += 1;
        }
        credential_binding_count += node
            .get("credentials")
            .and_then(Value::as_array)
            .map(|credentials| credentials.len())
            .unwrap_or_default();
    }

    crate::repositories::workflow::WorkflowSummaryEntity {
        node_count: nodes.len(),
        trigger_count,
        credential_binding_count,
        tag_count: template.tags.len(),
        latest_version: 0,
    }
}

fn build_history_diff(
    workflow_id: Uuid,
    from_entry: &crate::repositories::workflow::WorkflowHistoryEntryEntity,
    to_entry: &crate::repositories::workflow::WorkflowHistoryEntryEntity,
) -> WorkflowHistoryDiffResponse {
    let from_tags = extract_string_set(&from_entry.snapshot.tags);
    let to_tags = extract_string_set(&to_entry.snapshot.tags);

    let tags_added = to_tags.difference(&from_tags).cloned().collect();
    let tags_removed = from_tags.difference(&to_tags).cloned().collect();

    let from_nodes = node_map(&from_entry.snapshot.nodes);
    let to_nodes = node_map(&to_entry.snapshot.nodes);

    let mut nodes_added = Vec::new();
    let mut nodes_removed = Vec::new();
    let mut nodes_changed = Vec::new();

    for (node_id, node) in &to_nodes {
        if !from_nodes.contains_key(node_id) {
            nodes_added.push(node_name(node));
        }
    }

    for (node_id, node) in &from_nodes {
        if !to_nodes.contains_key(node_id) {
            nodes_removed.push(node_name(node));
        }
    }

    for (node_id, from_node) in &from_nodes {
        let Some(to_node) = to_nodes.get(node_id) else {
            continue;
        };

        let changed_fields = diff_node_fields(from_node, to_node);
        if !changed_fields.is_empty() {
            nodes_changed.push(WorkflowNodeChangeResponse {
                node_id: node_id.clone(),
                node_name: node_name(to_node),
                changed_fields,
            });
        }
    }

    nodes_added.sort();
    nodes_removed.sort();
    nodes_changed.sort_by(|left, right| left.node_name.cmp(&right.node_name));

    let from_connections = flatten_connections(&from_entry.snapshot.connections);
    let to_connections = flatten_connections(&to_entry.snapshot.connections);

    let connections_added = to_connections
        .difference(&from_connections)
        .cloned()
        .collect();
    let connections_removed = from_connections
        .difference(&to_connections)
        .cloned()
        .collect();

    WorkflowHistoryDiffResponse {
        workflow_id,
        from_version: from_entry.snapshot.version,
        to_version: to_entry.snapshot.version,
        from_name: from_entry.snapshot.name.clone(),
        to_name: to_entry.snapshot.name.clone(),
        name_changed: from_entry.snapshot.name != to_entry.snapshot.name,
        active_changed: from_entry.snapshot.active != to_entry.snapshot.active,
        tags_added,
        tags_removed,
        settings_changed: diff_settings(&from_entry.snapshot.settings, &to_entry.snapshot.settings),
        nodes_added,
        nodes_removed,
        nodes_changed,
        connections_added,
        connections_removed,
    }
}

fn extract_string_set(value: &Value) -> BTreeSet<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.as_str())
        .map(ToString::to_string)
        .collect()
}

fn node_map(nodes: &Value) -> BTreeMap<String, Value> {
    let mut mapped = BTreeMap::new();
    for node in nodes.as_array().into_iter().flatten() {
        let key = node
            .get("id")
            .and_then(Value::as_str)
            .or_else(|| node.get("name").and_then(Value::as_str))
            .unwrap_or_default()
            .to_string();
        if !key.is_empty() {
            mapped.insert(key, node.clone());
        }
    }
    mapped
}

fn node_name(node: &Value) -> String {
    node.get("name")
        .and_then(Value::as_str)
        .or_else(|| node.get("id").and_then(Value::as_str))
        .unwrap_or("Unknown Node")
        .to_string()
}

fn diff_node_fields(from_node: &Value, to_node: &Value) -> Vec<String> {
    let mut changed_fields = Vec::new();
    for (field, label) in [
        ("name", "name"),
        ("type", "type"),
        ("typeVersion", "typeVersion"),
        ("position", "position"),
        ("parameters", "parameters"),
        ("credentials", "credentials"),
        ("disabled", "disabled"),
    ] {
        let from_value = from_node.get(field).cloned().unwrap_or(Value::Null);
        let to_value = to_node.get(field).cloned().unwrap_or(Value::Null);
        if from_value != to_value {
            changed_fields.push(label.to_string());
        }
    }
    changed_fields
}

fn flatten_connections(connections: &Value) -> BTreeSet<String> {
    let mut flattened = BTreeSet::new();
    let Some(raw_connections) = connections.as_object() else {
        return flattened;
    };

    for (source_name, connection_group) in raw_connections {
        let groups = connection_group
            .get("main")
            .or_else(|| connection_group.get("Main"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        for targets in groups {
            for target in targets.as_array().into_iter().flatten() {
                let target_name = target
                    .get("node")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                let index = target
                    .get("index")
                    .and_then(Value::as_i64)
                    .unwrap_or_default();
                let connection_type = target.get("type").and_then(Value::as_str).unwrap_or("main");
                flattened.insert(format!(
                    "{source_name} -> {target_name} [{connection_type}:{index}]"
                ));
            }
        }
    }

    flattened
}

fn diff_settings(from_settings: &Value, to_settings: &Value) -> Vec<String> {
    let mut keys = BTreeSet::new();
    if let Some(object) = from_settings.as_object() {
        keys.extend(object.keys().cloned());
    }
    if let Some(object) = to_settings.as_object() {
        keys.extend(object.keys().cloned());
    }

    let mut changed = Vec::new();
    for key in keys {
        let from_value = from_settings.get(&key).cloned().unwrap_or(Value::Null);
        let to_value = to_settings.get(&key).cloned().unwrap_or(Value::Null);
        if from_value != to_value {
            changed.push(key);
        }
    }

    changed
}

fn is_trigger_node_type(node_type: &str) -> bool {
    matches!(
        node_type,
        "n8n-nodes-base.manualTrigger"
            | "barqflow-nodes.errorTrigger"
            | "barqflow-nodes.webhook"
            | "barqflow-nodes.cronTrigger"
    )
}

fn internal_error(error: impl ToString) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

fn not_found(message: &str) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::workflow::{WorkflowHistoryEntryEntity, WorkflowSummaryEntity};
    use barqflow_db::models::WorkflowHistorySnapshotEntity;
    use chrono::TimeZone;
    use serde_json::json;

    #[test]
    fn history_diff_reports_tags_nodes_and_settings_changes() {
        let from_entry = WorkflowHistoryEntryEntity {
            snapshot: WorkflowHistorySnapshotEntity {
                id: Uuid::parse_str("14390b11-9b68-4a6b-a16d-6278bd781f8e").unwrap(),
                workflow_id: Uuid::parse_str("ba00a445-bd4f-4efa-ae31-d3d59638f508").unwrap(),
                version: 1,
                source: "create".into(),
                name: "Incident Flow".into(),
                active: false,
                tags: json!(["ops"]),
                nodes: json!([
                    {"id": "node-1", "name": "Manual Trigger", "type": "n8n-nodes-base.manualTrigger", "parameters": {}, "credentials": [], "disabled": false}
                ]),
                connections: json!({}),
                settings: json!({"saveExecutionProgress": true}),
                created_at: Utc.with_ymd_and_hms(2026, 3, 10, 10, 0, 0).unwrap(),
            },
            summary: WorkflowSummaryEntity {
                node_count: 1,
                trigger_count: 1,
                credential_binding_count: 0,
                tag_count: 1,
                latest_version: 1,
            },
        };

        let to_entry = WorkflowHistoryEntryEntity {
            snapshot: WorkflowHistorySnapshotEntity {
                id: Uuid::parse_str("2866db2a-3f05-435c-b160-f6b1e2436eb0").unwrap(),
                workflow_id: from_entry.snapshot.workflow_id,
                version: 2,
                source: "update".into(),
                name: "Incident Flow v2".into(),
                active: true,
                tags: json!(["ops", "critical"]),
                nodes: json!([
                    {"id": "node-1", "name": "Manual Trigger", "type": "n8n-nodes-base.manualTrigger", "parameters": {"changed": true}, "credentials": [], "disabled": false},
                    {"id": "node-2", "name": "Notify Slack", "type": "barqflow-nodes.slack", "parameters": {}, "credentials": [], "disabled": false}
                ]),
                connections: json!({
                    "Manual Trigger": {
                        "main": [[{"node": "Notify Slack", "type": "main", "index": 0}]]
                    }
                }),
                settings: json!({"saveExecutionProgress": false, "timezone": "Asia/Dubai"}),
                created_at: Utc.with_ymd_and_hms(2026, 3, 10, 10, 10, 0).unwrap(),
            },
            summary: WorkflowSummaryEntity {
                node_count: 2,
                trigger_count: 1,
                credential_binding_count: 0,
                tag_count: 2,
                latest_version: 2,
            },
        };

        let diff = build_history_diff(from_entry.snapshot.workflow_id, &from_entry, &to_entry);
        assert!(diff.name_changed);
        assert!(diff.active_changed);
        assert_eq!(diff.tags_added, vec!["critical".to_string()]);
        assert_eq!(diff.nodes_added, vec!["Notify Slack".to_string()]);
        assert!(diff.settings_changed.contains(&"timezone".to_string()));
        assert_eq!(diff.nodes_changed.len(), 1);
        assert!(diff
            .connections_added
            .iter()
            .any(|entry| entry.contains("Notify Slack")));
    }

    #[test]
    fn parse_tags_csv_normalizes_empty_entries() {
        assert_eq!(
            parse_tags_csv(Some("ops, , critical,starter")),
            vec!["ops", "critical", "starter"]
        );
        assert!(parse_tags_csv(None).is_empty());
    }
}
