use crate::models::UserEntity;
use chrono::Utc;
use sqlx::{PgPool, Result};
use uuid::Uuid;

pub struct UserRepo {
    pool: PgPool,
}

impl UserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<UserEntity>> {
        sqlx::query_as::<_, UserEntity>(
            r#"
            SELECT id, email, password_hash, first_name, last_name, global_role, active_workspace_id, created_at, updated_at
            FROM users
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn get_by_email(&self, email: &str) -> Result<Option<UserEntity>> {
        sqlx::query_as::<_, UserEntity>(
            r#"
            SELECT id, email, password_hash, first_name, last_name, global_role, active_workspace_id, created_at, updated_at
            FROM users
            WHERE email = $1
            "#
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create(
        &self,
        email: &str,
        password_hash: &str,
        first_name: Option<String>,
        last_name: Option<String>,
        global_role: &str,
    ) -> Result<UserEntity> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query_as::<_, UserEntity>(
            r#"
            INSERT INTO users (
                id, email, password_hash, first_name, last_name, global_role, active_workspace_id, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, NULL, $7, $8)
            RETURNING id, email, password_hash, first_name, last_name, global_role, active_workspace_id, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(email)
        .bind(password_hash)
        .bind(first_name)
        .bind(last_name)
        .bind(global_role)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_password(
        &self,
        id: Uuid,
        password_hash: &str,
    ) -> Result<Option<UserEntity>> {
        let now = Utc::now();

        sqlx::query_as::<_, UserEntity>(
            r#"
            UPDATE users
            SET password_hash = $1, updated_at = $2
            WHERE id = $3
            RETURNING id, email, password_hash, first_name, last_name, global_role, active_workspace_id, created_at, updated_at
            "#
        )
        .bind(password_hash)
        .bind(now)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn set_active_workspace(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<UserEntity>> {
        let now = Utc::now();

        sqlx::query_as::<_, UserEntity>(
            r#"
            UPDATE users
            SET active_workspace_id = $1, updated_at = $2
            WHERE id = $3
            RETURNING id, email, password_hash, first_name, last_name, global_role, active_workspace_id, created_at, updated_at
            "#
        )
        .bind(workspace_id)
        .bind(now)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
    }
}
