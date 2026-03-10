use barqflow_db::models::{
    AuditLogEntity, PromotionRequestEntity, PromotionTargetEntity, SecretProviderEntity,
    WorkspacePolicyEntity,
};
use chrono::Utc;
use sqlx::{PgPool, Result};
use uuid::Uuid;

const SECRET_PROVIDER_COLUMNS: &str = r#"
    id,
    workspace_id,
    name,
    provider_type,
    config,
    status,
    last_validated_at,
    last_error,
    created_at,
    updated_at
"#;

const WORKSPACE_POLICY_COLUMNS: &str = r#"
    workspace_id,
    blocked_node_types,
    blocked_support_tiers,
    approval_required_node_types,
    max_workflow_nodes,
    created_at,
    updated_at
"#;

const PROMOTION_TARGET_COLUMNS: &str = r#"
    id,
    workspace_id,
    name,
    environment,
    git_repo_url,
    git_branch,
    requires_approval,
    created_at,
    updated_at
"#;

const PROMOTION_REQUEST_COLUMNS: &str = r#"
    id,
    workspace_id,
    workflow_id,
    target_id,
    requested_by_user_id,
    approved_by_user_id,
    status,
    source_control_ref,
    workflow_snapshot,
    notes,
    requested_at,
    approved_at
"#;

const AUDIT_LOG_COLUMNS: &str = r#"
    id,
    workspace_id,
    actor_user_id,
    actor_email,
    action,
    resource_type,
    resource_id,
    summary,
    metadata,
    created_at
"#;

pub struct GovernanceRepository {
    pool: PgPool,
}

