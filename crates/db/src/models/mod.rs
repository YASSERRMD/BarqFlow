use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CredentialEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub owner_user_id: Option<Uuid>,
    pub name: String,
    pub cred_type: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_tested_at: Option<DateTime<Utc>>,
    pub last_test_status: Option<String>,
    pub last_test_message: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub usage_count: i64,
    pub rotated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub owner_user_id: Option<Uuid>,
    pub name: String,
    pub active: bool,
    pub nodes: serde_json::Value,
    pub connections: serde_json::Value,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TagEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkspaceEntity {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_by_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkspaceMembershipEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowHistorySnapshotEntity {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub version: i32,
    pub source: String,
    pub name: String,
    pub active: bool,
    pub tags: serde_json::Value,
    pub nodes: serde_json::Value,
    pub connections: serde_json::Value,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ExecutionEntity {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub status: String,
    pub data: serde_json::Value,
    pub started_at: DateTime<Utc>,
    pub stopped_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ExecutionDispatchItemEntity {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub workspace_id: Uuid,
    pub workflow_id: Uuid,
    pub queue_kind: String,
    pub source: String,
    pub status: String,
    pub priority: i32,
    pub payload: serde_json::Value,
    pub available_at: DateTime<Utc>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub lease_expires_at: Option<DateTime<Utc>>,
    pub worker_id: Option<String>,
    pub attempt_count: i32,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ExecutionLogEntity {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub workflow_id: Uuid,
    pub level: String,
    pub event_type: Option<String>,
    pub message: String,
    pub node_id: Option<String>,
    pub node_name: Option<String>,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WaitResumeEntity {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub node_name: String,
    pub resume_token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub resumed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StaticDataEntity {
    pub id: Uuid,
    pub node_id: String,
    pub workflow_id: Uuid,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserEntity {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub global_role: String,
    pub active_workspace_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiKeyEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub key_hash: String,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SecretProviderEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub provider_type: String,
    pub config: serde_json::Value,
    pub status: String,
    pub last_validated_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkspacePolicyEntity {
    pub workspace_id: Uuid,
    pub blocked_node_types: serde_json::Value,
    pub blocked_support_tiers: serde_json::Value,
    pub approval_required_node_types: serde_json::Value,
    pub max_workflow_nodes: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PromotionTargetEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub environment: String,
    pub git_repo_url: Option<String>,
    pub git_branch: Option<String>,
    pub requires_approval: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PromotionRequestEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub workflow_id: Uuid,
    pub target_id: Uuid,
    pub requested_by_user_id: Option<Uuid>,
    pub approved_by_user_id: Option<Uuid>,
    pub status: String,
    pub source_control_ref: Option<String>,
    pub workflow_snapshot: serde_json::Value,
    pub notes: Option<String>,
    pub requested_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLogEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub actor_user_id: Option<Uuid>,
    pub actor_email: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub summary: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
