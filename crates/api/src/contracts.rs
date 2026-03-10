use barqflow_core::properties::INodeProperty;
use barqflow_core::schema::CredentialReference;
use barqflow_db::models::{CredentialEntity, ExecutionEntity, WorkflowEntity};
use barqflow_registry::registry::NodeInfo;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowResponse {
    pub id: Uuid,
    pub name: String,
    pub active: bool,
    pub nodes: Value,
    pub connections: Value,
    pub settings: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<WorkflowEntity> for WorkflowResponse {
    fn from(value: WorkflowEntity) -> Self {
        Self {
            id: value.id,
            name: value.name,
            active: value.active,
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
pub struct ExecutionResponse {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub status: String,
    pub data: Value,
    pub started_at: DateTime<Utc>,
    pub stopped_at: Option<DateTime<Utc>>,
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeSchemaResponse {
    pub name: String,
    pub display_name: String,
    pub description: String,
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
        credentials: Vec<CredentialReference>,
        defaults: Value,
    ) -> Self {
        Self {
            name: info.name,
            display_name: info.display_name,
            description: info.description,
            is_trigger: info.is_trigger,
            type_version: info.version,
            max_inputs: info.max_inputs,
            documentation_url: None,
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
    pub role: String,
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
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSettingsResponse {
    pub server_time: DateTime<Utc>,
    pub environment: String,
    pub node_types_count: usize,
    pub credential_types_count: usize,
    pub encryption_key_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialValidationResponse {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_type: Option<String>,
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
    }
}