impl GovernanceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_secret_providers(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<SecretProviderEntity>> {
        sqlx::query_as::<_, SecretProviderEntity>(&format!(
            r#"
            SELECT {SECRET_PROVIDER_COLUMNS}
            FROM secret_providers
            WHERE workspace_id = $1
            ORDER BY updated_at DESC, name ASC
            "#
        ))
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_secret_provider_in_workspace(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<SecretProviderEntity>> {
        sqlx::query_as::<_, SecretProviderEntity>(&format!(
            r#"
            SELECT {SECRET_PROVIDER_COLUMNS}
            FROM secret_providers
            WHERE workspace_id = $1 AND id = $2
            "#
        ))
        .bind(workspace_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create_secret_provider(
        &self,
        workspace_id: Uuid,
        name: &str,
        provider_type: &str,
        config: serde_json::Value,
        status: &str,
        last_error: Option<&str>,
    ) -> Result<SecretProviderEntity> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query_as::<_, SecretProviderEntity>(&format!(
            r#"
            INSERT INTO secret_providers (
                id,
                workspace_id,
                name,
                provider_type,
                config,
                status,
                last_validated_at,
                last_error,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, NULL, $7, $8, $8)
            RETURNING {SECRET_PROVIDER_COLUMNS}
            "#
        ))
        .bind(id)
        .bind(workspace_id)
        .bind(name)
        .bind(provider_type)
        .bind(config)
        .bind(status)
        .bind(last_error)
        .bind(now)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_secret_provider_validation(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        status: &str,
        last_error: Option<&str>,
    ) -> Result<Option<SecretProviderEntity>> {
        let now = Utc::now();
        sqlx::query_as::<_, SecretProviderEntity>(&format!(
            r#"
            UPDATE secret_providers
            SET status = $1,
                last_validated_at = $2,
                last_error = $3,
                updated_at = $2
            WHERE workspace_id = $4 AND id = $5
            RETURNING {SECRET_PROVIDER_COLUMNS}
            "#
        ))
        .bind(status)
        .bind(now)
        .bind(last_error)
        .bind(workspace_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_workspace_policy(
        &self,
        workspace_id: Uuid,
    ) -> Result<Option<WorkspacePolicyEntity>> {
        sqlx::query_as::<_, WorkspacePolicyEntity>(&format!(
            r#"
            SELECT {WORKSPACE_POLICY_COLUMNS}
            FROM workspace_policies
            WHERE workspace_id = $1
            "#
        ))
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn upsert_workspace_policy(
        &self,
        workspace_id: Uuid,
        blocked_node_types: serde_json::Value,
        blocked_support_tiers: serde_json::Value,
        approval_required_node_types: serde_json::Value,
        max_workflow_nodes: Option<i32>,
    ) -> Result<WorkspacePolicyEntity> {
        let now = Utc::now();
        sqlx::query_as::<_, WorkspacePolicyEntity>(&format!(
            r#"
            INSERT INTO workspace_policies (
                workspace_id,
                blocked_node_types,
                blocked_support_tiers,
                approval_required_node_types,
                max_workflow_nodes,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $6)
            ON CONFLICT (workspace_id)
            DO UPDATE SET blocked_node_types = EXCLUDED.blocked_node_types,
                          blocked_support_tiers = EXCLUDED.blocked_support_tiers,
                          approval_required_node_types = EXCLUDED.approval_required_node_types,
                          max_workflow_nodes = EXCLUDED.max_workflow_nodes,
                          updated_at = EXCLUDED.updated_at
            RETURNING {WORKSPACE_POLICY_COLUMNS}
            "#
        ))
        .bind(workspace_id)
        .bind(blocked_node_types)
        .bind(blocked_support_tiers)
        .bind(approval_required_node_types)
        .bind(max_workflow_nodes)
        .bind(now)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_promotion_targets(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<PromotionTargetEntity>> {
        sqlx::query_as::<_, PromotionTargetEntity>(&format!(
            r#"
            SELECT {PROMOTION_TARGET_COLUMNS}
            FROM promotion_targets
            WHERE workspace_id = $1
            ORDER BY environment ASC, name ASC
            "#
        ))
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_promotion_target_in_workspace(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<PromotionTargetEntity>> {
        sqlx::query_as::<_, PromotionTargetEntity>(&format!(
            r#"
            SELECT {PROMOTION_TARGET_COLUMNS}
            FROM promotion_targets
            WHERE workspace_id = $1 AND id = $2
            "#
        ))
        .bind(workspace_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create_promotion_target(
        &self,
        workspace_id: Uuid,
        name: &str,
        environment: &str,
        git_repo_url: Option<&str>,
        git_branch: Option<&str>,
        requires_approval: bool,
    ) -> Result<PromotionTargetEntity> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query_as::<_, PromotionTargetEntity>(&format!(
            r#"
            INSERT INTO promotion_targets (
                id,
                workspace_id,
                name,
                environment,
                git_repo_url,
                git_branch,
                requires_approval,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)
            RETURNING {PROMOTION_TARGET_COLUMNS}
            "#
        ))
        .bind(id)
        .bind(workspace_id)
        .bind(name)
        .bind(environment)
        .bind(git_repo_url)
        .bind(git_branch)
        .bind(requires_approval)
        .bind(now)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_promotion_requests(
        &self,
        workspace_id: Uuid,
        limit: usize,
    ) -> Result<Vec<PromotionRequestEntity>> {
        sqlx::query_as::<_, PromotionRequestEntity>(&format!(
            r#"
            SELECT {PROMOTION_REQUEST_COLUMNS}
            FROM promotion_requests
            WHERE workspace_id = $1
            ORDER BY requested_at DESC
            LIMIT $2
            "#
        ))
        .bind(workspace_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_promotion_request_in_workspace(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<PromotionRequestEntity>> {
        sqlx::query_as::<_, PromotionRequestEntity>(&format!(
            r#"
            SELECT {PROMOTION_REQUEST_COLUMNS}
            FROM promotion_requests
            WHERE workspace_id = $1 AND id = $2
            "#
        ))
        .bind(workspace_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create_promotion_request(
        &self,
        workspace_id: Uuid,
        workflow_id: Uuid,
        target_id: Uuid,
        requested_by_user_id: Option<Uuid>,
        status: &str,
        source_control_ref: Option<&str>,
        workflow_snapshot: serde_json::Value,
        notes: Option<&str>,
        approved_by_user_id: Option<Uuid>,
        approved_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<PromotionRequestEntity> {
        let id = Uuid::new_v4();
        let requested_at = Utc::now();
        sqlx::query_as::<_, PromotionRequestEntity>(&format!(
            r#"
            INSERT INTO promotion_requests (
                id,
                workspace_id,
                workflow_id,
                target_id,
                requested_by_user_id,
                approved_by_user_id,
                status,
                source_control_ref,
                workflow_snapshot,
                notes,
                requested_at,
                approved_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING {PROMOTION_REQUEST_COLUMNS}
            "#
        ))
        .bind(id)
        .bind(workspace_id)
        .bind(workflow_id)
        .bind(target_id)
        .bind(requested_by_user_id)
        .bind(approved_by_user_id)
        .bind(status)
        .bind(source_control_ref)
        .bind(workflow_snapshot)
        .bind(notes)
        .bind(requested_at)
        .bind(approved_at)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn approve_promotion_request(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        approved_by_user_id: Uuid,
        notes: Option<&str>,
    ) -> Result<Option<PromotionRequestEntity>> {
        let approved_at = Utc::now();
        sqlx::query_as::<_, PromotionRequestEntity>(&format!(
            r#"
            UPDATE promotion_requests
            SET status = 'approved',
                approved_by_user_id = $1,
                approved_at = $2,
                notes = COALESCE($3, notes)
            WHERE workspace_id = $4 AND id = $5
            RETURNING {PROMOTION_REQUEST_COLUMNS}
            "#
        ))
        .bind(approved_by_user_id)
        .bind(approved_at)
        .bind(notes)
        .bind(workspace_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create_audit_log(
        &self,
        workspace_id: Uuid,
        actor_user_id: Option<Uuid>,
        actor_email: Option<&str>,
        action: &str,
        resource_type: &str,
        resource_id: Option<Uuid>,
        summary: &str,
        metadata: serde_json::Value,
    ) -> Result<AuditLogEntity> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query_as::<_, AuditLogEntity>(&format!(
            r#"
            INSERT INTO audit_logs (
                id,
                workspace_id,
                actor_user_id,
                actor_email,
                action,
                resource_type,
                resource_id,
                summary,
                metadata,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING {AUDIT_LOG_COLUMNS}
            "#
        ))
        .bind(id)
        .bind(workspace_id)
        .bind(actor_user_id)
        .bind(actor_email)
        .bind(action)
        .bind(resource_type)
        .bind(resource_id)
        .bind(summary)
        .bind(metadata)
        .bind(now)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn list_audit_logs(
        &self,
        workspace_id: Uuid,
        limit: usize,
    ) -> Result<Vec<AuditLogEntity>> {
        sqlx::query_as::<_, AuditLogEntity>(&format!(
            r#"
            SELECT {AUDIT_LOG_COLUMNS}
            FROM audit_logs
            WHERE workspace_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#
        ))
        .bind(workspace_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
    }
}
