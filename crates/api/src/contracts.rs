use barqflow_core::properties::INodeProperty;
use barqflow_core::schema::CredentialReference;
use barqflow_db::models::{
    ApiKeyEntity, CredentialEntity, ExecutionEntity, ExecutionLogEntity, TagEntity, WorkflowEntity,
};
use barqflow_registry::registry::NodeInfo;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::repositories::workflow::{
    TagRecordEntity, WorkflowDocumentEntity, WorkflowHistoryEntryEntity, WorkflowSummaryEntity,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowTagResponse {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TagEntity> for WorkflowTagResponse {
    fn from(value: TagEntity) -> Self {
        Self {
            id: value.id,
            name: value.name,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagResponse {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub workflow_count: i64,
}

impl From<TagRecordEntity> for TagResponse {
    fn from(value: TagRecordEntity) -> Self {
        Self {
            id: value.tag.id,
            name: value.tag.name,
            created_at: value.tag.created_at,
            updated_at: value.tag.updated_at,
            workflow_count: value.workflow_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowSummaryResponse {
    pub node_count: usize,
    pub trigger_count: usize,
    pub credential_binding_count: usize,
    pub tag_count: usize,
    pub latest_version: i32,
}

impl From<WorkflowSummaryEntity> for WorkflowSummaryResponse {
    fn from(value: WorkflowSummaryEntity) -> Self {
        Self {
            node_count: value.node_count,
            trigger_count: value.trigger_count,
            credential_binding_count: value.credential_binding_count,
            tag_count: value.tag_count,
            latest_version: value.latest_version,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowResponse {
    pub id: Uuid,
    pub name: String,
    pub active: bool,
    pub tags: Vec<WorkflowTagResponse>,
    pub summary: WorkflowSummaryResponse,
    pub nodes: Value,
    pub connections: Value,
    pub settings: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<WorkflowDocumentEntity> for WorkflowResponse {
    fn from(value: WorkflowDocumentEntity) -> Self {
        let summary = value.summary();
        Self {
            id: value.workflow.id,
            name: value.workflow.name,
            active: value.workflow.active,
            tags: value
                .tags
                .into_iter()
                .map(WorkflowTagResponse::from)
                .collect(),
            summary: summary.into(),
            nodes: value.workflow.nodes,
            connections: value.workflow.connections,
            settings: value.workflow.settings,
            created_at: value.workflow.created_at,
            updated_at: value.workflow.updated_at,
        }
    }
}

impl From<WorkflowEntity> for WorkflowResponse {
    fn from(value: WorkflowEntity) -> Self {
        Self {
            id: value.id,
            name: value.name,
            active: value.active,
            tags: Vec::new(),
            summary: WorkflowSummaryResponse {
                node_count: value.nodes.as_array().map(Vec::len).unwrap_or_default(),
                trigger_count: 0,
                credential_binding_count: 0,
                tag_count: 0,
                latest_version: 0,
            },
            nodes: value.nodes,
            connections: value.connections,
            settings: value.settings,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowHistoryEntryResponse {
    pub version: i32,
    pub source: String,
    pub name: String,
    pub active: bool,
    pub tags: Vec<String>,
    pub summary: WorkflowSummaryResponse,
    pub created_at: DateTime<Utc>,
}

impl From<WorkflowHistoryEntryEntity> for WorkflowHistoryEntryResponse {
    fn from(value: WorkflowHistoryEntryEntity) -> Self {
        Self {
            version: value.snapshot.version,
            source: value.snapshot.source,
            name: value.snapshot.name,
            active: value.snapshot.active,
            tags: value
                .snapshot
                .tags
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|entry| entry.as_str())
                .map(ToString::to_string)
                .collect(),
            summary: value.summary.into(),
            created_at: value.snapshot.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowNodeChangeResponse {
    pub node_id: String,
    pub node_name: String,
    pub changed_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowHistoryDiffResponse {
    pub workflow_id: Uuid,
    pub from_version: i32,
    pub to_version: i32,
    pub from_name: String,
    pub to_name: String,
    pub name_changed: bool,
    pub active_changed: bool,
    pub tags_added: Vec<String>,
    pub tags_removed: Vec<String>,
    pub settings_changed: Vec<String>,
    pub nodes_added: Vec<String>,
    pub nodes_removed: Vec<String>,
    pub nodes_changed: Vec<WorkflowNodeChangeResponse>,
    pub connections_added: Vec<String>,
    pub connections_removed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowTemplateResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub difficulty: String,
    pub tags: Vec<String>,
    pub highlights: Vec<String>,
    pub summary: WorkflowSummaryResponse,
    pub nodes: Value,
    pub connections: Value,
    pub settings: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowExportResponse {
    pub format: String,
    pub exported_at: DateTime<Utc>,
    pub workflow: WorkflowResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionResponse {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub status: String,
    pub data: Value,
    pub started_at: DateTime<Utc>,
    pub stopped_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionLogResponse {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub workflow_id: Uuid,
    pub level: String,
    pub event_type: Option<String>,
    pub message: String,
    pub node_id: Option<String>,
    pub node_name: Option<String>,
    pub payload: Value,
    pub created_at: DateTime<Utc>,
}

impl From<ExecutionLogEntity> for ExecutionLogResponse {
    fn from(value: ExecutionLogEntity) -> Self {
        Self {
            id: value.id,
            execution_id: value.execution_id,
            workflow_id: value.workflow_id,
            level: value.level,
            event_type: value.event_type,
            message: value.message,
            node_id: value.node_id,
            node_name: value.node_name,
            payload: value.payload,
            created_at: value.created_at,
        }
    }
}

impl From<ExecutionEntity> for ExecutionResponse {
    fn from(value: ExecutionEntity) -> Self {
        Self {
            id: value.id,
            workflow_id: value.workflow_id,
            status: value.status,
            data: value.data,
            started_at: value.started_at,
            stopped_at: value.stopped_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialResponse {
    pub id: Uuid,
    pub name: String,
    pub credential_type: String,
    pub data: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_tested_at: Option<DateTime<Utc>>,
    pub last_test_status: Option<String>,
    pub last_test_message: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub usage_count: i64,
    pub rotated_at: Option<DateTime<Utc>>,
}

impl CredentialResponse {
    pub fn from_masked_entity(value: CredentialEntity, masked_data: Value) -> Self {
        Self {
            id: value.id,
            name: value.name,
            credential_type: value.cred_type,
            data: masked_data,
            created_at: value.created_at,
            updated_at: value.updated_at,
            last_tested_at: value.last_tested_at,
            last_test_status: value.last_test_status,
            last_test_message: value.last_test_message,
            last_used_at: value.last_used_at,
            usage_count: value.usage_count,
            rotated_at: value.rotated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeSchemaResponse {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub support_tier: String,
    pub support_note: Option<String>,
    pub is_trigger: bool,
    pub type_version: f32,
    pub max_inputs: usize,
    pub documentation_url: Option<String>,
    pub properties: Vec<INodeProperty>,
    pub credentials: Vec<CredentialReference>,
    pub defaults: Value,
}

impl NodeSchemaResponse {
    pub fn from_node_info(
        info: NodeInfo,
        category: impl Into<String>,
        support_tier: impl Into<String>,
        support_note: Option<String>,
        documentation_url: Option<String>,
        credentials: Vec<CredentialReference>,
        defaults: Value,
    ) -> Self {
        Self {
            name: info.name,
            display_name: info.display_name,
            description: info.description,
            category: category.into(),
            support_tier: support_tier.into(),
            support_note,
            is_trigger: info.is_trigger,
            type_version: info.version,
            max_inputs: info.max_inputs,
            documentation_url,
            properties: info.properties.properties,
            credentials,
            defaults,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthUserResponse {
    pub id: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: String,
    pub workspace_role: String,
    pub active_workspace: WorkspaceSummaryResponse,
    pub workspaces: Vec<WorkspaceSummaryResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
    pub user: AuthUserResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfileResponse {
    pub id: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: String,
    pub workspace_role: String,
    pub active_workspace: WorkspaceSummaryResponse,
    pub workspaces: Vec<WorkspaceSummaryResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSettingsResponse {
    pub server_time: DateTime<Utc>,
    pub environment: String,
    pub node_types_count: usize,
    pub credential_types_count: usize,
    pub encryption_key_configured: bool,
    pub execution_mode: String,
    pub worker_concurrency: usize,
    pub queue_capacity: usize,
    pub pruning_enabled: bool,
    pub execution_retention_days: u64,
    pub tracing_enabled: bool,
    pub trace_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionDispatchMetricsResponse {
    pub mode: String,
    pub worker_concurrency: usize,
    pub queue_capacity: usize,
    pub queued_count: usize,
    pub running_count: usize,
    pub total_enqueued: u64,
    pub total_started: u64,
    pub total_finished: u64,
    pub total_failed_to_dispatch: u64,
    pub last_enqueued_at: Option<DateTime<Utc>>,
    pub last_started_at: Option<DateTime<Utc>>,
    pub last_finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionPruningStatusResponse {
    pub enabled: bool,
    pub retention_days: u64,
    pub interval_minutes: u64,
    pub last_run_at: Option<DateTime<Utc>>,
    pub last_cutoff_at: Option<DateTime<Utc>>,
    pub last_executions_deleted: u64,
    pub last_wait_resumes_deleted: u64,
    pub last_logs_deleted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetrySettingsResponse {
    pub enabled: bool,
    pub format: String,
    pub service_name: String,
    pub environment: String,
    pub request_id_header: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationsOverviewResponse {
    pub dispatch: ExecutionDispatchMetricsResponse,
    pub pruning: ExecutionPruningStatusResponse,
    pub telemetry: TelemetrySettingsResponse,
    pub active_executions: usize,
    pub webhook_endpoint_count: usize,
    pub webhook_workflow_count: usize,
    pub cron_workflow_count: usize,
    pub cron_job_count: usize,
    pub node_types_count: usize,
    pub credential_types_count: usize,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PruneExecutionsResponse {
    pub cutoff: DateTime<Utc>,
    pub ran_at: DateTime<Utc>,
    pub executions_deleted: u64,
    pub wait_resumes_deleted: u64,
    pub logs_deleted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialValidationResponse {
    pub valid: bool,
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialOAuthConnectResponse {
    pub credential_id: Uuid,
    pub credential_type: String,
    pub connect_url: String,
    pub redirect_uri: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSummaryResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceMemberResponse {
    pub membership_id: Uuid,
    pub user_id: Uuid,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ApiKeyEntity> for ApiKeyResponse {
    fn from(value: ApiKeyEntity) -> Self {
        Self {
            id: value.id,
            name: value.name,
            key_prefix: value.key_prefix,
            workspace_id: value.workspace_id,
            user_id: value.user_id,
            last_used_at: value.last_used_at,
            expires_at: value.expires_at,
            revoked_at: value.revoked_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyCreateResponse {
    pub api_key: String,
    pub key: ApiKeyResponse,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn workflow_response_serializes_timestamps_as_camel_case() {
        let response = WorkflowResponse {
            id: Uuid::parse_str("7f595f8b-61d2-4cb3-b9f7-d4590f258fb2").unwrap(),
            name: "Example".to_string(),
            active: false,
            tags: vec![WorkflowTagResponse {
                id: Uuid::parse_str("6bda2600-1dc2-4f67-9496-2354eef9a3f6").unwrap(),
                name: "starter".to_string(),
                created_at: DateTime::parse_from_rfc3339("2026-03-10T09:50:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339("2026-03-10T09:50:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            }],
            summary: WorkflowSummaryResponse {
                node_count: 2,
                trigger_count: 1,
                credential_binding_count: 0,
                tag_count: 1,
                latest_version: 3,
            },
            nodes: json!([]),
            connections: json!({}),
            settings: json!({}),
            created_at: DateTime::parse_from_rfc3339("2026-03-10T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339("2026-03-10T10:05:00Z")
                .unwrap()
                .with_timezone(&Utc),
        };

        let value = serde_json::to_value(response).unwrap();
        assert!(value.get("createdAt").is_some());
        assert!(value.get("updatedAt").is_some());
        assert!(value.get("created_at").is_none());
        assert!(value.get("updated_at").is_none());
        assert_eq!(value["summary"]["latestVersion"], 3);
        assert_eq!(value["tags"][0]["name"], "starter");
    }
}
